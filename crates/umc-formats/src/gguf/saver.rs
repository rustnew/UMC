use super::spec::{GgmlType, GgufMetaValueType, GGUF_MAGIC};
use std::io::{BufWriter, Write};
/// GGUF v3 saver — writes UniversalIR back to GGUF format.
///
/// Compliance with regles.md:
/// - Bit-identical round-trip when raw metadata is in ExtensionStore.
/// - Sémantique round-trip otherwise (same data, inferred types).
/// - Zero-copy tensor data: bytes written directly from TensorData.
/// - 32-byte data segment alignment (default; overridable via general.alignment).
use std::path::Path;
use umc_core::{
    DType, FormatSaver, MetaValue, ProgressCallback, SaveOptions, UmcError, UniversalIR,
};

pub struct GgufSaver;

impl FormatSaver for GgufSaver {
    fn format_name(&self) -> &'static str {
        "GGUF"
    }

    fn default_extension(&self) -> &'static str {
        "gguf"
    }

    fn save(
        &self,
        ir: &UniversalIR,
        path: &Path,
        _opts: &SaveOptions,
        progress: &ProgressCallback,
    ) -> Result<(), UmcError> {
        progress.report("Writing GGUF v3…");

        let file = std::fs::File::create(path).map_err(UmcError::Io)?;
        let mut w = BufWriter::with_capacity(8 * 1024 * 1024, file);

        // ── 1. Collect metadata KV pairs ─────────────────────────────────
        // Check if we have raw metadata bytes in ExtensionStore (from a previous GGUF load)
        // for bit-identical round-trip.
        let raw_meta: Option<Vec<u8>> = ir
            .extensions
            .get("GGUF@v3/metadata_kv_raw")
            .map(|b| b.to_vec());

        // Collect tensor entries in insertion order
        let tensors: Vec<_> = ir.tensors.iter().collect();
        let tensor_count = tensors.len() as u64;

        // ── 2. Determine alignment ────────────────────────────────────────
        let alignment: u64 = ir
            .metadata
            .get_i64("general.alignment")
            .map(|v| v as u64)
            .unwrap_or(32);

        // ── 3. Compute tensor data offsets ────────────────────────────────
        // Offsets are relative to the data segment start. Each tensor is aligned.
        let mut offsets: Vec<u64> = Vec::with_capacity(tensors.len());
        let mut cumulative: u64 = 0;
        for (_, t) in &tensors {
            offsets.push(cumulative);
            let byte_size = t.data.len() as u64;
            // Align to next boundary
            cumulative = (cumulative + byte_size + alignment - 1) / alignment * alignment;
        }

        // ── 4. Serialize metadata KV bytes (used below) ───────────────────
        let (metadata_kv_bytes, metadata_kv_count): (Vec<u8>, u64) = if let Some(raw) = raw_meta {
            // Read count prefix from raw bytes (first 8 bytes = count, then pairs)
            // Actually, we stored count separately; raw is just the KV pairs.
            let count = ir.metadata.len() as u64;
            (raw, count)
        } else {
            let mut kv_buf: Vec<u8> = Vec::new();
            let count = serialize_metadata(&ir.metadata, &mut kv_buf)?;
            (kv_buf, count)
        };

        // ── 6. Write header ───────────────────────────────────────────────
        w.write_all(GGUF_MAGIC).map_err(UmcError::Io)?;
        w.write_all(&3u32.to_le_bytes()).map_err(UmcError::Io)?;
        w.write_all(&tensor_count.to_le_bytes())
            .map_err(UmcError::Io)?;
        w.write_all(&metadata_kv_count.to_le_bytes())
            .map_err(UmcError::Io)?;

        // ── 7. Write metadata KV pairs ────────────────────────────────────
        w.write_all(&metadata_kv_bytes).map_err(UmcError::Io)?;

        // ── 8. Write tensor info headers (with correct offsets) ───────────
        for (idx, (name, t)) in tensors.iter().enumerate() {
            write_tensor_info_with_offset(&mut w, name.as_str(), t, offsets[idx])?;
        }

        // ── 9. Alignment padding after header ─────────────────────────────
        let pos_after_header = {
            // Estimate position: magic(4)+version(4)+tensor_count(8)+metadata_kv_count(8)
            // + metadata_kv_bytes.len() + tensor_info_total_bytes
            let ti_total: usize = tensors
                .iter()
                .map(|(name, t)| {
                    8 + name.len()           // name_len(u64) + name
                + 4                       // n_dims (u32)
                + t.shape.len() * 8       // shape dims (each u64)
                + 4                       // ggml_type (u32)
                + 8 // offset (u64)
                })
                .sum();
            4 + 4 + 8 + 8 + metadata_kv_bytes.len() + ti_total
        };
        let pad_needed = ((pos_after_header as u64 + alignment - 1) / alignment * alignment)
            as usize
            - pos_after_header;
        let zeroes = vec![0u8; pad_needed];
        w.write_all(&zeroes).map_err(UmcError::Io)?;

        // ── 10. Write tensor data ─────────────────────────────────────────
        progress.set_total(tensor_count);
        let mut written_bytes: u64 = 0;
        for (idx, (name, t)) in tensors.iter().enumerate() {
            let bytes = t
                .data
                .as_bytes()
                .map_err(|e| UmcError::Other(format!("Cannot read tensor '{}': {}", name, e)))?;
            w.write_all(bytes).map_err(UmcError::Io)?;
            written_bytes += bytes.len() as u64;

            // Pad each tensor to alignment boundary
            let pad = ((written_bytes + alignment - 1) / alignment * alignment) - written_bytes;
            if pad > 0 && idx + 1 < tensors.len() {
                w.write_all(&vec![0u8; pad as usize])
                    .map_err(UmcError::Io)?;
                written_bytes += pad;
            }
            progress.increment(&format!("Wrote '{}'", name));
        }

        w.flush().map_err(UmcError::Io)?;
        progress.report("GGUF written successfully.");
        Ok(())
    }
}

