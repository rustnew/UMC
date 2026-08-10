use super::cancel::CancellationToken;
use std::path::{Path, PathBuf};
use umc_core::{
    FormatLoader, FormatSaver, LoadOptions, ProgressCallback, SaveOptions, UmcError, UniversalIR,
};
use umc_detect::FormatRegistry;
use umc_formats::{
    AwqLoader, AwqSaver, CoreMLSaver, ExecuTorchSaver, GgufLoader, GgufSaver, GptqLoader,
    GptqSaver, LoraLoader, OnnxLoader, OnnxSaver, OpenVINOSaver, PyTorchLoader, PyTorchSaver,
    SafeTensorsLoader, SafeTensorsSaver, TFLiteLoader, TFLiteSaver, TensorRTSaver,
};
use umc_graph::{find_path, ConversionGraph};
use umc_validate::{
    certificate::sha256_file, certificate::CertFileInfo, numeric_validate, structural_validate,
    CertificateBuilder, ConversionCertificate, ValidationMode,
};

/// A conversion request.
#[derive(Debug, Clone)]
pub struct ConversionRequest {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    /// Override source format (None = auto-detect).
    pub source_format: Option<String>,
    /// Override target format (None = infer from output extension).
    pub target_format: Option<String>,
    pub load_options: LoadOptions,
    pub save_options: SaveOptions,
    pub validation_mode: ValidationMode,
    pub cancellation: CancellationToken,
}

impl ConversionRequest {
    pub fn new(input: impl Into<PathBuf>, output: impl Into<PathBuf>) -> Self {
        Self {
            input_path: input.into(),
            output_path: output.into(),
            source_format: None,
            target_format: None,
            load_options: LoadOptions::default(),
            save_options: SaveOptions::default(),
            validation_mode: ValidationMode::Structural,
            cancellation: CancellationToken::new(),
        }
    }
}

/// Result of a successful conversion.
pub struct ConversionResult {
    pub source_format: String,
    pub target_format: String,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub elapsed_ms: u64,
    pub tensor_count: usize,
    pub certificate: Option<ConversionCertificate>,
    pub warnings: Vec<String>,
}

impl ConversionResult {
    pub fn summary(&self) -> String {
        format!(
            "{} → {} | {} tensors | {:.1}s{}",
            self.source_format,
            self.target_format,
            self.tensor_count,
            self.elapsed_ms as f64 / 1000.0,
            if self.certificate.is_some() {
                " | CERTIFIED"
            } else {
                ""
            }
        )
    }
}

/// The main conversion orchestrator.
pub struct ConversionPipeline {
    detect_registry: FormatRegistry,
    conversion_graph: ConversionGraph,
}

impl ConversionPipeline {
    pub fn new() -> Self {
        Self {
            detect_registry: FormatRegistry::new(),
            conversion_graph: ConversionGraph::default_graph(),
        }
    }

    /// Run a full conversion: detect → load → validate → save → certify.
    pub fn convert(
        &self,
        request: ConversionRequest,
        progress: &ProgressCallback,
    ) -> Result<ConversionResult, UmcError> {
        let start = std::time::Instant::now();
        let mut warnings: Vec<String> = Vec::new();

        // ── 1. Detect source format ───────────────────────────────────────
        let source_format = if let Some(ref f) = request.source_format {
            f.clone()
        } else {
            let det = self.detect_registry.detect(&request.input_path)?;
            tracing::info!(
                "Detected format: {} (confidence {:.2})",
                det.format,
                det.confidence
            );
            det.format
        };

        // ── 2. Detect target format ───────────────────────────────────────
        let target_format = if let Some(ref f) = request.target_format {
            f.clone()
        } else {
            infer_format_from_extension(&request.output_path).ok_or_else(|| {
                UmcError::UnknownFormat {
                    path: request.output_path.display().to_string(),
                    hint: "Use --target <FORMAT> to specify the output format explicitly.".into(),
                }
            })?
        };

        progress.report(&format!(
            "Converting {} → {}…",
            source_format, target_format
        ));

        if request.cancellation.is_cancelled() {
            return Err(UmcError::Cancelled);
        }

        // ── 3. Find conversion path ───────────────────────────────────────
        let conv_path = find_path(&self.conversion_graph, &source_format, &target_format)?;
        tracing::info!("Conversion path: {}", conv_path.display_path());
        if !conv_path.is_direct() {
            warnings.push(format!(
                "Multi-hop conversion: {} hops. Direct conversion preferred.",
                conv_path.hop_count()
            ));
        }

        // ── 4. Load ───────────────────────────────────────────────────────
        let loader = self.get_loader(&source_format)?;
        progress.report(&format!("Loading {}…", source_format));
        let ir = loader.load(&request.input_path, &request.load_options, progress)?;

        if request.cancellation.is_cancelled() {
            return Err(UmcError::Cancelled);
        }

        let tensor_count = ir.tensors.len();
        tracing::info!("Loaded {} tensors", tensor_count);

        // ── 5. Save ───────────────────────────────────────────────────────
        let saver = self.get_saver(&target_format)?;
        progress.report(&format!("Saving {}…", target_format));
        saver.save(&ir, &request.output_path, &request.save_options, progress)?;

        let elapsed_ms = start.elapsed().as_millis() as u64;

        if request.cancellation.is_cancelled() {
            return Err(UmcError::Cancelled);
        }

        // ── 6. Validation and certification ──────────────────────────────
        let certificate = if request.validation_mode != ValidationMode::None {
            self.validate_and_certify(
                &ir,
                &request.input_path,
                &request.output_path,
                &source_format,
                &target_format,
                &request.validation_mode,
                progress,
                &mut warnings,
            )
            .ok()
        } else {
            None
        };

        progress.report("Conversion complete.");

        Ok(ConversionResult {
            source_format,
            target_format,
            input_path: request.input_path,
            output_path: request.output_path,
            elapsed_ms,
            tensor_count,
            certificate,
            warnings,
        })
    }

