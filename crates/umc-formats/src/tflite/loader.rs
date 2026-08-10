use super::flatbuf::parse_tflite;
use memmap2::Mmap;
use std::path::Path;
use std::sync::Arc;
use umc_core::ir::provenance::ProvenanceEntryData;
use umc_core::ir::{GraphContent, MetaValue};
use umc_core::UMC_VERSION;
use umc_core::{FormatLoader, LoadOptions, ProgressCallback};
use umc_core::{Tensor, UmcError, UniversalIR};

pub struct TFLiteLoader;

impl FormatLoader for TFLiteLoader {
    fn format_name(&self) -> &'static str {
        "TFLite"
    }

    fn can_load(&self, path: &Path) -> bool {
        if path.extension().map_or(false, |e| e == "tflite") {
            return true;
        }
        // Check FlatBuffer magic at bytes 4..8
        use std::io::Read;
        let mut f = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return false,
        };
        let mut buf = [0u8; 8];
        if f.read_exact(&mut buf).is_err() {
            return false;
        }
        matches!(&buf[4..8], b"TFL3" | b"TFL2" | b"TFL1")
    }

    fn load(
        &self,
        path: &Path,
        _opts: &LoadOptions,
        progress: &ProgressCallback,
    ) -> Result<UniversalIR, UmcError> {
        // mmap the file
        let file = std::fs::File::open(path).map_err(UmcError::Io)?;
        let mmap = unsafe { Mmap::map(&file) }.map_err(|e| UmcError::Mmap {
            context: path.to_string_lossy().into(),
            msg: e.to_string(),
        })?;
        let mmap = Arc::new(mmap);

        let model =
            parse_tflite(&mmap[..]).map_err(|e| UmcError::Other(format!("TFLite parse: {}", e)))?;

        let mut ir = UniversalIR::new("TFLite", path);
        ir.metadata
            .insert("source_format", MetaValue::String("TFLite".into()));
        ir.metadata
            .insert("tensor_count", MetaValue::I64(model.tensors.len() as i64));

        progress.set_total(model.tensors.len() as u64);

        for tf_tensor in &model.tensors {
            let shape: Vec<usize> = tf_tensor.shape.iter().map(|&s| s as usize).collect();
            let n_elems: usize = shape.iter().product();
            let elem_bytes = tf_tensor.dtype.bytes_per_element().unwrap_or(4.0) as usize;
            let expected_bytes = n_elems * elem_bytes;

            let buf = model.buffers.get(tf_tensor.buffer_idx).ok_or_else(|| {
                UmcError::Other(format!(
                    "TFLite: tensor '{}' references buffer {} which does not exist",
                    tf_tensor.name, tf_tensor.buffer_idx
                ))
            })?;

            // Skip empty buffers (e.g. graph inputs/outputs with no weights)
            if buf.is_empty() && n_elems > 0 {
                continue;
            }

            let raw = if buf.len() >= expected_bytes {
                buf[..expected_bytes].to_vec()
            } else {
                buf.clone()
            };
            let tensor =
                Tensor::from_bytes(tf_tensor.name.clone(), tf_tensor.dtype.clone(), shape, raw);
            ir.tensors
                .insert(tensor)
                .map_err(|e| UmcError::Other(e.to_string()))?;
            progress.increment(&tf_tensor.name);
        }

        ir.graph = GraphContent::WeightsOnly {
            architecture: "TFLite".into(),
            template_available: false,
            template_name: None,
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        ir.provenance.append(ProvenanceEntryData {
            timestamp,
            source_format: "TFLite".into(),
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

#[cfg(test)]
mod tests {
    use super::super::saver::TFLiteSaver;
    use super::*;

    #[test]
    fn test_tflite_can_load_extension() {
        let loader = TFLiteLoader;
        assert!(loader.can_load(std::path::Path::new("model.tflite")));
        assert!(!loader.can_load(std::path::Path::new("model.onnx")));
    }

    #[test]
    fn test_tflite_round_trip() {
        use tempfile::NamedTempFile;
        use umc_core::{FormatSaver, SaveOptions, Tensor};

        let mut ir = UniversalIR::new("test", std::path::Path::new("model.tflite"));
        let data: Vec<f32> = (0..16).map(|i| i as f32).collect();
        let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_le_bytes()).collect();
        ir.tensors
            .insert(Tensor::from_bytes(
                "dense/kernel",
                umc_core::DType::F32,
                vec![4, 4],
                bytes.clone(),
            ))
            .unwrap();

        let saver = TFLiteSaver;
        let tmp = NamedTempFile::new().unwrap();
        saver
            .save(
                &ir,
                tmp.path(),
                &SaveOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();

        let loader = TFLiteLoader;
        let loaded = loader
            .load(
                tmp.path(),
                &LoadOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();
        assert_eq!(loaded.tensors.len(), 1);
        let t = loaded.tensors.get("dense/kernel").unwrap();
        assert_eq!(t.shape, vec![4, 4]);
        assert_eq!(t.data.as_bytes().unwrap(), bytes.as_slice());
    }
}
