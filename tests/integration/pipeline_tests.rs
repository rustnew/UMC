use std::io::Write;
use tempfile::NamedTempFile;
use umc_core::ProgressCallback;
use umc_pipeline::{ConversionPipeline, ConversionRequest};
use umc_validate::ValidationMode;

fn write_minimal_gguf() -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".gguf").unwrap();
    f.write_all(b"GGUF").unwrap();
    f.write_all(&3u32.to_le_bytes()).unwrap();
    f.write_all(&0u64.to_le_bytes()).unwrap();
    f.write_all(&0u64.to_le_bytes()).unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn test_pipeline_gguf_to_safetensors_no_validate() {
    let input = write_minimal_gguf();
    let output = NamedTempFile::with_suffix(".safetensors").unwrap();

    let pipeline = ConversionPipeline::new();
    let mut req = ConversionRequest::new(input.path(), output.path());
    req.validation_mode = ValidationMode::None;

    let result = pipeline.convert(req, &ProgressCallback::noop()).unwrap();
    assert_eq!(result.source_format, "GGUF");
    assert_eq!(result.target_format, "SafeTensors");
    assert!(output.path().exists());
    assert_eq!(result.tensor_count, 0);
}

#[test]
fn test_pipeline_gguf_to_safetensors_with_structural_validation() {
    let input = write_minimal_gguf();
    let output = NamedTempFile::with_suffix(".safetensors").unwrap();

    let pipeline = ConversionPipeline::new();
    let mut req = ConversionRequest::new(input.path(), output.path());
    req.validation_mode = ValidationMode::Structural;

    let result = pipeline.convert(req, &ProgressCallback::noop()).unwrap();
    assert!(output.path().exists());
    assert!(result.certificate.is_some(), "Structural validation passed → certificate expected");
}

#[test]
fn test_pipeline_unknown_format_error() {
    let mut f = NamedTempFile::with_suffix(".xyz").unwrap();
    f.write_all(b"UNKNOWN_FORMAT_DATA").unwrap();
    f.flush().unwrap();

    let output = NamedTempFile::with_suffix(".safetensors").unwrap();
    let pipeline = ConversionPipeline::new();
    let req = ConversionRequest::new(f.path(), output.path());

    let result = pipeline.convert(req, &ProgressCallback::noop());
    assert!(result.is_err(), "Unknown format should produce an error");
}

#[test]
fn test_pipeline_result_summary() {
    let input = write_minimal_gguf();
    let output = NamedTempFile::with_suffix(".safetensors").unwrap();

    let pipeline = ConversionPipeline::new();
    let mut req = ConversionRequest::new(input.path(), output.path());
    req.validation_mode = ValidationMode::None;

    let result = pipeline.convert(req, &ProgressCallback::noop()).unwrap();
    let summary = result.summary();
    assert!(summary.contains("GGUF"));
    assert!(summary.contains("SafeTensors"));
}

#[test]
fn test_pipeline_output_file_created() {
    let input = write_minimal_gguf();
    let output = NamedTempFile::with_suffix(".safetensors").unwrap();
    let output_path = output.path().to_path_buf();

    let pipeline = ConversionPipeline::new();
    let mut req = ConversionRequest::new(input.path(), &output_path);
    req.validation_mode = ValidationMode::None;

    pipeline.convert(req, &ProgressCallback::noop()).unwrap();
    assert!(output_path.exists(), "Output file must be created");
    assert!(std::fs::metadata(&output_path).unwrap().len() > 0, "Output file must not be empty");
}
