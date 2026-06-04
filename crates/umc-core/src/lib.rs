/// UMC Core — Universal Intermediate Representation, traits, and error types.
/// Every crate in the workspace depends on this crate.

pub mod error;
pub mod dtype;
pub mod ir;
pub mod traits;

pub use error::UmcError;
pub use dtype::DType;
pub use ir::{
    UniversalIR, GraphContent, MetadataStore, MetaValue,
    ArchitectureConfig, GenerationConfig, RopeScalingConfig,
    ConversionHintsMap, ConversionHints, TiedWeightsPolicy,
    SubModelGraph, SubModelRole,
};
pub use ir::tensor::{Tensor, TensorData, TensorStore, Layout, SecurityBounds};
pub use ir::quantization::{
    TensorQuantization, QuantScheme, StorageOrder, CanonicalQuantization,
    RequantizationSupport, QuantizationStore,
};
pub use ir::extension::{ExtensionStore, FormatExtension};
pub use ir::provenance::{ProvenanceChain, ProvenanceEntry, ProvenanceEntryData};
pub use ir::graph::{
    ComputeGraph, ComputeNode, ComputeEdge, GraphTensor,
    UniversalOp, OpAttributes, PadMode, ConstantValue,
};
pub use ir::tokenizer::{TokenizerStore, TokenizerType};
pub use ir::adapter::{AdapterInfo, AdapterType};
pub use traits::{FormatLoader, FormatSaver, LoadOptions, SaveOptions, ProgressCallback};

/// UMC version string — embedded in all certificates and provenance entries.
pub const UMC_VERSION: &str = "0.1.0";
