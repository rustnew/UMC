use crate::{DType, UmcError};

// ── QuantScheme ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum QuantScheme {
    GgufQ2K, GgufQ3KS, GgufQ3KM, GgufQ3KL,
    GgufQ4_0, GgufQ4_1, GgufQ4KS, GgufQ4KM,
    GgufQ5_0, GgufQ5_1, GgufQ5KS, GgufQ5KM,
    GgufQ6K, GgufQ8_0, GgufQ8K,
    AwqGemm4, AwqGemv4, AwqGemm8,
    Gptq { bits: u8, sym: bool },
    BnbNF4, BnbFP4,
    SymmetricInt8, AsymmetricInt8,
    Custom(String),
}

impl QuantScheme {
    pub fn bit_width(&self) -> u8 {
        match self {
            Self::GgufQ2K | Self::Gptq { bits: 2, .. } => 2,
            Self::GgufQ3KS | Self::GgufQ3KM | Self::GgufQ3KL => 3,
            Self::GgufQ4_0 | Self::GgufQ4_1 | Self::GgufQ4KS | Self::GgufQ4KM
            | Self::AwqGemm4 | Self::AwqGemv4 | Self::Gptq { bits: 4, .. }
            | Self::BnbNF4 | Self::BnbFP4 => 4,
            Self::GgufQ5_0 | Self::GgufQ5_1 | Self::GgufQ5KS | Self::GgufQ5KM => 5,
            Self::GgufQ6K => 6,
            Self::GgufQ8_0 | Self::GgufQ8K | Self::AwqGemm8
            | Self::SymmetricInt8 | Self::AsymmetricInt8 => 8,
            _ => 0,
        }
    }
}

// ── StorageOrder ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum StorageOrder {
    Sequential,   // weights then scales, separate (GPTQ)
    Interleaved,  // weights and scales interleaved per group (AWQ)
    BlockPacked,  // full blocks with embedded scales (GGUF)
}

// ── TensorQuantization ────────────────────────────────────────────────────────

/// Per-tensor quantization metadata — all fields required for correct dequantization.
#[derive(Debug, Clone)]
pub struct TensorQuantization {
    pub scheme: QuantScheme,
    pub block_size: usize,
    pub superblock_size: Option<usize>,
    pub scale_dtype: DType,
    pub zero_point_dtype: DType,
    pub storage_order: StorageOrder,
    pub calibration_dataset: Option<String>,
    pub calibration_method: Option<String>,
    pub group_size: Option<usize>,
}

// ── QuantizationStore ─────────────────────────────────────────────────────────

/// Global quantization information for the whole model.
#[derive(Debug, Clone)]
pub struct QuantizationStore {
    pub scheme: QuantScheme,
    pub description: String,
}

// ── CanonicalQuantization ─────────────────────────────────────────────────────

/// Universal bridge between all quantization schemes.
#[derive(Debug, Clone)]
pub struct CanonicalQuantization {
    pub bit_width: u8,
    pub block_size: usize,
    pub superblock_size: Option<usize>,
    pub scales: Vec<f32>,
    pub zero_points: Vec<f32>,
    pub scales_dtype: DType,
    pub quantized_data: Vec<u8>,
    pub storage_order: StorageOrder,
}

impl CanonicalQuantization {
    /// Dequantize to F32 — 100% native Rust, no external dependencies.
    pub fn dequantize_to_f32(&self) -> Result<Vec<f32>, UmcError> {
        match self.storage_order {
            StorageOrder::BlockPacked => self.dequantize_block_packed(),
            StorageOrder::Sequential => self.dequantize_sequential(),
            StorageOrder::Interleaved => self.dequantize_interleaved(),
        }
    }

    fn dequantize_block_packed(&self) -> Result<Vec<f32>, UmcError> {
        let block_size = self.block_size;
        let bytes_per_block = (block_size * self.bit_width as usize).div_ceil(8);
        let num_blocks = if bytes_per_block > 0 {
            self.quantized_data.len() / bytes_per_block
        } else {
            0
        };
        let mut result = Vec::with_capacity(num_blocks * block_size);

        for (block_idx, chunk) in self.quantized_data.chunks(bytes_per_block).enumerate() {
            let scale = self.scales.get(block_idx).copied().unwrap_or(1.0);
            let zero = self.zero_points.get(block_idx).copied().unwrap_or(0.0);
            match self.bit_width {
                4 => {
                    for &byte in chunk {
                        let lo = (byte & 0x0F) as f32;
                        let hi = (byte >> 4) as f32;
                        result.push(scale * (lo - zero));
                        result.push(scale * (hi - zero));
                    }
                }
                8 => {
                    for &byte in chunk {
                        result.push(scale * (byte as f32 - zero));
                    }
                }
                bw => return Err(UmcError::UnsupportedBitWidth(bw)),
            }
        }
        Ok(result)
    }

