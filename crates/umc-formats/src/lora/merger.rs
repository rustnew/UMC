/// LoRA weight merger: fuses lora_A and lora_B into base model weights.
/// Formula: W_new = W_base + (lora_B @ lora_A) * (alpha / rank)
/// Used when saving to formats that don't natively support LoRA (GGUF, ONNX, TFLite).
use std::collections::HashMap;
use std::sync::Arc;
use umc_core::{DType, UmcError, UniversalIR, Tensor, TensorData};

/// Merge all LoRA adapters from `adapter_ir` into `base_ir`.
/// Modifies base_ir tensors in-place.
pub fn merge_lora_into_base(
    base_ir: &mut UniversalIR,
    adapter_ir: &UniversalIR,
) -> Result<MergeStats, UmcError> {
    let mut stats = MergeStats::default();

    for adapter in &adapter_ir.adapters {
        let rank = adapter.rank.unwrap_or(8);
        let alpha = adapter.alpha.unwrap_or(rank as f64);
        let scale = (alpha / rank as f64) as f32;

        // Group lora_A / lora_B pairs by layer name
        let pairs = find_lora_pairs(&adapter.tensors);
        stats.pairs_found += pairs.len();

        for (layer_name, (key_a, key_b)) in &pairs {
            // Get raw bytes for lora_A and lora_B
            let a_bytes = adapter.tensors.get(key_a)
                .ok_or_else(|| UmcError::Other(format!("LoRA: lora_A '{}' not found", key_a)))?;
            let b_bytes = adapter.tensors.get(key_b)
                .ok_or_else(|| UmcError::Other(format!("LoRA: lora_B '{}' not found", key_b)))?;

            // Get base weight tensor
            let base_key = layer_to_base_key(layer_name);
            let base_tensor = match base_ir.tensors.get_mut(&base_key) {
                Some(t) => t,
                None => {
                    stats.skipped_not_found += 1;
                    continue;
                }
            };

            if base_tensor.dtype != DType::F32 {
                stats.skipped_wrong_dtype += 1;
                continue;
            }

            let shape = base_tensor.shape.clone();
            if shape.len() != 2 {
                stats.skipped_wrong_shape += 1;
                continue;
            }
            let (rows, cols) = (shape[0], shape[1]);

            // Parse lora_A: [rank × cols] and lora_B: [rows × rank]
            let lora_a = bytes_to_f32_vec(a_bytes);
            let lora_b = bytes_to_f32_vec(b_bytes);

            let lora_a_rows = lora_a.len() / cols;
            let lora_b_cols = lora_b.len() / rows;

            if lora_a_rows == 0 || lora_b_cols == 0 || lora_a_rows != lora_b_cols {
                stats.skipped_shape_mismatch += 1;
                continue;
            }

            // Compute delta = (lora_B @ lora_A) * scale
            // lora_B: [rows × lora_rank], lora_A: [lora_rank × cols]
            // Result: [rows × cols]
            let lora_rank = lora_a_rows;
            let mut delta = vec![0f32; rows * cols];
            for r in 0..rows {
                for c in 0..cols {
                    let mut acc = 0f32;
                    for k in 0..lora_rank {
                        acc += lora_b[r * lora_rank + k] * lora_a[k * cols + c];
                    }
                    delta[r * cols + c] = acc * scale;
                }
            }

            // Add delta to base weight
            let base_bytes = base_tensor.data.as_bytes()
                .map_err(|e| UmcError::Other(format!("LoRA merge: {}", e)))?;
            let mut base_f32 = bytes_to_f32_vec(base_bytes);

            if base_f32.len() != delta.len() {
                stats.skipped_shape_mismatch += 1;
                continue;
            }

            for (w, d) in base_f32.iter_mut().zip(delta.iter()) {
                *w += d;
            }

            let new_bytes: Vec<u8> = base_f32.iter()
                .flat_map(|f| f.to_le_bytes())
                .collect();
            base_tensor.data = TensorData::Owned(Arc::new(new_bytes));
            stats.merged += 1;
        }
    }

    Ok(stats)
}

#[derive(Debug, Default)]
pub struct MergeStats {
    pub pairs_found: usize,
    pub merged: usize,
    pub skipped_not_found: usize,
    pub skipped_wrong_dtype: usize,
    pub skipped_wrong_shape: usize,
    pub skipped_shape_mismatch: usize,
}

