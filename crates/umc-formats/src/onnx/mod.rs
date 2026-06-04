/// ONNX format loader and saver — fully native Rust via prost protobuf.
/// Supports opset 1-21, ONNX IR version 3-10.
/// Spec: https://github.com/onnx/onnx/blob/main/onnx/onnx.proto3

pub mod proto;
mod dtype_map;
mod loader;
mod saver;

pub use loader::OnnxLoader;
pub use saver::OnnxSaver;
