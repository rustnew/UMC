/// LoRA / PEFT adapter loader.
/// Detects lora_A / lora_B weight patterns in SafeTensors files and stores
/// them in ir.adapters. Also reads adapter_config.json (PEFT format).
use std::path::Path;
use umc_core::{UmcError, UniversalIR, AdapterInfo, AdapterType, UMC_VERSION};
use umc_core::ir::MetaValue;
use umc_core::ir::provenance::ProvenanceEntryData;
use umc_core::{FormatLoader, LoadOptions, ProgressCallback};

pub struct LoraLoader;

impl FormatLoader for LoraLoader {
    fn format_name(&self) -> &'static str { "LoRA" }

    fn can_load(&self, path: &Path) -> bool {
        // File with lora tensor patterns (safetensors or .bin; extension check is lenient)
        if path.is_file() && has_lora_tensors(path) {
            return true;
        }
        // PEFT directory with adapter_config.json
        if path.is_dir() && path.join("adapter_config.json").exists() {
            return true;
        }
        false
    }

    fn load(&self, path: &Path, opts: &LoadOptions, progress: &ProgressCallback)
        -> Result<UniversalIR, UmcError>
    {
        let adapter_path = if path.is_dir() {
            // Find adapter_model.safetensors
            let candidates = ["adapter_model.safetensors", "adapter_model.bin"];
            candidates.iter()
                .map(|name| path.join(name))
                .find(|p| p.exists())
                .ok_or_else(|| UmcError::Other(
                    "LoRA: no adapter_model.safetensors found in directory".into()
                ))?
        } else {
            path.to_path_buf()
        };

        // Load the safetensors file
        let st_loader = crate::safetensors::SafeTensorsLoader;
        let mut ir = st_loader.load(&adapter_path, opts, progress)?;

        // Parse adapter_config.json if present
        let config_dir = if path.is_dir() { path.to_path_buf() }
            else { path.parent().unwrap_or(path).to_path_buf() };

        let mut rank: Option<usize> = None;
        let mut alpha: Option<f64> = None;
        let mut target_modules: Vec<String> = Vec::new();
        let mut adapter_type = AdapterType::LoRA;

        if let Ok(cfg_str) = std::fs::read_to_string(config_dir.join("adapter_config.json")) {
            if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&cfg_str) {
                rank = cfg.get("r").or(cfg.get("rank")).and_then(|v| v.as_u64())
                    .map(|v| v as usize);
                alpha = cfg.get("lora_alpha").and_then(|v| v.as_f64());
                if let Some(mods) = cfg.get("target_modules").and_then(|v| v.as_array()) {
                    target_modules = mods.iter()
                        .filter_map(|m| m.as_str().map(|s| s.to_string()))
                        .collect();
                }
                if let Some(peft_type) = cfg.get("peft_type").and_then(|v| v.as_str()) {
                    if peft_type.to_lowercase().contains("lora") {
                        adapter_type = AdapterType::LoRA;
                    } else if peft_type.to_lowercase().contains("qlora") {
                        adapter_type = AdapterType::QLoRA;
                    } else {
                        adapter_type = AdapterType::PEFT;
                    }
                }
            }
        }

        // Build AdapterInfo with all LoRA tensor pairs
        let lora_a_keys: Vec<String> = ir.tensors.iter()
            .filter(|(k, _)| k.contains("lora_A") || k.contains("lora_a"))
            .map(|(k, _)| k.clone())
            .collect();

        let mut adapter_tensors: indexmap::IndexMap<String, Vec<u8>> = indexmap::IndexMap::new();
        for key in ir.tensors.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>() {
            let raw = ir.tensors.get(&key).unwrap().data.as_bytes()
                .map_err(|e| UmcError::Other(e.to_string()))?.to_vec();
            adapter_tensors.insert(key.clone(), raw);
        }

        let adapter = AdapterInfo {
            adapter_type,
            rank,
            alpha,
            target_modules,
            tensors: adapter_tensors,
        };
        ir.adapters.push(adapter);

        ir.metadata.insert("source_format", MetaValue::String("LoRA".into()));
        if let Some(r) = rank { ir.metadata.insert("lora.rank", MetaValue::I64(r as i64)); }
        if let Some(a) = alpha { ir.metadata.insert("lora.alpha", MetaValue::F64(a)); }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        ir.provenance.append(ProvenanceEntryData {
            timestamp,
            source_format: "LoRA".into(),
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

fn has_lora_tensors(path: &Path) -> bool {
    use std::io::Read;
    let mut f = match std::fs::File::open(path) { Ok(f) => f, Err(_) => return false };
    let mut size_buf = [0u8; 8];
    if f.read_exact(&mut size_buf).is_err() { return false; }
    let header_size = u64::from_le_bytes(size_buf) as usize;
    if header_size > 10_000_000 { return false; }
    let mut header_buf = vec![0u8; header_size];
    if f.read_exact(&mut header_buf).is_err() { return false; }
    let h = std::str::from_utf8(&header_buf).unwrap_or("");
    h.contains("lora_A") || h.contains("lora_B") || h.contains("lora_a") || h.contains("lora_b")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::safetensors::SafeTensorsSaver;
    use umc_core::{FormatSaver, SaveOptions, Tensor, DType};
    use tempfile::NamedTempFile;

    fn make_lora_ir() -> UniversalIR {
        let mut ir = UniversalIR::new("LoRA", std::path::Path::new("adapter.safetensors"));
        let bytes_a: Vec<u8> = (0..32u8).collect();
        let bytes_b: Vec<u8> = (0..48u8).collect();
        ir.tensors.insert(Tensor::from_bytes("base_model.model.layer.lora_A.weight", DType::F32, vec![2, 4], bytes_a)).unwrap();
        ir.tensors.insert(Tensor::from_bytes("base_model.model.layer.lora_B.weight", DType::F32, vec![3, 4], bytes_b)).unwrap();
        ir
    }

    #[test]
    fn test_lora_detection_and_load() {
        let ir = make_lora_ir();
        let saver = SafeTensorsSaver;
        let tmp = NamedTempFile::new().unwrap();
        saver.save(&ir, tmp.path(), &SaveOptions::default(), &ProgressCallback::noop()).unwrap();

        let loader = LoraLoader;
        assert!(loader.can_load(tmp.path()), "LoRA file should be detected");

        let loaded = loader.load(tmp.path(), &LoadOptions::default(), &ProgressCallback::noop()).unwrap();
        assert_eq!(loaded.adapters.len(), 1);
        assert_eq!(loaded.adapters[0].tensors.len(), 2);
    }
}
