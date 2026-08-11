use memmap2::Mmap;
use std::path::Path;
use std::sync::Arc;
use umc_core::{
    ir::provenance::ProvenanceEntryData, DType, FormatLoader, GraphContent, LoadOptions,
    ProgressCallback, Tensor, UmcError, UniversalIR, UMC_VERSION,
};

fn st_dtype_to_umc(s: &str) -> Result<DType, UmcError> {
    match s {
        "F64" => Ok(DType::F64),
        "F32" => Ok(DType::F32),
        "F16" => Ok(DType::F16),
        "BF16" => Ok(DType::BF16),
        "I64" => Ok(DType::I64),
        "I32" => Ok(DType::I32),
        "I16" => Ok(DType::I16),
        "I8" => Ok(DType::I8),
        "U64" => Ok(DType::U64),
        "U32" => Ok(DType::U32),
        "U16" => Ok(DType::U16),
        "U8" => Ok(DType::U8),
        "BOOL" => Ok(DType::Bool),
        "F8_E4M3" => Ok(DType::F8E4M3),
        "F8_E5M2" => Ok(DType::F8E5M2),
        other => Ok(DType::Custom(other.to_string())),
    }
}

/// Native SafeTensors format loader.
pub struct SafeTensorsLoader;

impl FormatLoader for SafeTensorsLoader {
    fn format_name(&self) -> &'static str {
        "SafeTensors"
    }

    fn can_load(&self, path: &Path) -> bool {
        // Check magic: 8 bytes LE size + '{'
        let Ok(mut f) = std::fs::File::open(path) else {
            return false;
        };
        let mut buf = [0u8; 9];
        use std::io::Read;
        if f.read_exact(&mut buf).is_err() {
            return false;
        }
        let size = u64::from_le_bytes(buf[0..8].try_into().unwrap_or([0; 8]));
        buf[8] == b'{' && size > 2 && size < 100_000_000
    }

    fn load(
        &self,
        path: &Path,
        options: &LoadOptions,
        progress: &ProgressCallback,
    ) -> Result<UniversalIR, UmcError> {
        progress.report("Opening SafeTensors file…");

        let file = std::fs::File::open(path).map_err(UmcError::Io)?;
        let file_size = file.metadata().map_err(UmcError::Io)?.len();
        let mmap = Arc::new(unsafe {
            Mmap::map(&file).map_err(|e| UmcError::Mmap {
                context: path.display().to_string(),
                msg: e.to_string(),
            })?
        });

        let data: &[u8] = &mmap;

        if data.len() < 9 {
            return Err(UmcError::FileTruncated {
                path: path.display().to_string(),
                expected: 9,
                actual: data.len(),
            });
        }

        let header_size = u64::from_le_bytes(data[0..8].try_into().unwrap_or([0; 8])) as usize;

        if 8 + header_size > data.len() {
            return Err(UmcError::FileTruncated {
                path: path.display().to_string(),
                expected: 8 + header_size,
                actual: data.len(),
            });
        }

        let header_bytes = &data[8..8 + header_size];
        let header: serde_json::Value =
            serde_json::from_slice(header_bytes).map_err(UmcError::Json)?;

        let header_obj = header
            .as_object()
            .ok_or_else(|| UmcError::Other("SafeTensors header is not a JSON object".into()))?;

        // Security: max 1M tensors
        if header_obj.len() > 1_000_001 {
            return Err(UmcError::SecurityViolation {
                field: "safetensors_tensor_count".into(),
                value: header_obj.len(),
                limit: 1_000_000,
            });
        }

        let data_start: usize = 8 + header_size;
        let mut ir = UniversalIR::new("SafeTensors", path);

        // Extract __metadata__ if present
        if let Some(meta_val) = header_obj.get("__metadata__") {
            if let Some(meta_obj) = meta_val.as_object() {
                for (k, v) in meta_obj {
                    if let Some(s) = v.as_str() {
                        ir.metadata
                            .insert(k.clone(), umc_core::MetaValue::String(s.to_string()));
                    }
                }
            }
        }

        if options.metadata_only {
            return Ok(ir);
        }

        progress.set_total(header_obj.len() as u64);

        for (name, tensor_info) in header_obj {
            if name == "__metadata__" {
                continue;
            }

            let dtype_str = tensor_info["dtype"]
                .as_str()
                .ok_or_else(|| UmcError::Other(format!("Tensor '{}' missing dtype", name)))?;
            let dtype = st_dtype_to_umc(dtype_str)?;

            let shape: Vec<usize> = tensor_info["shape"]
                .as_array()
                .ok_or_else(|| UmcError::Other(format!("Tensor '{}' missing shape", name)))?
                .iter()
                .map(|v| v.as_u64().unwrap_or(0) as usize)
                .collect();

            let offsets = tensor_info["data_offsets"].as_array().ok_or_else(|| {
                UmcError::Other(format!("Tensor '{}' missing data_offsets", name))
            })?;
            let start = offsets[0].as_u64().unwrap_or(0) as usize;
            let end = offsets[1].as_u64().unwrap_or(0) as usize;

            let abs_start = data_start + start;
            let abs_end = data_start + end;
            let length = end - start;

            // Security bounds check
            if abs_end as u64 > file_size {
                return Err(UmcError::TensorOutOfBounds {
                    name: name.clone(),
                    offset: abs_start as u64,
                    length,
                    end: abs_end as u64,
                    file_size,
                });
            }

            let tensor =
                Tensor::from_mmap(name, dtype, shape, Arc::clone(&mmap), abs_start, length);
            ir.tensors.insert(tensor)?;
            progress.increment(&format!("Loaded '{}'", name));
        }

        ir.graph = GraphContent::WeightsOnly {
            architecture: ir
                .metadata
                .get_str("architecture")
                .unwrap_or("unknown")
                .to_string(),
            template_available: false,
            template_name: None,
        };

        ir.provenance.append(ProvenanceEntryData {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            source_format: "SafeTensors".into(),
            target_format: "IR".into(),
            tool: format!("umc/{}", UMC_VERSION),
            input_hash: "unknown".into(),
            output_hash: None,
            roundtrip_level: "bit_identical".into(),
            max_divergence: None,
            warnings: vec![],
        });

        progress.report("SafeTensors loaded.");
        Ok(ir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_st_file(tensors: &[(&str, &str, Vec<usize>, &[f32])]) -> NamedTempFile {
        let mut header_map = serde_json::Map::new();
        let mut data_parts: Vec<Vec<u8>> = Vec::new();
        let mut offset: u64 = 0;

        for (name, dtype, shape, values) in tensors {
            let bytes: Vec<u8> = values.iter().flat_map(|f| f.to_le_bytes()).collect();
            let end = offset + bytes.len() as u64;
            header_map.insert(
                name.to_string(),
                serde_json::json!({
                    "dtype": dtype,
                    "shape": shape,
                    "data_offsets": [offset, end],
                }),
            );
            offset = end;
            data_parts.push(bytes);
        }

        let header_json = serde_json::to_string(&serde_json::Value::Object(header_map)).unwrap();
        let header_bytes = header_json.as_bytes();

        let mut f = NamedTempFile::new().unwrap();
        f.write_all(&(header_bytes.len() as u64).to_le_bytes())
            .unwrap();
        f.write_all(header_bytes).unwrap();
        for part in &data_parts {
            f.write_all(part).unwrap();
        }
        f.flush().unwrap();
        f
    }

    #[test]
    fn test_load_f32_tensor() {
        let f = make_st_file(&[("weight", "F32", vec![2, 2], &[1.0, 2.0, 3.0, 4.0])]);
        let loader = SafeTensorsLoader;
        let ir = loader
            .load(f.path(), &LoadOptions::default(), &ProgressCallback::noop())
            .unwrap();
        assert_eq!(ir.tensors.len(), 1);
        let t = ir.tensors.get("weight").unwrap();
        assert_eq!(t.dtype, DType::F32);
        assert_eq!(t.shape, vec![2, 2]);
    }

    #[test]
    fn test_load_bf16_tensor() {
        // BF16 1.0 = 0x3F80, stored as 2 bytes each
        let bf16_bytes: Vec<u8> = vec![0x80u8, 0x3F, 0x80, 0x3F];
        let header = serde_json::json!({
            "w": { "dtype": "BF16", "shape": [2], "data_offsets": [0, 4] }
        });
        let header_str = header.to_string();
        let mut f = NamedTempFile::new().unwrap();
        use std::io::Write;
        f.write_all(&(header_str.len() as u64).to_le_bytes())
            .unwrap();
        f.write_all(header_str.as_bytes()).unwrap();
        f.write_all(&bf16_bytes).unwrap();
        f.flush().unwrap();

        let loader = SafeTensorsLoader;
        let ir = loader
            .load(f.path(), &LoadOptions::default(), &ProgressCallback::noop())
            .unwrap();
        assert_eq!(ir.tensors.get("w").unwrap().dtype, DType::BF16);
    }

    #[test]
    fn test_reject_out_of_bounds_tensor() {
        let header = serde_json::json!({
            "w": { "dtype": "F32", "shape": [100], "data_offsets": [0, 40000] }
        });
        let header_str = header.to_string();
        let mut f = NamedTempFile::new().unwrap();
        use std::io::Write;
        f.write_all(&(header_str.len() as u64).to_le_bytes())
            .unwrap();
        f.write_all(header_str.as_bytes()).unwrap();
        f.write_all(&[0u8; 4]).unwrap(); // Only 4 bytes, not 40000
        f.flush().unwrap();

        let loader = SafeTensorsLoader;
        let err = loader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop());
        assert!(matches!(err, Err(UmcError::TensorOutOfBounds { .. })));
    }

    #[test]
    fn test_metadata_only_skips_tensors() {
        let f = make_st_file(&[("weight", "F32", vec![4], &[1.0, 2.0, 3.0, 4.0])]);
        let loader = SafeTensorsLoader;
        let mut opts = LoadOptions::default();
        opts.metadata_only = true;
        let ir = loader
            .load(f.path(), &opts, &ProgressCallback::noop())
            .unwrap();
        assert!(ir.tensors.is_empty());
    }
}