// ── Metadata serialization ────────────────────────────────────────────────────

fn serialize_metadata(meta: &umc_core::MetadataStore, out: &mut Vec<u8>) -> Result<u64, UmcError> {
    let mut count = 0u64;
    for (key, value) in meta.iter() {
        write_gguf_string(out, key.as_bytes());
        serialize_meta_value(out, value);
        count += 1;
    }
    Ok(count)
}

fn write_gguf_string(out: &mut Vec<u8>, s: &[u8]) {
    out.extend_from_slice(&(s.len() as u64).to_le_bytes());
    out.extend_from_slice(s);
}

fn serialize_meta_value(out: &mut Vec<u8>, v: &MetaValue) {
    match v {
        MetaValue::I64(n) => {
            if *n >= 0 && *n <= u32::MAX as i64 {
                out.extend_from_slice(&(GgufMetaValueType::Uint32 as u32).to_le_bytes());
                out.extend_from_slice(&(*n as u32).to_le_bytes());
            } else if *n >= i32::MIN as i64 && *n <= i32::MAX as i64 {
                out.extend_from_slice(&(GgufMetaValueType::Int32 as u32).to_le_bytes());
                out.extend_from_slice(&(*n as i32).to_le_bytes());
            } else {
                out.extend_from_slice(&(GgufMetaValueType::Int64 as u32).to_le_bytes());
                out.extend_from_slice(&n.to_le_bytes());
            }
        }
        MetaValue::F64(f) => {
            // Prefer Float32 if no precision loss
            let f32v = *f as f32;
            if (f32v as f64 - f).abs() < 1e-7 * f.abs().max(1.0) {
                out.extend_from_slice(&(GgufMetaValueType::Float32 as u32).to_le_bytes());
                out.extend_from_slice(&f32v.to_le_bytes());
            } else {
                out.extend_from_slice(&(GgufMetaValueType::Float64 as u32).to_le_bytes());
                out.extend_from_slice(&f.to_le_bytes());
            }
        }
        MetaValue::Bool(b) => {
            out.extend_from_slice(&(GgufMetaValueType::Bool as u32).to_le_bytes());
            out.push(*b as u8);
        }
        MetaValue::String(s) => {
            out.extend_from_slice(&(GgufMetaValueType::String as u32).to_le_bytes());
            write_gguf_string(out, s.as_bytes());
        }
        MetaValue::Array(arr) => {
            out.extend_from_slice(&(GgufMetaValueType::Array as u32).to_le_bytes());
            let elem_type = infer_array_elem_type(arr);
            out.extend_from_slice(&(elem_type as u32).to_le_bytes());
            out.extend_from_slice(&(arr.len() as u64).to_le_bytes());
            for elem in arr {
                serialize_meta_value_untyped(out, elem, elem_type);
            }
        }
        MetaValue::Raw(_) => {
            // Cannot represent raw bytes in GGUF metadata — skip by writing as empty string
            out.extend_from_slice(&(GgufMetaValueType::String as u32).to_le_bytes());
            out.extend_from_slice(&0u64.to_le_bytes());
        }
    }
}

