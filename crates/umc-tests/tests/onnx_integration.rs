/// Integration tests for the ONNX loader and saver.
use umc_core::{DType, Tensor, UniversalIR, FormatLoader, FormatSaver, LoadOptions, SaveOptions, ProgressCallback};
use umc_formats::{OnnxLoader, OnnxSaver};
use std::io::Write;

fn make_onnx_bytes_f32(tensors: &[(&str, Vec<usize>, Vec<f32>)]) -> Vec<u8> {
    use prost::Message;

    let initializers: Vec<_> = tensors.iter().map(|(name, shape, values)| {
        let raw: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
        umc_formats::onnx::proto::TensorProto {
            dims: shape.iter().map(|&d| d as i64).collect(),
            data_type: Some(1), // FLOAT
            name: Some(name.to_string()),
            raw_data: Some(raw),
            ..Default::default()
        }
    }).collect();

    let graph = umc_formats::onnx::proto::GraphProto {
        name: Some("test".into()),
        initializer: initializers,
        ..Default::default()
    };
    let model = umc_formats::onnx::proto::ModelProto {
        ir_version: Some(8),
        graph: Some(graph),
        opset_import: vec![umc_formats::onnx::proto::OperatorSetIdProto {
            domain: Some(String::new()),
            version: Some(21),
        }],
        ..Default::default()
    };
    let mut buf = Vec::new();
    model.encode(&mut buf).unwrap();
    buf
}

#[test]
fn test_onnx_load_f32_tensor() {
    let buf = make_onnx_bytes_f32(&[("weight", vec![2, 2], vec![1.0, 2.0, 3.0, 4.0])]);
    let mut f = tempfile::NamedTempFile::with_suffix(".onnx").unwrap();
    f.write_all(&buf).unwrap(); f.flush().unwrap();

    let ir = OnnxLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    assert_eq!(ir.tensors.len(), 1);
    let t = ir.tensors.get("weight").unwrap();
    assert_eq!(t.dtype, DType::F32);
    assert_eq!(t.shape, vec![2, 2]);
}

#[test]
fn test_onnx_round_trip_f32() {
    let original_vals = vec![1.0f32, -1.5, 0.0, 3.14, 2.718, -0.5];
    let buf = make_onnx_bytes_f32(&[("layer.weight", vec![2, 3], original_vals.clone())]);
    let mut f = tempfile::NamedTempFile::with_suffix(".onnx").unwrap();
    f.write_all(&buf).unwrap(); f.flush().unwrap();

    // Load ONNX → IR
    let ir = OnnxLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();

    // Save IR → ONNX
    let out = tempfile::NamedTempFile::with_suffix(".onnx").unwrap();
    OnnxSaver.save(&ir, out.path(), &SaveOptions::default(), &ProgressCallback::noop()).unwrap();

    // Reload and compare
    let ir2 = OnnxLoader.load(out.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    let t = ir2.tensors.get("layer.weight").unwrap();
    let loaded: Vec<f32> = t.data.as_bytes().unwrap().chunks(4)
        .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
        .collect();

    assert_eq!(loaded.len(), original_vals.len());
    for (a, b) in loaded.iter().zip(original_vals.iter()) {
        assert!((a - b).abs() < 1e-6, "ONNX round-trip divergence: {} vs {}", a, b);
    }
}

#[test]
fn test_onnx_round_trip_multiple_tensors() {
    let tensors_data = vec![
        ("encoder.weight",   vec![4, 4], (0..16).map(|i| i as f32).collect::<Vec<_>>()),
        ("decoder.bias",     vec![4],    vec![0.1, 0.2, 0.3, 0.4]),
        ("norm.weight",      vec![4],    vec![1.0, 1.0, 1.0, 1.0]),
    ];
    let refs: Vec<(&str, Vec<usize>, Vec<f32>)> = tensors_data.iter()
        .map(|(n, s, v)| (*n, s.clone(), v.clone()))
        .collect();

    let buf = make_onnx_bytes_f32(&refs);
    let mut f = tempfile::NamedTempFile::with_suffix(".onnx").unwrap();
    f.write_all(&buf).unwrap(); f.flush().unwrap();

    let ir = OnnxLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    assert_eq!(ir.tensors.len(), 3);

    let out = tempfile::NamedTempFile::with_suffix(".onnx").unwrap();
    OnnxSaver.save(&ir, out.path(), &SaveOptions::default(), &ProgressCallback::noop()).unwrap();

    let ir2 = OnnxLoader.load(out.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    assert_eq!(ir2.tensors.len(), 3);

    for (name, _, original) in &tensors_data {
        let t = ir2.tensors.get(*name).unwrap();
        let loaded: Vec<f32> = t.data.as_bytes().unwrap().chunks(4)
            .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        for (a, b) in loaded.iter().zip(original.iter()) {
            assert!((a - b).abs() < 1e-6, "tensor {} divergence: {} vs {}", name, a, b);
        }
    }
}

#[test]
fn test_onnx_metadata_preserved() {
    let buf = make_onnx_bytes_f32(&[("w", vec![1], vec![1.0])]);
    let mut f = tempfile::NamedTempFile::with_suffix(".onnx").unwrap();
    f.write_all(&buf).unwrap(); f.flush().unwrap();

    let ir = OnnxLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    assert!(ir.metadata.get("onnx.ir_version").is_some(), "ir_version missing");
    assert!(ir.metadata.get("onnx.opset_version").is_some(), "opset_version missing");
    assert!(ir.provenance.verify(), "provenance chain invalid");
}

#[test]
fn test_onnx_provenance_source_format() {
    let buf = make_onnx_bytes_f32(&[("w", vec![1], vec![0.0])]);
    let mut f = tempfile::NamedTempFile::with_suffix(".onnx").unwrap();
    f.write_all(&buf).unwrap(); f.flush().unwrap();

    let ir = OnnxLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    let last = ir.provenance.last_entry().unwrap();
    assert_eq!(last.source_format, "ONNX");
}
