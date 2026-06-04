/// OpenVINO saver — converts ONNX → OpenVINO IR (.xml + .bin) via Model Optimizer (mo).
/// Requires: OpenVINO toolkit. Tool may be `mo`, `mo.py`, or `openvino-mo`.
use std::path::Path;
use umc_core::{UmcError, UniversalIR};
use umc_core::{FormatSaver, SaveOptions, ProgressCallback};
use super::run_external;

pub struct OpenVINOSaver;

impl FormatSaver for OpenVINOSaver {
    fn format_name(&self) -> &'static str { "OpenVINO" }
    fn default_extension(&self) -> &'static str { "xml" }

    fn save(&self, ir: &UniversalIR, path: &Path, opts: &SaveOptions, progress: &ProgressCallback)
        -> Result<(), UmcError>
    {
        progress.report("OpenVINO: saving via ONNX intermediate + model optimizer");

        let onnx_tmp = path.with_extension("_umc_tmp.onnx");
        let onnx_saver = crate::onnx::OnnxSaver;
        onnx_saver.save(ir, &onnx_tmp, opts, progress)?;

        let output_dir = path.parent().unwrap_or(std::path::Path::new("."));
        let model_name = path.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "model".to_string());

        // Try 'mo' first, then 'openvino-mo'
        let tool = if run_external("mo", &["--help"]).is_ok() { "mo" } else { "openvino-mo" };

        let onnx_path = onnx_tmp.to_string_lossy().into_owned();
        let out_dir = output_dir.to_string_lossy().into_owned();
        let args = [
            "--input_model", &onnx_path,
            "--output_dir", &out_dir,
            "--model_name", &model_name,
        ];
        let result = run_external(tool, &args);
        let _ = std::fs::remove_file(&onnx_tmp);

        result.map(|_| ())?;
        progress.report("OpenVINO: IR saved (.xml + .bin)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_openvino_tool_absent() {
        let result = run_external("mo", &["--help"]);
        let _ = result; // Don't panic whether installed or not
    }
}
