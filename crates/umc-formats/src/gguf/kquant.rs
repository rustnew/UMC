/// Native K-quant dequantization following the GGUF / llama.cpp specification.
/// Reference: https://github.com/ggerganov/llama.cpp/blob/master/ggml/src/ggml-quants.c
///
/// All block sizes and bit-layouts are taken verbatim from that source.

// ── F16 → F32 helper ──────────────────────────────────────────────────────────

#[inline]
fn f16_to_f32(h: u16) -> f32 {
    // IEEE 754 half-precision to single-precision.
    let sign = ((h as u32 & 0x8000) << 16) as u32;
    let exp = (h as u32 & 0x7C00) >> 10;
    let frac = (h as u32 & 0x03FF) as u32;
    let bits: u32 = if exp == 0 {
        if frac == 0 {
            sign
        } else {
            let mut e = 127 - 14;
            let mut f = frac;
            while f & 0x0400 == 0 {
                f <<= 1;
                e -= 1;
            }
            sign | ((e as u32) << 23) | ((f & 0x03FF) << 13)
        }
    } else if exp == 0x1F {
        sign | 0x7F800000 | (frac << 13)
    } else {
        sign | ((exp + 112) << 23) | (frac << 13)
    };
    f32::from_bits(bits)
}

// ── Scale extraction helpers ──────────────────────────────────────────────────

/// Extract 6-bit scale and min from the packed K4 scales byte array (12 bytes → 8 pairs).
#[inline]
fn get_scale_min_k4(j: usize, q: &[u8]) -> (u8, u8) {
    if j < 4 {
        (q[j] & 63, q[j + 4] & 63)
    } else {
        (
            (q[j + 4] & 0x0F) | ((q[j - 4] >> 6) << 4),
            (q[j + 4] >> 4) | ((q[j - 0] >> 6) << 4),
        )
    }
}

// ── Q2_K: 256 elements, 84 bytes/block ───────────────────────────────────────
// Layout: scales[16] | qs[64] | d(f16)[2] | dmin(f16)[2]
// 16 groups of 16 elements; each group has a 4-bit scale and 4-bit min.

pub const Q2_K_BLOCK_SIZE: usize = 256;
pub const Q2_K_BLOCK_BYTES: usize = 84;

pub fn dequantize_q2_k(data: &[u8], n_elements: usize) -> Vec<f32> {
    let n_blocks = n_elements / Q2_K_BLOCK_SIZE;
    let mut out = vec![0f32; n_blocks * Q2_K_BLOCK_SIZE];

    for b in 0..n_blocks {
        let block = &data[b * Q2_K_BLOCK_BYTES..];
        let scales_raw = &block[0..16];
        let qs_raw = &block[16..80];
        let d = f16_to_f32(u16::from_le_bytes([block[80], block[81]]));
        let dmin = f16_to_f32(u16::from_le_bytes([block[82], block[83]]));

        let dst = &mut out[b * Q2_K_BLOCK_SIZE..];
        let mut is = 0usize;
        let mut qs_off = 0usize;

        for _ in 0..16 {
            let scale = d * (scales_raw[is] & 0xF) as f32;
            let min = dmin * (scales_raw[is] >> 4) as f32;
            is += 1;

            for j in 0..16 {
                let q = (qs_raw[qs_off + j / 4] >> (2 * (j % 4))) & 3;
                dst[is * 16 - 16 + j] = scale * q as f32 - min;
            }
            qs_off += 4;
        }
    }
    out
}

// ── Q3_K: 256 elements, 110 bytes/block ──────────────────────────────────────
// Layout: hmask[32] | qs[64] | scales[12] | d(f16)[2]
// 8 super-groups; hmask provides the high bit of each 3-bit value.

pub const Q3_K_BLOCK_SIZE: usize = 256;
pub const Q3_K_BLOCK_BYTES: usize = 110;

/// Extract one signed byte from a packed u32 array at flat index `i`.
#[inline]
fn aux_scale_byte(aux: &[u32; 4], i: usize) -> i8 {
    ((aux[i / 4] >> ((i % 4) * 8)) & 0xFF) as i8
}

