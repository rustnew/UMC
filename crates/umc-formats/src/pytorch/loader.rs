use super::pickle::{self, Pv};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use umc_core::ir::provenance::ProvenanceEntryData;
use umc_core::ir::MetaValue;
use umc_core::{DType, Tensor, UmcError, UniversalIR};
use umc_core::{FormatLoader, LoadOptions, ProgressCallback, UMC_VERSION};
use zip::ZipArchive;

pub struct PyTorchLoader;

// ── Dtype mapping ──────────────────────────────────────────────────────────────

fn storage_to_dtype(class: &str) -> DType {
    match class {
        s if s.contains("Float") || s.contains("float32") => DType::F32,
        s if s.contains("Half") || s.contains("float16") => DType::F16,
        s if s.contains("BFloat16") || s.contains("bfloat16") => DType::BF16,
        s if s.contains("Double") || s.contains("float64") => DType::F64,
        s if s.contains("Long") || s.contains("int64") => DType::I64,
        s if s.contains("Int") || s.contains("int32") => DType::I32,
        s if s.contains("Short") || s.contains("int16") => DType::I16,
        s if s.contains("Byte") || s.contains("uint8") => DType::U8,
        s if s.contains("Char") || s.contains("int8") => DType::I8,
        s if s.contains("Bool") || s.contains("bool") => DType::Bool,
        _ => DType::F32,
    }
}

impl FormatLoader for PyTorchLoader {
    fn format_name(&self) -> &'static str {
        "PyTorch"
    }

    fn can_load(&self, path: &Path) -> bool {
        path.extension().map_or(false, |e| {
            let e = e.to_string_lossy();
            e == "pt" || e == "pth" || e == "bin"
        })
    }

    fn load(
        &self,
        path: &Path,
        _opts: &LoadOptions,
        progress: &ProgressCallback,
    ) -> Result<UniversalIR, UmcError> {
        let file = std::fs::File::open(path).map_err(UmcError::Io)?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| UmcError::Other(format!("PyTorch: not a valid ZIP: {}", e)))?;

        // ── 1. Read data.pkl ────────────────────────────────────────────────
        let pkl_name = {
            let names: Vec<String> = (0..archive.len())
                .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
                .collect();
            names
                .into_iter()
                .find(|n| n.ends_with("data.pkl"))
                .ok_or_else(|| UmcError::Other("PyTorch: no data.pkl in archive".into()))?
        };

        let pkl_bytes = {
            let mut f = archive
                .by_name(&pkl_name)
                .map_err(|e| UmcError::Other(format!("PyTorch: cannot read pkl: {}", e)))?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).map_err(UmcError::Io)?;
            buf
        };

        // ── 2. Parse pickle ────────────────────────────────────────────────
        let root = pickle::parse(&pkl_bytes)
            .map_err(|e| UmcError::Other(format!("PyTorch pickle: {}", e)))?;

        // Extract state_dict as a flat list of (name, Pv::PtTensor)
        let entries = flatten_state_dict(root);
        progress.set_total(entries.len() as u64);

        // ── 3. Load storage buffers from archive ────────────────────────────
        let mut storages: HashMap<String, Vec<u8>> = HashMap::new();
        for i in 0..archive.len() {
            let name = archive
                .by_index(i)
                .map_err(|e| UmcError::Other(e.to_string()))?
                .name()
                .to_string();
            // Pattern: "archive/data/0" or "archive/data/1" etc.
            if name.contains("/data/") && !name.ends_with('/') {
                let key = name.rsplit('/').next().unwrap_or(&name).to_string();
                let mut f = archive
                    .by_name(&name)
                    .map_err(|e| UmcError::Other(e.to_string()))?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf).map_err(UmcError::Io)?;
                storages.insert(key, buf);
            }
        }

        // ── 4. Build IR ─────────────────────────────────────────────────────
        let mut ir = UniversalIR::new("PyTorch", path);

        for (name, pv) in &entries {
            if let Pv::PtTensor {
                storage_key,
                dtype_class,
                storage_offset,
                shape,
                ..
            } = pv
            {
                let dtype = storage_to_dtype(dtype_class);
                let elem_bytes = dtype.bytes_per_element().unwrap_or(4.0) as usize;
                let n_elems: usize = shape.iter().product();
                let byte_count = n_elems * elem_bytes;

                let storage = storages.get(storage_key.as_str()).ok_or_else(|| {
                    UmcError::Other(format!("storage '{}' not found", storage_key))
                })?;

                let byte_offset = storage_offset * elem_bytes;
                let end = byte_offset + byte_count;
                if end > storage.len() {
                    return Err(UmcError::Other(format!(
                        "tensor '{}': storage OOB (offset={}, n={}, storage_len={})",
                        name,
                        byte_offset,
                        byte_count,
                        storage.len()
                    )));
                }

                let raw = storage[byte_offset..end].to_vec();
                let tensor = Tensor::from_bytes(name.clone(), dtype, shape.clone(), raw);
                ir.tensors
                    .insert(tensor)
                    .map_err(|e| UmcError::Other(e.to_string()))?;
                progress.increment(name);
            }
        }

        ir.metadata
            .insert("source_format", MetaValue::String("PyTorch".into()));
        ir.metadata
            .insert("tensor_count", MetaValue::I64(ir.tensors.len() as i64));

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        ir.provenance.append(ProvenanceEntryData {
            timestamp,
            source_format: "PyTorch".into(),
            target_format: "IR".into(),
            tool: format!("umc/{}", UMC_VERSION),
            input_hash: "unknown".into(),
            output_hash: None,
            roundtrip_level: "structural".into(),
            max_divergence: None,
            warnings: vec![],
        });

        Ok(ir)
    }
}

