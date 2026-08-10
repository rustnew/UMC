use super::dtype_map::{ggml_row_size, ggml_type_to_dtype};
use super::spec::{GgmlType, GgufMetaValueType, GGUF_MAGIC};
use memmap2::Mmap;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use umc_core::{
    ir::provenance::ProvenanceEntryData, ir::tokenizer::TokenizerType, ArchitectureConfig,
    FormatLoader, GraphContent, LoadOptions, MetaValue, MetadataStore, ProgressCallback, Tensor,
    TokenizerStore, UmcError, UniversalIR, UMC_VERSION,
};

// ── Binary reader helper ──────────────────────────────────────────────────────

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
    version: u32,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8], version: u32) -> Self {
        Self {
            data,
            pos: 0,
            version,
        }
    }

    #[allow(dead_code)]
    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], UmcError> {
        if self.pos + n > self.data.len() {
            return Err(UmcError::UnexpectedEof {
                context: "GGUF binary read".into(),
                offset: self.pos as u64,
            });
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    fn read_u8(&mut self) -> Result<u8, UmcError> {
        Ok(self.read_bytes(1)?[0])
    }

    fn read_u16_le(&mut self) -> Result<u16, UmcError> {
        let b = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn read_u32_le(&mut self) -> Result<u32, UmcError> {
        let b = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_i32_le(&mut self) -> Result<i32, UmcError> {
        Ok(self.read_u32_le()? as i32)
    }

    fn read_u64_le(&mut self) -> Result<u64, UmcError> {
        let b = self.read_bytes(8)?;
        Ok(u64::from_le_bytes(b.try_into().map_err(|_| {
            UmcError::UnexpectedEof {
                context: "read_u64".into(),
                offset: self.pos as u64,
            }
        })?))
    }

    fn read_i64_le(&mut self) -> Result<i64, UmcError> {
        Ok(self.read_u64_le()? as i64)
    }

    fn read_f32_le(&mut self) -> Result<f32, UmcError> {
        let b = self.read_bytes(4)?;
        Ok(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn read_f64_le(&mut self) -> Result<f64, UmcError> {
        let b = self.read_bytes(8)?;
        Ok(f64::from_le_bytes(b.try_into().map_err(|_| {
            UmcError::UnexpectedEof {
                context: "read_f64".into(),
                offset: self.pos as u64,
            }
        })?))
    }

    /// Read a GGUF string: u64 length + UTF-8 bytes (v2+) or u32 + bytes (v1).
    fn read_string(&mut self) -> Result<String, UmcError> {
        let len = if self.version >= 2 {
            let v = self.read_u64_le()?;
            if v > 1_048_576 {
                return Err(UmcError::SecurityViolation {
                    field: "gguf_string_length".into(),
                    value: v as usize,
                    limit: 1_048_576,
                });
            }
            v as usize
        } else {
            self.read_u32_le()? as usize
        };
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|e| UmcError::Utf8 {
            context: "GGUF string".into(),
            msg: e.to_string(),
        })
    }

    /// Read the count field (u64 in v2+, u32 in v1).
    fn read_count(&mut self) -> Result<u64, UmcError> {
        if self.version >= 2 {
            self.read_u64_le()
        } else {
            Ok(self.read_u32_le()? as u64)
        }
    }
}

// ── Metadata value parsing ────────────────────────────────────────────────────

fn read_meta_value(cur: &mut Cursor, vtype: GgufMetaValueType) -> Result<MetaValue, UmcError> {
    match vtype {
        GgufMetaValueType::Uint8 => Ok(MetaValue::I64(cur.read_u8()? as i64)),
        GgufMetaValueType::Int8 => Ok(MetaValue::I64(cur.read_u8()? as i8 as i64)),
        GgufMetaValueType::Uint16 => Ok(MetaValue::I64(cur.read_u16_le()? as i64)),
        GgufMetaValueType::Int16 => Ok(MetaValue::I64(cur.read_u16_le()? as i16 as i64)),
        GgufMetaValueType::Uint32 => Ok(MetaValue::I64(cur.read_u32_le()? as i64)),
        GgufMetaValueType::Int32 => Ok(MetaValue::I64(cur.read_i32_le()? as i64)),
        GgufMetaValueType::Uint64 => Ok(MetaValue::I64(cur.read_u64_le()? as i64)),
        GgufMetaValueType::Int64 => Ok(MetaValue::I64(cur.read_i64_le()?)),
        GgufMetaValueType::Float32 => Ok(MetaValue::F64(cur.read_f32_le()? as f64)),
        GgufMetaValueType::Float64 => Ok(MetaValue::F64(cur.read_f64_le()?)),
        GgufMetaValueType::Bool => Ok(MetaValue::Bool(cur.read_u8()? != 0)),
        GgufMetaValueType::String => Ok(MetaValue::String(cur.read_string()?)),
        GgufMetaValueType::Array => {
            let elem_type_raw = cur.read_u32_le()?;
            let elem_type = GgufMetaValueType::from_u32(elem_type_raw).ok_or_else(|| {
                UmcError::Other(format!("Unknown array elem type: {}", elem_type_raw))
            })?;
            let count = cur.read_count()?;
            // Security: max 100k elements per array
            if count > 100_000 {
                return Err(UmcError::SecurityViolation {
                    field: "gguf_array_length".into(),
                    value: count as usize,
                    limit: 100_000,
                });
            }
            let mut arr = Vec::with_capacity(count as usize);
            for _ in 0..count {
                arr.push(read_meta_value(cur, elem_type)?);
            }
            Ok(MetaValue::Array(arr))
        }
    }
}

// ── Tensor info ───────────────────────────────────────────────────────────────

struct TensorInfo {
    name: String,
    shape: Vec<usize>,
    ggml_type: GgmlType,
    offset: u64,
    byte_size: usize,
}

// ── GgufLoader ────────────────────────────────────────────────────────────────

/// Native GGUF format loader. Supports v1, v2, v3.
pub struct GgufLoader;

impl FormatLoader for GgufLoader {
    fn format_name(&self) -> &'static str {
        "GGUF"
    }

    fn can_load(&self, path: &Path) -> bool {
        let Ok(mut f) = std::fs::File::open(path) else {
            return false;
        };
        let mut buf = [0u8; 4];
        use std::io::Read;
        f.read_exact(&mut buf).map_or(false, |_| &buf == GGUF_MAGIC)
    }

    fn load(
        &self,
        path: &Path,
        options: &LoadOptions,
        progress: &ProgressCallback,
    ) -> Result<UniversalIR, UmcError> {
        progress.report("Opening GGUF file…");

        let file = std::fs::File::open(path).map_err(UmcError::Io)?;
        let file_size = file.metadata().map_err(UmcError::Io)?.len();
        let mmap = Arc::new(unsafe {
            Mmap::map(&file).map_err(|e| UmcError::Mmap {
                context: path.display().to_string(),
                msg: e.to_string(),
            })?
        });

        let data: &[u8] = &mmap;

        // ── Check magic ────────────────────────────────────────────────────
        if data.len() < 8 {
            return Err(UmcError::FileTruncated {
                path: path.display().to_string(),
                expected: 8,
                actual: data.len(),
            });
        }
        if &data[0..4] != GGUF_MAGIC {
            return Err(UmcError::InvalidMagic {
                path: path.display().to_string(),
                expected: GGUF_MAGIC.to_vec(),
                found: data[0..4].to_vec(),
            });
        }

        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        if version < 1 || version > 3 {
            return Err(UmcError::UnsupportedFormatVersion {
                format: "GGUF".into(),
                version: format!("v{}", version),
            });
        }

        let mut cur = Cursor::new(data, version);
        cur.pos = 8; // skip magic + version

        progress.report(&format!("GGUF v{} detected", version));

        // ── Header: tensor_count + metadata_kv_count ──────────────────────
        let tensor_count = cur.read_count()?;
        let metadata_kv_count = cur.read_count()?;

        // Security limits
        if tensor_count > 1_000_000 {
            return Err(UmcError::SecurityViolation {
                field: "tensor_count".into(),
                value: tensor_count as usize,
                limit: 1_000_000,
            });
        }
        if metadata_kv_count > 10_000 {
            return Err(UmcError::SecurityViolation {
                field: "metadata_kv_count".into(),
                value: metadata_kv_count as usize,
                limit: 10_000,
            });
        }

        progress.report(&format!("Reading {} metadata entries…", metadata_kv_count));

        // ── Metadata ──────────────────────────────────────────────────────
        let mut metadata = MetadataStore::default();
        for _ in 0..metadata_kv_count {
            let key = cur.read_string()?;
            let vtype_raw = cur.read_u32_le()?;
            let vtype = GgufMetaValueType::from_u32(vtype_raw).ok_or_else(|| {
                UmcError::Other(format!("Unknown metadata value type: {}", vtype_raw))
            })?;
            let value = read_meta_value(&mut cur, vtype)?;
            metadata.insert(key, value);
        }

        progress.set_total(tensor_count);
        progress.report(&format!("Reading {} tensor infos…", tensor_count));

        // ── Tensor infos ──────────────────────────────────────────────────
        let mut tensor_infos = Vec::with_capacity(tensor_count as usize);
        for _ in 0..tensor_count {
            let name = cur.read_string()?;
            let n_dims = cur.read_u32_le()?;
            if n_dims > 8 {
                return Err(UmcError::SecurityViolation {
                    field: "tensor_n_dims".into(),
                    value: n_dims as usize,
                    limit: 8,
                });
            }
            let mut shape = Vec::with_capacity(n_dims as usize);
            for _ in 0..n_dims {
                let dim = cur.read_u64_le()?;
                if dim > 1_000_000_000 {
                    return Err(UmcError::SecurityViolation {
                        field: "tensor_dim".into(),
                        value: dim as usize,
                        limit: 1_000_000_000,
                    });
                }
                shape.push(dim as usize);
            }
            // GGUF stores shape in reverse order (innermost first)
            shape.reverse();

            let ggml_type_raw = cur.read_u32_le()?;
            let ggml_type = GgmlType::from_u32(ggml_type_raw)
                .ok_or_else(|| UmcError::Other(format!("Unknown GGML type: {}", ggml_type_raw)))?;

            let offset = cur.read_u64_le()?;

            let n_elems: usize = shape.iter().product::<usize>().max(1);
            let byte_size = ggml_row_size(ggml_type, n_elems).ok_or_else(|| {
                UmcError::Other(format!(
                    "Cannot compute byte size for tensor '{}' with type {:?} and {} elements",
                    name, ggml_type, n_elems
                ))
            })?;

            tensor_infos.push(TensorInfo {
                name,
                shape,
                ggml_type,
                offset,
                byte_size,
            });
        }

        // Alignment padding: GGUF aligns tensor data to ALIGNMENT bytes
        // The default alignment is 32, can be overridden in metadata
        let alignment: u64 = metadata
            .get_i64("general.alignment")
            .map(|v| v as u64)
            .unwrap_or(32);

        // Data segment starts after the header — cur.pos is now end of headers
        let header_end = cur.pos as u64;
        // Round up to alignment boundary
        let data_offset = (header_end + alignment - 1) / alignment * alignment;

        if !options.metadata_only {
            // Validate all tensor offsets are within the file
            for info in &tensor_infos {
                let abs_offset = data_offset + info.offset;
                let abs_end = abs_offset + info.byte_size as u64;
                if abs_end > file_size {
                    return Err(UmcError::TensorOutOfBounds {
                        name: info.name.clone(),
                        offset: abs_offset,
                        length: info.byte_size,
                        end: abs_end,
                        file_size,
                    });
                }
            }
        }

        // ── Build IR ──────────────────────────────────────────────────────
        let mut ir = UniversalIR::new("GGUF", path);
        ir.metadata = metadata;

        // ── Architecture config ───────────────────────────────────────────
        ir.architecture = build_architecture_config(&ir.metadata);

        // ── Tensors ───────────────────────────────────────────────────────
        if !options.metadata_only {
            progress.report("Loading tensors via mmap (zero-copy)…");
            for info in &tensor_infos {
                let abs_offset = (data_offset + info.offset) as usize;
                let dtype = ggml_type_to_dtype(info.ggml_type);
                let tensor = Tensor::from_mmap(
                    &info.name,
                    dtype,
                    info.shape.clone(),
                    Arc::clone(&mmap),
                    abs_offset,
                    info.byte_size,
                );
                ir.tensors.insert(tensor)?;
                progress.increment(&format!("Loaded '{}'", info.name));
            }
        }

        // ── Graph content: always WeightsOnly for GGUF ────────────────────
        let arch = ir.architecture.architecture.clone();
        ir.graph = GraphContent::WeightsOnly {
            template_available: is_known_architecture(&arch),
            architecture: arch,
            template_name: None,
        };

        // ── Tokenizer (if present) ─────────────────────────────────────────
        ir.tokenizer = build_tokenizer(&ir.metadata);

        // ── Quantization store ────────────────────────────────────────────
        ir.quantization = build_quantization_store(&tensor_infos);

        // ── Provenance ────────────────────────────────────────────────────
        let input_hash = compute_file_hash(path).unwrap_or_else(|_| "unknown".into());
        ir.provenance.append(ProvenanceEntryData {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            source_format: "GGUF".into(),
            target_format: "IR".into(),
            tool: format!("umc/{}", UMC_VERSION),
            input_hash,
            output_hash: None,
            roundtrip_level: "bit_identical".into(),
            max_divergence: None,
            warnings: vec![],
        });

        progress.report("GGUF loaded successfully.");
        Ok(ir)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn build_architecture_config(meta: &MetadataStore) -> ArchitectureConfig {
    let arch = meta
        .get_str("general.architecture")
        .unwrap_or("unknown")
        .to_string();

    // Build prefix for this architecture's metadata keys
    let p = &arch;

    ArchitectureConfig {
        architecture: arch.clone(),
        model_type: meta.get_str("general.name").unwrap_or("").to_string(),
        hidden_size: meta
            .get_i64(&format!("{}.embedding_length", p))
            .or_else(|| meta.get_i64(&format!("{}.n_embd", p)))
            .unwrap_or(0) as usize,
        num_layers: meta
            .get_i64(&format!("{}.block_count", p))
            .or_else(|| meta.get_i64(&format!("{}.n_layer", p)))
            .unwrap_or(0) as usize,
        num_heads: meta
            .get_i64(&format!("{}.attention.head_count", p))
            .or_else(|| meta.get_i64(&format!("{}.n_head", p)))
            .unwrap_or(0) as usize,
        num_kv_heads: meta
            .get_i64(&format!("{}.attention.head_count_kv", p))
            .map(|v| v as usize),
        intermediate_size: meta
            .get_i64(&format!("{}.feed_forward_length", p))
            .or_else(|| meta.get_i64(&format!("{}.n_ff", p)))
            .unwrap_or(0) as usize,
        max_position_embeddings: meta
            .get_i64(&format!("{}.context_length", p))
            .or_else(|| meta.get_i64(&format!("{}.n_ctx_train", p)))
            .unwrap_or(0) as usize,
        vocab_size: meta
            .get_i64("tokenizer.ggml.tokens")
            .map(|_| {
                // tokens is an array — get its length via metadata
                0usize // placeholder; computed from actual vocab
            })
            .unwrap_or_else(|| meta.get_i64(&format!("{}.vocab_size", p)).unwrap_or(0) as usize),
        rms_norm_eps: meta.get_f64(&format!("{}.attention.layer_norm_rms_epsilon", p)),
        layer_norm_eps: meta.get_f64(&format!("{}.attention.layer_norm_epsilon", p)),
        rope_theta: meta.get_f64(&format!("{}.rope.freq_base", p)),
        rope_scaling: None, // TODO: parse rope_scaling from GGUF
        attention_bias: false,
        tie_word_embeddings: false,
        torch_dtype: None,
        transformers_version: None,
        extra_fields: Default::default(),
    }
}

fn build_tokenizer(meta: &MetadataStore) -> Option<TokenizerStore> {
    // Only build if tokenizer data is present
    let _model_type = meta.get_str("tokenizer.ggml.model")?;
    let mut tok = TokenizerStore::new(TokenizerType::BPE, 0);

    if let Some(s) = meta.get_str("tokenizer.chat_template") {
        tok.chat_template = Some(s.to_string());
    }
    if let Some(MetaValue::I64(bos)) = meta.get("tokenizer.ggml.bos_token_id") {
        tok.bos_token = Some(format!("{}", bos));
    }
    if let Some(MetaValue::I64(eos)) = meta.get("tokenizer.ggml.eos_token_id") {
        tok.eos_token = Some(format!("{}", eos));
    }

    Some(tok)
}

fn build_quantization_store(
    tensor_infos: &[TensorInfo],
) -> Option<umc_core::ir::quantization::QuantizationStore> {
    use umc_core::ir::quantization::{QuantScheme, QuantizationStore};

    // Determine the dominant quantization scheme from tensor types
    let mut q4km_count = 0usize;
    let mut q4ks_count = 0usize;
    let mut q8_count = 0usize;
    let mut q5km_count = 0usize;
    let mut f16_count = 0usize;
    let mut f32_count = 0usize;

    for info in tensor_infos {
        match info.ggml_type {
            GgmlType::Q4KM => q4km_count += 1,
            GgmlType::Q4KS => q4ks_count += 1,
            GgmlType::Q8_0 | GgmlType::Q8_1 | GgmlType::Q8K => q8_count += 1,
            GgmlType::Q5KM => q5km_count += 1,
            GgmlType::F16 | GgmlType::BF16 => f16_count += 1,
            GgmlType::F32 => f32_count += 1,
            _ => {}
        }
    }

    let total = tensor_infos.len();
    if total == 0 {
        return None;
    }

    let dominant = if q4km_count * 2 > total {
        QuantScheme::GgufQ4KM
    } else if q4ks_count * 2 > total {
        QuantScheme::GgufQ4KS
    } else if q5km_count * 2 > total {
        QuantScheme::GgufQ5KM
    } else if q8_count * 2 > total {
        QuantScheme::GgufQ8_0
    } else if f16_count * 2 > total {
        return None; // Float model
    } else if f32_count * 2 > total {
        return None; // Float model
    } else {
        return None;
    };

    Some(QuantizationStore {
        scheme: dominant,
        description: "GGUF quantized model".into(),
    })
}

fn is_known_architecture(arch: &str) -> bool {
    matches!(
        arch,
        "llama"
            | "mistral"
            | "phi"
            | "phi2"
            | "gemma"
            | "qwen"
            | "qwen2"
            | "falcon"
            | "gpt2"
            | "bloom"
            | "mpt"
            | "stablelm"
            | "phi3"
            | "starcoder2"
            | "deepseek"
            | "deepseek2"
            | "command-r"
    )
}

fn compute_file_hash(path: &Path) -> Result<String, UmcError> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(UmcError::Io)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(UmcError::Io)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_gguf_v3_minimal() -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        // magic
        f.write_all(b"GGUF").unwrap();
        // version = 3
        f.write_all(&3u32.to_le_bytes()).unwrap();
        // tensor_count = 0 (u64)
        f.write_all(&0u64.to_le_bytes()).unwrap();
        // metadata_kv_count = 0 (u64)
        f.write_all(&0u64.to_le_bytes()).unwrap();
        f.flush().unwrap();
        f
    }

    fn write_gguf_v3_with_metadata() -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"GGUF").unwrap();
        f.write_all(&3u32.to_le_bytes()).unwrap();
        // tensor_count = 0
        f.write_all(&0u64.to_le_bytes()).unwrap();
        // metadata_kv_count = 2
        f.write_all(&2u64.to_le_bytes()).unwrap();

        // Key 1: "general.architecture" = "phi"
        let key1 = b"general.architecture";
        f.write_all(&(key1.len() as u64).to_le_bytes()).unwrap();
        f.write_all(key1).unwrap();
        f.write_all(&(GgufMetaValueType::String as u32).to_le_bytes())
            .unwrap();
        let val1 = b"phi";
        f.write_all(&(val1.len() as u64).to_le_bytes()).unwrap();
        f.write_all(val1).unwrap();

        // Key 2: "phi.block_count" = 32 (u32)
        let key2 = b"phi.block_count";
        f.write_all(&(key2.len() as u64).to_le_bytes()).unwrap();
        f.write_all(key2).unwrap();
        f.write_all(&(GgufMetaValueType::Uint32 as u32).to_le_bytes())
            .unwrap();
        f.write_all(&32u32.to_le_bytes()).unwrap();

        f.flush().unwrap();
        f
    }

    #[test]
    fn test_can_load_gguf() {
        let f = write_gguf_v3_minimal();
        let loader = GgufLoader;
        assert!(loader.can_load(f.path()));
    }

    #[test]
    fn test_load_gguf_empty() {
        let f = write_gguf_v3_minimal();
        let loader = GgufLoader;
        let ir = loader
            .load(f.path(), &LoadOptions::default(), &ProgressCallback::noop())
            .unwrap();
        assert_eq!(ir.tensors.len(), 0);
    }

    #[test]
    fn test_load_gguf_with_metadata() {
        let f = write_gguf_v3_with_metadata();
        let loader = GgufLoader;
        let ir = loader
            .load(f.path(), &LoadOptions::default(), &ProgressCallback::noop())
            .unwrap();
        assert_eq!(ir.metadata.get_str("general.architecture"), Some("phi"));
        assert_eq!(ir.metadata.get_i64("phi.block_count"), Some(32));
        assert_eq!(ir.architecture.architecture, "phi");
        assert_eq!(ir.architecture.num_layers, 32);
    }

    #[test]
    fn test_reject_bad_magic() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"EVIL\x03\x00\x00\x00").unwrap();
        f.flush().unwrap();
        let loader = GgufLoader;
        let err = loader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop());
        assert!(matches!(err, Err(UmcError::InvalidMagic { .. })));
    }

    #[test]
    fn test_reject_unsupported_version() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"GGUF").unwrap();
        f.write_all(&99u32.to_le_bytes()).unwrap();
        f.write_all(&0u64.to_le_bytes()).unwrap();
        f.write_all(&0u64.to_le_bytes()).unwrap();
        f.flush().unwrap();
        let loader = GgufLoader;
        let err = loader.load(f.path(), &LoadOptions::default(), &ProgressCallback::noop());
        assert!(matches!(
            err,
            Err(UmcError::UnsupportedFormatVersion { .. })
        ));
    }

    #[test]
    fn test_metadata_only_skips_tensors() {
        let f = write_gguf_v3_with_metadata();
        let loader = GgufLoader;
        let mut opts = LoadOptions::default();
        opts.metadata_only = true;
        let ir = loader
            .load(f.path(), &opts, &ProgressCallback::noop())
            .unwrap();
        assert!(ir.tensors.is_empty());
        assert_eq!(ir.architecture.architecture, "phi");
    }

    #[test]
    fn test_provenance_chain_appended() {
        let f = write_gguf_v3_minimal();
        let loader = GgufLoader;
        let ir = loader
            .load(f.path(), &LoadOptions::default(), &ProgressCallback::noop())
            .unwrap();
        assert_eq!(ir.provenance.len(), 1);
        assert!(ir.provenance.verify());
    }
}
