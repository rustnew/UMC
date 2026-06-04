use umc_core::DType;
use super::proto::OnnxDataType;

/// Map ONNX TensorProto.DataType (i32) to UMC DType.
pub fn onnx_dtype_to_umc(onnx_type: i32) -> Option<DType> {
    match onnx_type {
        x if x == OnnxDataType::Float     as i32 => Some(DType::F32),
        x if x == OnnxDataType::Double    as i32 => Some(DType::F64),
        x if x == OnnxDataType::Float16   as i32 => Some(DType::F16),
        x if x == OnnxDataType::Bfloat16  as i32 => Some(DType::BF16),
        x if x == OnnxDataType::Int8      as i32 => Some(DType::I8),
        x if x == OnnxDataType::Int16     as i32 => Some(DType::I16),
        x if x == OnnxDataType::Int32     as i32 => Some(DType::I32),
        x if x == OnnxDataType::Int64     as i32 => Some(DType::I64),
        x if x == OnnxDataType::Uint8     as i32 => Some(DType::U8),
        x if x == OnnxDataType::Uint16    as i32 => Some(DType::U16),
        x if x == OnnxDataType::Uint32    as i32 => Some(DType::U32),
        x if x == OnnxDataType::Uint64    as i32 => Some(DType::U64),
        x if x == OnnxDataType::Bool      as i32 => Some(DType::Bool),
        x if x == OnnxDataType::Float8e4m3fn   as i32 => Some(DType::F8E4M3),
        x if x == OnnxDataType::Float8e5m2     as i32 => Some(DType::F8E5M2),
        _ => None,
    }
}

/// Map UMC DType to ONNX TensorProto.DataType (i32).
pub fn umc_dtype_to_onnx(dtype: &DType) -> Option<i32> {
    match dtype {
        DType::F32  => Some(OnnxDataType::Float     as i32),
        DType::F64  => Some(OnnxDataType::Double    as i32),
        DType::F16  => Some(OnnxDataType::Float16   as i32),
        DType::BF16 => Some(OnnxDataType::Bfloat16  as i32),
        DType::I8   => Some(OnnxDataType::Int8      as i32),
        DType::I16  => Some(OnnxDataType::Int16     as i32),
        DType::I32  => Some(OnnxDataType::Int32     as i32),
        DType::I64  => Some(OnnxDataType::Int64     as i32),
        DType::U8   => Some(OnnxDataType::Uint8     as i32),
        DType::U16  => Some(OnnxDataType::Uint16    as i32),
        DType::U32  => Some(OnnxDataType::Uint32    as i32),
        DType::U64  => Some(OnnxDataType::Uint64    as i32),
        DType::Bool => Some(OnnxDataType::Bool      as i32),
        DType::F8E4M3 => Some(OnnxDataType::Float8e4m3fn as i32),
        DType::F8E5M2 => Some(OnnxDataType::Float8e5m2   as i32),
        _ => None, // quantized types not natively representable in ONNX
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_f32() {
        let onnx = umc_dtype_to_onnx(&DType::F32).unwrap();
        assert_eq!(onnx_dtype_to_umc(onnx), Some(DType::F32));
    }

    #[test]
    fn test_unknown_onnx_type() {
        assert!(onnx_dtype_to_umc(99).is_none());
    }
}
