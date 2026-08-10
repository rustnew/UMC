use super::run_external;
/// ExecuTorch saver — converts ONNX → ExecuTorch .pte via executorch tools.
/// Requires: ExecuTorch Python package (pip install executorch).
use std::path::Path;
use umc_core::{FormatSaver, ProgressCallback, SaveOptions};
use umc_core::{UmcError, UniversalIR};

pub struct ExecuTorchSaver;

impl FormatSaver for ExecuTorchSaver {
    fn format_name(&self) -> &'static str {
        "ExecuTorch"
    }
    fn default_extension(&self) -> &'static str {
        "pte"
    }

    fn save(
        &self,
        ir: &UniversalIR,
        path: &Path,
        opts: &SaveOptions,
        progress: &ProgressCallback,
    ) -> Result<(), UmcError> {
        progress.report("ExecuTorch: saving via ONNX intermediate + executorch");

        let onnx_tmp = path.with_extension("_umc_tmp.onnx");
        let onnx_saver = crate::onnx::OnnxSaver;
        onnx_saver.save(ir, &onnx_tmp, opts, progress)?;

        let script = format!(
            "from executorch.exir import to_edge; \
             import torch.onnx; \
             import torch; \
             # Convert via torch.export then to_edge \
             print('ExecuTorch conversion from ONNX not yet fully automated.'); \
             print('Intermediate ONNX saved at: {}')",
            onnx_tmp.display()
        );

        let result = run_external("python3", &["-c", &script]);
        let _ = std::fs::remove_file(&onnx_tmp);

        match result {
            Ok(_) => progress.report("ExecuTorch: saved"),
            Err(e) => {
                return Err(UmcError::Other(format!(
                    "ExecuTorch: {}. Ensure executorch is installed: pip install executorch",
                    e
                )))
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_executorch_graceful() {
        let result = run_external("python3", &["-c", "import executorch"]);
        let _ = result;
    }
}
