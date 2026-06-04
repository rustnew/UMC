use std::io::Write;
use tempfile::NamedTempFile;
use umc_core::{FormatLoader, LoadOptions, ProgressCallback};
use umc_formats::GgufLoader;

fn write_gguf_v3_with_tensors() -> NamedTempFile {
    use umc_formats::gguf_test_helpers::write_gguf_with_f32_tensors;
    write_gguf_with_f32_tensors(&[
        ("token_embd.weight", vec![32, 8]),
        ("blk.0.attn_q.weight", vec![8, 8]),
        ("blk.0.attn_k.weight", vec![8, 8]),
        ("output.weight", vec![32, 8]),
    ])
}

fn write_minimal_gguf() -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"GGUF").unwrap();
    f.write_all(&3u32.to_le_bytes()).unwrap();
    f.write_all(&0u64.to_le_bytes()).unwrap();
    f.write_all(&0u64.to_le_bytes()).unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn test_gguf_load_empty_model() {
    let f = write_minimal_gguf();
    let loader = GgufLoader;
    let ir = loader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    assert_eq!(ir.tensors.len(), 0);
    assert!(ir.provenance.verify());
}

#[test]
fn test_gguf_load_metadata() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"GGUF").unwrap();
    f.write_all(&3u32.to_le_bytes()).unwrap();
    f.write_all(&0u64.to_le_bytes()).unwrap(); // tensor_count
    f.write_all(&1u64.to_le_bytes()).unwrap(); // kv_count = 1

    let key = b"general.architecture";
    f.write_all(&(key.len() as u64).to_le_bytes()).unwrap();
    f.write_all(key).unwrap();
    f.write_all(&8u32.to_le_bytes()).unwrap(); // String type
    let val = b"phi";
    f.write_all(&(val.len() as u64).to_le_bytes()).unwrap();
    f.write_all(val).unwrap();
    f.flush().unwrap();

    let loader = GgufLoader;
    let ir = loader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    assert_eq!(ir.architecture.architecture, "phi");
}

#[test]
fn test_gguf_metadata_only_option() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"GGUF").unwrap();
    f.write_all(&3u32.to_le_bytes()).unwrap();
    f.write_all(&0u64.to_le_bytes()).unwrap();
    f.write_all(&0u64.to_le_bytes()).unwrap();
    f.flush().unwrap();

    let loader = GgufLoader;
    let mut opts = LoadOptions::default();
    opts.metadata_only = true;
    let ir = loader.load(f.path(), &opts, &ProgressCallback::noop()).unwrap();
    assert!(ir.tensors.is_empty());
}

#[test]
fn test_gguf_reject_wrong_magic() {
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"EVIL\x03\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00").unwrap();
    f.flush().unwrap();
    let loader = GgufLoader;
    let err = loader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop());
    assert!(err.is_err());
}

#[test]
fn test_gguf_provenance_verified() {
    let f = write_minimal_gguf();
    let loader = GgufLoader;
    let ir = loader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    assert!(ir.provenance.verify(), "Provenance chain must be valid after load");
    assert_eq!(ir.provenance.len(), 1);
    assert_eq!(ir.provenance.last_entry().unwrap().source_format, "GGUF");
}

#[test]
fn test_gguf_v1_v2_v3_all_accepted() {
    for version in [1u32, 2, 3] {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"GGUF").unwrap();
        f.write_all(&version.to_le_bytes()).unwrap();
        // v1 uses u32 counts, v2+ uses u64
        if version >= 2 {
            f.write_all(&0u64.to_le_bytes()).unwrap();
            f.write_all(&0u64.to_le_bytes()).unwrap();
        } else {
            f.write_all(&0u32.to_le_bytes()).unwrap();
            f.write_all(&0u32.to_le_bytes()).unwrap();
        }
        f.flush().unwrap();
        let loader = GgufLoader;
        let result = loader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop());
        assert!(result.is_ok(), "GGUF v{} should be accepted", version);
    }
}
