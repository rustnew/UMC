use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Tamper-evident provenance chain — append-only audit log.
///
/// `entry[n].chain_hash = SHA256(entry[n-1].chain_hash || entry[n].content_hash)`
///
/// Any tampering with a past entry is detectable via `verify()`.
#[derive(Debug, Clone)]
pub struct ProvenanceChain {
    entries: Vec<ProvenanceEntry>,
    root_hash: String,
}

/// A single conversion event in the chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceEntry {
    pub timestamp: u64,
    pub source_format: String,
    pub target_format: String,
    pub tool: String,
    pub input_hash: String,
    pub output_hash: Option<String>,
    /// "bit_identical" | "semantic" | "structural"
    pub roundtrip_level: String,
    pub max_divergence: Option<f64>,
    pub warnings: Vec<String>,
    /// SHA256 of this entry's content fields.
    pub content_hash: String,
    /// SHA256(prev_chain_hash || content_hash).
    pub chain_hash: String,
}

/// Input data for appending a new provenance entry.
pub struct ProvenanceEntryData {
    pub timestamp: u64,
    pub source_format: String,
    pub target_format: String,
    pub tool: String,
    pub input_hash: String,
    pub output_hash: Option<String>,
    pub roundtrip_level: String,
    pub max_divergence: Option<f64>,
    pub warnings: Vec<String>,
}

impl ProvenanceChain {
    /// Create a new chain seeded with the source file.
    pub fn new(source_format: &str, source_path: &std::path::Path) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let seed = format!("UMC_ROOT_{}_{}", timestamp, source_path.display());
        let root_hash = hex::encode(Sha256::digest(seed.as_bytes()));
        let _ = source_format; // used in seed indirectly via path
        Self {
            entries: Vec::new(),
            root_hash,
        }
    }

    /// Append a new entry — the only mutation allowed.
    pub fn append(&mut self, data: ProvenanceEntryData) -> &ProvenanceEntry {
        let prev_chain_hash = self
            .entries
            .last()
            .map(|e| e.chain_hash.as_str())
            .unwrap_or(self.root_hash.as_str());

        let content = serde_json::json!({
            "timestamp": data.timestamp,
            "source_format": data.source_format,
            "target_format": data.target_format,
            "tool": data.tool,
            "input_hash": data.input_hash,
            "output_hash": data.output_hash,
            "roundtrip_level": data.roundtrip_level,
            "max_divergence": data.max_divergence,
            "warnings": data.warnings,
        });
        let content_hash = hex::encode(Sha256::digest(content.to_string().as_bytes()));
        let chain_input = format!("{}{}", prev_chain_hash, content_hash);
        let chain_hash = hex::encode(Sha256::digest(chain_input.as_bytes()));

        self.entries.push(ProvenanceEntry {
            timestamp: data.timestamp,
            source_format: data.source_format,
            target_format: data.target_format,
            tool: data.tool,
            input_hash: data.input_hash,
            output_hash: data.output_hash,
            roundtrip_level: data.roundtrip_level,
            max_divergence: data.max_divergence,
            warnings: data.warnings,
            content_hash,
            chain_hash,
        });
        self.entries.last().unwrap()
    }

    /// Verify chain integrity — O(n) hash verification.
    /// Checks both content hash recomputation AND chain link integrity.
    pub fn verify(&self) -> bool {
        let mut prev = self.root_hash.clone();
        for entry in &self.entries {
            // 1. Recompute content_hash from fields — detects field tampering.
            let content = serde_json::json!({
                "timestamp": entry.timestamp,
                "source_format": entry.source_format,
                "target_format": entry.target_format,
                "tool": entry.tool,
                "input_hash": entry.input_hash,
                "output_hash": entry.output_hash,
                "roundtrip_level": entry.roundtrip_level,
                "max_divergence": entry.max_divergence,
                "warnings": entry.warnings,
            });
            let recomputed_content_hash =
                hex::encode(Sha256::digest(content.to_string().as_bytes()));
            if recomputed_content_hash != entry.content_hash {
                return false;
            }
            // 2. Verify chain link — detects reordering.
            let expected_chain = hex::encode(Sha256::digest(
                format!("{}{}", prev, entry.content_hash).as_bytes(),
            ));
            if expected_chain != entry.chain_hash {
                return false;
            }
            prev = entry.chain_hash.clone();
        }
        true
    }

    pub fn entries(&self) -> &[ProvenanceEntry] {
        &self.entries
    }

    pub fn root_hash(&self) -> &str {
        &self.root_hash
    }

    pub fn last_entry(&self) -> Option<&ProvenanceEntry> {
        self.entries.last()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn make_entry(src: &str, tgt: &str) -> ProvenanceEntryData {
        ProvenanceEntryData {
            timestamp: 1_700_000_000,
            source_format: src.into(),
            target_format: tgt.into(),
            tool: "umc/0.1.0".into(),
            input_hash: "aaa".into(),
            output_hash: Some("bbb".into()),
            roundtrip_level: "semantic".into(),
            max_divergence: Some(1e-6),
            warnings: vec![],
        }
    }

    #[test]
    fn test_empty_chain_verifies() {
        let chain = ProvenanceChain::new("GGUF", Path::new("model.gguf"));
        assert!(chain.verify());
        assert_eq!(chain.len(), 0);
    }

    #[test]
    fn test_append_and_verify() {
        let mut chain = ProvenanceChain::new("GGUF", Path::new("model.gguf"));
        chain.append(make_entry("GGUF", "ONNX"));
        chain.append(make_entry("ONNX", "GGUF"));
        assert_eq!(chain.len(), 2);
        assert!(chain.verify());
    }

    #[test]
    fn test_tamper_detection() {
        let mut chain = ProvenanceChain::new("GGUF", Path::new("model.gguf"));
        chain.append(make_entry("GGUF", "ONNX"));
        // Tamper with the first entry
        if let Some(entry) = chain.entries.first_mut() {
            entry.source_format = "EVIL".into();
        }
        assert!(!chain.verify());
    }

    #[test]
    fn test_last_entry() {
        let mut chain = ProvenanceChain::new("GGUF", Path::new("model.gguf"));
        chain.append(make_entry("GGUF", "ONNX"));
        let last = chain.last_entry().unwrap();
        assert_eq!(last.target_format, "ONNX");
    }
}
