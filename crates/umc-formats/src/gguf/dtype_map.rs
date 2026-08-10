use super::spec::GgmlType;
use umc_core::DType;

/// Map GGML tensor type → UMC DType.
pub fn ggml_type_to_dtype(t: GgmlType) -> DType {
    match t {
        GgmlType::F32 => DType::F32,
        GgmlType::F16 => DType::F16,
        GgmlType::BF16 => DType::BF16,
        GgmlType::Q4_0 => DType::Q4_0,
        GgmlType::Q4_1 => DType::Q4_1,
        GgmlType::Q5_0 => DType::Q5_0,
        GgmlType::Q5_1 => DType::Q5_1,
        GgmlType::Q8_0 => DType::Q8_0,
        GgmlType::Q8_1 => DType::Q8_0, // treat as Q8_0
        GgmlType::Q2K => DType::Q2K,
        GgmlType::Q3KS => DType::Q3KS,
        GgmlType::Q3KM => DType::Q3KM,
        GgmlType::Q3KL => DType::Q3KL,
        GgmlType::Q4KS => DType::Q4KS,
        GgmlType::Q4KM => DType::Q4KM,
        GgmlType::Q5KS => DType::Q5KS,
        GgmlType::Q5KM => DType::Q5KM,
        GgmlType::Q6K => DType::Q6K,
        GgmlType::Q8K => DType::Q8K,
        GgmlType::I8 => DType::I8,
        GgmlType::I16 => DType::I16,
        GgmlType::I32 => DType::I32,
        GgmlType::I64 => DType::I64,
        GgmlType::F64 => DType::F64,
        // IQ variants — map to custom
        GgmlType::IQ2XXS => DType::Custom("IQ2_XXS".into()),
        GgmlType::IQ2XS => DType::Custom("IQ2_XS".into()),
        GgmlType::IQ2S => DType::Custom("IQ2_S".into()),
        GgmlType::IQ3XXS => DType::Custom("IQ3_XXS".into()),
        GgmlType::IQ3S => DType::Custom("IQ3_S".into()),
        GgmlType::IQ1S => DType::Custom("IQ1_S".into()),
        GgmlType::IQ1M => DType::Custom("IQ1_M".into()),
        GgmlType::IQ4NL => DType::Custom("IQ4_NL".into()),
        GgmlType::IQ4XS => DType::Custom("IQ4_XS".into()),
    }
}

/// Bytes per row for a given GGML type and number of elements.
/// Returns None if the type has a variable or unknown row size.
pub fn ggml_row_size(t: GgmlType, n_elems: usize) -> Option<usize> {
    // Block sizes (number of elements per quantization block)
    let (block_elements, bytes_per_block): (usize, usize) = match t {
        GgmlType::F32 => return Some(n_elems * 4),
        GgmlType::F16 => return Some(n_elems * 2),
        GgmlType::BF16 => return Some(n_elems * 2),
        GgmlType::I8 => return Some(n_elems),
        GgmlType::I16 => return Some(n_elems * 2),
        GgmlType::I32 => return Some(n_elems * 4),
        GgmlType::I64 => return Some(n_elems * 8),
        GgmlType::F64 => return Some(n_elems * 8),
        // Q4_0: 32 elements, 2 bytes scale (F16) + 16 bytes data = 18 bytes/block
        GgmlType::Q4_0 => (32, 18),
        // Q4_1: 32 elements, 2 bytes scale + 2 bytes min + 16 bytes data = 20 bytes/block
        GgmlType::Q4_1 => (32, 20),
        // Q5_0: 32 elements, 2 bytes scale + 20 bytes data = 22 bytes/block
        GgmlType::Q5_0 => (32, 22),
        // Q5_1: 32 elements, 2+2 bytes scale/min + 20 bytes data = 24 bytes/block
        GgmlType::Q5_1 => (32, 24),
        // Q8_0: 32 elements, 2 bytes scale + 32 bytes data = 34 bytes/block
        GgmlType::Q8_0 => (32, 34),
        GgmlType::Q8_1 => (32, 36),
        // Q2_K: 256 elements, 84 bytes/block
        GgmlType::Q2K => (256, 84),
        // Q3_K: 256 elements, 110 bytes/block
        GgmlType::Q3KS | GgmlType::Q3KM | GgmlType::Q3KL => (256, 110),
        // Q4_K: 256 elements, 144 bytes/block
        GgmlType::Q4KS | GgmlType::Q4KM => (256, 144),
        // Q5_K: 256 elements, 176 bytes/block
        GgmlType::Q5KS | GgmlType::Q5KM => (256, 176),
        // Q6_K: 256 elements, 210 bytes/block
        GgmlType::Q6K => (256, 210),
        // Q8_K: 256 elements, 292 bytes/block
        GgmlType::Q8K => (256, 292),
        // IQ variants
        GgmlType::IQ2XXS => (256, 66),
        GgmlType::IQ2XS => (256, 74),
        GgmlType::IQ2S => (256, 82),
        GgmlType::IQ3XXS => (256, 98),
        GgmlType::IQ3S => (256, 98),
        GgmlType::IQ1S => (256, 50),
        GgmlType::IQ1M => (256, 56),
        GgmlType::IQ4NL => (32, 18),
        GgmlType::IQ4XS => (256, 136),
    };

    if n_elems % block_elements != 0 {
        return None;
    }
    Some((n_elems / block_elements) * bytes_per_block)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_size() {
        assert_eq!(ggml_row_size(GgmlType::F32, 128), Some(512));
    }

    #[test]
    fn test_f16_size() {
        assert_eq!(ggml_row_size(GgmlType::F16, 128), Some(256));
    }

    #[test]
    fn test_q4_0_size() {
        // 32 elements = 1 block = 18 bytes
        assert_eq!(ggml_row_size(GgmlType::Q4_0, 32), Some(18));
        assert_eq!(ggml_row_size(GgmlType::Q4_0, 64), Some(36));
    }

    #[test]
    fn test_q4km_size() {
        // 256 elements = 1 block = 144 bytes
        assert_eq!(ggml_row_size(GgmlType::Q4KM, 256), Some(144));
    }

    #[test]
    fn test_dtype_mapping() {
        assert_eq!(ggml_type_to_dtype(GgmlType::F32), DType::F32);
        assert_eq!(ggml_type_to_dtype(GgmlType::Q4KM), DType::Q4KM);
        assert_eq!(ggml_type_to_dtype(GgmlType::BF16), DType::BF16);
    }
}
