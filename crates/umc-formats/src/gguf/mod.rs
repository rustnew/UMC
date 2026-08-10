mod dtype_map;
pub(crate) mod kquant;
mod reader;
mod saver;
/// GGUF format loader/saver — fully native Rust, zero external dependencies.
///
/// Supports GGUF v1, v2, v3 (read) and v3 (write).
/// Spec: https://github.com/ggerganov/ggml/blob/master/docs/gguf.md
mod spec;

pub use reader::GgufLoader;
pub use saver::GgufSaver;
