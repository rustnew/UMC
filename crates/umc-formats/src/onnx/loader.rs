use std::path::Path;
use memmap2::Mmap;
use prost::Message;
use umc_core::{
    UmcError, UniversalIR,
    FormatLoader, LoadOptions, ProgressCallback,
    Tensor,
    MetaValue, ArchitectureConfig,
    ir::provenance::ProvenanceEntryData,
    UMC_VERSION,
};
use super::proto::{ModelProto, TensorProto};
use super::dtype_map::onnx_dtype_to_umc;

/// Native ONNX loader — parses the ModelProto protobuf, extracts initializer
/// tensors (model weights), metadata, and operator graph.
/// Supports opset 1-21, ONNX IR version 3-10.
pub struct OnnxLoader;

impl FormatLoader for OnnxLoader {
    fn format_name(&self) -> &'static str { "ONNX" }

    fn can_load(&self, path: &Path) -> bool {
        path.extension().map_or(false, |e| e == "onnx")
    }

    fn load(
        &self,
        path: &Path,
        options: &LoadOptions,
        progress: &ProgressCallback,
    ) -> Result<UniversalIR, UmcError> {
        progress.report("Opening ONNX file…");

        // Zero-copy mmap for the file
        let file = std::fs::File::open(path).map_err(UmcError::Io)?;
        let mmap = unsafe {
            Mmap::map(&file).map_err(|e| UmcError::Mmap {
                context: path.display().to_string(),
                msg: e.to_string(),
            })?
        };

        progress.report("Decoding ONNX protobuf…");
        let model = ModelProto::decode(mmap.as_ref()).map_err(|e| {
            UmcError::ProtobufDecode(format!("{}: {}", path.display(), e))
        })?;

        let graph = model.graph.as_ref().ok_or_else(|| {
            UmcError::Other("ONNX: ModelProto has no graph".into())
        })?;

        // ── IR construction ───────────────────────────────────────────────
        let mut ir = UniversalIR::new("ONNX", path);

        // Architecture config
        ir.architecture = build_arch_config(&model, graph);

        // Metadata from model props
        for prop in &model.metadata_props {
            if let (Some(k), Some(v)) = (&prop.key, &prop.value) {
                ir.metadata.insert(k.clone(), MetaValue::String(v.clone()));
            }
        }
        if let Some(v) = model.ir_version {
            ir.metadata.insert("onnx.ir_version".to_string(), MetaValue::I64(v));
        }
        for opset in &model.opset_import {
            if let (Some(d), Some(v)) = (&opset.domain, opset.version) {
                let key = if d.is_empty() {
                    "onnx.opset_version".to_string()
                } else {
                    format!("onnx.opset_{}", d)
                };
                ir.metadata.insert(key, MetaValue::I64(v));
            }
        }

        // ── Tensors from initializers (weights) ───────────────────────────
        if !options.metadata_only {
            progress.set_total(graph.initializer.len() as u64);

            for tensor_proto in &graph.initializer {
                match tensor_proto_to_tensor(tensor_proto) {
                    Ok(Some(tensor)) => {
                        progress.increment(&format!("Loaded '{}'", tensor.name));
                        ir.tensors.insert(tensor).map_err(|e| {
                            UmcError::Other(format!("TensorStore insert: {}", e))
                        })?;
                    }
                    Ok(None) => {} // unsupported dtype — skip silently
                    Err(e) => {
                        tracing::warn!("Skipping tensor: {}", e);
                    }
                }
            }
        }

        // ── Provenance ────────────────────────────────────────────────────
        let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        ir.provenance.append(ProvenanceEntryData {
            timestamp,
            source_format: "ONNX".into(),
            target_format: "IR".into(),
            tool: format!("umc-formats/{}", UMC_VERSION),
            input_hash: format!("size:{}", file_size),
            output_hash: None,
            roundtrip_level: "lossless".into(),
            max_divergence: None,
            warnings: vec![],
        });

        Ok(ir)
    }
}

/// Convert a TensorProto to a UMC Tensor.
/// Returns Ok(None) if the dtype is unknown/unsupported.
fn tensor_proto_to_tensor(
    proto: &TensorProto,
) -> Result<Option<Tensor>, UmcError> {
    let name = proto.name.as_deref().unwrap_or("").to_string();
    if name.is_empty() { return Ok(None); }

    let dtype = match proto.data_type.and_then(onnx_dtype_to_umc) {
        Some(d) => d,
        None => {
            tracing::warn!("Unsupported ONNX dtype {} for tensor '{}'",
                proto.data_type.unwrap_or(0), name);
            return Ok(None);
        }
    };

    let shape: Vec<usize> = proto.dims.iter()
        .map(|&d| d.max(0) as usize)
        .collect();

    // Prefer raw_data, then fall back to typed fields.
    let bytes: Vec<u8> = if let Some(raw) = &proto.raw_data {
        raw.clone()
    } else {
        typed_fields_to_bytes(proto, &dtype)?
    };
    let tensor = Tensor::from_bytes(&name, dtype, shape, bytes);
    Ok(Some(tensor))
}