// ── Helper: flatten a state_dict Pv tree into (name, PtTensor) pairs ──────────

fn flatten_state_dict(root: Pv) -> Vec<(String, Pv)> {
    let mut out = Vec::new();
    match root {
        Pv::Dict(items) => {
            for (k, v) in items {
                let name = match &k {
                    Pv::Str(s) => s.clone(),
                    Pv::Bytes(b) => String::from_utf8_lossy(b).into_owned(),
                    _ => continue,
                };
                collect_tensors(name, v, &mut out);
            }
        }
        // Real torch.save wraps the dict in an OrderedDict Object — unwrap it
        Pv::Object { args, .. } => {
            return flatten_state_dict(*args);
        }
        Pv::Tuple(items) => {
            // Flat alternating [key, value, key, value, ...] from some pickle variants
            let mut i = 0;
            while i + 1 < items.len() {
                if let Pv::Str(name) = &items[i] {
                    collect_tensors(name.clone(), items[i + 1].clone(), &mut out);
                }
                i += 2;
            }
        }
        Pv::PtTensor { .. } => out.push(("tensor".into(), root)),
        _ => {}
    }
    out
}

fn collect_tensors(prefix: String, value: Pv, out: &mut Vec<(String, Pv)>) {
    match value {
        Pv::PtTensor { .. } => out.push((prefix, value)),
        Pv::Dict(items) => {
            for (k, v) in items {
                if let Pv::Str(name) = k {
                    let full = if prefix.is_empty() {
                        name
                    } else {
                        format!("{}.{}", prefix, name)
                    };
                    collect_tensors(full, v, out);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pytorch::saver::PyTorchSaver;
    use tempfile::NamedTempFile;
    use umc_core::{DType, FormatSaver, LoadOptions, ProgressCallback, SaveOptions};

    fn make_test_ir() -> UniversalIR {
        let mut ir = UniversalIR::new("test", std::path::Path::new("test.pt"));
        let data: Vec<f32> = (0..12).map(|i| i as f32 * 0.5).collect();
        let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_le_bytes()).collect();
        ir.tensors
            .insert(Tensor::from_bytes("weight", DType::F32, vec![3, 4], bytes))
            .unwrap();
        ir
    }

    #[test]
    fn test_pytorch_round_trip() {
        let ir = make_test_ir();
        let saver = PyTorchSaver;
        let loader = PyTorchLoader;

        let tmp = NamedTempFile::new().unwrap();
        let opts = SaveOptions::default();
        let progress = ProgressCallback::noop();

        saver.save(&ir, tmp.path(), &opts, &progress).unwrap();
        let loaded = loader
            .load(tmp.path(), &LoadOptions::default(), &progress)
            .unwrap();

        assert_eq!(loaded.tensors.len(), 1);
        let t = loaded.tensors.get("weight").unwrap();
        assert_eq!(t.shape, vec![3, 4]);

        let orig_bytes = ir.tensors.get("weight").unwrap().data.as_bytes().unwrap();
        let loaded_bytes = t.data.as_bytes().unwrap();
        assert_eq!(orig_bytes, loaded_bytes);
    }
}
