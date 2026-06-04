/// SafeTensors format — native Rust loader and saver.
/// Spec: https://github.com/huggingface/safetensors

mod saver;
mod loader;

pub use saver::SafeTensorsSaver;
pub use loader::SafeTensorsLoader;
