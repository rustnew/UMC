use super::run_external;
/// TensorRT saver — converts ONNX → TensorRT .engine via trtexec.
/// Requires: TensorRT toolkit with trtexec in PATH.
use std::path::Path;
use umc_core::{FormatSaver, ProgressCallback, SaveOptions};
use umc_core::{UmcError, UniversalIR};

pub struct TensorRTSaver;

impl FormatSaver for TensorRTSaver {
    fn format_name(&self) -> &'static str {
        "TensorRT"
    }
    fn default_extension(&self) -> &'static str {
        "engine"
    }

    fn save(
        &self,
        ir: &UniversalIR,
        path: &Path,
        opts: &SaveOptions,
        progress: &ProgressCallback,
    ) -> Result<(), UmcError> {
        progress.report("TensorRT: saving via ONNX intermediate + trtexec");

        // Save as ONNX first
        let onnx_tmp = path.with_extension("_umc_tmp.onnx");
        let onnx_saver = crate::onnx::OnnxSaver;
        onnx_saver.save(ir, &onnx_tmp, opts, progress)?;

        // Determine precision flags
        let precision = ir.metadata.get_str("tensorrt.precision").unwrap_or("fp32");
        let mut args = vec![
            format!("--onnx={}", onnx_tmp.display()),
            format!("--saveEngine={}", path.display()),
        ];
        match precision {
            "fp16" => args.push("--fp16".to_string()),
            "int8" => {
                args.push("--fp16".to_string());
                args.push("--int8".to_string());
            }
            _ => {}
        }
        let arg_strs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let result = run_external("trtexec", &arg_strs);
        let _ = std::fs::remove_file(&onnx_tmp);

        result.map(|_| ())?;
        progress.report("TensorRT: engine saved");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_tensorrt_graceful_missing_tool() {
        // trtexec is likely not available; just verify the error is descriptive
        let result = run_external("trtexec", &["--help"]);
        match result {
            Ok(_) => {} // installed, fine
            Err(e) => {
                assert!(e.to_string().contains("not found") || e.to_string().contains("failed"))
            }
        }
    }
}
