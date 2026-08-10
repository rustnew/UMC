use super::dtype_map::umc_dtype_to_onnx;
use super::proto::{
    GraphProto, ModelProto, OperatorSetIdProto, StringStringEntryProto, TensorProto,
};
use crate::gguf::kquant::dequantize_any_to_f32_bytes;
use prost::Message;
use std::io::Write;
use std::path::Path;
use umc_core::DType;
use umc_core::{FormatSaver, ProgressCallback, SaveOptions, UmcError, UniversalIR, UMC_VERSION};

/// ONNX format saver.
///
/// Writes initializer (weight) tensors into an ONNX ModelProto.
/// The compute graph is not reconstructed (weights-only ONNX).
pub struct OnnxSaver;

impl FormatSaver for OnnxSaver {
    fn format_name(&self) -> &'static str {
        "ONNX"
    }
    fn default_extension(&self) -> &'static str {
        "onnx"
    }

    fn save(
        &self,
        ir: &UniversalIR,
        path: &Path,
        _options: &SaveOptions,
        progress: &ProgressCallback,
    ) -> Result<(), UmcError> {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let tmp = parent.join(format!(
            ".umc_tmp_{}.onnx",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));

        let result = self.write_to_path(ir, &tmp, progress);
        match result {
            Ok(()) => std::fs::rename(&tmp, path).map_err(|e| UmcError::AtomicRename {
                src: tmp.display().to_string(),
                dst: path.display().to_string(),
                msg: e.to_string(),
            }),
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                Err(e)
            }
        }
    }
}

impl OnnxSaver {
    fn write_to_path(
        &self,
        ir: &UniversalIR,
        path: &Path,
        progress: &ProgressCallback,
    ) -> Result<(), UmcError> {
        progress.set_total(ir.tensors.len() as u64);

        let mut initializers: Vec<TensorProto> = Vec::with_capacity(ir.tensors.len());

        for (name, tensor) in ir.tensors.iter() {
            let (data_type, raw_bytes): (i32, std::borrow::Cow<[u8]>) =
                match umc_dtype_to_onnx(&tensor.dtype) {
                    Some(dt) => {
                        let raw = tensor
                            .data
                            .as_bytes()
                            .map_err(|e| UmcError::Other(format!("Tensor '{}': {}", name, e)))?;
                        (dt, std::borrow::Cow::Borrowed(raw))
                    }
                    None => {
                        // Quantized dtype — dequantize to F32
                        let raw = tensor
                            .data
                            .as_bytes()
                            .map_err(|e| UmcError::Other(format!("Tensor '{}': {}", name, e)))?;
                        let n_elems: usize = tensor.shape.iter().product::<usize>().max(1);
                        let f32_bytes = dequantize_any_to_f32_bytes(raw, &tensor.dtype, n_elems)
                            .map_err(|e| {
                                UmcError::Other(format!(
                                    "ONNX dequant tensor '{}' ({:?}): {}",
                                    name, tensor.dtype, e
                                ))
                            })?;
                        let onnx_f32 = umc_dtype_to_onnx(&DType::F32).unwrap();
                        progress
                            .report(&format!("Dequantizing '{}' ({:?})→F32", name, tensor.dtype));
                        (onnx_f32, std::borrow::Cow::Owned(f32_bytes))
                    }
                };

            let f32_shape: Vec<i64> = if matches!(umc_dtype_to_onnx(&tensor.dtype), None) {
                // Shape stays the same after dequantization
                tensor.shape.iter().map(|&d| d as i64).collect()
            } else {
                tensor.shape.iter().map(|&d| d as i64).collect()
            };

            let proto = TensorProto {
                dims: f32_shape,
                data_type: Some(data_type),
                name: Some(name.clone()),
                raw_data: Some(raw_bytes.into_owned()),
                ..Default::default()
            };
            initializers.push(proto);
            progress.increment(&format!("Packed '{}'", name));
        }

        // Build metadata_props from IR metadata + provenance
        let mut metadata_props = vec![
            StringStringEntryProto {
                key: Some("umc_version".into()),
                value: Some(UMC_VERSION.into()),
            },
            StringStringEntryProto {
                key: Some("architecture".into()),
                value: Some(ir.architecture.architecture.clone()),
            },
        ];
        if let Some(last) = ir.provenance.last_entry() {
            metadata_props.push(StringStringEntryProto {
                key: Some("source_format".into()),
                value: Some(last.source_format.clone()),
            });
        }

        let model = ModelProto {
            ir_version: Some(8),
            opset_import: vec![OperatorSetIdProto {
                domain: Some(String::new()),
                version: Some(21),
            }],
            graph: Some(GraphProto {
                name: Some(ir.architecture.architecture.clone()),
                initializer: initializers,
                ..Default::default()
            }),
            metadata_props,
            ..Default::default()
        };

        let mut buf = Vec::with_capacity(1024 * 1024);
        model
            .encode(&mut buf)
            .map_err(|e| UmcError::Other(format!("ONNX encode error: {}", e)))?;

        let mut file = std::fs::File::create(path).map_err(UmcError::Io)?;
        file.write_all(&buf).map_err(UmcError::Io)?;
        file.flush().map_err(UmcError::Io)?;

        progress.report("ONNX file written.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::loader::OnnxLoader;
    use super::*;
    use tempfile::NamedTempFile;
    use umc_core::ProgressCallback;
    use umc_core::{DType, FormatLoader, LoadOptions, Tensor, UniversalIR};

    fn make_f32_ir() -> UniversalIR {
        let mut ir = UniversalIR::new("TEST", std::path::Path::new("x.onnx"));
        let data: Vec<u8> = [1.0f32, 2.0, 3.0, 4.0]
            .iter()
            .flat_map(|v: &f32| v.to_le_bytes())
            .collect();
        ir.tensors
            .insert(Tensor::from_bytes("weight", DType::F32, vec![2, 2], data))
            .unwrap();
        ir
    }

    #[test]
    fn test_save_creates_valid_onnx() {
        let ir = make_f32_ir();
        let f = NamedTempFile::with_suffix(".onnx").unwrap();
        OnnxSaver
            .save(
                &ir,
                f.path(),
                &SaveOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();
        assert!(f.path().metadata().unwrap().len() > 0);
    }

    #[test]
    fn test_round_trip_f32() {
        let ir = make_f32_ir();
        let f = NamedTempFile::with_suffix(".onnx").unwrap();
        OnnxSaver
            .save(
                &ir,
                f.path(),
                &SaveOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();

        let ir2 = OnnxLoader
            .load(f.path(), &LoadOptions::default(), &ProgressCallback::noop())
            .unwrap();
        assert_eq!(ir2.tensors.len(), 1);
        let t = ir2.tensors.get("weight").unwrap();
        assert_eq!(t.dtype, DType::F32);
        assert_eq!(t.shape, vec![2, 2]);

        let original: Vec<f32> = [1.0f32, 2.0, 3.0, 4.0].to_vec();
        let loaded: Vec<f32> = t
            .data
            .as_bytes()
            .unwrap()
            .chunks(4)
            .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        assert_eq!(
            loaded, original,
            "F32 ONNX round-trip must be bit-identical"
        );
    }
}
