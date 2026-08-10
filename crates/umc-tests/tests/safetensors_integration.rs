use std::path::Path;
use umc_core::{
    DType, FormatLoader, FormatSaver, LoadOptions, ProgressCallback, SaveOptions, Tensor,
    UniversalIR,
};
use umc_formats::{SafeTensorsLoader, SafeTensorsSaver};
use umc_tests::write_safetensors_f32;

#[test]
fn test_safetensors_load_f32_tensor() {
    let f = write_safetensors_f32(&[("weight", vec![2, 2], vec![1.0f32, 2.0, 3.0, 4.0])]);
    let ir = SafeTensorsLoader
        .load(f.path(), &LoadOptions::default(), &ProgressCallback::noop())
        .unwrap();
    assert_eq!(ir.tensors.len(), 1);
    let t = ir.tensors.get("weight").unwrap();
    assert_eq!(t.dtype, DType::F32);
    assert_eq!(t.shape, vec![2, 2]);
    assert_eq!(t.num_elements(), 4);
}

#[test]
fn test_safetensors_save_and_reload_preserves_values() {
    let values = vec![1.5f32, -2.5, 0.0, 100.0, -0.001, 999.9];
    let mut ir = UniversalIR::new("TEST", Path::new("test.bin"));
    let data: Vec<u8> = values.iter().flat_map(|f: &f32| f.to_le_bytes()).collect();
    ir.tensors
        .insert(Tensor::from_bytes(
            "weight",
            DType::F32,
            vec![values.len()],
            data,
        ))
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
    let t = ir2.tensors.get("weight").unwrap();
    let loaded: Vec<f32> = t
        .data
        .as_bytes()
        .unwrap()
        .chunks(4)
        .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
        .collect();
    assert_eq!(loaded, values, "F32 round-trip must be bit-identical");
}

#[test]
fn test_safetensors_save_atomic_rename_leaves_no_temp_files() {
    let mut ir = UniversalIR::new("TEST", Path::new("test.bin"));
    ir.tensors
        .insert(Tensor::from_bytes("w", DType::F32, vec![1], vec![0u8; 4]))
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
    assert!(out.path().exists());

    let parent = out.path().parent().unwrap();
    let tmp_count = std::fs::read_dir(parent)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with(".umc_tmp_"))
        .count();
    assert_eq!(
        tmp_count, 0,
        "No temp files should remain after successful save"
    );
}

#[test]
fn test_safetensors_multiple_dtypes() {
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::new().unwrap();
    let header = serde_json::json!({
        "f32_tensor": { "dtype": "F32", "shape": [2], "data_offsets": [0, 8] },
        "f16_tensor": { "dtype": "F16", "shape": [2], "data_offsets": [8, 12] },
    });
    let header_str = header.to_string();
    f.write_all(&(header_str.len() as u64).to_le_bytes())
        .unwrap();
    f.write_all(header_str.as_bytes()).unwrap();
    f.write_all(&[0u8; 12]).unwrap(); // 8 bytes F32 + 4 bytes F16
    f.flush().unwrap();

    let ir = SafeTensorsLoader
        .load(f.path(), &LoadOptions::default(), &ProgressCallback::noop())
        .unwrap();
    assert_eq!(ir.tensors.len(), 2);
    assert_eq!(ir.tensors.get("f32_tensor").unwrap().dtype, DType::F32);
    assert_eq!(ir.tensors.get("f16_tensor").unwrap().dtype, DType::F16);
}
