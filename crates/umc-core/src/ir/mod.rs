pub mod tensor;
pub mod quantization;
pub mod extension;
pub mod provenance;
pub mod graph;
pub mod tokenizer;
pub mod adapter;

use serde::{Deserialize, Serialize};

pub use tensor::{Tensor, TensorData, TensorStore, Layout, SecurityBounds};
pub use quantization::{TensorQuantization, QuantScheme, StorageOrder, QuantizationStore};
pub use extension::ExtensionStore;
pub use provenance::ProvenanceChain;
pub use graph::ComputeGraph;
pub use tokenizer::TokenizerStore;
pub use adapter::AdapterInfo;

/// Universal Intermediate Representation — the heart of UMC.
///
/// Every format is loaded into this structure and saved from it.
/// The IR is the mathematical superset of all supported formats.
///
/// # Guarantees
/// - Zero information loss: every field not representable natively
///   is stored in `extensions` and restored on round-trip.
/// - Security: all insertions are validated against `SecurityBounds`.
/// - Provenance: every conversion is appended to `provenance` (append-only).
#[derive(Debug, Clone)]
pub struct UniversalIR {
    /// All model tensors (weights, biases, embeddings…).
    pub tensors: TensorStore,

    /// Graph content — distinguishes weights-only from formats with an explicit graph.
    pub graph: GraphContent,

    /// General model metadata (key-value).
    pub metadata: MetadataStore,

    /// Architecture hyperparameters.
    pub architecture: ArchitectureConfig,

    /// Tokenizer (LLM models).
    pub tokenizer: Option<TokenizerStore>,

    /// Global quantization scheme (if any).
    pub quantization: Option<QuantizationStore>,

    /// Adapters (LoRA, QLoRA, PEFT…).
    pub adapters: Vec<AdapterInfo>,

    /// Immutable provenance chain — tamper-evident audit log.
    pub provenance: ProvenanceChain,

    /// Opaque extension blobs — zero information loss guarantee.
    /// Keys are namespaced: "FORMAT@VERSION/field.path"
    pub extensions: ExtensionStore,

    /// Per-pair conversion hints (layout transposes, tied-weight policies, etc.)
    pub conversion_hints: ConversionHintsMap,
}

impl UniversalIR {
    /// Create a new empty IR for a given source format and file path.
    pub fn new(source_format: &str, source_path: &std::path::Path) -> Self {
        Self {
            tensors: TensorStore::new(),
            graph: GraphContent::WeightsOnly {
                architecture: String::new(),
                template_available: false,
                template_name: None,
            },
            metadata: MetadataStore::default(),
            architecture: ArchitectureConfig::default(),
            tokenizer: None,
            quantization: None,
            adapters: Vec::new(),
            provenance: ProvenanceChain::new(source_format, source_path),
            extensions: ExtensionStore::default(),
            conversion_hints: ConversionHintsMap::default(),
        }
    }

    /// Returns the number of parameters (elements, not bytes).
    pub fn num_parameters(&self) -> u64 {
        self.tensors
            .iter()
            .map(|(_, t)| t.shape.iter().product::<usize>() as u64)
            .sum()
    }

    /// Human-readable summary line.
    pub fn summary(&self) -> String {
        let n_params = self.num_parameters();
        let arch = &self.architecture.architecture;
        let n_tensors = self.tensors.len();
        format!(
            "Architecture: {} | Tensors: {} | Parameters: {:.2}B",
            if arch.is_empty() { "unknown" } else { arch },
            n_tensors,
            n_params as f64 / 1e9
        )
    }
}

// ── GraphContent ─────────────────────────────────────────────────────────────

/// Distinguishes formats with an explicit compute graph from weights-only formats.
#[derive(Debug, Clone)]
pub enum GraphContent {
    /// Formats with an explicit compute graph (ONNX, PyTorch, TFSavedModel, TFLite…).
    Explicit(ComputeGraph),

    /// Weights-only formats (GGUF, SafeTensors, AWQ, GPTQ, bitsandbytes…).
    /// A graph can be reconstructed via GraphTemplate when converting to a graph format.
    WeightsOnly {
        architecture: String,
        template_available: bool,
        template_name: Option<String>,
    },

    /// Composite formats with multiple sub-models (Diffusers).
    Composite(Vec<SubModelGraph>),

    /// Pure tokenizer format (SentencePiece, TikToken).
    TokenizerOnly,
}

#[derive(Debug, Clone)]
pub struct SubModelGraph {
    pub name: String,
    pub graph: ComputeGraph,
    pub role: SubModelRole,
    pub format_hint: String,
}

#[derive(Debug, Clone)]
pub enum SubModelRole {
    TextEncoder,
    TextEncoder2,
    ImageEncoder,
    Denoiser,
    Decoder,
    Encoder,
    Scheduler,
    Custom(String),
}

// ── MetadataStore ─────────────────────────────────────────────────────────────

