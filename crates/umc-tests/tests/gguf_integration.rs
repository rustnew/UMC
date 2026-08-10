use std::io::Write;
use tempfile::NamedTempFile;
use umc_core::{FormatLoader, LoadOptions, ProgressCallback};
use umc_formats::GgufLoader;
use umc_tests::{write_gguf_v3_with_f32_tensors, write_minimal_gguf};

#[test]
fn test_gguf_load_empty_model() {
    let f = write_minimal_gguf();
    let ir = GgufLoader
        .load(f.path(), &LoadOptions::default(), &ProgressCallback::noop())
        .unwrap();
    assert_eq!(ir.tensors.len(), 0);
    assert!(ir.provenance.verify());
    assert_eq!(ir.provenance.len(), 1);
}

#[test]
fn test_gguf_load_single_f32_tensor() {
    let values = vec![1.0f32, 2.0, 3.0, 4.0];
    let f = write_gguf_v3_with_f32_tensors(&[("weight", vec![2, 2], values.clone())]);
    let ir = GgufLoader
        .load(f.path(), &LoadOptions::default(), &ProgressCallback::noop())
        .unwrap();
    assert_eq!(ir.tensors.len(), 1);
    let t = ir.tensors.get("weight").unwrap();
    assert_eq!(t.shape, vec![2, 2]);
    let bytes = t.data.as_bytes().unwrap();
    let loaded: Vec<f32> = bytes
        .chunks(4)
        .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
        .collect();
    assert_eq!(loaded, values);
}

#[test]
fn test_gguf_load_multiple_tensors() {
    let f = write_gguf_v3_with_f32_tensors(&[
        ("embed.weight", vec![32, 8], vec![0.5f32; 256]),
        ("norm.weight", vec![8], vec![1.0f32; 8]),
        ("output.weight", vec![32, 8], vec![0.1f32; 256]),
    ]);
    let ir = GgufLoader
        .load(f.path(), &LoadOptions::default(), &ProgressCallback::noop())
        .unwrap();
    assert_eq!(ir.tensors.len(), 3);
    assert!(ir.tensors.get("embed.weight").is_some());
    assert!(ir.tensors.get("norm.weight").is_some());
    assert!(ir.tensors.get("output.weight").is_some());
}

#[test]
fn test_gguf_provenance_chain_valid() {
    let f = write_gguf_v3_with_f32_tensors(&[("layer.0.weight", vec![4, 4], vec![0.0f32; 16])]);
    let ir = GgufLoader
        .load(f.path(), &LoadOptions::default(), &ProgressCallback::noop())
        .unwrap();
    assert!(
        ir.provenance.verify(),
        "Provenance chain must pass integrity check"
    );
    let last = ir.provenance.last_entry().unwrap();
    assert_eq!(last.source_format, "GGUF");
    assert_eq!(last.target_format, "IR");
}

#[test]
fn test_gguf_metadata_only_loads_zero_tensors() {
    let f = write_gguf_v3_with_f32_tensors(&[("w", vec![4], vec![1.0f32; 4])]);
    let mut opts = LoadOptions::default();
    opts.metadata_only = true;
    let ir = GgufLoader
        .load(f.path(), &opts, &ProgressCallback::noop())
        .unwrap();
    assert!(ir.tensors.is_empty());
}

#[test]
fn test_gguf_reject_wrong_magic() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"EVIL\x03\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")
        .unwrap();
    f.flush().unwrap();
    let err = GgufLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop());
    assert!(err.is_err(), "Wrong magic bytes should produce an error");
}

#[test]
fn test_gguf_v1_v2_v3_all_accepted() {
    for version in [1u32, 2, 3] {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"GGUF").unwrap();
        f.write_all(&version.to_le_bytes()).unwrap();
        if version >= 2 {
            f.write_all(&0u64.to_le_bytes()).unwrap();
            f.write_all(&0u64.to_le_bytes()).unwrap();
        } else {
            f.write_all(&0u32.to_le_bytes()).unwrap();
            f.write_all(&0u32.to_le_bytes()).unwrap();
        }
        f.flush().unwrap();
        let result = GgufLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop());
        assert!(
            result.is_ok(),
            "GGUF v{} should be accepted: {:?}",
            version,
            result
        );
    }
}

#[test]
fn test_gguf_reject_unsupported_version() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"GGUF").unwrap();
    f.write_all(&99u32.to_le_bytes()).unwrap();
    f.write_all(&0u64.to_le_bytes()).unwrap();
    f.write_all(&0u64.to_le_bytes()).unwrap();
    f.flush().unwrap();
    let err = GgufLoader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop());
    assert!(err.is_err());
}
