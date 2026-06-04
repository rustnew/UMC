/// GGUF format loader/saver — fully native Rust, zero external dependencies.
///
/// Supports GGUF v1, v2, v3 (read) and v3 (write).
/// Spec: https://github.com/ggerganov/ggml/blob/master/docs/gguf.md

mod spec;
mod reader;
mod saver;
mod dtype_map;
pub(crate) mod kquant;

pub use reader::GgufLoader;
pub use saver::GgufSaver;
