use std::path::Path;
use umc_core::{
    DType, FormatLoader, FormatSaver, LoadOptions, ProgressCallback, SaveOptions, Tensor,
    UniversalIR,
};
use umc_formats::{GgufLoader, SafeTensorsLoader, SafeTensorsSaver};
use umc_tests::{write_gguf_v3_with_f32_tensors, write_minimal_gguf};
use umc_validate::{numeric_validate, structural_validate};

#[test]
fn test_round_trip_gguf_to_safetensors_empty() {
    let input = write_minimal_gguf();
    let ir = GgufLoader
        .load(
            input.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = tempfile::NamedTempFile::new().unwrap();
    SafeTensorsSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let ir2 = SafeTensorsLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let report = structural_validate(&ir, &ir2).unwrap();
    assert!(
        report.passed,
        "Structural validation: {:?}",
        report.shape_mismatches
    );
}

#[test]
fn test_round_trip_gguf_to_safetensors_with_tensors() {
    let values: Vec<f32> = (0..16).map(|i| i as f32 * 0.5).collect();
    let input =
        write_gguf_v3_with_f32_tensors(&[("embed.weight", vec![4, 4], values[..16].to_vec())]);
    let ir = GgufLoader
        .load(
            input.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out = tempfile::NamedTempFile::new().unwrap();
    SafeTensorsSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let ir2 = SafeTensorsLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    // Structural check
    let struct_report = structural_validate(&ir, &ir2).unwrap();
    assert!(
        struct_report.passed,
        "Structural: {:?}",
        struct_report.shape_mismatches
    );

    // Numeric check — F32 round-trip must be bit-identical
    let num_report = numeric_validate(&ir, &ir2, Some(0.0)).unwrap();
    assert!(
        num_report.passed,
        "F32 round-trip divergence: {:.2e}",
        num_report.global_max_divergence
    );
    assert_eq!(
        num_report.global_max_divergence, 0.0,
        "F32 must be bit-identical"
    );
}

#[test]
fn test_round_trip_safetensors_to_safetensors() {
    let mut ir = UniversalIR::new("TEST", Path::new("x.bin"));
    for i in 0..5 {
        let data: Vec<u8> = (0..16u8)
            .map(|j| j + i)
            .flat_map(|v| (v as f32).to_le_bytes())
            .collect();
        ir.tensors
            .insert(Tensor::from_bytes(
                &format!("layer.{}.weight", i),
                DType::F32,
                vec![4, 4],
                data,
            ))
            .unwrap();
    }

    let out1 = tempfile::NamedTempFile::new().unwrap();
    SafeTensorsSaver
        .save(
            &ir,
            out1.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let ir2 = SafeTensorsLoader
        .load(
            out1.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let out2 = tempfile::NamedTempFile::new().unwrap();
    SafeTensorsSaver
        .save(
            &ir2,
            out2.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let ir3 = SafeTensorsLoader
        .load(
            out2.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let report = numeric_validate(&ir, &ir3, Some(0.0)).unwrap();
    assert!(
        report.passed,
        "Double round-trip must be bit-identical: {:.2e}",
        report.global_max_divergence
    );
}

#[test]
fn test_round_trip_large_model_simulation() {
    // Simulate a small model with realistic tensor naming
    let tensors: Vec<(&str, Vec<usize>, Vec<f32>)> = vec![
        (
            "model.embed_tokens.weight",
            vec![100, 32],
            vec![0.01f32; 3200],
        ),
        (
            "model.layers.0.self_attn.q_proj.weight",
            vec![32, 32],
            vec![0.1f32; 1024],
        ),
        (
            "model.layers.0.self_attn.k_proj.weight",
            vec![32, 32],
            vec![0.2f32; 1024],
        ),
        (
            "model.layers.0.self_attn.v_proj.weight",
            vec![32, 32],
            vec![0.3f32; 1024],
        ),
        (
            "model.layers.0.mlp.gate_proj.weight",
            vec![64, 32],
            vec![0.4f32; 2048],
        ),
        ("model.norm.weight", vec![32], vec![1.0f32; 32]),
        ("lm_head.weight", vec![100, 32], vec![0.01f32; 3200]),
    ];

    let mut ir = UniversalIR::new("TEST", Path::new("model.bin"));
    for (name, shape, vals) in &tensors {
        let data: Vec<u8> = vals.iter().flat_map(|f: &f32| f.to_le_bytes()).collect();
        ir.tensors
            .insert(Tensor::from_bytes(*name, DType::F32, shape.clone(), data))
            .unwrap();
    }

    let out = tempfile::NamedTempFile::new().unwrap();
    SafeTensorsSaver
        .save(
            &ir,
            out.path(),
            &SaveOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();
    let ir2 = SafeTensorsLoader
        .load(
            out.path(),
            &LoadOptions::default(),
            &ProgressCallback::noop(),
        )
        .unwrap();

    let struct_report = structural_validate(&ir, &ir2).unwrap();
    assert!(struct_report.passed);
    assert_eq!(struct_report.tensor_count_before, tensors.len());
    assert_eq!(struct_report.tensor_count_after, tensors.len());

    let num_report = numeric_validate(&ir, &ir2, Some(0.0)).unwrap();
    assert_eq!(num_report.global_max_divergence, 0.0);
}
