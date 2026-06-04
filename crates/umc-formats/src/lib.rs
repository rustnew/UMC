/// UMC native format implementations.
/// Native: GGUF, SafeTensors, ONNX, PyTorch, AWQ, GPTQ, TFLite, LoRA.
/// External (subprocess wrappers): CoreML, TensorRT, OpenVINO, ExecuTorch.

pub mod gguf;
pub mod safetensors;
pub mod onnx;
pub mod pytorch;
pub mod awq;
pub mod gptq;
pub mod tflite;
pub mod external;
pub mod lora;

pub use gguf::{GgufLoader, GgufSaver};
pub use safetensors::{SafeTensorsSaver, SafeTensorsLoader};
pub use onnx::{OnnxLoader, OnnxSaver};
pub use pytorch::{PyTorchLoader, PyTorchSaver};
pub use awq::{AwqLoader, AwqSaver};
pub use gptq::{GptqLoader, GptqSaver};
pub use tflite::{TFLiteLoader, TFLiteSaver};
pub use external::{CoreMLSaver, TensorRTSaver, OpenVINOSaver, ExecuTorchSaver};
pub use lora::LoraLoader;

use umc_core::{FormatLoader, FormatSaver};

/// Return all built-in loaders.
pub fn all_loaders() -> Vec<Box<dyn FormatLoader>> {
    vec![
        Box::new(GgufLoader),
        Box::new(SafeTensorsLoader),
        Box::new(OnnxLoader),
        Box::new(PyTorchLoader),
        Box::new(AwqLoader),
        Box::new(GptqLoader),
        Box::new(TFLiteLoader),
        Box::new(LoraLoader),
    ]
}

/// Return all built-in savers.
pub fn all_savers() -> Vec<Box<dyn FormatSaver>> {
    vec![
        Box::new(GgufSaver),
        Box::new(SafeTensorsSaver),
        Box::new(OnnxSaver),
        Box::new(PyTorchSaver),
        Box::new(AwqSaver),
        Box::new(GptqSaver),
        Box::new(TFLiteSaver),
        Box::new(CoreMLSaver),
        Box::new(TensorRTSaver),
        Box::new(OpenVINOSaver),
        Box::new(ExecuTorchSaver),
    ]
}
