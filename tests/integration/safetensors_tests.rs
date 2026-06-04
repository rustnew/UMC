use std::io::Write;
use tempfile::NamedTempFile;
use umc_core::{DType, FormatLoader, FormatSaver, LoadOptions, ProgressCallback, SaveOptions, Tensor, UniversalIR};
use umc_formats::{SafeTensorsLoader, SafeTensorsSaver};
use std::path::Path;

fn make_st_file(tensors: &[(&str, &str, Vec<usize>, Vec<f32>)]) -> NamedTempFile {
    let mut header_map = serde_json::Map::new();
    let mut data_parts: Vec<Vec<u8>> = Vec::new();
    let mut offset: u64 = 0;

    for (name, dtype, shape, values) in tensors {
        let bytes: Vec<u8> = values.iter().flat_map(|f| f.to_le_bytes()).collect();
        let end = offset + bytes.len() as u64;
        header_map.insert(name.to_string(), serde_json::json!({
            "dtype": dtype,
            "shape": shape,
            "data_offsets": [offset, end],
        }));
        offset = end;
        data_parts.push(bytes);
    }

    let json = serde_json::to_string(&serde_json::Value::Object(header_map)).unwrap();
    let json_bytes = json.as_bytes();

    let mut f = NamedTempFile::new().unwrap();
    f.write_all(&(json_bytes.len() as u64).to_le_bytes()).unwrap();
    f.write_all(json_bytes).unwrap();
    for part in &data_parts {
        f.write_all(part).unwrap();
    }
    f.flush().unwrap();
    f
}

#[test]
fn test_safetensors_load_single_tensor() {
    let f = make_st_file(&[("weight", "F32", vec![2, 2], vec![1.0, 2.0, 3.0, 4.0])]);
    let loader = SafeTensorsLoader;
    let ir = loader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    assert_eq!(ir.tensors.len(), 1);
    let t = ir.tensors.get("weight").unwrap();
    assert_eq!(t.dtype, DType::F32);
    assert_eq!(t.shape, vec![2, 2]);
    assert_eq!(t.num_elements(), 4);
}

#[test]
fn test_safetensors_load_multiple_tensors() {
    let f = make_st_file(&[
        ("embed", "F32", vec![100, 8], vec![0.0f32; 800]),
        ("norm",  "F32", vec![8],      vec![1.0f32; 8]),
    ]);
    let loader = SafeTensorsLoader;
    let ir = loader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    assert_eq!(ir.tensors.len(), 2);
}

#[test]
fn test_safetensors_load_bf16_tensor() {
    let bf16_val: u16 = 0x3F80; // 1.0 in BF16
    let bytes: Vec<u8> = vec![bf16_val, bf16_val, bf16_val, bf16_val]
        .iter().flat_map(|v: &u16| v.to_le_bytes()).collect();
    let header = serde_json::json!({
        "w": { "dtype": "BF16", "shape": [4], "data_offsets": [0, 8] }
    });
    let header_str = header.to_string();
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(&(header_str.len() as u64).to_le_bytes()).unwrap();
    f.write_all(header_str.as_bytes()).unwrap();
    f.write_all(&bytes).unwrap();
    f.flush().unwrap();

    let loader = SafeTensorsLoader;
    let ir = loader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    assert_eq!(ir.tensors.get("w").unwrap().dtype, DType::BF16);
}

#[test]
fn test_safetensors_save_and_reload() {
    let mut ir = UniversalIR::new("GGUF", Path::new("test.gguf"));
    let data: Vec<u8> = vec![1.0f32, 2.0, 3.0, 4.0]
        .iter().flat_map(|f: &f32| f.to_le_bytes()).collect();
    ir.tensors.insert(Tensor::from_bytes("embed.weight", DType::F32, vec![2, 2], data)).unwrap();

    let out = NamedTempFile::new().unwrap();
    let saver = SafeTensorsSaver;
    saver.save(&ir, out.path(), &SaveOptions::default(), &ProgressCallback::noop()).unwrap();

    let loader = SafeTensorsLoader;
    let ir2 = loader.load(out.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
    assert_eq!(ir2.tensors.len(), 1);
    let t = ir2.tensors.get("embed.weight").unwrap();
    assert_eq!(t.dtype, DType::F32);
    assert_eq!(t.shape, vec![2, 2]);

    // Verify the values are preserved
    let bytes = t.data.as_bytes().unwrap();
    let vals: Vec<f32> = bytes.chunks(4)
        .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
        .collect();
    assert_eq!(vals, vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_safetensors_save_atomic_rename() {
    // Verify the file either fully exists or does not exist
    let mut ir = UniversalIR::new("GGUF", Path::new("test.gguf"));
    ir.tensors.insert(Tensor::from_bytes("w", DType::F32, vec![1], vec![0u8; 4])).unwrap();

    let out = NamedTempFile::new().unwrap();
    let saver = SafeTensorsSaver;
    saver.save(&ir, out.path(), &SaveOptions::default(), &ProgressCallback::noop()).unwrap();
    assert!(out.path().exists());

    // Verify no temp files left behind
    let parent = out.path().parent().unwrap();
    let tmp_count = std::fs::read_dir(parent).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with(".umc_tmp_"))
        .count();
    assert_eq!(tmp_count, 0, "No temp files should remain after successful save");
}
