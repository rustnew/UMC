#[derive(Debug, Clone)]
pub enum AdapterType {
    LoRA,
    QLoRA,
    PEFT,
    Custom(String),
}

/// Adapter (LoRA / QLoRA / PEFT) information.
#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub adapter_type: AdapterType,
    pub rank: Option<usize>,
    pub alpha: Option<f64>,
    pub target_modules: Vec<String>,
    /// Raw adapter tensors stored by name.
    pub tensors: indexmap::IndexMap<String, Vec<u8>>,
}