fn bytes_to_f32_vec(bytes: &[u8]) -> Vec<f32> {
    bytes.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Find lora_A / lora_B pairs in adapter tensors.
/// Returns: HashMap<layer_name, (key_A, key_B)>
fn find_lora_pairs(tensors: &indexmap::IndexMap<String, Vec<u8>>) -> HashMap<String, (String, String)> {
    let mut pairs: HashMap<String, (String, String)> = HashMap::new();
    let mut a_keys: HashMap<String, String> = HashMap::new();
    let mut b_keys: HashMap<String, String> = HashMap::new();

    for key in tensors.keys() {
        // Extract layer name as the prefix before ".lora_A" / ".lora_B"
        if let Some(pos) = key.find(".lora_A").or_else(|| key.find(".lora_a")) {
            let layer = key[..pos].to_string();
            a_keys.insert(layer, key.clone());
        } else if let Some(pos) = key.find(".lora_B").or_else(|| key.find(".lora_b")) {
            let layer = key[..pos].to_string();
            b_keys.insert(layer, key.clone());
        }
    }

    for (layer, key_a) in &a_keys {
        if let Some(key_b) = b_keys.get(layer) {
            pairs.insert(layer.clone(), (key_a.clone(), key_b.clone()));
        }
    }
    pairs
}

/// Convert PEFT tensor key to base model key.
/// "base_model.model.layer.weight" → "model.layer.weight" or just "layer.weight"
fn layer_to_base_key(layer_name: &str) -> String {
    // Strip "base_model.model." prefix common in PEFT
    let stripped = layer_name
        .strip_prefix("base_model.model.")
        .unwrap_or(layer_name);
    // Append ".weight" if not present
    if stripped.ends_with(".weight") || stripped.ends_with(".bias") {
        stripped.to_string()
    } else {
        format!("{}.weight", stripped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_simple_lora() {
        // Base: [2×3] identity-like weight
        let base_data: Vec<f32> = vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        let base_bytes: Vec<u8> = base_data.iter().flat_map(|f| f.to_le_bytes()).collect();

        let mut base_ir = UniversalIR::new("base", std::path::Path::new("base.safetensors"));
        base_ir.tensors.insert(Tensor::from_bytes("linear.weight", DType::F32, vec![2, 3], base_bytes)).unwrap();

        // lora_A: [2×3] = [[0.1, 0.0, 0.0], [0.0, 0.1, 0.0]]
        let a_data: Vec<f32> = vec![0.1, 0.0, 0.0, 0.0, 0.1, 0.0];
        let a_bytes: Vec<u8> = a_data.iter().flat_map(|f| f.to_le_bytes()).collect();
        // lora_B: [2×2] = [[1.0, 0.0], [0.0, 1.0]]
        let b_data: Vec<f32> = vec![1.0, 0.0, 0.0, 1.0];
        let b_bytes: Vec<u8> = b_data.iter().flat_map(|f| f.to_le_bytes()).collect();

        let mut adapter_tensors = indexmap::IndexMap::new();
        adapter_tensors.insert("base_model.model.linear.lora_A.weight".into(), a_bytes);
        adapter_tensors.insert("base_model.model.linear.lora_B.weight".into(), b_bytes);

        let adapter_ir = {
            let mut ir = UniversalIR::new("lora", std::path::Path::new("adapter.safetensors"));
            ir.adapters.push(umc_core::AdapterInfo {
                adapter_type: umc_core::AdapterType::LoRA,
                rank: Some(2),
                alpha: Some(2.0),
                target_modules: vec!["linear".into()],
                tensors: adapter_tensors,
            });
            ir
        };

        let stats = merge_lora_into_base(&mut base_ir, &adapter_ir).unwrap();
        assert_eq!(stats.merged, 1);

        let merged = base_ir.tensors.get("linear.weight").unwrap();
        let merged_f32 = bytes_to_f32_vec(merged.data.as_bytes().unwrap());
        // delta = (B @ A) * 1.0 = [[0.1,0.0,0.0],[0.0,0.1,0.0]]
        // result = base + delta
        assert!((merged_f32[0] - 1.1).abs() < 1e-5);
        assert!((merged_f32[4] - 1.1).abs() < 1e-5);
    }

    #[test]
    fn test_find_lora_pairs() {
        let mut tensors = indexmap::IndexMap::new();
        tensors.insert("model.layer.lora_A.weight".into(), vec![]);
        tensors.insert("model.layer.lora_B.weight".into(), vec![]);
        tensors.insert("model.other.weight".into(), vec![]);

        let pairs = find_lora_pairs(&tensors);
        assert_eq!(pairs.len(), 1);
        assert!(pairs.contains_key("model.layer"));
    }
}