    fn dequantize_sequential(&self) -> Result<Vec<f32>, UmcError> {
        let num_elements = self.quantized_data.len() * 8 / self.bit_width as usize;
        let group_size = self.block_size;
        let mut result = Vec::with_capacity(num_elements);

        for elem_idx in 0..num_elements {
            let group_idx = elem_idx / group_size;
            let scale = self.scales.get(group_idx).copied().unwrap_or(1.0);
            let zero = self.zero_points.get(group_idx).copied().unwrap_or(0.0);
            let byte_pos = (elem_idx * self.bit_width as usize) / 8;
            let bit_offset = (elem_idx * self.bit_width as usize) % 8;
            let mask = (1u8 << self.bit_width) - 1;
            let raw_byte = self.quantized_data.get(byte_pos).copied().unwrap_or(0);
            let q = (raw_byte >> bit_offset) & mask;
            result.push(scale * (q as f32 - zero));
        }
        Ok(result)
    }

    fn dequantize_interleaved(&self) -> Result<Vec<f32>, UmcError> {
        let group_size = self.block_size;
        let num_groups = self.scales.len();
        let mut result = Vec::with_capacity(num_groups * group_size);

        for group_idx in 0..num_groups {
            let scale = self.scales[group_idx];
            let zero = self.zero_points.get(group_idx).copied().unwrap_or(0.0);
            let bytes_per_group = group_size * self.bit_width as usize / 8;
            let group_start = group_idx * bytes_per_group;
            let group_end = (group_start + bytes_per_group).min(self.quantized_data.len());
            let group_bytes = &self.quantized_data[group_start..group_end];

            for (i, &byte) in group_bytes.iter().enumerate() {
                let lo = (byte & 0x0F) as f32;
                let hi = (byte >> 4) as f32;
                if i * 2 < group_size {
                    result.push(scale * (lo - zero));
                }
                if i * 2 + 1 < group_size {
                    result.push(scale * (hi - zero));
                }
            }
        }
        Ok(result)
    }

    /// Whether re-quantization to a target scheme is supported without calibration data.
    pub fn can_requantize(&self, target: &QuantScheme) -> RequantizationSupport {
        match target {
            QuantScheme::GgufQ4KM
            | QuantScheme::GgufQ5KM
            | QuantScheme::GgufQ8_0
            | QuantScheme::GgufQ4_0
            | QuantScheme::GgufQ6K => RequantizationSupport::Supported,
            QuantScheme::AwqGemm4 | QuantScheme::AwqGemv4 => {
                RequantizationSupport::RequiresCalibration {
                    reason: "AWQ requires a calibration dataset to compute optimal scales. \
                             Convert to F16 instead."
                        .into(),
                }
            }
            QuantScheme::Gptq { .. } => RequantizationSupport::RequiresCalibration {
                reason: "GPTQ uses second-order (Hessian) optimization. \
                         Direct re-quantization produces sub-optimal results."
                    .into(),
            },
            QuantScheme::BnbNF4 | QuantScheme::BnbFP4 => RequantizationSupport::Unsupported {
                reason: "NF4/FP4 requires the bitsandbytes library. \
                         Convert to F16 then re-quantize to GGUF instead."
                    .into(),
            },
            _ => RequantizationSupport::Supported,
        }
    }
}

pub enum RequantizationSupport {
    Supported,
    RequiresCalibration { reason: String },
    Unsupported { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dequantize_4bit_block_packed() {
        // Two bytes → 4 nibbles = 4 elements in one block
        let quant = CanonicalQuantization {
            bit_width: 4,
            block_size: 4,
            superblock_size: None,
            scales: vec![1.0],
            zero_points: vec![0.0],
            scales_dtype: DType::F32,
            quantized_data: vec![0x21, 0x43],  // nibbles: 1,2,3,4
            storage_order: StorageOrder::BlockPacked,
        };
        let out = quant.dequantize_to_f32().unwrap();
        assert_eq!(out, vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_dequantize_8bit_block_packed() {
        let quant = CanonicalQuantization {
            bit_width: 8,
            block_size: 3,
            superblock_size: None,
            scales: vec![0.5],
            zero_points: vec![0.0],
            scales_dtype: DType::F32,
            quantized_data: vec![2, 4, 6],
            storage_order: StorageOrder::BlockPacked,
        };
        let out = quant.dequantize_to_f32().unwrap();
        assert_eq!(out, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_quant_scheme_bit_width() {
        assert_eq!(QuantScheme::GgufQ4KM.bit_width(), 4);
        assert_eq!(QuantScheme::GgufQ8_0.bit_width(), 8);
        assert_eq!(QuantScheme::GgufQ2K.bit_width(), 2);
    }
}
