use std::path::Path;
use std::io::{BufWriter, Write};
use umc_core::{
    UmcError, UniversalIR, DType,
    FormatSaver, SaveOptions, ProgressCallback,
    UMC_VERSION,
};
use crate::gguf::kquant::kquant_to_f32_bytes;
use serde_json::{json, Value};

/// Maps UMC DType to SafeTensors dtype string.
fn dtype_to_st_str(dtype: &DType) -> Option<&'static str> {
    match dtype {
        DType::F64  => Some("F64"),
        DType::F32  => Some("F32"),
        DType::F16  => Some("F16"),
        DType::BF16 => Some("BF16"),
        DType::I64  => Some("I64"),
        DType::I32  => Some("I32"),
        DType::I16  => Some("I16"),
        DType::I8   => Some("I8"),
        DType::U64  => Some("U64"),
        DType::U32  => Some("U32"),
        DType::U16  => Some("U16"),
        DType::U8   => Some("U8"),
        DType::Bool => Some("BOOL"),
        DType::F8E4M3 => Some("F8_E4M3"),
        DType::F8E5M2 => Some("F8_E5M2"),
        _ => None,  // Quantized types not directly representable
    }
}

/// Dequantize a tensor to F32 bytes (for quantized → SafeTensors conversion).
fn dequantize_to_f32(tensor_data: &[u8], dtype: &DType, shape: &[usize]) -> Result<Vec<u8>, UmcError> {
    let n_elems: usize = shape.iter().product::<usize>().max(1);
    let mut output = vec![0u8; n_elems * 4];

    match dtype {
        // Q4_0: 32 elements per block, 18 bytes per block (2 bytes F16 scale + 16 bytes Q4)
        DType::Q4_0 => {
            let block_size: usize = 32;
            let bytes_per_block: usize = 18;
            for (block_idx, block) in tensor_data.chunks(bytes_per_block).enumerate() {
                if block.len() < bytes_per_block { break; }
                let scale_bits = u16::from_le_bytes([block[0], block[1]]);
                let scale = f16_to_f32(scale_bits);
                let quant_bytes = &block[2..18];
                for (i, &byte) in quant_bytes.iter().enumerate() {
                    let lo = (byte & 0x0F) as i32 - 8;
                    let hi = ((byte >> 4) & 0x0F) as i32 - 8;
                    let idx0 = block_idx * block_size + i * 2;
                    let idx1 = idx0 + 1;
                    if idx0 < n_elems {
                        let v = (lo as f32) * scale;
                        output[idx0*4..(idx0+1)*4].copy_from_slice(&v.to_le_bytes());
                    }
                    if idx1 < n_elems {
                        let v = (hi as f32) * scale;
                        output[idx1*4..(idx1+1)*4].copy_from_slice(&v.to_le_bytes());
                    }
                }
            }
        }
        // Q8_0: 32 elements per block, 34 bytes per block (2 bytes F16 scale + 32 bytes I8)
        DType::Q8_0 => {
            let block_size: usize = 32;
            let bytes_per_block: usize = 34;
            for (block_idx, block) in tensor_data.chunks(bytes_per_block).enumerate() {
                if block.len() < bytes_per_block { break; }
                let scale_bits = u16::from_le_bytes([block[0], block[1]]);
                let scale = f16_to_f32(scale_bits);
                for (i, &byte) in block[2..].iter().enumerate() {
                    let idx = block_idx * block_size + i;
                    if idx < n_elems {
                        let v = (byte as i8 as f32) * scale;
                        output[idx*4..(idx+1)*4].copy_from_slice(&v.to_le_bytes());
                    }
                }
            }
        }
        // K-quant types: full block-level decode
        DType::Q2K | DType::Q3KS | DType::Q3KM | DType::Q3KL |
        DType::Q4KS | DType::Q4KM |
        DType::Q5KS | DType::Q5KM |
        DType::Q6K  | DType::Q8K => {
            return kquant_to_f32_bytes(tensor_data, dtype, n_elems);
        }
        // F16 → F32 upcasting
        DType::F16 => {
            for i in 0..n_elems {
                if i * 2 + 1 < tensor_data.len() {
                    let bits = u16::from_le_bytes([tensor_data[i*2], tensor_data[i*2+1]]);
                    let v = f16_to_f32(bits);
                    output[i*4..(i+1)*4].copy_from_slice(&v.to_le_bytes());
                }
            }
        }
        // BF16 → F32 upcasting
        DType::BF16 => {
            for i in 0..n_elems {
                if i * 2 + 1 < tensor_data.len() {
                    let bits = u16::from_le_bytes([tensor_data[i*2], tensor_data[i*2+1]]);
                    let v = bf16_to_f32(bits);
                    output[i*4..(i+1)*4].copy_from_slice(&v.to_le_bytes());
                }
            }
        }
        _ => {}
    }

    Ok(output)
}

