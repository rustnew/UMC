/// Round-trip tests — GGUF → SafeTensors → compare.

use std::io::Write;
use tempfile::NamedTempFile;
use umc_core::{DType, FormatLoader, FormatSaver, LoadOptions, ProgressCallback, SaveOptions, Tensor, UniversalIR};
use umc_formats::{GgufLoader, SafeTensorsLoader, SafeTensorsSaver};
use umc_validate::{structural_validate, numeric_validate};
use std::path::Path;

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
fn test_gguf_to_safetensors_empty_model() {
    let gguf = write_minimal_gguf();
    let loader = GgufLoader;
    let ir = loader.load(gguf.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();

    let out = NamedTempFile::new().unwrap();
    let saver = SafeTensorsSaver;
    saver.save(&ir, out.path(), &SaveOptions::default(), &ProgressCallback::noop()).unwrap();

    // Reload and validate
    let st_loader = SafeTensorsLoader;
    let ir2 = st_loader.load(out.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    let report = structural_validate(&ir, &ir2).unwrap();
    assert!(report.passed, "Structural validation must pass: {:?}", report.shape_mismatches);
}

#[test]
fn test_safetensors_round_trip_f32_values() {
    // Create IR with known F32 values
    let mut ir = UniversalIR::new("test", Path::new("x.bin"));
    let values = vec![1.5f32, -2.5, 0.0, 100.0, -0.001, 999.9];
    let data: Vec<u8> = values.iter().flat_map(|f: &f32| f.to_le_bytes()).collect();
    ir.tensors.insert(Tensor::from_bytes("weight", DType::F32, vec![values.len()], data)).unwrap();

    // Save to SafeTensors
    let out = NamedTempFile::new().unwrap();
    let saver = SafeTensorsSaver;
    saver.save(&ir, out.path(), &SaveOptions::default(), &ProgressCallback::noop()).unwrap();

    // Reload
    let loader = SafeTensorsLoader;
    let ir2 = loader.load(out.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();

    // Numeric validation
    let report = numeric_validate(&ir, &ir2, None).unwrap();
    assert!(report.passed, "Numeric validation failed: max divergence = {:.2e}", report.global_max_divergence);
    assert_eq!(report.global_max_divergence, 0.0, "F32 round-trip must be bit-identical");
}

#[test]
fn test_structural_validate_passes_for_identical_irs() {
    let mut ir = UniversalIR::new("TEST", Path::new("x.bin"));
    for i in 0..5 {
        ir.tensors.insert(Tensor::from_bytes(
            &format!("layer_{}", i),
            DType::F32,
            vec![4, 4],
            vec![0u8; 64],
        )).unwrap();
    }

    let report = structural_validate(&ir, &ir).unwrap();
    assert!(report.passed);
    assert_eq!(report.tensor_count_before, 5);
    assert_eq!(report.tensor_count_after, 5);
    assert!(report.shape_mismatches.is_empty());
}

#[test]
fn test_round_trip_multiple_tensors() {
    let mut ir = UniversalIR::new("TEST", Path::new("x.bin"));
    let tensors = [
        ("embed.weight",  vec![32usize, 8usize]),
        ("norm.weight",   vec![8]),
        ("lm_head.weight", vec![32, 8]),
    ];
    for (name, shape) in &tensors {
        let n: usize = shape.iter().product();
        let data: Vec<u8> = (0..n).flat_map(|i| (i as f32).to_le_bytes()).collect();
        ir.tensors.insert(Tensor::from_bytes(*name, DType::F32, shape.clone(), data)).unwrap();
    }

    let out = NamedTempFile::new().unwrap();
    SafeTensorsSaver.save(&ir, out.path(), &SaveOptions::default(), &ProgressCallback::noop()).unwrap();
    let ir2 = SafeTensorsLoader.load(out.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();

    let struct_report = structural_validate(&ir, &ir2).unwrap();
    assert!(struct_report.passed, "Structural: {:?}", struct_report.shape_mismatches);

    let num_report = numeric_validate(&ir, &ir2, Some(0.0)).unwrap();
    assert!(num_report.passed, "Numeric: max error = {:.2e}", num_report.global_max_divergence);
}