/// Flatten typed proto fields into little-endian bytes.
fn typed_fields_to_bytes(proto: &TensorProto, dtype: &umc_core::DType) -> Result<Vec<u8>, UmcError> {
    use umc_core::DType::*;
    match dtype {
        F32 => Ok(proto.float_data.iter().flat_map(|v| v.to_le_bytes()).collect()),
        F64 => Ok(proto.double_data.iter().flat_map(|v| v.to_le_bytes()).collect()),
        I32 => Ok(proto.int32_data.iter().flat_map(|v| v.to_le_bytes()).collect()),
        I64 => Ok(proto.int64_data.iter().flat_map(|v| v.to_le_bytes()).collect()),
        U64 => Ok(proto.uint64_data.iter().flat_map(|v| v.to_le_bytes()).collect()),
        Bool | U8 | I8 => {
            // stored as int32 in ONNX, take low byte
            Ok(proto.int32_data.iter().map(|&v| v as u8).collect())
        }
        I16 | U16 | F16 | BF16 => {
            // stored as int32 in ONNX (each value fits in 16 bits)
            Ok(proto.int32_data.iter().flat_map(|&v| (v as u16).to_le_bytes()).collect())
        }
        _ => Ok(vec![]),
    }
}

fn build_arch_config(
    model: &ModelProto,
    graph: &super::proto::GraphProto,
) -> ArchitectureConfig {
    let mut cfg = ArchitectureConfig::default();

    // Try to infer hidden_size from the first weight initializer
    if let Some(first) = graph.initializer.first() {
        if first.dims.len() >= 2 {
            cfg.hidden_size = first.dims[1].max(0) as usize;
        }
    }

    // Infer num_layers by counting unique layer indices in tensor names
    let max_layer = graph.initializer.iter()
        .filter_map(|t| t.name.as_deref())
        .filter_map(|n| {
            // match patterns like "encoder.layer.0." or "layers.0."
            n.split('.').find_map(|part| part.parse::<usize>().ok())
        })
        .max()
        .unwrap_or(0);
    cfg.num_layers = if max_layer > 0 { max_layer + 1 } else { 0 };

    // Opset version
    for opset in &model.opset_import {
        if opset.domain.as_deref().unwrap_or("") == "" {
            cfg.architecture = format!("onnx_opset_{}", opset.version.unwrap_or(0));
        }
    }

    cfg
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_minimal_onnx() -> Vec<u8> {
        use prost::Message;
        use super::super::proto::*;

        let tensor = TensorProto {
            dims: vec![2, 2],
            data_type: Some(1), // FLOAT
            name: Some("weight".into()),
            raw_data: Some(vec![
                0u8, 0, 128, 63,  // 1.0f32 LE
                0, 0, 0, 64,      // 2.0f32 LE
                0, 0, 64, 64,     // 3.0f32 LE
                0, 0, 128, 64,    // 4.0f32 LE
            ]),
            ..Default::default()
        };

        let graph = GraphProto {
            name: Some("test_graph".into()),
            initializer: vec![tensor],
            ..Default::default()
        };

        let model = ModelProto {
            ir_version: Some(8),
            graph: Some(graph),
            opset_import: vec![OperatorSetIdProto {
                domain: Some(String::new()),
                version: Some(17),
            }],
            ..Default::default()
        };

        let mut buf = Vec::new();
        model.encode(&mut buf).unwrap();
        buf
    }

    #[test]
    fn test_load_minimal_onnx() {
        let buf = make_minimal_onnx();
        let mut f = NamedTempFile::with_suffix(".onnx").unwrap();
        f.write_all(&buf).unwrap();
        f.flush().unwrap();

        let ir = OnnxLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
        assert_eq!(ir.tensors.len(), 1);
        let t = ir.tensors.get("weight").unwrap();
        assert_eq!(t.dtype, umc_core::DType::F32);
        assert_eq!(t.shape, vec![2, 2]);
    }

    #[test]
    fn test_load_metadata_only() {
        let buf = make_minimal_onnx();
        let mut f = NamedTempFile::with_suffix(".onnx").unwrap();
        f.write_all(&buf).unwrap();
        f.flush().unwrap();

        let mut opts = LoadOptions::default();
        opts.metadata_only = true;
        let ir = OnnxLoader.load(f.path(), &opts, &ProgressCallback::noop()).unwrap();
        assert!(ir.tensors.is_empty());
    }

    #[test]
    fn test_provenance_chain_valid() {
        let buf = make_minimal_onnx();
        let mut f = NamedTempFile::with_suffix(".onnx").unwrap();
        f.write_all(&buf).unwrap();
        f.flush().unwrap();

        let ir = OnnxLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
        assert!(ir.provenance.verify());
        assert_eq!(ir.provenance.last_entry().unwrap().source_format, "ONNX");
    }

    #[test]
    fn test_load_onnx_ir_version_in_metadata() {
        let buf = make_minimal_onnx();
        let mut f = NamedTempFile::with_suffix(".onnx").unwrap();
        f.write_all(&buf).unwrap();
        f.flush().unwrap();

        let ir = OnnxLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
        assert!(ir.metadata.get("onnx.ir_version").is_some());
        assert!(ir.metadata.get("onnx.opset_version").is_some());
    }
}