pub fn dequantize_q3_k(data: &[u8], n_elements: usize) -> Vec<f32> {
    let n_blocks = n_elements / Q3_K_BLOCK_SIZE;
    let mut out = vec![0f32; n_blocks * Q3_K_BLOCK_SIZE];

    const KMASK1: u32 = 0x03030303;
    const KMASK2: u32 = 0x0f0f0f0f;

    for b in 0..n_blocks {
        let block = &data[b * Q3_K_BLOCK_BYTES..];
        let hmask = &block[0..32]; // 32 bytes: 1 high bit per element
        let qs = &block[32..96]; // 64 bytes: 2 low bits per element
        let scdata = &block[96..108]; // 12 bytes: packed 6-bit scales
        let d_all = f16_to_f32(u16::from_le_bytes([block[108], block[109]]));

        // Decode 12-byte packed scales identical to llama.cpp kmask manipulation.
        let s0 = u32::from_le_bytes(scdata[0..4].try_into().unwrap_or([0; 4]));
        let s1 = u32::from_le_bytes(scdata[4..8].try_into().unwrap_or([0; 4]));
        let tmp = u32::from_le_bytes(scdata[8..12].try_into().unwrap_or([0; 4]));
        let mut aux = [0u32; 4];
        aux[2] = ((s0 >> 4) & KMASK2) | (((tmp >> 4) & KMASK1) << 6);
        aux[3] = ((s1 >> 4) & KMASK2) | (((tmp >> 6) & KMASK1) << 6);
        aux[0] = (s0 & KMASK2) | (((tmp >> 0) & KMASK1) << 6);
        aux[1] = (s1 & KMASK2) | (((tmp >> 2) & KMASK1) << 6);

        let dst = &mut out[b * Q3_K_BLOCK_SIZE..];

        // Process each element directly using its bit position.
        // For element i (0..255):
        //   lo = qs[i/4] bits (i%4)*2 and (i%4)*2+1   — shift 0,2,4,6: never overflows u8
        //   hi = hmask[i/8] bit i%8
        //   q  = lo | (hi << 2)  → unsigned 0..7, subtract 4 → signed -4..3
        for i in 0..Q3_K_BLOCK_SIZE {
            let group = i / 32;
            let sc = aux_scale_byte(&aux, group);
            let d = d_all * sc as f32;
            let lo = (qs[i / 4] >> ((i % 4) * 2)) & 3;
            let hi = (hmask[i / 8] >> (i % 8)) & 1;
            let q = lo | (hi << 2);
            dst[i] = d * (q as i32 - 4) as f32;
        }
    }
    out
}

// ── Q4_K: 256 elements, 144 bytes/block ──────────────────────────────────────
// Layout: d(f16)[2] | dmin(f16)[2] | scales[12] | qs[128]
// 8 groups of 32 elements. Each group has a 6-bit scale and 6-bit min.

pub const Q4_K_BLOCK_SIZE: usize = 256;
pub const Q4_K_BLOCK_BYTES: usize = 144;

pub fn dequantize_q4_k(data: &[u8], n_elements: usize) -> Vec<f32> {
    let n_blocks = n_elements / Q4_K_BLOCK_SIZE;
    let mut out = vec![0f32; n_blocks * Q4_K_BLOCK_SIZE];

    for b in 0..n_blocks {
        let block = &data[b * Q4_K_BLOCK_BYTES..];
        let d = f16_to_f32(u16::from_le_bytes([block[0], block[1]]));
        let dmin = f16_to_f32(u16::from_le_bytes([block[2], block[3]]));
        let scales = &block[4..16];
        let qs_raw = &block[16..144];

        let dst = &mut out[b * Q4_K_BLOCK_SIZE..];
        let mut qs_off = 0usize;

        for group in 0..8 {
            let (sc, mn) = get_scale_min_k4(group, scales);
            let scale = d * sc as f32;
            let min = dmin * mn as f32;

            for j in 0..16 {
                let byte = qs_raw[qs_off + j];
                dst[group * 32 + j] = scale * (byte & 0x0F) as f32 - min;
                dst[group * 32 + j + 16] = scale * (byte >> 4) as f32 - min;
            }
            qs_off += 16;
        }
    }
    out
}