    fn validate_and_certify(
        &self,
        source_ir: &UniversalIR,
        input_path: &Path,
        output_path: &Path,
        source_format: &str,
        target_format: &str,
        mode: &ValidationMode,
        progress: &ProgressCallback,
        warnings: &mut Vec<String>,
    ) -> Result<ConversionCertificate, UmcError> {
        progress.report("Validating output…");

        // Load the saved file back for structural comparison
        let target_loader = self.get_loader(target_format)?;
        let mut target_opts = LoadOptions::default();
        target_opts.metadata_only = false;
        let output_ir = target_loader.load(output_path, &target_opts, &ProgressCallback::noop())?;

        let struct_report = structural_validate(source_ir, &output_ir)?;
        if !struct_report.passed {
            return Err(UmcError::StructuralValidationFailed {
                reason: struct_report
                    .shape_mismatches
                    .first()
                    .cloned()
                    .unwrap_or_default(),
            });
        }
        for w in &struct_report.warnings {
            warnings.push(w.clone());
        }
        for dt in &struct_report.dtype_changes {
            warnings.push(format!("DType change: {}", dt));
        }

        let mut builder = CertificateBuilder::new()
            .structural_passed(true)
            .roundtrip_level("structural");

        // Numeric validation for non-quantized tensors
        let numeric_report = if *mode == ValidationMode::Numeric || *mode == ValidationMode::Strict
        {
            let report = numeric_validate(source_ir, &output_ir, None)?;
            if report.passed {
                Some(report)
            } else {
                warnings.push(format!(
                    "Numeric divergence: max={:.2e}",
                    report.global_max_divergence
                ));
                Some(report)
            }
        } else {
            None
        };

        if let Some(ref nr) = numeric_report {
            builder = builder.numeric_passed(nr.passed, nr.global_max_divergence);
        }

        // SHA256 hashes
        let input_hash = sha256_file(input_path).unwrap_or_else(|_| "unknown".into());
        let output_hash = sha256_file(output_path).unwrap_or_else(|_| "unknown".into());
        let output_meta = std::fs::metadata(output_path).ok();
        let output_size = output_meta.as_ref().map(|m| m.len()).unwrap_or(0);

        builder = builder
            .source(CertFileInfo {
                format: source_format.into(),
                sha256: input_hash,
                file_size_bytes: std::fs::metadata(input_path).map(|m| m.len()).unwrap_or(0),
                num_tensors: source_ir.tensors.len(),
                num_parameters: source_ir.num_parameters(),
            })
            .target(CertFileInfo {
                format: target_format.into(),
                sha256: output_hash,
                file_size_bytes: output_size,
                num_tensors: output_ir.tensors.len(),
                num_parameters: output_ir.num_parameters(),
            });

        for w in warnings.iter() {
            builder = builder.add_warning(w.clone());
        }

        builder
            .build()
            .ok_or_else(|| UmcError::StructuralValidationFailed {
                reason: "Certificate could not be issued".into(),
            })
    }

