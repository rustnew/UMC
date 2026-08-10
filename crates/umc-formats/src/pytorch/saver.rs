use std::io::Write;
use std::path::Path;
use umc_core::{DType, UmcError, UniversalIR};
use umc_core::{FormatSaver, ProgressCallback, SaveOptions, UMC_VERSION};
use zip::{write::FileOptions, ZipWriter};

pub struct PyTorchSaver;

impl FormatSaver for PyTorchSaver {
    fn format_name(&self) -> &'static str {
        "PyTorch"
    }
    fn default_extension(&self) -> &'static str {
        "pt"
    }

    fn save(
        &self,
        ir: &UniversalIR,
        path: &Path,
        _opts: &SaveOptions,
        progress: &ProgressCallback,
    ) -> Result<(), UmcError> {
        let tmp_path = path.with_extension("pt.tmp");
        {
            let file = std::fs::File::create(&tmp_path).map_err(UmcError::Io)?;
            let mut zip = ZipWriter::new(file);
            let compress =
                FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

            // ── Sort tensors for deterministic output ────────────────────────
            let mut names: Vec<&str> = ir.tensors.iter().map(|(n, _)| n.as_str()).collect();
            names.sort();
            progress.set_total(names.len() as u64);

            // ── Write each tensor's raw storage ──────────────────────────────
            // storage_index → (name, dtype, shape, storage_offset=0, stride)
            let mut storage_index: u32 = 0;
            let mut tensor_entries: Vec<PtTensorEntry> = Vec::with_capacity(names.len());

            for name in &names {
                let tensor = ir.tensors.get(*name).unwrap();
                let raw = tensor
                    .data
                    .as_bytes()
                    .map_err(|e| UmcError::Other(format!("PyTorch saver: '{}': {}", name, e)))?;

                let storage_file = format!("archive/data/{}", storage_index);
                zip.start_file(&storage_file, compress)
                    .map_err(|e| UmcError::Other(e.to_string()))?;
                zip.write_all(raw).map_err(UmcError::Io)?;

                let stride = c_contiguous_strides(&tensor.shape);
                tensor_entries.push(PtTensorEntry {
                    name: name.to_string(),
                    storage_idx: storage_index,
                    dtype: tensor.dtype.clone(),
                    shape: tensor.shape.clone(),
                    stride,
                    numel: tensor.shape.iter().product(),
                });

                storage_index += 1;
                progress.increment(name);
            }

            // ── Write byteorder ──────────────────────────────────────────────
            zip.start_file("archive/byteorder", compress)
                .map_err(|e| UmcError::Other(e.to_string()))?;
            zip.write_all(b"little").map_err(UmcError::Io)?;

            // ── Write data.pkl ───────────────────────────────────────────────
            let pkl = write_state_dict_pickle(&tensor_entries);
            zip.start_file("archive/data.pkl", compress)
                .map_err(|e| UmcError::Other(e.to_string()))?;
            zip.write_all(&pkl).map_err(UmcError::Io)?;

            zip.finish().map_err(|e| UmcError::Other(e.to_string()))?;
        }

        std::fs::rename(&tmp_path, path).map_err(UmcError::Io)?;
        Ok(())
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

struct PtTensorEntry {
    name: String,
    storage_idx: u32,
    dtype: DType,
    shape: Vec<usize>,
    stride: Vec<usize>,
    numel: usize,
}

fn c_contiguous_strides(shape: &[usize]) -> Vec<usize> {
    let mut strides = vec![1usize; shape.len()];
    for i in (0..shape.len().saturating_sub(1)).rev() {
        strides[i] = strides[i + 1] * shape[i + 1];
    }
    strides
}

fn dtype_to_storage_class(dtype: &DType) -> &'static str {
    match dtype {
        DType::F32 => "FloatStorage",
        DType::F16 => "HalfStorage",
        DType::BF16 => "BFloat16Storage",
        DType::F64 => "DoubleStorage",
        DType::I64 => "LongStorage",
        DType::I32 => "IntStorage",
        DType::I16 => "ShortStorage",
        DType::U8 => "ByteStorage",
        DType::I8 => "CharStorage",
        DType::Bool => "BoolStorage",
        _ => "FloatStorage",
    }
}