fn infer_array_elem_type(arr: &[MetaValue]) -> GgufMetaValueType {
    for v in arr {
        match v {
            MetaValue::String(_) => return GgufMetaValueType::String,
            MetaValue::F64(_) => return GgufMetaValueType::Float32,
            MetaValue::Bool(_) => return GgufMetaValueType::Bool,
            _ => {}
        }
    }
    GgufMetaValueType::Uint32
}

fn serialize_meta_value_untyped(out: &mut Vec<u8>, v: &MetaValue, _expected: GgufMetaValueType) {
    // Write just the data bytes (no type prefix) for array elements
    match v {
        MetaValue::I64(n) => {
            out.extend_from_slice(&(*n as u32).to_le_bytes());
        }
        MetaValue::F64(f) => {
            out.extend_from_slice(&(*f as f32).to_le_bytes());
        }
        MetaValue::Bool(b) => {
            out.push(*b as u8);
        }
        MetaValue::String(s) => {
            write_gguf_string(out, s.as_bytes());
        }
        _ => {}
    }
}

// ── Tensor info ───────────────────────────────────────────────────────────────

fn write_tensor_info_with_offset<W: Write>(
    w: &mut W,
    name: &str,
    t: &umc_core::Tensor,
    offset: u64,
) -> Result<(), UmcError> {
    // Name
    w.write_all(&(name.len() as u64).to_le_bytes())
        .map_err(UmcError::Io)?;
    w.write_all(name.as_bytes()).map_err(UmcError::Io)?;

    // n_dims
    let n_dims = t.shape.len() as u32;
    w.write_all(&n_dims.to_le_bytes()).map_err(UmcError::Io)?;

    // Shape in GGUF order (innermost first = reverse of our order)
    for &dim in t.shape.iter().rev() {
        w.write_all(&(dim as u64).to_le_bytes())
            .map_err(UmcError::Io)?;
    }

    // GgmlType
    let ggml = dtype_to_ggml_type(&t.dtype).ok_or_else(|| {
        UmcError::Other(format!(
            "Cannot map DType {:?} to GGML type for tensor '{}'",
            t.dtype, name
        ))
    })?;
    w.write_all(&(ggml as u32).to_le_bytes())
        .map_err(UmcError::Io)?;

    // Data offset (relative to data segment start)
    w.write_all(&offset.to_le_bytes()).map_err(UmcError::Io)?;

    Ok(())
}

// ── DType → GgmlType reverse mapping ─────────────────────────────────────────

