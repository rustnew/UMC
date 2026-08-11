pub mod dtype;
/// UMC Core — Universal Intermediate Representation, traits, and error types.
/// Every crate in the workspace depends on this crate.
pub mod error;
pub mod ir;
pub mod traits;

pub use dtype::DType;
pub use error::UmcError;
pub use ir::adapter::{AdapterInfo, AdapterType};
pub use ir::extension::{ExtensionStore, FormatExtension};
pub use ir::graph::{
    ComputeEdge, ComputeGraph, ComputeNode, ConstantValue, GraphTensor, OpAttributes, PadMode,
    UniversalOp,
};
pub use ir::provenance::{ProvenanceChain, ProvenanceEntry, ProvenanceEntryData};
pub use ir::quantization::{
    CanonicalQuantization, QuantScheme, QuantizationStore, RequantizationSupport, StorageOrder,
    TensorQuantization,
};
pub use ir::tensor::{Layout, SecurityBounds, Tensor, TensorData, TensorStore};
pub use ir::tokenizer::{TokenizerStore, TokenizerType};
pub use ir::{
    ArchitectureConfig, ConversionHints, ConversionHintsMap, GenerationConfig, GraphContent,
    MetaValue, MetadataStore, RopeScalingConfig, SubModelGraph, SubModelRole, TiedWeightsPolicy,
    UniversalIR,
};
pub use traits::{FormatLoader, FormatSaver, LoadOptions, ProgressCallback, SaveOptions};

/// UMC version string — embedded in all certificates and provenance entries.
pub const UMC_VERSION: &str = "1.0.0";
