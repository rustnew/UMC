/// GPTQ (Generative Pre-trained Quantization) loader.
/// GPTQ files are SafeTensors with: qweight (int32), qzeros (int32), scales (f16), g_idx (int32)
use std::path::Path;
use umc_core::ir::provenance::ProvenanceEntryData;
use umc_core::ir::{MetaValue, QuantScheme, QuantizationStore};
use umc_core::{FormatLoader, LoadOptions, ProgressCallback};
use umc_core::{UmcError, UniversalIR, UMC_VERSION};

pub struct GptqLoader;

impl FormatLoader for GptqLoader {
    fn format_name(&self) -> &'static str {
        "GPTQ"
    }

    fn can_load(&self, path: &Path) -> bool {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        if ext != "safetensors" {
            return false;
        }
        is_gptq_file(path)
    }

    fn load(
        &self,
        path: &Path,
        opts: &LoadOptions,
        progress: &ProgressCallback,
    ) -> Result<UniversalIR, UmcError> {
        let st_loader = crate::safetensors::SafeTensorsLoader;
        let mut ir = st_loader.load(path, opts, progress)?;

        ir.metadata
            .insert("source_format", MetaValue::String("GPTQ".into()));
        ir.metadata
            .insert("quantization.method", MetaValue::String("gptq".into()));
        ir.metadata.insert("quantization.bits", MetaValue::I64(4));

        // Parse quantization config from config.json
        let mut bits = 4i64;
        let mut group_size = 128usize;

        if let Some(parent) = path.parent() {
            if let Ok(config) = std::fs::read_to_string(parent.join("config.json")) {
                if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&config) {
                    if let Some(qcfg) = cfg.get("quantization_config") {
                        if let Some(b) = qcfg.get("bits").and_then(|v| v.as_i64()) {
                            bits = b;
                            ir.metadata
                                .insert("quantization.bits", MetaValue::I64(bits));
                        }
                        if let Some(gs) = qcfg.get("group_size").and_then(|v| v.as_i64()) {
                            group_size = gs as usize;
                            ir.metadata
                                .insert("quantization.group_size", MetaValue::I64(gs));
                        }
                    }
                }
            }
        }

        ir.quantization = Some(QuantizationStore {
            scheme: QuantScheme::Gptq {
                bits: bits as u8,
                sym: false,
            },
            description: format!("GPTQ {}-bit, group_size={}", bits, group_size),
        });

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        ir.provenance.append(ProvenanceEntryData {
            timestamp,
            source_format: "GPTQ".into(),
            target_format: "IR".into(),
            tool: format!("umc/{}", UMC_VERSION),
            input_hash: "unknown".into(),
            output_hash: None,
            roundtrip_level: "structural".into(),
            max_divergence: None,
            warnings: vec![],
        });

        Ok(ir)
    }
}

fn is_gptq_file(path: &Path) -> bool {
    // Check SafeTensors header for GPTQ tensor name patterns
    use std::io::Read;
    let mut f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut size_buf = [0u8; 8];
    if f.read_exact(&mut size_buf).is_err() {
        return false;
    }
    let header_size = u64::from_le_bytes(size_buf) as usize;
    if header_size > 10_000_000 {
        return false;
    }
    let mut header_buf = vec![0u8; header_size];
    if f.read_exact(&mut header_buf).is_err() {
        return false;
    }
    let h = std::str::from_utf8(&header_buf).unwrap_or("");
    h.contains("qweight") && (h.contains("qzeros") || h.contains("scales")) || {
        path.parent().map_or(false, |p| {
            if let Ok(cfg) = std::fs::read_to_string(p.join("config.json")) {
                cfg.contains("\"gptq\"") || cfg.contains("\"GPTQ\"")
            } else {
                false
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gptq::saver::GptqSaver;
    use tempfile::NamedTempFile;
    use umc_core::{DType, FormatSaver, SaveOptions, Tensor};

    fn make_gptq_ir() -> UniversalIR {
        let mut ir = UniversalIR::new("GPTQ", std::path::Path::new("model.safetensors"));
        let qweight: Vec<u8> = (0..64u8).collect();
        let scales: Vec<u8> = vec![0u8; 16];
        let qzeros: Vec<u8> = vec![0u8; 4];
        ir.tensors
            .insert(Tensor::from_bytes(
                "layer.qweight",
                DType::I32,
                vec![4, 4],
                qweight,
            ))
            .unwrap();
        ir.tensors
            .insert(Tensor::from_bytes(
                "layer.scales",
                DType::F16,
                vec![1, 4],
                scales,
            ))
            .unwrap();
        ir.tensors
            .insert(Tensor::from_bytes(
                "layer.qzeros",
                DType::I32,
                vec![1, 1],
                qzeros,
            ))
            .unwrap();
        ir.metadata
            .insert("quantization.method", MetaValue::String("gptq".into()));
        ir.metadata.insert("quantization.bits", MetaValue::I64(4));
        ir
    }

    #[test]
    fn test_gptq_round_trip() {
        let ir = make_gptq_ir();
        let saver = GptqSaver;
        let tmp = NamedTempFile::new().unwrap();
        saver
            .save(
                &ir,
                tmp.path(),
                &SaveOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();

        let st_loader = crate::safetensors::SafeTensorsLoader;
        let loaded = st_loader
            .load(
                tmp.path(),
                &LoadOptions::default(),
                &ProgressCallback::noop(),
            )
            .unwrap();
        assert_eq!(loaded.tensors.len(), 3);
        assert!(loaded.tensors.get("layer.qweight").is_some());
    }
}