pub fn dtype_to_ggml_type(dtype: &DType) -> Option<GgmlType> {
    match dtype {
        DType::F32 => Some(GgmlType::F32),
        DType::F16 => Some(GgmlType::F16),
        DType::BF16 => Some(GgmlType::BF16),
        DType::Q4_0 => Some(GgmlType::Q4_0),
        DType::Q4_1 => Some(GgmlType::Q4_1),
        DType::Q5_0 => Some(GgmlType::Q5_0),
        DType::Q5_1 => Some(GgmlType::Q5_1),
        DType::Q8_0 => Some(GgmlType::Q8_0),
        DType::Q2K => Some(GgmlType::Q2K),
        DType::Q3KS => Some(GgmlType::Q3KS),
        DType::Q3KM => Some(GgmlType::Q3KM),
        DType::Q3KL => Some(GgmlType::Q3KL),
        DType::Q4KS => Some(GgmlType::Q4KS),
        DType::Q4KM => Some(GgmlType::Q4KM),
        DType::Q5KS => Some(GgmlType::Q5KS),
        DType::Q5KM => Some(GgmlType::Q5KM),
        DType::Q6K => Some(GgmlType::Q6K),
        DType::Q8K => Some(GgmlType::Q8K),
        DType::I8 => Some(GgmlType::I8),
        DType::I16 => Some(GgmlType::I16),
        DType::I32 => Some(GgmlType::I32),
        DType::I64 => Some(GgmlType::I64),
        DType::F64 => Some(GgmlType::F64),
        DType::Custom(s) => match s.as_str() {
            "IQ2_XXS" => Some(GgmlType::IQ2XXS),
            "IQ2_XS" => Some(GgmlType::IQ2XS),
            "IQ2_S" => Some(GgmlType::IQ2S),
            "IQ3_XXS" => Some(GgmlType::IQ3XXS),
            "IQ3_S" => Some(GgmlType::IQ3S),
            "IQ1_S" => Some(GgmlType::IQ1S),
            "IQ1_M" => Some(GgmlType::IQ1M),
            "IQ4_NL" => Some(GgmlType::IQ4NL),
            "IQ4_XS" => Some(GgmlType::IQ4XS),
            _ => None,
        },
        _ => None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gguf::reader::GgufLoader;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use umc_core::{DType, FormatLoader, LoadOptions, ProgressCallback};

    fn write_minimal_gguf_v3() -> NamedTempFile {
        let mut f = NamedTempFile::with_suffix(".gguf").unwrap();
        f.write_all(b"GGUF").unwrap();
        f.write_all(&3u32.to_le_bytes()).unwrap();
        f.write_all(&0u64.to_le_bytes()).unwrap(); // tensor_count
        f.write_all(&0u64.to_le_bytes()).unwrap(); // metadata_kv_count
        f.flush().unwrap();
        f
    }

    fn write_gguf_v3_with_metadata_and_tensor() -> NamedTempFile {
        let mut f = NamedTempFile::with_suffix(".gguf").unwrap();
        f.write_all(b"GGUF").unwrap();
        f.write_all(&3u32.to_le_bytes()).unwrap();
        f.write_all(&1u64.to_le_bytes()).unwrap(); // tensor_count = 1
        f.write_all(&2u64.to_le_bytes()).unwrap(); // metadata_kv_count = 2

        // Key: "general.architecture" = "phi"
        let key1 = b"general.architecture";
        f.write_all(&(key1.len() as u64).to_le_bytes()).unwrap();
        f.write_all(key1).unwrap();
        f.write_all(&8u32.to_le_bytes()).unwrap(); // String type
        let val1 = b"phi";
        f.write_all(&(val1.len() as u64).to_le_bytes()).unwrap();
        f.write_all(val1).unwrap();

        // Key: "phi.block_count" = 2 (Uint32)
        let key2 = b"phi.block_count";
        f.write_all(&(key2.len() as u64).to_le_bytes()).unwrap();
        f.write_all(key2).unwrap();
        f.write_all(&4u32.to_le_bytes()).unwrap(); // Uint32 type
        f.write_all(&2u32.to_le_bytes()).unwrap();

        // Tensor info: "weight" shape=[4,4] F32 offset=0
        let tname = b"weight";
        f.write_all(&(tname.len() as u64).to_le_bytes()).unwrap();
        f.write_all(tname).unwrap();
        f.write_all(&2u32.to_le_bytes()).unwrap(); // n_dims = 2
                                                   // shape: GGUF stores innermost first (reversed), so [4,4] → [4,4]
        f.write_all(&4u64.to_le_bytes()).unwrap();
        f.write_all(&4u64.to_le_bytes()).unwrap();
        f.write_all(&0u32.to_le_bytes()).unwrap(); // F32
        f.write_all(&0u64.to_le_bytes()).unwrap(); // offset=0

        // Alignment padding (32 bytes default)
        let header_end = f.as_file().metadata().unwrap().len() as usize;
        let aligned = (header_end + 31) / 32 * 32;
        let pad = aligned - header_end;
        f.write_all(&vec![0u8; pad]).unwrap();

        // Tensor data: 4×4 F32 = 64 bytes
        let data: Vec<f32> = (0..16).map(|i| i as f32).collect();
        for v in &data {
            f.write_all(&v.to_le_bytes()).unwrap();
        }
        f.flush().unwrap();
        f
    }

    #[test]
    fn test_save_empty_gguf() {
        let src = write_minimal_gguf_v3();
        let loader = GgufLoader;
        let ir = loader
            .load(
                src.path(),
                &LoadOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();

        let out = NamedTempFile::with_suffix(".gguf").unwrap();
        let saver = GgufSaver;
        saver
            .save(
                &ir,
                out.path(),
                &SaveOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();

        // Can be reloaded
        let ir2 = loader
            .load(
                out.path(),
                &LoadOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();
        assert_eq!(ir2.tensors.len(), 0);
    }

    #[test]
    fn test_save_and_reload_with_metadata() {
        let src = write_gguf_v3_with_metadata_and_tensor();
        let loader = GgufLoader;
        let ir = loader
            .load(
                src.path(),
                &LoadOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();
        assert_eq!(ir.tensors.len(), 1);
        assert_eq!(ir.metadata.get_str("general.architecture"), Some("phi"));

        let out = NamedTempFile::with_suffix(".gguf").unwrap();
        let saver = GgufSaver;
        saver
            .save(
                &ir,
                out.path(),
                &SaveOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();

        let ir2 = loader
            .load(
                out.path(),
                &LoadOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();
        assert_eq!(ir2.tensors.len(), 1);
        assert_eq!(ir2.metadata.get_str("general.architecture"), Some("phi"));
        assert_eq!(ir2.metadata.get_i64("phi.block_count"), Some(2));
        assert_eq!(ir2.architecture.architecture, "phi");
        assert_eq!(ir2.architecture.num_layers, 2);
    }

    #[test]
    fn test_tensor_data_preserved() {
        let src = write_gguf_v3_with_metadata_and_tensor();
        let loader = GgufLoader;
        let ir = loader
            .load(
                src.path(),
                &LoadOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();

        let out = NamedTempFile::with_suffix(".gguf").unwrap();
        GgufSaver
            .save(
                &ir,
                out.path(),
                &SaveOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();

        let ir2 = loader
            .load(
                out.path(),
                &LoadOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();
        let t2 = ir2
            .tensors
            .get("weight")
            .expect("tensor 'weight' not found");

        let bytes = t2.data.as_bytes().unwrap();
        let floats: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        assert_eq!(floats.len(), 16);
        for (i, v) in floats.iter().enumerate() {
            assert!(
                (v - i as f32).abs() < 1e-6,
                "float[{}] = {} expected {}",
                i,
                v,
                i
            );
        }
    }

    #[test]
    fn test_dtype_to_ggml_round_trip() {
        let types = [
            DType::F32,
            DType::F16,
            DType::BF16,
            DType::Q4KM,
            DType::Q5KM,
            DType::Q8_0,
            DType::Q2K,
            DType::Q6K,
        ];
        for dt in &types {
            let g = dtype_to_ggml_type(dt);
            assert!(g.is_some(), "dtype_to_ggml_type({:?}) returned None", dt);
        }
    }
}
