use std::io::Read;
use std::path::Path;
use umc_core::UmcError;

/// Detection confidence and method for a single format match.
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub format: String,
    /// 0.0 to 1.0 — higher means more confident.
    pub confidence: f32,
    pub method: DetectionMethod,
    pub format_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DetectionMethod {
    /// Magic bytes matched (most reliable).
    MagicBytes,
    /// Only file extension matched.
    Extension,
    /// Content-based heuristic.
    ContentAnalysis,
    /// User explicitly specified the format.
    ManualOverride,
}

/// Trait for individual format detectors.
pub trait FormatDetector: Send + Sync {
    fn format_name(&self) -> &'static str;
    /// Lower = higher priority (1 = magic bytes, 2 = extension, 3 = content).
    fn priority(&self) -> u8;
    /// Return confidence 0.0–1.0 (0.0 = not this format).
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32;
    fn detect_version(&self, path: &Path, first_bytes: &[u8]) -> Option<String>;
}

/// Central registry — holds all format detectors.
pub struct FormatRegistry {
    detectors: Vec<Box<dyn FormatDetector>>,
}

impl FormatRegistry {
    pub fn new() -> Self {
        let mut r = Self { detectors: Vec::new() };
        r.register_all_builtin();
        r
    }

    fn register_all_builtin(&mut self) {
        // Priority 1 — Magic bytes (most reliable)
        self.detectors.push(Box::new(GgufDetector));
        self.detectors.push(Box::new(GgmlDetector));
        self.detectors.push(Box::new(TFLiteDetector));
        self.detectors.push(Box::new(HDF5Detector));
        // Priority 2 — Extension + partial magic / header analysis
        self.detectors.push(Box::new(OnnxDetector));
        self.detectors.push(Box::new(PyTorchDetector));
        self.detectors.push(Box::new(SentencePieceDetector));
        // Priority 2 — Content analysis (must run before SafeTensors to refine)
        self.detectors.push(Box::new(AwqDetector));
        self.detectors.push(Box::new(GptqDetector));
        self.detectors.push(Box::new(LoraDetector));
        // Priority 1 — SafeTensors (fallback for .safetensors after AWQ/GPTQ/LoRA)
        self.detectors.push(Box::new(SafeTensorsDetector));
        // Priority 3 — Content analysis
        self.detectors.push(Box::new(DiffusersDetector));
    }

    pub fn register(&mut self, detector: Box<dyn FormatDetector>) {
        self.detectors.push(detector);
    }

    /// Detect format using cascade. Returns an actionable error if unknown.
    pub fn detect(&self, path: &Path) -> Result<DetectionResult, UmcError> {
        let first_bytes = self.read_magic_bytes(path, 512)?;

        let mut candidates: Vec<(&dyn FormatDetector, f32)> = self
            .detectors
            .iter()
            .filter_map(|d| {
                let c = d.confidence(path, &first_bytes);
                if c > 0.0 { Some((d.as_ref(), c)) } else { None }
            })
            .collect();

        candidates.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.priority().cmp(&b.0.priority()))
        });

        let (detector, confidence) = candidates.first().ok_or_else(|| {
            UmcError::UnknownFormat {
                path: path.to_string_lossy().to_string(),
                hint: "Run `umc formats` to see supported formats, \
                       or use `--format <FORMAT>` to specify manually."
                    .into(),
            }
        })?;

        // Warn on ambiguous detection (two formats with < 0.1 confidence gap)
        if candidates.len() > 1 && confidence - candidates[1].1 < 0.1 {
            tracing::warn!(
                "Ambiguous detection for {}: {} ({:.2}) vs {} ({:.2}). \
                 Use --format to disambiguate.",
                path.display(),
                detector.format_name(), confidence,
                candidates[1].0.format_name(), candidates[1].1,
            );
        }

        Ok(DetectionResult {
            format: detector.format_name().to_string(),
            confidence: *confidence,
            method: match detector.priority() {
                1 => DetectionMethod::MagicBytes,
                2 => DetectionMethod::Extension,
                _ => DetectionMethod::ContentAnalysis,
            },
            format_version: detector.detect_version(path, &first_bytes),
        })
    }

    fn read_magic_bytes(&self, path: &Path, n: usize) -> Result<Vec<u8>, UmcError> {
        let mut file = std::fs::File::open(path).map_err(UmcError::Io)?;
        let mut buf = vec![0u8; n];
        let read = file.read(&mut buf).map_err(UmcError::Io)?;
        buf.truncate(read);
        Ok(buf)
    }

    /// Return all registered format names.
    pub fn format_names(&self) -> Vec<&'static str> {
        let mut names: Vec<&'static str> = self.detectors.iter().map(|d| d.format_name()).collect();
        names.dedup();
        names
    }
}

