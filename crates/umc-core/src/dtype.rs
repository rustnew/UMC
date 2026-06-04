use serde::{Deserialize, Serialize};

/// Universal dtype enum — superset of all supported formats.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DType {
    // IEEE 754 floats
    F64,
    F32,
    F16,
    BF16,
    // FP8 (H100+)
    F8E4M3,
    F8E5M2,
    // Signed integers
    I64,
    I32,
    I16,
    I8,
    // Unsigned integers
    U64,
    U32,
    U16,
    U8,
    // Boolean
    Bool,
    // GGUF K-quants (block-wise)
    Q2K,
    Q3KS,
    Q3KM,
    Q3KL,
    Q4_0,
    Q4_1,
    Q4KS,
    Q4KM,
    Q5_0,
    Q5_1,
    Q5KS,
    Q5KM,
    Q6K,
    Q8_0,
    Q8K,
    // AWQ / GPTQ (channel-wise)
    Awq4,
    Awq8,
    Gptq2,
    Gptq3,
    Gptq4,
    Gptq8,
    // bitsandbytes
    NF4,
    FP4,
    // Custom / unknown
    Custom(String),
}

impl DType {
    /// Number of bytes per element, or None for sub-byte types without integer factor.
    pub fn bytes_per_element(&self) -> Option<f64> {
        match self {
            Self::F64 | Self::I64 | Self::U64 => Some(8.0),
            Self::F32 | Self::I32 | Self::U32 => Some(4.0),
            Self::F16 | Self::BF16 | Self::I16 | Self::U16 => Some(2.0),
            Self::F8E4M3 | Self::F8E5M2 | Self::I8 | Self::U8 | Self::Bool => Some(1.0),
            Self::Q8_0 | Self::Q8K | Self::Awq8 | Self::Gptq8 => Some(1.0),
            Self::Q4_0 | Self::Q4_1 | Self::Q4KS | Self::Q4KM => Some(0.5),
            Self::Q5_0 | Self::Q5_1 | Self::Q5KS | Self::Q5KM => Some(0.625),
            Self::Q6K => Some(0.75),
            Self::Q2K | Self::Gptq2 => Some(0.25),
            Self::Q3KS | Self::Q3KM | Self::Q3KL | Self::Gptq3 => Some(0.375),
            Self::NF4 | Self::FP4 | Self::Awq4 | Self::Gptq4 => Some(0.5),
            Self::Custom(_) => None,
        }
    }

    /// Returns true if this dtype involves lossy quantization (information loss vs F32).
    pub fn is_quantized(&self) -> bool {
        matches!(
            self,
            Self::Q2K
                | Self::Q3KS
                | Self::Q3KM
                | Self::Q3KL
                | Self::Q4_0
                | Self::Q4_1
                | Self::Q4KS
                | Self::Q4KM
                | Self::Q5_0
                | Self::Q5_1
                | Self::Q5KS
                | Self::Q5KM
                | Self::Q6K
                | Self::Q8_0
                | Self::Q8K
                | Self::Awq4
                | Self::Awq8
                | Self::Gptq2
                | Self::Gptq3
                | Self::Gptq4
                | Self::Gptq8
                | Self::NF4
                | Self::FP4
        )
    }

    /// Returns true if upcasting from `self` to `target` is lossless.
    pub fn is_lossless_upcast_to(&self, target: &DType) -> bool {
        matches!(
            (self, target),
            (DType::F16, DType::F32)
                | (DType::F16, DType::F64)
                | (DType::BF16, DType::F32)
                | (DType::BF16, DType::F64)
                | (DType::F32, DType::F64)
                | (DType::I8, DType::I16)
                | (DType::I8, DType::I32)
                | (DType::I8, DType::I64)
                | (DType::I16, DType::I32)
                | (DType::I16, DType::I64)
                | (DType::I32, DType::I64)
                | (DType::U8, DType::U16)
                | (DType::U8, DType::U32)
                | (DType::U8, DType::U64)
                | (DType::U16, DType::U32)
                | (DType::U16, DType::U64)
                | (DType::U32, DType::U64)
        )
    }

    /// Human-readable name for display.
    pub fn as_str(&self) -> &str {
        match self {
            Self::F64 => "F64",
            Self::F32 => "F32",
            Self::F16 => "F16",
            Self::BF16 => "BF16",
            Self::F8E4M3 => "F8E4M3",
            Self::F8E5M2 => "F8E5M2",
            Self::I64 => "I64",
            Self::I32 => "I32",
            Self::I16 => "I16",
            Self::I8 => "I8",
            Self::U64 => "U64",
            Self::U32 => "U32",
            Self::U16 => "U16",
            Self::U8 => "U8",
            Self::Bool => "Bool",
            Self::Q2K => "Q2_K",
            Self::Q3KS => "Q3_K_S",
            Self::Q3KM => "Q3_K_M",
            Self::Q3KL => "Q3_K_L",
            Self::Q4_0 => "Q4_0",
            Self::Q4_1 => "Q4_1",
            Self::Q4KS => "Q4_K_S",
            Self::Q4KM => "Q4_K_M",
            Self::Q5_0 => "Q5_0",
            Self::Q5_1 => "Q5_1",
            Self::Q5KS => "Q5_K_S",
            Self::Q5KM => "Q5_K_M",
            Self::Q6K => "Q6_K",
            Self::Q8_0 => "Q8_0",
            Self::Q8K => "Q8_K",
            Self::Awq4 => "AWQ_4",
            Self::Awq8 => "AWQ_8",
            Self::Gptq2 => "GPTQ_2",
            Self::Gptq3 => "GPTQ_3",
            Self::Gptq4 => "GPTQ_4",
            Self::Gptq8 => "GPTQ_8",
            Self::NF4 => "NF4",
            Self::FP4 => "FP4",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for DType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_per_element() {
        assert_eq!(DType::F32.bytes_per_element(), Some(4.0));
        assert_eq!(DType::F16.bytes_per_element(), Some(2.0));
        assert_eq!(DType::Q4KM.bytes_per_element(), Some(0.5));
        assert_eq!(DType::Q8_0.bytes_per_element(), Some(1.0));
        assert_eq!(DType::Q6K.bytes_per_element(), Some(0.75));
    }

    #[test]
    fn test_is_quantized() {
        assert!(DType::Q4KM.is_quantized());
        assert!(DType::Q8_0.is_quantized());
        assert!(!DType::F32.is_quantized());
        assert!(!DType::F16.is_quantized());
    }

    #[test]
    fn test_lossless_upcast() {
        assert!(DType::F16.is_lossless_upcast_to(&DType::F32));
        assert!(!DType::F32.is_lossless_upcast_to(&DType::F16));
        assert!(DType::I8.is_lossless_upcast_to(&DType::I32));
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", DType::Q4KM), "Q4_K_M");
        assert_eq!(format!("{}", DType::F32), "F32");
    }
}
