/// Minimal ONNX protobuf message definitions, written with prost annotations.
/// Covers the subset needed for loading/saving weight tensors (opset 21).
/// Reference: https://github.com/onnx/onnx/blob/main/onnx/onnx.proto3

use prost::Message;

// ── TensorProto ───────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Message)]
pub struct TensorProto {
    #[prost(int64, repeated, tag = "1")]
    pub dims: Vec<i64>,

    #[prost(int32, optional, tag = "2")]
    pub data_type: Option<i32>,

    #[prost(string, optional, tag = "8")]
    pub name: Option<String>,

    #[prost(string, optional, tag = "12")]
    pub doc_string: Option<String>,

    /// Raw binary blob — present for all numeric types in well-formed ONNX files.
    #[prost(bytes = "vec", optional, tag = "9")]
    pub raw_data: Option<Vec<u8>>,

    #[prost(float, repeated, packed = "false", tag = "4")]
    pub float_data: Vec<f32>,

    #[prost(int32, repeated, packed = "false", tag = "5")]
    pub int32_data: Vec<i32>,

    #[prost(int64, repeated, packed = "false", tag = "7")]
    pub int64_data: Vec<i64>,

    #[prost(double, repeated, packed = "false", tag = "10")]
    pub double_data: Vec<f64>,

    #[prost(uint64, repeated, packed = "false", tag = "11")]
    pub uint64_data: Vec<u64>,

    #[prost(bytes = "vec", repeated, tag = "6")]
    pub string_data: Vec<Vec<u8>>,
}

/// ONNX TensorProto.DataType enum values.
#[repr(i32)]
#[allow(dead_code)]
pub enum OnnxDataType {
    Undefined = 0,
    Float     = 1,
    Uint8     = 2,
    Int8      = 3,
    Uint16    = 4,
    Int16     = 5,
    Int32     = 6,
    Int64     = 7,
    String    = 8,
    Bool      = 9,
    Float16   = 10,
    Double    = 11,
    Uint32    = 12,
    Uint64    = 13,
    Complex64 = 14,
    Complex128= 15,
    Bfloat16  = 16,
    Float8e4m3fn  = 17,
    Float8e4m3fnuz= 18,
    Float8e5m2    = 19,
    Float8e5m2fnuz= 20,
}

// ── ValueInfoProto / TypeProto (minimal — only shape extraction) ──────────────

#[derive(Clone, PartialEq, Message)]
pub struct TypeProtoTensor {
    #[prost(int32, optional, tag = "1")]
    pub elem_type: Option<i32>,

    #[prost(message, optional, tag = "2")]
    pub shape: Option<TensorShapeProto>,
}

#[derive(Clone, PartialEq, Message)]
pub struct TypeProto {
    #[prost(message, optional, tag = "1")]
    pub tensor_type: Option<TypeProtoTensor>,
}

#[derive(Clone, PartialEq, Message)]
pub struct TensorShapeProtoDimension {
    /// Static (integer) dimension value; -1 for dynamic.
    #[prost(int64, optional, tag = "1")]
    pub dim_value: Option<i64>,
    /// Symbolic dimension parameter (for dynamic shapes).
    #[prost(string, optional, tag = "2")]
    pub dim_param: Option<String>,
}

#[derive(Clone, PartialEq, Message)]
pub struct TensorShapeProto {
    #[prost(message, repeated, tag = "1")]
    pub dim: Vec<TensorShapeProtoDimension>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ValueInfoProto {
    #[prost(string, optional, tag = "1")]
    pub name: Option<String>,

    #[prost(message, optional, tag = "2")]
    pub r#type: Option<TypeProto>,
}

// ── AttributeProto ────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Message)]
pub struct AttributeProto {
    #[prost(string, optional, tag = "1")]
    pub name: Option<String>,

    #[prost(float, optional, tag = "4")]
    pub f: Option<f32>,

    #[prost(int64, optional, tag = "3")]
    pub i: Option<i64>,

    #[prost(bytes = "vec", optional, tag = "5")]
    pub s: Option<Vec<u8>>,

    #[prost(message, optional, tag = "6")]
    pub t: Option<TensorProto>,

    #[prost(float, repeated, packed = "false", tag = "7")]
    pub floats: Vec<f32>,

    #[prost(int64, repeated, packed = "false", tag = "8")]
    pub ints: Vec<i64>,

    #[prost(bytes = "vec", repeated, tag = "9")]
    pub strings: Vec<Vec<u8>>,

    #[prost(message, repeated, tag = "10")]
    pub tensors: Vec<TensorProto>,

    #[prost(message, repeated, tag = "11")]
    pub graphs: Vec<GraphProto>,

    #[prost(int32, optional, tag = "20")]
    pub r#type: Option<i32>,
}

// ── NodeProto ─────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Message)]
pub struct NodeProto {
    #[prost(string, repeated, tag = "1")]
    pub input: Vec<String>,

    #[prost(string, repeated, tag = "2")]
    pub output: Vec<String>,

    #[prost(string, optional, tag = "3")]
    pub name: Option<String>,

    #[prost(string, optional, tag = "4")]
    pub op_type: Option<String>,

    #[prost(string, optional, tag = "7")]
    pub domain: Option<String>,

    #[prost(message, repeated, tag = "5")]
    pub attribute: Vec<AttributeProto>,
}

// ── StringStringEntryProto ────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Message)]
pub struct StringStringEntryProto {
    #[prost(string, optional, tag = "1")]
    pub key: Option<String>,

    #[prost(string, optional, tag = "2")]
    pub value: Option<String>,
}

// ── OperatorSetIdProto ────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Message)]
pub struct OperatorSetIdProto {
    #[prost(string, optional, tag = "1")]
    pub domain: Option<String>,

    #[prost(int64, optional, tag = "2")]
    pub version: Option<i64>,
}

// ── GraphProto ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Message)]
pub struct GraphProto {
    #[prost(message, repeated, tag = "1")]
    pub node: Vec<NodeProto>,

    #[prost(string, optional, tag = "2")]
    pub name: Option<String>,

    #[prost(message, repeated, tag = "5")]
    pub initializer: Vec<TensorProto>,

    #[prost(message, repeated, tag = "11")]
    pub input: Vec<ValueInfoProto>,

    #[prost(message, repeated, tag = "12")]
    pub output: Vec<ValueInfoProto>,

    #[prost(message, repeated, tag = "13")]
    pub value_info: Vec<ValueInfoProto>,
}

// ── ModelProto ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Message)]
pub struct ModelProto {
    #[prost(int64, optional, tag = "1")]
    pub ir_version: Option<i64>,

    #[prost(message, repeated, tag = "8")]
    pub opset_import: Vec<OperatorSetIdProto>,

    #[prost(string, optional, tag = "2")]
    pub domain: Option<String>,

    #[prost(int64, optional, tag = "5")]
    pub model_version: Option<i64>,

    #[prost(string, optional, tag = "6")]
    pub doc_string: Option<String>,

    #[prost(message, optional, tag = "7")]
    pub graph: Option<GraphProto>,

    #[prost(message, repeated, tag = "14")]
    pub metadata_props: Vec<StringStringEntryProto>,
}