impl Default for FormatRegistry {
    fn default() -> Self { Self::new() }
}

// ── Individual detectors ──────────────────────────────────────────────────────

/// GGUF: magic "GGUF" (4 bytes)
pub struct GgufDetector;
impl FormatDetector for GgufDetector {
    fn format_name(&self) -> &'static str { "GGUF" }
    fn priority(&self) -> u8 { 1 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        if first_bytes.starts_with(b"GGUF") {
            0.99
        } else if path.extension().map_or(false, |e| e == "gguf") {
            0.70
        } else {
            0.0
        }
    }
    fn detect_version(&self, _path: &Path, first_bytes: &[u8]) -> Option<String> {
        if first_bytes.len() >= 8 {
            let ver = u32::from_le_bytes(first_bytes[4..8].try_into().ok()?);
            Some(format!("v{}", ver))
        } else {
            None
        }
    }
}

/// GGML legacy: magic "GGML"
pub struct GgmlDetector;
impl FormatDetector for GgmlDetector {
    fn format_name(&self) -> &'static str { "GGML" }
    fn priority(&self) -> u8 { 1 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        if first_bytes.starts_with(b"GGML") { 0.99 }
        else if path.extension().map_or(false, |e| e == "bin") { 0.20 }
        else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

/// SafeTensors: 8 bytes LE size followed by '{'
pub struct SafeTensorsDetector;
impl FormatDetector for SafeTensorsDetector {
    fn format_name(&self) -> &'static str { "SafeTensors" }
    fn priority(&self) -> u8 { 1 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        if first_bytes.len() >= 9 {
            let json_size = u64::from_le_bytes(
                first_bytes[0..8].try_into().unwrap_or([0; 8])
            );
            if first_bytes[8] == b'{' && json_size >= 2 && json_size < 100_000_000 {
                return 0.99;
            }
        }
        if path.extension().map_or(false, |e| e == "safetensors") { 0.75 }
        else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> {
        Some("1.0".into())
    }
}

/// TFLite: FlatBuffers magic at offset 4 (TFL3/TFL2/TFL1)
pub struct TFLiteDetector;
impl FormatDetector for TFLiteDetector {
    fn format_name(&self) -> &'static str { "TFLite" }
    fn priority(&self) -> u8 { 1 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        if first_bytes.len() >= 8 {
            let m = &first_bytes[4..8];
            if m == b"TFL3" || m == b"TFL2" || m == b"TFL1" { return 0.99; }
        }
        if path.extension().map_or(false, |e| e == "tflite") { 0.75 }
        else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, first_bytes: &[u8]) -> Option<String> {
        if first_bytes.len() >= 8 {
            Some(std::str::from_utf8(&first_bytes[4..8]).unwrap_or("?").to_string())
        } else { None }
    }
}

/// HDF5/Keras: magic \x89HDF
pub struct HDF5Detector;
impl FormatDetector for HDF5Detector {
    fn format_name(&self) -> &'static str { "KerasH5" }
    fn priority(&self) -> u8 { 1 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        if first_bytes.starts_with(&[0x89, 0x48, 0x44, 0x46]) {
            if path.extension().map_or(false, |e| e == "h5" || e == "keras" || e == "hdf5") {
                0.99
            } else {
                0.85
            }
        } else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

/// ONNX: protobuf (starts with 0x08 or 0x0a) + .onnx extension
pub struct OnnxDetector;
impl FormatDetector for OnnxDetector {
    fn format_name(&self) -> &'static str { "ONNX" }
    fn priority(&self) -> u8 { 2 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        let proto_magic = first_bytes.first().map_or(false, |&b| b == 0x08 || b == 0x0a);
        let ext_ok = path.extension().map_or(false, |e| e == "onnx");
        match (proto_magic, ext_ok) {
            (true, true) => 0.95,
            (true, false) => 0.50,
            (false, true) => 0.80,
            _ => 0.0,
        }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

/// PyTorch: ZIP magic (PK\x03\x04) + .pt/.pth/.bin extension
pub struct PyTorchDetector;
impl FormatDetector for PyTorchDetector {
    fn format_name(&self) -> &'static str { "PyTorch" }
    fn priority(&self) -> u8 { 2 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        let zip = first_bytes.starts_with(b"PK\x03\x04");
        let ext_ok = path.extension().map_or(false, |e| e == "pt" || e == "pth" || e == "bin");
        match (zip, ext_ok) {
            (true, true) => 0.90,
            (true, false) => 0.40,
            (false, true) => 0.55,
            _ => 0.0,
        }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

/// SentencePiece: proto magic
pub struct SentencePieceDetector;
impl FormatDetector for SentencePieceDetector {
    fn format_name(&self) -> &'static str { "SentencePiece" }
    fn priority(&self) -> u8 { 2 }
    fn confidence(&self, path: &Path, _first_bytes: &[u8]) -> f32 {
        if path.extension().map_or(false, |e| e == "model" || e == "spm") { 0.75 }
        else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

/// AWQ: SafeTensors with AWQ quantization markers in header or config.json
pub struct AwqDetector;
impl FormatDetector for AwqDetector {
    fn format_name(&self) -> &'static str { "AWQ" }
    fn priority(&self) -> u8 { 2 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        // Must have SafeTensors structure
        if first_bytes.len() < 9 { return 0.0; }
        let json_size = u64::from_le_bytes(first_bytes[0..8].try_into().unwrap_or([0;8]));
        if first_bytes[8] != b'{' || json_size < 2 || json_size >= 100_000_000 { return 0.0; }
        // Check header for AWQ tensor naming
        let header = std::str::from_utf8(&first_bytes[8..]).unwrap_or("");
        if header.contains("\"awq\"") || header.contains("scales") && header.contains("zeros") {
            return 0.95;
        }
        // Check config.json for AWQ metadata
        if let Some(parent) = path.parent() {
            if let Ok(cfg) = std::fs::read_to_string(parent.join("config.json")) {
                if cfg.contains("\"awq\"") || cfg.contains("AWQ") {
                    return 0.90;
                }
            }
        }
        0.0
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> {
        Some("AWQ-4bit".into())
    }
}

/// GPTQ: SafeTensors with GPTQ tensor patterns (qweight, qzeros, scales)
pub struct GptqDetector;
impl FormatDetector for GptqDetector {
    fn format_name(&self) -> &'static str { "GPTQ" }
    fn priority(&self) -> u8 { 2 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        if first_bytes.len() < 9 { return 0.0; }
        let json_size = u64::from_le_bytes(first_bytes[0..8].try_into().unwrap_or([0;8]));
        if first_bytes[8] != b'{' || json_size < 2 || json_size >= 100_000_000 { return 0.0; }
        let header = std::str::from_utf8(&first_bytes[8..]).unwrap_or("");
        if header.contains("qweight") && (header.contains("qzeros") || header.contains("scales")) {
            return 0.95;
        }
        if let Some(parent) = path.parent() {
            if let Ok(cfg) = std::fs::read_to_string(parent.join("config.json")) {
                if cfg.contains("\"gptq\"") || cfg.contains("\"GPTQ\"") {
                    return 0.90;
                }
            }
        }
        0.0
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> {
        Some("GPTQ-4bit".into())
    }
}

/// LoRA/PEFT: SafeTensors with lora_A/lora_B tensor names
pub struct LoraDetector;
impl FormatDetector for LoraDetector {
    fn format_name(&self) -> &'static str { "LoRA" }
    fn priority(&self) -> u8 { 2 }
    fn confidence(&self, path: &Path, first_bytes: &[u8]) -> f32 {
        // PEFT directory with adapter_config.json
        if path.is_dir() && path.join("adapter_config.json").exists() {
            return 0.95;
        }
        if first_bytes.len() < 9 { return 0.0; }
        let json_size = u64::from_le_bytes(first_bytes[0..8].try_into().unwrap_or([0;8]));
        if first_bytes[8] != b'{' || json_size < 2 || json_size >= 100_000_000 { return 0.0; }
        let header = std::str::from_utf8(&first_bytes[8..]).unwrap_or("");
        if header.contains("lora_A") || header.contains("lora_B")
            || header.contains("lora_a") || header.contains("lora_b") {
            return 0.95;
        }
        0.0
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

/// Diffusers: directory with model_index.json
pub struct DiffusersDetector;
impl FormatDetector for DiffusersDetector {
    fn format_name(&self) -> &'static str { "Diffusers" }
    fn priority(&self) -> u8 { 3 }
    fn confidence(&self, path: &Path, _first_bytes: &[u8]) -> f32 {
        if path.is_dir() && path.join("model_index.json").exists() { 0.95 }
        else { 0.0 }
    }
    fn detect_version(&self, _path: &Path, _first_bytes: &[u8]) -> Option<String> { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_magic(magic: &[u8]) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(magic).unwrap();
        f
    }

    #[test]
    fn test_detect_gguf() {
        let mut magic = b"GGUF".to_vec();
        magic.extend_from_slice(&3u32.to_le_bytes()); // version 3
        magic.extend_from_slice(&[0u8; 500]);
        let f = write_magic(&magic);
        let reg = FormatRegistry::new();
        let result = reg.detect(f.path()).unwrap();
        assert_eq!(result.format, "GGUF");
        assert_eq!(result.method, DetectionMethod::MagicBytes);
        assert_eq!(result.format_version, Some("v3".into()));
        assert!(result.confidence > 0.95);
    }

    #[test]
    fn test_detect_safetensors() {
        // 8 bytes LE size (value=2) then '{'
        let json_content = b"{}";
        let size: u64 = json_content.len() as u64;
        let mut magic = size.to_le_bytes().to_vec();
        magic.extend_from_slice(b"{");
        magic.extend_from_slice(&[0u8; 500]);
        let f = write_magic(&magic);
        let reg = FormatRegistry::new();
        let result = reg.detect(f.path()).unwrap();
        assert_eq!(result.format, "SafeTensors");
    }

    #[test]
    fn test_detect_unknown_format() {
        let f = write_magic(&[0xFF, 0xFE, 0x00, 0x01]);
        let reg = FormatRegistry::new();
        let result = reg.detect(f.path());
        assert!(matches!(result, Err(UmcError::UnknownFormat { .. })));
    }

    #[test]
    fn test_format_names() {
        let reg = FormatRegistry::new();
        let names = reg.format_names();
        assert!(names.contains(&"GGUF"));
        assert!(names.contains(&"SafeTensors"));
        assert!(names.contains(&"ONNX"));
    }

    #[test]
    fn test_detect_tflite() {
        let mut magic = vec![0u8; 4];
        magic.extend_from_slice(b"TFL3");
        magic.extend_from_slice(&[0u8; 500]);
        let f = write_magic(&magic);
        let reg = FormatRegistry::new();
        let r = reg.detect(f.path()).unwrap();
        assert_eq!(r.format, "TFLite");
    }
}
