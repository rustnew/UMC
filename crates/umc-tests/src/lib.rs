// Integration test helpers and shared fixtures.

/// Write a minimal GGUF v3 file with zero tensors and zero metadata.
pub fn write_minimal_gguf() -> tempfile::NamedTempFile {
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::with_suffix(".gguf").unwrap();
    f.write_all(b"GGUF").unwrap();
    f.write_all(&3u32.to_le_bytes()).unwrap();
    f.write_all(&0u64.to_le_bytes()).unwrap(); // tensor_count
    f.write_all(&0u64.to_le_bytes()).unwrap(); // metadata_kv_count
    f.flush().unwrap();
    f
}

/// Write a GGUF v3 file with F32 tensors (for round-trip tests).
pub fn write_gguf_v3_with_f32_tensors(
    tensors: &[(&str, Vec<usize>, Vec<f32>)],
) -> tempfile::NamedTempFile {
    use std::io::Write;

    let mut f = tempfile::NamedTempFile::with_suffix(".gguf").unwrap();
    f.write_all(b"GGUF").unwrap();
    f.write_all(&3u32.to_le_bytes()).unwrap();
    // tensor_count (u64)
    f.write_all(&(tensors.len() as u64).to_le_bytes()).unwrap();
    // metadata_kv_count (u64)
    f.write_all(&0u64.to_le_bytes()).unwrap();

    // Alignment = 32 bytes
    let alignment: u64 = 32;

    // Compute tensor data layout
    let mut tensor_data_parts: Vec<Vec<u8>> = Vec::new();
    let mut running_offset: u64 = 0;

    // Write tensor info section
    for (name, shape, values) in tensors {
        // name
        let name_bytes = name.as_bytes();
        f.write_all(&(name_bytes.len() as u64).to_le_bytes())
            .unwrap();
        f.write_all(name_bytes).unwrap();
        // n_dims
        f.write_all(&(shape.len() as u32).to_le_bytes()).unwrap();
        // dims in reverse order (GGUF stores innermost first)
        for &dim in shape.iter().rev() {
            f.write_all(&(dim as u64).to_le_bytes()).unwrap();
        }
        // ggml_type = F32 = 0
        f.write_all(&0u32.to_le_bytes()).unwrap();
        // offset relative to data segment
        f.write_all(&running_offset.to_le_bytes()).unwrap();

        let bytes: Vec<u8> = values.iter().flat_map(|v: &f32| v.to_le_bytes()).collect();
        running_offset += bytes.len() as u64;
        tensor_data_parts.push(bytes);
    }

    // Pad to alignment
    let header_end = f.as_file().metadata().unwrap().len();
    let aligned_start = (header_end + alignment - 1) / alignment * alignment;
    let pad = (aligned_start - header_end) as usize;
    if pad > 0 {
        f.write_all(&vec![0u8; pad]).unwrap();
    }

    // Write tensor data
    for part in &tensor_data_parts {
        f.write_all(part).unwrap();
    }
    f.flush().unwrap();
    f
}

/// Write a SafeTensors file from F32 tensors.
pub fn write_safetensors_f32(tensors: &[(&str, Vec<usize>, Vec<f32>)]) -> tempfile::NamedTempFile {
    use std::io::Write;

    let mut header_map = serde_json::Map::new();
    let mut data_parts: Vec<Vec<u8>> = Vec::new();
    let mut offset: u64 = 0;

    for (name, shape, values) in tensors {
        let bytes: Vec<u8> = values.iter().flat_map(|v: &f32| v.to_le_bytes()).collect();
        let end = offset + bytes.len() as u64;
        header_map.insert(
            name.to_string(),
            serde_json::json!({
                "dtype": "F32",
                "shape": shape,
                "data_offsets": [offset, end],
            }),
        );
        offset = end;
        data_parts.push(bytes);
    }

    let json = serde_json::to_string(&serde_json::Value::Object(header_map)).unwrap();
    let json_bytes = json.as_bytes();

    let mut f = tempfile::NamedTempFile::with_suffix(".safetensors").unwrap();
    f.write_all(&(json_bytes.len() as u64).to_le_bytes())
        .unwrap();
    f.write_all(json_bytes).unwrap();
    for part in &data_parts {
        f.write_all(part).unwrap();
    }
    f.flush().unwrap();
    f
}