/// IEEE 754 F16 to F32.
fn f16_to_f32(bits: u16) -> f32 {
    let sign = ((bits & 0x8000) as u32) << 16;
    let exp = (bits & 0x7c00) as u32;
    let mant = (bits & 0x03ff) as u32;
    let bits32 = if exp == 0x7c00 {
        sign | 0x7f800000 | (mant << 13)
    } else if exp == 0 {
        if mant == 0 {
            sign
        } else {
            let mut e = 0u32;
            let mut m = mant;
            while m & 0x0400 == 0 { m <<= 1; e += 1; }
            let normalized_exp = 127 - 15 - e + 1;
            sign | (normalized_exp << 23) | ((m & 0x03ff) << 13)
        }
    } else {
        sign | ((exp >> 10) + (127 - 15)) << 23 | (mant << 13)
    };
    f32::from_bits(bits32)
}

/// BF16 to F32.
fn bf16_to_f32(bits: u16) -> f32 {
    f32::from_bits((bits as u32) << 16)
}

/// SafeTensors format saver.
///
/// Writes tensors in the SafeTensors binary format:
/// [8 bytes LE header_size][header_size bytes JSON][tensor_data…]
pub struct SafeTensorsSaver;

impl FormatSaver for SafeTensorsSaver {
    fn format_name(&self) -> &'static str { "SafeTensors" }
    fn default_extension(&self) -> &'static str { "safetensors" }

    fn save(
        &self,
        ir: &UniversalIR,
        path: &Path,
        options: &SaveOptions,
        progress: &ProgressCallback,
    ) -> Result<(), UmcError> {
        progress.report("Preparing SafeTensors output…");

        // Write to a temp file then atomic rename
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let tmp_path = parent.join(format!(
            ".umc_tmp_{}.safetensors",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));

        let result = self.write_to_path(ir, &tmp_path, options, progress);

        match result {
            Ok(()) => {
                // Atomic rename
                std::fs::rename(&tmp_path, path).map_err(|e| UmcError::AtomicRename {
                    src: tmp_path.display().to_string(),
                    dst: path.display().to_string(),
                    msg: e.to_string(),
                })?;
                progress.report("SafeTensors file written successfully.");
                Ok(())
            }
            Err(e) => {
                // Clean up temp file on failure
                let _ = std::fs::remove_file(&tmp_path);
                Err(e)
            }
        }
    }
}

impl SafeTensorsSaver {
    fn write_to_path(
        &self,
        ir: &UniversalIR,
        path: &Path,
        options: &SaveOptions,
        progress: &ProgressCallback,
    ) -> Result<(), UmcError> {
        let target_dtype = options.dtype.as_ref();

        // ── Phase 1: Sorted tensor list + output dtype/size metadata ──────
        // No data is loaded or copied here — only shapes and dtype strings.
        let mut tensor_list: Vec<(&str, &umc_core::Tensor)> = ir.tensors.iter()
            .map(|(n, t)| (n.as_str(), t))
            .collect();
        tensor_list.sort_by_key(|(n, _)| *n);
        progress.set_total(tensor_list.len() as u64);

        struct TensorMeta<'a> {
            name: &'a str,
            tensor: &'a umc_core::Tensor,
            st_dtype_str: &'static str,
            byte_len: u64,
        }

        let mut meta: Vec<TensorMeta> = Vec::with_capacity(tensor_list.len());
        for (name, tensor) in &tensor_list {
            let effective = target_dtype.unwrap_or(&tensor.dtype);
            let (st_str, byte_len) = if let Some(s) = dtype_to_st_str(effective) {
                // native dtype — byte count from raw data length
                let n_elems: usize = tensor.shape.iter().product();
                let elem_bytes = effective.bytes_per_element().unwrap_or(0.0) as usize;
                (s, (n_elems * elem_bytes) as u64)
            } else {
                // quantized → will be dequantized to F32
                let n_elems: usize = tensor.shape.iter().product();
                ("F32", (n_elems * 4) as u64)
            };
            meta.push(TensorMeta { name, tensor, st_dtype_str: st_str, byte_len });
        }