// ── Q5_K: 256 elements, 176 bytes/block ──────────────────────────────────────
// Layout: d(f16)[2] | dmin(f16)[2] | scales[12] | qh[32] | qs[128]
// Like Q4_K but each element gets a 5th bit from qh.

pub const Q5_K_BLOCK_SIZE: usize = 256;
pub const Q5_K_BLOCK_BYTES: usize = 176;

pub fn dequantize_q5_k(data: &[u8], n_elements: usize) -> Vec<f32> {
    let n_blocks = n_elements / Q5_K_BLOCK_SIZE;
    let mut out = vec![0f32; n_blocks * Q5_K_BLOCK_SIZE];

    for b in 0..n_blocks {
        let block = &data[b * Q5_K_BLOCK_BYTES..];
        let d = f16_to_f32(u16::from_le_bytes([block[0], block[1]]));
        let dmin = f16_to_f32(u16::from_le_bytes([block[2], block[3]]));
        let scales = &block[4..16];
        let qh_raw = &block[16..48];
        let qs_raw = &block[48..176];

        let dst = &mut out[b * Q5_K_BLOCK_SIZE..];
        let mut qs_off = 0usize;
        let mut qh_off = 0usize;
        let mut hshift: u32 = 0;

        for group in 0..8 {
            let (sc, mn) = get_scale_min_k4(group, scales);
            let scale = d * sc as f32;
            let min = dmin * mn as f32;

            for j in 0..16 {
                let lo_lo = qs_raw[qs_off + j] & 0x0F;
                let lo_hi = qs_raw[qs_off + j] >> 4;
                let hi_lo = (qh_raw[qh_off + j / 8] >> (hshift + (j % 8) as u32)) & 1;
                let hi_hi = (qh_raw[qh_off + (j + 16) / 8] >> (hshift + ((j + 16) % 8) as u32)) & 1;
                dst[group * 32 + j] = scale * (lo_lo | (hi_lo << 4)) as f32 - min;
                dst[group * 32 + j + 16] = scale * (lo_hi | (hi_hi << 4)) as f32 - min;
            }
            qs_off += 16;
            if (group & 3) == 3 {
                qh_off += 4;
                hshift = 0;
            } else {
                hshift += 1;
            }
        }
    }
    out
}

// ── Q6_K: 256 elements, 210 bytes/block ──────────────────────────────────────
// Layout: ql[128] | qh[64] | scales[16] | d(f16)[2]
// 16 groups of 16 elements. scales are i8.

pub const Q6_K_BLOCK_SIZE: usize = 256;
pub const Q6_K_BLOCK_BYTES: usize = 210;

pub fn dequantize_q6_k(data: &[u8], n_elements: usize) -> Vec<f32> {
    let n_blocks = n_elements / Q6_K_BLOCK_SIZE;
    let mut out = vec![0f32; n_blocks * Q6_K_BLOCK_SIZE];

    for b in 0..n_blocks {
        let block = &data[b * Q6_K_BLOCK_BYTES..];
        let ql_raw = &block[0..128];
        let qh_raw = &block[128..192];
        let sc_raw = &block[192..208];
        let d = f16_to_f32(u16::from_le_bytes([block[208], block[209]]));

        let dst = &mut out[b * Q6_K_BLOCK_SIZE..];

        for group in 0..16 {
            let scale = d * (sc_raw[group] as i8) as f32;
            let base = group * 16;
            let ql_off = group * 8;
            let qh_off = (group / 2) * 8;
            let hshift = (group % 2) * 4;

            for j in 0..8 {
                let ql0 = ql_raw[ql_off + j] & 0x0F;
                let ql1 = ql_raw[ql_off + j] >> 4;
                let qh0 = (qh_raw[qh_off + j] >> hshift) & 0x0F;
                let q0 = (ql0 | ((qh0 & 3) << 4)) as i32 - 32;
                let q1 = (ql1 | (((qh0 >> 2) & 3) << 4)) as i32 - 32;
                dst[base + j] = scale * q0 as f32;
                dst[base + j + 8] = scale * q1 as f32;
            }
        }
    }
    out
}