// ── Minimal pickle v2 writer ──────────────────────────────────────────────────
// Writes a valid Python OrderedDict of tensors using protocol 2.

fn write_state_dict_pickle(entries: &[PtTensorEntry]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();

    // PROTO 2
    out.extend_from_slice(&[0x80, 0x02]);

    // EMPTY_DICT  {}
    out.push(b'}');

    // MARK  (  — all key/value pairs will be added via SETITEMS
    out.push(b'(');

    // For each tensor, write key (SHORT_BINUNICODE) + value (_rebuild_tensor_v2)
    let mut memo_idx: u8 = 0;

    for entry in entries {
        // ── Write tensor name (key) ──────────────────────────────────────
        let name_bytes = entry.name.as_bytes();
        out.push(0x8C); // SHORT_BINUNICODE
        out.push(name_bytes.len() as u8);
        out.extend_from_slice(name_bytes);

        // ── Write _rebuild_tensor_v2 call ────────────────────────────────
        // GLOBAL torch._utils._rebuild_tensor_v2
        out.push(b'c');
        out.extend_from_slice(b"torch._utils\n_rebuild_tensor_v2\n");

        // MARK for args tuple
        out.push(b'(');

        // Arg 0: storage (BINPERSID with persistent tuple)
        // Push the persistent ID tuple
        out.push(b'('); // MARK
                        // 'storage' string
        out.push(0x8C);
        out.push(7);
        out.extend_from_slice(b"storage");
        // storage class
        out.push(b'c');
        out.extend_from_slice(b"torch\n");
        let sc = dtype_to_storage_class(&entry.dtype);
        out.extend_from_slice(sc.as_bytes());
        out.push(b'\n');
        // key (storage index as string)
        let key_str = entry.storage_idx.to_string();
        out.push(0x8C);
        out.push(key_str.len() as u8);
        out.extend_from_slice(key_str.as_bytes());
        // device
        out.push(0x8C);
        out.push(3);
        out.extend_from_slice(b"cpu");
        // numel
        write_binint(&mut out, entry.numel as i64);
        out.push(b't'); // TUPLE (from MARK)
        out.push(b'Q'); // BINPERSID

        // Arg 1: storage_offset = 0
        out.push(b'K');
        out.push(0); // BININT1 = 0

        // Arg 2: shape (tuple of dims)
        out.push(b'(');
        for &d in &entry.shape {
            write_binint(&mut out, d as i64);
        }
        out.push(b't'); // TUPLE

        // Arg 3: stride (tuple)
        out.push(b'(');
        for &s in &entry.stride {
            write_binint(&mut out, s as i64);
        }
        out.push(b't'); // TUPLE

        // Arg 4: requires_grad = False
        out.push(0x89); // NEWFALSE

        // Arg 5: backward_hooks = OrderedDict()
        out.push(b'c');
        out.extend_from_slice(b"collections\nOrderedDict\n");
        out.push(b')'); // EMPTY_TUPLE
        out.push(b'R'); // REDUCE → OrderedDict()

        // Close args tuple
        out.push(b't'); // TUPLE
                        // REDUCE
        out.push(b'R');
    }

    // SETITEMS  u  — pops all key-value pairs from the mark and adds to the dict
    out.push(b'u');

    // STOP
    out.push(b'.');

    out
}

fn write_binint(out: &mut Vec<u8>, v: i64) {
    if v >= 0 && v <= 0xFF {
        out.push(b'K');
        out.push(v as u8);
    } else if v >= 0 && v <= 0xFFFF {
        out.push(b'M');
        let u = v as u16;
        out.extend_from_slice(&u.to_le_bytes());
    } else {
        out.push(b'J');
        let i = v as i32;
        out.extend_from_slice(&i.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_contiguous_strides() {
        assert_eq!(c_contiguous_strides(&[3, 4]), vec![4, 1]);
        assert_eq!(c_contiguous_strides(&[2, 3, 4]), vec![12, 4, 1]);
        assert_eq!(c_contiguous_strides(&[5]), vec![1]);
    }
}