        // ── Phase 2: Build JSON header from sizes (no data copy) ──────────
        let mut header_map = serde_json::Map::with_capacity(meta.len() + 1);
        let mut data_offset: u64 = 0;
        for m in &meta {
            let end = data_offset + m.byte_len;
            header_map.insert(m.name.to_string(), json!({
                "dtype": m.st_dtype_str,
                "shape": m.tensor.shape,
                "data_offsets": [data_offset, end],
            }));
            data_offset = end;
        }
        header_map.insert("__metadata__".into(), json!({
            "umc_version": UMC_VERSION,
            "source_format": ir.provenance.last_entry()
                .map(|e| e.source_format.as_str()).unwrap_or("unknown"),
            "architecture": ir.architecture.architecture,
        }));
        let header_json = serde_json::to_string(&Value::Object(header_map))
            .map_err(UmcError::Json)?;

        // ── Phase 3: Open file, write header ─────────────────────────────
        let file = std::fs::File::create(path).map_err(UmcError::Io)?;
        let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file);
        writer.write_all(&(header_json.len() as u64).to_le_bytes()).map_err(UmcError::Io)?;
        writer.write_all(header_json.as_bytes()).map_err(UmcError::Io)?;

        // ── Phase 4: Stream each tensor — dequantize, write, discard ─────
        // Peak RAM = max(single tensor F32 size), not total model size.
        progress.report("Streaming tensor data…");
        for m in &meta {
            let raw = m.tensor.data.as_bytes().map_err(|e| {
                UmcError::Other(format!("Tensor '{}': {}", m.name, e))
            })?;

            if dtype_to_st_str(&m.tensor.dtype).is_some() {
                writer.write_all(raw).map_err(UmcError::Io)?;
            } else {
                let f32_bytes = dequantize_to_f32(raw, &m.tensor.dtype, &m.tensor.shape)?;
                writer.write_all(&f32_bytes).map_err(UmcError::Io)?;
                // f32_bytes dropped here — only one tensor in RAM at a time
            }

            progress.increment(&format!("Wrote '{}'", m.name));
        }

        writer.flush().map_err(UmcError::Io)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use umc_core::{Tensor, UniversalIR};
    use std::path::Path;
    use tempfile::NamedTempFile;

    fn make_f32_ir() -> UniversalIR {
        let mut ir = UniversalIR::new("GGUF", Path::new("test.gguf"));
        let data: Vec<u8> = vec![0.0f32, 1.0f32, 2.0f32, 3.0f32]
            .iter().flat_map(|f: &f32| f.to_le_bytes()).collect();
        let t = Tensor::from_bytes("linear.weight", DType::F32, vec![2, 2], data);
        ir.tensors.insert(t).unwrap();
        ir
    }

    #[test]
    fn test_save_f32() {
        let ir = make_f32_ir();
        let f = NamedTempFile::new().unwrap();
        let saver = SafeTensorsSaver;
        saver.save(&ir, f.path(), &SaveOptions::default(), &ProgressCallback::noop()).unwrap();

        // Verify the file has valid SafeTensors header
        let bytes = std::fs::read(f.path()).unwrap();
        assert!(bytes.len() > 8);
        let header_size = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        assert!(header_size > 0);
        assert_eq!(bytes[8], b'{');
    }

    #[test]
    fn test_f16_to_f32_conversion() {
        // 1.0 in F16 = 0x3C00
        let v = f16_to_f32(0x3C00);
        assert!((v - 1.0).abs() < 1e-5, "Expected 1.0 got {}", v);
    }

    #[test]
    fn test_bf16_to_f32_conversion() {
        // 1.0 in BF16 = 0x3F80
        let v = bf16_to_f32(0x3F80);
        assert!((v - 1.0).abs() < 1e-5, "Expected 1.0 got {}", v);
    }

    #[test]
    fn test_save_creates_valid_json_header() {
        let ir = make_f32_ir();
        let f = NamedTempFile::new().unwrap();
        let saver = SafeTensorsSaver;
        saver.save(&ir, f.path(), &SaveOptions::default(), &ProgressCallback::noop()).unwrap();

        let bytes = std::fs::read(f.path()).unwrap();
        let header_size = u64::from_le_bytes(bytes[0..8].try_into().unwrap()) as usize;
        let header_json = std::str::from_utf8(&bytes[8..8+header_size]).unwrap();
        let header: serde_json::Value = serde_json::from_str(header_json).unwrap();
        assert!(header.get("linear.weight").is_some());
        assert_eq!(header["linear.weight"]["dtype"], "F32");
    }
}
