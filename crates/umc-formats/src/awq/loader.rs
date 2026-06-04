/// AWQ (Activation-aware Weight Quantization) loader.
/// AWQ files are HuggingFace SafeTensors with 4-bit packed weights + per-channel scales/zeros.
/// Tensor naming: {layer}.weight (int32 packed), {layer}.scales (f16), {layer}.zeros (int32)
use std::path::Path;
use umc_core::{UmcError, UniversalIR, UMC_VERSION};
use umc_core::ir::{MetaValue, QuantizationStore, QuantScheme};
use umc_core::ir::provenance::ProvenanceEntryData;
use umc_core::{FormatLoader, LoadOptions, ProgressCallback};

pub struct AwqLoader;

impl FormatLoader for AwqLoader {
    fn format_name(&self) -> &'static str { "AWQ" }

    fn can_load(&self, path: &Path) -> bool {
        let ext = path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
        if ext != "safetensors" { return false; }
        // Check for AWQ metadata in the safetensors header
        is_awq_file(path)
    }

    fn load(&self, path: &Path, opts: &LoadOptions, progress: &ProgressCallback)
        -> Result<UniversalIR, UmcError>
    {
        // Reuse SafeTensors loader for the heavy lifting
        let st_loader = crate::safetensors::SafeTensorsLoader;
        let mut ir = st_loader.load(path, opts, progress)?;

        // Override source format and mark as AWQ
        ir.metadata.insert("source_format", MetaValue::String("AWQ".into()));
        ir.metadata.insert("quantization.method", MetaValue::String("awq".into()));
        ir.metadata.insert("quantization.bits", MetaValue::I64(4));

        // Parse AWQ config if present in same directory
        if let Some(parent) = path.parent() {
            if let Ok(config) = std::fs::read_to_string(parent.join("quant_config.json"))
                .or_else(|_| std::fs::read_to_string(parent.join("quantize_config.json")))
            {
                if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&config) {
                    if let Some(bits) = cfg.get("w_bit").or(cfg.get("bits"))
                        .and_then(|v| v.as_i64())
                    {
                        ir.metadata.insert("quantization.bits", MetaValue::I64(bits));
                    }
                    if let Some(gs) = cfg.get("q_group_size").or(cfg.get("group_size"))
                        .and_then(|v| v.as_i64())
                    {
                        ir.metadata.insert("quantization.group_size", MetaValue::I64(gs));
                    }
                }
            }
        }

        ir.quantization = Some(QuantizationStore {
            scheme: QuantScheme::AwqGemm4,
            description: format!(
                "AWQ 4-bit, group_size={}",
                ir.metadata.get_i64("quantization.group_size").unwrap_or(128)
            ),
        });

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        ir.provenance.append(ProvenanceEntryData {
            timestamp,
            source_format: "AWQ".into(),
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

fn is_awq_file(path: &Path) -> bool {
    // Quick check: open safetensors header and look for AWQ metadata
    use std::io::Read;
    let mut f = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut size_buf = [0u8; 8];
    if f.read_exact(&mut size_buf).is_err() { return false; }
    let header_size = u64::from_le_bytes(size_buf) as usize;
    if header_size > 10_000_000 { return false; }
    let mut header_buf = vec![0u8; header_size];
    if f.read_exact(&mut header_buf).is_err() { return false; }
    let header_str = std::str::from_utf8(&header_buf).unwrap_or("");
    // AWQ files have scales/zeros or awq in metadata
    header_str.contains("\"quant_type\"") && header_str.contains("awq")
        || header_str.contains(".scales") && header_str.contains(".zeros")
        || {
            // Check parent dir for quant config
            path.parent().map_or(false, |p| {
                p.join("quant_config.json").exists()
                    || p.join("quantize_config.json").exists()
            })
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::awq::saver::AwqSaver;
    use umc_core::{FormatSaver, SaveOptions, Tensor, DType};
    use tempfile::NamedTempFile;

    fn make_awq_ir() -> UniversalIR {
        let mut ir = UniversalIR::new("AWQ", std::path::Path::new("model.safetensors"));
        let weight_bytes: Vec<u8> = (0..32u8).collect();
        let scales_bytes: Vec<u8> = vec![0u8; 32];
        ir.tensors.insert(Tensor::from_bytes("layer.weight", DType::I32, vec![4, 2], weight_bytes)).unwrap();
        ir.tensors.insert(Tensor::from_bytes("layer.scales", DType::F16, vec![4], scales_bytes)).unwrap();
        ir.metadata.insert("quantization.method", MetaValue::String("awq".into()));
        ir.metadata.insert("quantization.bits", MetaValue::I64(4));
        ir
    }

    #[test]
    fn test_awq_save_load_round_trip() {
        let ir = make_awq_ir();
        let saver = AwqSaver;
        let tmp = NamedTempFile::new().unwrap();
        saver.save(&ir, tmp.path(), &SaveOptions::default(), &ProgressCallback::noop()).unwrap();

        // Load back as SafeTensors (since AWQ = SafeTensors + metadata)
        let loader = crate::safetensors::SafeTensorsLoader;
        let loaded = loader.load(tmp.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();

        assert_eq!(loaded.tensors.len(), 2);
        assert!(loaded.tensors.get("layer.weight").is_some());
        assert!(loaded.tensors.get("layer.scales").is_some());
    }
}
