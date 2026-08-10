/// GGUF file format constants and types.
/// Spec: https://github.com/ggerganov/ggml/blob/master/docs/gguf.md

pub const GGUF_MAGIC: &[u8; 4] = b"GGUF";

#[allow(dead_code)]
pub const GGUF_VERSION_1: u32 = 1;
#[allow(dead_code)]
pub const GGUF_VERSION_2: u32 = 2;
#[allow(dead_code)]
pub const GGUF_VERSION_3: u32 = 3;

/// GGUF metadata value type tag.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum GgufMetaValueType {
    Uint8 = 0,
    Int8 = 1,
    Uint16 = 2,
    Int16 = 3,
    Uint32 = 4,
    Int32 = 5,
    Float32 = 6,
    Bool = 7,
    String = 8,
    Array = 9,
    Uint64 = 10,
    Int64 = 11,
    Float64 = 12,
}

impl GgufMetaValueType {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Uint8),
            1 => Some(Self::Int8),
            2 => Some(Self::Uint16),
            3 => Some(Self::Int16),
            4 => Some(Self::Uint32),
            5 => Some(Self::Int32),
            6 => Some(Self::Float32),
            7 => Some(Self::Bool),
            8 => Some(Self::String),
            9 => Some(Self::Array),
            10 => Some(Self::Uint64),
            11 => Some(Self::Int64),
            12 => Some(Self::Float64),
            _ => None,
        }
    }
}

/// GGUF tensor type (quantization scheme).
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
pub enum GgmlType {
    F32 = 0,
    F16 = 1,
    Q4_0 = 2,
    Q4_1 = 3,
    // 4,5 unused
    Q5_0 = 6,
    Q5_1 = 7,
    Q8_0 = 8,
    Q8_1 = 9,
    Q2K = 10,
    Q3KS = 11,
    Q3KM = 12,
    Q3KL = 13,
    Q4KS = 14,
    Q4KM = 15,
    Q5KS = 16,
    Q5KM = 17,
    Q6K = 18,
    Q8K = 19,
    IQ2XXS = 20,
    IQ2XS = 21,
    IQ3XXS = 22,
    IQ1S = 23,
    IQ4NL = 24,
    IQ3S = 25,
    IQ2S = 26,
    IQ4XS = 27,
    I8 = 28,
    I16 = 29,
    I32 = 30,
    I64 = 31,
    F64 = 32,
    IQ1M = 33,
    BF16 = 34,
}

impl GgmlType {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::F32),
            1 => Some(Self::F16),
            2 => Some(Self::Q4_0),
            3 => Some(Self::Q4_1),
            6 => Some(Self::Q5_0),
            7 => Some(Self::Q5_1),
            8 => Some(Self::Q8_0),
            9 => Some(Self::Q8_1),
            10 => Some(Self::Q2K),
            11 => Some(Self::Q3KS),
            12 => Some(Self::Q3KM),
            13 => Some(Self::Q3KL),
            14 => Some(Self::Q4KS),
            15 => Some(Self::Q4KM),
            16 => Some(Self::Q5KS),
            17 => Some(Self::Q5KM),
            18 => Some(Self::Q6K),
            19 => Some(Self::Q8K),
            20 => Some(Self::IQ2XXS),
            21 => Some(Self::IQ2XS),
            22 => Some(Self::IQ3XXS),
            23 => Some(Self::IQ1S),
            24 => Some(Self::IQ4NL),
            25 => Some(Self::IQ3S),
            26 => Some(Self::IQ2S),
            27 => Some(Self::IQ4XS),
            28 => Some(Self::I8),
            29 => Some(Self::I16),
            30 => Some(Self::I32),
            31 => Some(Self::I64),
            32 => Some(Self::F64),
            33 => Some(Self::IQ1M),
            34 => Some(Self::BF16),
            _ => None,
        }
    }
}
