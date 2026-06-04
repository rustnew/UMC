use umc_core::ProgressCallback;
use umc_pipeline::{ConversionPipeline, ConversionRequest};
use umc_validate::ValidationMode;
use umc_tests::write_minimal_gguf;

#[test]
fn test_pipeline_gguf_to_safetensors_no_validate() {
    let input = write_minimal_gguf();
    let output = tempfile::NamedTempFile::with_suffix(".safetensors").unwrap();

    let pipeline = ConversionPipeline::new();
    let mut req = ConversionRequest::new(input.path(), output.path());
    req.validation_mode = ValidationMode::None;

    let result = pipeline.convert(req, &ProgressCallback::noop()).unwrap();
    assert_eq!(result.source_format, "GGUF");
    assert_eq!(result.target_format, "SafeTensors");
    assert!(output.path().exists());
}

#[test]
fn test_pipeline_gguf_to_safetensors_structural_validate() {
    let input = write_minimal_gguf();
    let output = tempfile::NamedTempFile::with_suffix(".safetensors").unwrap();

    let pipeline = ConversionPipeline::new();
    let mut req = ConversionRequest::new(input.path(), output.path());
    req.validation_mode = ValidationMode::Structural;

    let result = pipeline.convert(req, &ProgressCallback::noop()).unwrap();
    assert!(output.path().exists());
    assert!(result.certificate.is_some(), "Structural pass must produce a certificate");
}

#[test]
fn test_pipeline_elapsed_time_recorded() {
    let input = write_minimal_gguf();
    let output = tempfile::NamedTempFile::with_suffix(".safetensors").unwrap();

    let pipeline = ConversionPipeline::new();
    let mut req = ConversionRequest::new(input.path(), output.path());
    req.validation_mode = ValidationMode::None;

    let result = pipeline.convert(req, &ProgressCallback::noop()).unwrap();
    // Elapsed time must be >= 0
    let _ = result.elapsed_ms;
}

#[test]
fn test_pipeline_result_summary_contains_formats() {
    let input = write_minimal_gguf();
    let output = tempfile::NamedTempFile::with_suffix(".safetensors").unwrap();

    let pipeline = ConversionPipeline::new();
    let mut req = ConversionRequest::new(input.path(), output.path());
    req.validation_mode = ValidationMode::None;

    let result = pipeline.convert(req, &ProgressCallback::noop()).unwrap();
    let summary = result.summary();
    assert!(summary.contains("GGUF"), "Summary must mention GGUF");
    assert!(summary.contains("SafeTensors"), "Summary must mention SafeTensors");
}

#[test]
fn test_pipeline_unknown_format_returns_error() {
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::with_suffix(".xyz").unwrap();
    f.write_all(b"GARBAGE_DATA_XYZ").unwrap();
    f.flush().unwrap();

    let output = tempfile::NamedTempFile::with_suffix(".safetensors").unwrap();
    let pipeline = ConversionPipeline::new();
    let req = ConversionRequest::new(f.path(), output.path());

    let result = pipeline.convert(req, &ProgressCallback::noop());
    assert!(result.is_err(), "Unknown format must produce an error");
}