/// Ordered key-value metadata store.
#[derive(Debug, Clone, Default)]
pub struct MetadataStore {
    entries: indexmap::IndexMap<String, MetaValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetaValue {
    String(String),
    I64(i64),
    F64(f64),
    Bool(bool),
    Array(Vec<MetaValue>),
    Raw(Vec<u8>),
}

impl MetadataStore {
    pub fn insert(&mut self, key: impl Into<String>, value: MetaValue) {
        self.entries.insert(key.into(), value);
    }
    pub fn get(&self, key: &str) -> Option<&MetaValue> {
        self.entries.get(key)
    }
    pub fn get_str(&self, key: &str) -> Option<&str> {
        match self.entries.get(key)? {
            MetaValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        match self.entries.get(key)? {
            MetaValue::I64(v) => Some(*v),
            _ => None,
        }
    }
    pub fn get_u32(&self, key: &str) -> Option<u32> {
        self.get_i64(key).and_then(|v| u32::try_from(v).ok())
    }
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        match self.entries.get(key)? {
            MetaValue::F64(v) => Some(*v),
            _ => None,
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &MetaValue)> {
        self.entries.iter()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ── ArchitectureConfig ────────────────────────────────────────────────────────

/// Architecture hyperparameters — common fields across all LLM families.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchitectureConfig {
    pub architecture: String,
    pub model_type: String,
    pub hidden_size: usize,
    pub num_layers: usize,
    pub num_heads: usize,
    pub num_kv_heads: Option<usize>,
    pub intermediate_size: usize,
    pub max_position_embeddings: usize,
    pub vocab_size: usize,
    pub rms_norm_eps: Option<f64>,
    pub layer_norm_eps: Option<f64>,
    pub rope_theta: Option<f64>,
    pub rope_scaling: Option<RopeScalingConfig>,
    pub attention_bias: bool,
    pub tie_word_embeddings: bool,
    pub torch_dtype: Option<String>,
    pub transformers_version: Option<String>,
    #[serde(default)]
    pub extra_fields: indexmap::IndexMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RopeScalingConfig {
    pub scaling_type: String,
    pub factor: f64,
    pub original_max_position_embeddings: Option<usize>,
    pub low_freq_factor: Option<f64>,
    pub high_freq_factor: Option<f64>,
    #[serde(default)]
    pub extra: indexmap::IndexMap<String, serde_json::Value>,
}

/// Generation / inference configuration (LLM).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub bos_token_id: Option<u32>,
    pub eos_token_id: Option<Vec<u32>>,
    pub pad_token_id: Option<u32>,
    pub max_new_tokens: Option<usize>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<usize>,
    pub repetition_penalty: Option<f64>,
    pub do_sample: bool,
    #[serde(default)]
    pub extra: indexmap::IndexMap<String, serde_json::Value>,
}

// ── ConversionHintsMap ────────────────────────────────────────────────────────

/// Per-pair conversion hints — resolves the 20% of edge cases where N+M IR is not enough.
#[derive(Debug, Clone, Default)]
pub struct ConversionHintsMap {
    hints: std::collections::HashMap<(String, String), ConversionHints>,
}

impl ConversionHintsMap {
    pub fn get(&self, source: &str, target: &str) -> Option<&ConversionHints> {
        self.hints.get(&(source.to_string(), target.to_string()))
    }
    pub fn insert(&mut self, source: impl Into<String>, target: impl Into<String>, hints: ConversionHints) {
        self.hints.insert((source.into(), target.into()), hints);
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConversionHints {
    pub layout_transpose: Option<Vec<usize>>,
    pub fuse_batchnorm: bool,
    pub tied_weights_policy: TiedWeightsPolicy,
    pub decompose_ops: Vec<String>,
    pub weight_alignment: Option<usize>,
    pub note: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum TiedWeightsPolicy {
    #[default]
    PreserveShared,
    Duplicate,
    Deduplicate,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_universal_ir_new() {
        let ir = UniversalIR::new("GGUF", Path::new("model.gguf"));
        assert!(ir.tensors.is_empty());
        assert_eq!(ir.adapters.len(), 0);
        assert!(ir.tokenizer.is_none());
    }

    #[test]
    fn test_metadata_store() {
        let mut meta = MetadataStore::default();
        meta.insert("name", MetaValue::String("phi-2".into()));
        meta.insert("layers", MetaValue::I64(32));
        meta.insert("eps", MetaValue::F64(1e-5));

        assert_eq!(meta.get_str("name"), Some("phi-2"));
        assert_eq!(meta.get_i64("layers"), Some(32));
        assert!((meta.get_f64("eps").unwrap() - 1e-5).abs() < 1e-10);
        assert_eq!(meta.get_str("missing"), None);
    }

    #[test]
    fn test_num_parameters() {
        let ir = UniversalIR::new("GGUF", Path::new("model.gguf"));
        assert_eq!(ir.num_parameters(), 0);
    }
}
