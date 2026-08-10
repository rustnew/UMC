use super::run_external;
/// CoreML saver — converts ONNX → CoreML .mlpackage via coremltools Python package.
/// Requires: pip install coremltools
use std::path::Path;
use umc_core::{FormatSaver, ProgressCallback, SaveOptions};
use umc_core::{UmcError, UniversalIR};

pub struct CoreMLSaver;

impl FormatSaver for CoreMLSaver {
    fn format_name(&self) -> &'static str {
        "CoreML"
    }
    fn default_extension(&self) -> &'static str {
        "mlpackage"
    }

    fn save(
        &self,
        ir: &UniversalIR,
        path: &Path,
        opts: &SaveOptions,
        progress: &ProgressCallback,
    ) -> Result<(), UmcError> {
        progress.report("CoreML: saving via ONNX intermediate + coremltools");

        // Step 1: Save IR as ONNX to a temp file
        let onnx_tmp = path.with_extension("_umc_tmp.onnx");
        let onnx_saver = crate::onnx::OnnxSaver;
        onnx_saver.save(ir, &onnx_tmp, opts, progress)?;

        // Step 2: Convert ONNX → CoreML via Python script
        let script = format!(
            "import coremltools as ct; \
             model = ct.convert('{}', convert_to='mlprogram'); \
             model.save('{}')",
            onnx_tmp.display(),
            path.display()
        );

        let result = run_external("python3", &["-c", &script]);
        let _ = std::fs::remove_file(&onnx_tmp);

        result.map(|_| ())?;
        progress.report("CoreML: conversion complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coreml_tool_check() {
        // CoreML tools may not be available; just ensure graceful error
        let result = run_external("python3", &["-c", "import coremltools"]);
        // Either succeeds or returns a descriptive error — no panic
        let _ = result;
    }
}