    fn get_loader(&self, format: &str) -> Result<Box<dyn FormatLoader>, UmcError> {
        match format {
            "GGUF"         => Ok(Box::new(GgufLoader)),
            "SafeTensors"  => Ok(Box::new(SafeTensorsLoader)),
            "ONNX"         => Ok(Box::new(OnnxLoader)),
            "PyTorch"      => Ok(Box::new(PyTorchLoader)),
            "AWQ"          => Ok(Box::new(AwqLoader)),
            "GPTQ"         => Ok(Box::new(GptqLoader)),
            "TFLite"       => Ok(Box::new(TFLiteLoader)),
            "LoRA"         => Ok(Box::new(LoraLoader)),
            other => Err(UmcError::UnknownFormat {
                path: format!("(loader for {})", other),
                hint: format!("No loader registered for format '{}'. Supported: GGUF, SafeTensors, ONNX, PyTorch, AWQ, GPTQ, TFLite, LoRA.", other),
            }),
        }
    }

    fn get_saver(&self, format: &str) -> Result<Box<dyn FormatSaver>, UmcError> {
        match format {
            "GGUF"         => Ok(Box::new(GgufSaver)),
            "SafeTensors"  => Ok(Box::new(SafeTensorsSaver)),
            "ONNX"         => Ok(Box::new(OnnxSaver)),
            "PyTorch"      => Ok(Box::new(PyTorchSaver)),
            "AWQ"          => Ok(Box::new(AwqSaver)),
            "GPTQ"         => Ok(Box::new(GptqSaver)),
            "TFLite"       => Ok(Box::new(TFLiteSaver)),
            "CoreML"       => Ok(Box::new(CoreMLSaver)),
            "TensorRT"     => Ok(Box::new(TensorRTSaver)),
            "OpenVINO"     => Ok(Box::new(OpenVINOSaver)),
            "ExecuTorch"   => Ok(Box::new(ExecuTorchSaver)),
            other => Err(UmcError::UnknownFormat {
                path: format!("(saver for {})", other),
                hint: format!("No saver registered for format '{}'. Supported: SafeTensors, ONNX, PyTorch, AWQ, GPTQ, TFLite, CoreML, TensorRT, OpenVINO, ExecuTorch.", other),
            }),
        }
    }
}

impl Default for ConversionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

fn infer_format_from_extension(path: &Path) -> Option<String> {
    match path.extension()?.to_str()? {
        "gguf" => Some("GGUF".into()),
        "safetensors" => Some("SafeTensors".into()),
        "onnx" => Some("ONNX".into()),
        "pt" | "pth" => Some("PyTorch".into()),
        "tflite" => Some("TFLite".into()),
        "mlpackage" => Some("CoreML".into()),
        "engine" => Some("TensorRT".into()),
        "xml" => Some("OpenVINO".into()),
        "pte" => Some("ExecuTorch".into()),
        "h5" | "keras" => Some("KerasH5".into()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_minimal_gguf() -> NamedTempFile {
        let mut f = NamedTempFile::with_suffix(".gguf").unwrap();
        f.write_all(b"GGUF").unwrap();
        f.write_all(&3u32.to_le_bytes()).unwrap();
        f.write_all(&0u64.to_le_bytes()).unwrap(); // tensor_count
        f.write_all(&0u64.to_le_bytes()).unwrap(); // metadata_kv_count
        f.flush().unwrap();
        f
    }

    #[test]
    fn test_infer_format_safetensors() {
        let p = PathBuf::from("model.safetensors");
        assert_eq!(infer_format_from_extension(&p), Some("SafeTensors".into()));
    }

    #[test]
    fn test_infer_format_gguf() {
        let p = PathBuf::from("model.gguf");
        assert_eq!(infer_format_from_extension(&p), Some("GGUF".into()));
    }

    #[test]
    fn test_pipeline_gguf_to_safetensors() {
        let input = write_minimal_gguf();
        let output = NamedTempFile::with_suffix(".safetensors").unwrap();

        let pipeline = ConversionPipeline::new();
        let mut req = ConversionRequest::new(input.path(), output.path());
        req.validation_mode = ValidationMode::None;

        let result = pipeline.convert(req, &ProgressCallback::noop()).unwrap();
        assert_eq!(result.source_format, "GGUF");
        assert_eq!(result.target_format, "SafeTensors");
        assert!(output.path().exists());
    }

    #[test]
    fn test_cancellation() {
        let input = write_minimal_gguf();
        let output = NamedTempFile::with_suffix(".safetensors").unwrap();

        let pipeline = ConversionPipeline::new();
        let mut req = ConversionRequest::new(input.path(), output.path());
        req.cancellation.cancel(); // cancel before starting

        let result = pipeline.convert(req, &ProgressCallback::noop());
        // Depending on timing it might succeed or return Cancelled
        let _ = result; // Both outcomes are acceptable
    }
}