// ── Q8_K: 256 elements, 292 bytes/block ──────────────────────────────────────
// Layout: d(f64)[8] | qs[256] | bsums[16×i16] | unused[12]
// Simple: each element is i8 × f64 scale.

pub const Q8_K_BLOCK_SIZE: usize = 256;
pub const Q8_K_BLOCK_BYTES: usize = 292;

pub fn dequantize_q8_k(data: &[u8], n_elements: usize) -> Vec<f32> {
    let n_blocks = n_elements / Q8_K_BLOCK_SIZE;
    let mut out = vec![0f32; n_blocks * Q8_K_BLOCK_SIZE];

    for b in 0..n_blocks {
        let block = &data[b * Q8_K_BLOCK_BYTES..];
        let d = f64::from_le_bytes(block[0..8].try_into().unwrap_or([0; 8])) as f32;
        let qs = &block[8..264];

        let dst = &mut out[b * Q8_K_BLOCK_SIZE..];
        for i in 0..Q8_K_BLOCK_SIZE {
            dst[i] = d * (qs[i] as i8) as f32;
        }
    }
    out
}

// ── Dispatch ─────────────────────────────────────────────────────────────────

use umc_core::{DType, UmcError};

/// Dequantize ANY quantized tensor to F32 bytes.
/// Covers Q4_0, Q4_1, Q5_0, Q5_1, Q8_0 (legacy quants) and all K-quant types.
pub fn dequantize_any_to_f32_bytes(
    data: &[u8],
    dtype: &DType,
    n_elements: usize,
) -> Result<Vec<u8>, UmcError> {
    match dtype {
        // ── Legacy quants ─────────────────────────────────────────────────
        DType::Q4_0 => {
            let (block_size, bpb) = (32usize, 18usize);
            let mut out = vec![0f32; n_elements];
            for (bi, block) in data.chunks(bpb).enumerate() {
                if block.len() < bpb {
                    break;
                }
                let scale = f16_to_f32(u16::from_le_bytes([block[0], block[1]]));
                for (i, &byte) in block[2..18].iter().enumerate() {
                    let lo = (byte & 0x0F) as i32 - 8;
                    let hi = ((byte >> 4) & 0x0F) as i32 - 8;
                    let i0 = bi * block_size + i * 2;
                    let i1 = i0 + 1;
                    if i0 < n_elements {
                        out[i0] = lo as f32 * scale;
                    }
                    if i1 < n_elements {
                        out[i1] = hi as f32 * scale;
                    }
                }
            }
            Ok(out.iter().flat_map(|f| f.to_le_bytes()).collect())
        }
        DType::Q4_1 => {
            let (block_size, bpb) = (32usize, 20usize);
            let mut out = vec![0f32; n_elements];
            for (bi, block) in data.chunks(bpb).enumerate() {
                if block.len() < bpb {
                    break;
                }
                let scale = f16_to_f32(u16::from_le_bytes([block[0], block[1]]));
                let min = f16_to_f32(u16::from_le_bytes([block[2], block[3]]));
                for (i, &byte) in block[4..20].iter().enumerate() {
                    let lo = (byte & 0x0F) as f32;
                    let hi = ((byte >> 4) & 0x0F) as f32;
                    let i0 = bi * block_size + i * 2;
                    let i1 = i0 + 1;
                    if i0 < n_elements {
                        out[i0] = lo * scale + min;
                    }
                    if i1 < n_elements {
                        out[i1] = hi * scale + min;
                    }
                }
            }
            Ok(out.iter().flat_map(|f| f.to_le_bytes()).collect())
        }
        DType::Q5_0 => {
            let (block_size, bpb) = (32usize, 22usize);
            let mut out = vec![0f32; n_elements];
            for (bi, block) in data.chunks(bpb).enumerate() {
                if block.len() < bpb {
                    break;
                }
                let scale = f16_to_f32(u16::from_le_bytes([block[0], block[1]]));
                let qh = u32::from_le_bytes([block[2], block[3], block[4], block[5]]);
                for (i, &byte) in block[6..22].iter().enumerate() {
                    let lo = (byte & 0x0F) as i32 + (((qh >> (i * 2)) & 1) << 4) as i32 - 16;
                    let hi =
                        ((byte >> 4) & 0x0F) as i32 + (((qh >> (i * 2 + 1)) & 1) << 4) as i32 - 16;
                    let i0 = bi * block_size + i * 2;
                    let i1 = i0 + 1;
                    if i0 < n_elements {
                        out[i0] = lo as f32 * scale;
                    }
                    if i1 < n_elements {
                        out[i1] = hi as f32 * scale;
                    }
                }
            }
            Ok(out.iter().flat_map(|f| f.to_le_bytes()).collect())
        }
        DType::Q5_1 => {
            let (block_size, bpb) = (32usize, 24usize);
            let mut out = vec![0f32; n_elements];
            for (bi, block) in data.chunks(bpb).enumerate() {
                if block.len() < bpb {
                    break;
                }
                let scale = f16_to_f32(u16::from_le_bytes([block[0], block[1]]));
                let min = f16_to_f32(u16::from_le_bytes([block[2], block[3]]));
                let qh = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);
                for (i, &byte) in block[8..24].iter().enumerate() {
                    let lo = ((byte & 0x0F) + (((qh >> (i * 2)) & 1) << 4) as u8) as f32;
                    let hi = (((byte >> 4) & 0x0F) + (((qh >> (i * 2 + 1)) & 1) << 4) as u8) as f32;
                    let i0 = bi * block_size + i * 2;
                    let i1 = i0 + 1;
                    if i0 < n_elements {
                        out[i0] = lo * scale + min;
                    }
                    if i1 < n_elements {
                        out[i1] = hi * scale + min;
                    }
                }
            }
            Ok(out.iter().flat_map(|f| f.to_le_bytes()).collect())
        }
        DType::Q8_0 => {
            let (block_size, bpb) = (32usize, 34usize);
            let mut out = vec![0f32; n_elements];
            for (bi, block) in data.chunks(bpb).enumerate() {
                if block.len() < bpb {
                    break;
                }
                let scale = f16_to_f32(u16::from_le_bytes([block[0], block[1]]));
                for (i, &byte) in block[2..34].iter().enumerate() {
                    let idx = bi * block_size + i;
                    if idx < n_elements {
                        out[idx] = (byte as i8) as f32 * scale;
                    }
                }
            }
            Ok(out.iter().flat_map(|f| f.to_le_bytes()).collect())
        }
        // ── K-quant types (delegate) ──────────────────────────────────────
        _ => kquant_to_f32_bytes(data, dtype, n_elements),
    }
}

