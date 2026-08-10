/// External format converters — delegate to installed CLI tools.
/// These savers run ONNX as the intermediate format and invoke external tools via subprocess.
/// Graph weight in ConversionGraph = 2.0 (non-native).
pub mod coreml;
pub mod executorch;
pub mod openvino;
pub mod tensorrt;

pub use coreml::CoreMLSaver;
pub use executorch::ExecuTorchSaver;
pub use openvino::OpenVINOSaver;
pub use tensorrt::TensorRTSaver;

use std::process::{Command, Output};
use umc_core::UmcError;

/// Run an external command, returning its stdout on success.
pub(crate) fn run_external(program: &str, args: &[&str]) -> Result<Output, UmcError> {
    let output = Command::new(program).args(args).output().map_err(|e| {
        UmcError::Other(format!(
            "External tool '{}' not found or failed to start: {}. \
             Please install it and ensure it is in PATH.",
            program, e
        ))
    })?;

    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(UmcError::Other(format!(
            "External tool '{}' failed (exit {}): {}",
            program,
            output.status,
            stderr.trim()
        )))
    }
}

/// Check if an external tool is available in PATH.
pub fn tool_available(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