/// Dequantize any K-quant tensor to F32 bytes, dispatching by DType.
pub fn kquant_to_f32_bytes(
    data: &[u8],
    dtype: &DType,
    n_elements: usize,
) -> Result<Vec<u8>, UmcError> {
    let floats: Vec<f32> = match dtype {
        DType::Q2K => dequantize_q2_k(data, n_elements),
        DType::Q3KS | DType::Q3KM | DType::Q3KL => dequantize_q3_k(data, n_elements),
        DType::Q4KS | DType::Q4KM => dequantize_q4_k(data, n_elements),
        DType::Q5KS | DType::Q5KM => dequantize_q5_k(data, n_elements),
        DType::Q6K => dequantize_q6_k(data, n_elements),
        DType::Q8K => dequantize_q8_k(data, n_elements),
        _ => {
            return Err(UmcError::Other(format!(
                "kquant_to_f32: {:?} is not a supported K-quant type",
                dtype
            )))
        }
    };
    Ok(floats.iter().flat_map(|f| f.to_le_bytes()).collect())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_q4_k_block_zeros() -> Vec<u8> {
        // d=1.0, dmin=0.0, all 8 groups have sc=1, mn=0.
        // scales layout (indices into scales[0..12], which are block[4..16]):
        //   groups 0-3: sc = scales[j]&63,         mn = scales[j+4]&63
        //   groups 4-7: sc = (scales[j+4]&0xF) | ((scales[j-4]>>6)<<4)
        //               mn = (scales[j+4]>>4)   | ((scales[j]  >>6)<<4)
        // For sc=1, mn=0 in all groups:
        //   scales[0..3]=1  → groups 0-3 sc=1 (no bits 6-7 set on these)
        //   scales[4..7]=0  → mn=0 for groups 0-3
        //   scales[8..11]=1 → groups 4-7 sc = (1&0xF)|(0) = 1
        let mut block = vec![0u8; Q4_K_BLOCK_BYTES];
        block[0] = 0x00;
        block[1] = 0x3C; // d  = 1.0 f16 LE
        block[2] = 0x00;
        block[3] = 0x00; // dmin = 0.0
                         // scales[0..3]: sc for groups 0-3
        block[4] = 1;
        block[5] = 1;
        block[6] = 1;
        block[7] = 1;
        // scales[4..7]: mn for groups 0-3 (0 = already zero)
        // scales[8..11]: sc for groups 4-7
        block[12] = 1;
        block[13] = 1;
        block[14] = 1;
        block[15] = 1;
        block
    }

    #[test]
    fn test_q4_k_all_zero_quants() {
        // all qs = 0 → all values = scale*0 - min = -min = 0 (since min=0)
        let block = make_q4_k_block_zeros();
        let floats = dequantize_q4_k(&block, Q4_K_BLOCK_SIZE);
        assert_eq!(floats.len(), Q4_K_BLOCK_SIZE);
        for &v in &floats {
            assert_eq!(v, 0.0, "Expected 0.0 for zero quants");
        }
    }

    #[test]
    fn test_q4_k_max_quants() {
        let mut block = make_q4_k_block_zeros();
        // Set all qs = 0xFF → lo nibble=15, hi nibble=15
        for i in 16..144 {
            block[i] = 0xFF;
        }
        // d=1.0, scale[0]=1 → value = 1.0 * 15 - 0 = 15.0
        let floats = dequantize_q4_k(&block, Q4_K_BLOCK_SIZE);
        for &v in &floats {
            assert!((v - 15.0).abs() < 1e-4, "Expected ~15.0, got {}", v);
        }
    }

    #[test]
    fn test_q6_k_zero_quants() {
        // All ql=0x88, qh=0xAA → q0=(8|0)-32=-24, q1=(8|0)-32=-24 with d=1.0
        let block = vec![0u8; Q6_K_BLOCK_BYTES * 2];
        let floats = dequantize_q6_k(&block, Q6_K_BLOCK_SIZE);
        assert_eq!(floats.len(), Q6_K_BLOCK_SIZE);
        // d=0.0 → all zeros
        for &v in &floats {
            assert_eq!(v, 0.0);
        }
    }

    #[test]
    fn test_q8_k_identity() {
        let mut block = vec![0u8; Q8_K_BLOCK_BYTES];
        // d = 1.0f64
        let d_bytes = 1.0f64.to_le_bytes();
        block[0..8].copy_from_slice(&d_bytes);
        // qs[i] = i as i8 for i in 0..256 (cast as u8)
        for i in 0..256usize {
            block[8 + i] = (i as i8) as u8;
        }
        let floats = dequantize_q8_k(&block, Q8_K_BLOCK_SIZE);
        for (i, &v) in floats.iter().enumerate() {
            let expected = (i as i8) as f32;
            assert!(
                (v - expected).abs() < 1e-6,
                "i={} expected {} got {}",
                i,
                expected,
                v
            );
        }
    }

    #[test]
    fn test_q2_k_zero_d() {
        let block = vec![0u8; Q2_K_BLOCK_BYTES];
        let floats = dequantize_q2_k(&block, Q2_K_BLOCK_SIZE);
        for &v in &floats {
            assert_eq!(v, 0.0);
        }
    }
}
