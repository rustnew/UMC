use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use umc_core::UMC_VERSION;

/// A conversion certificate — cryptographically signed report of a conversion.
///
/// Produced only when ALL requested validations have passed.
/// Never produced for partial or in-progress conversions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionCertificate {
    pub umc_version: String,
    pub timestamp: u64,
    pub source: CertFileInfo,
    pub target: CertFileInfo,
    pub validation: ValidationSummary,
    pub guarantees: Vec<Guarantee>,
    /// SHA256 of the certificate body (before signing).
    pub body_hash: String,
    /// ed25519 signature placeholder (full crypto in Enterprise tier).
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertFileInfo {
    pub format: String,
    pub sha256: String,
    pub file_size_bytes: u64,
    pub num_tensors: usize,
    pub num_parameters: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub structural_passed: bool,
    pub numeric_passed: Option<bool>,
    pub roundtrip_level: String,
    pub max_divergence: Option<f64>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guarantee {
    #[serde(rename = "type")]
    pub guarantee_type: String,
    pub description: String,
    pub verified: bool,
}

/// Builder for constructing a `ConversionCertificate`.
#[derive(Default)]
pub struct CertificateBuilder {
    source: Option<CertFileInfo>,
    target: Option<CertFileInfo>,
    structural_passed: bool,
    numeric_passed: Option<bool>,
    max_divergence: Option<f64>,
    roundtrip_level: String,
    warnings: Vec<String>,
}

impl CertificateBuilder {
    pub fn new() -> Self {
        Self {
            roundtrip_level: "structural".into(),
            ..Default::default()
        }
    }

    pub fn source(mut self, info: CertFileInfo) -> Self {
        self.source = Some(info); self
    }

    pub fn target(mut self, info: CertFileInfo) -> Self {
        self.target = Some(info); self
    }

    pub fn structural_passed(mut self, passed: bool) -> Self {
        self.structural_passed = passed; self
    }

    pub fn numeric_passed(mut self, passed: bool, max_divergence: f64) -> Self {
        self.numeric_passed = Some(passed);
        self.max_divergence = Some(max_divergence);
        self
    }

    pub fn roundtrip_level(mut self, level: impl Into<String>) -> Self {
        self.roundtrip_level = level.into(); self
    }

    pub fn add_warning(mut self, w: impl Into<String>) -> Self {
        self.warnings.push(w.into()); self
    }

    /// Build the certificate. Returns None if not all validations passed.
    pub fn build(self) -> Option<ConversionCertificate> {
        if !self.structural_passed {
            return None;
        }
        if self.numeric_passed == Some(false) {
            return None;
        }

        let source = self.source?;
        let target = self.target?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut guarantees = vec![
            Guarantee {
                guarantee_type: "structural_integrity".into(),
                description: "All tensors preserved with correct shapes and dtypes".into(),
                verified: self.structural_passed,
            },
        ];

        if let Some(np) = self.numeric_passed {
            guarantees.push(Guarantee {
                guarantee_type: "numeric_precision".into(),
                description: format!(
                    "Max divergence: {:.2e}",
                    self.max_divergence.unwrap_or(0.0)
                ),
                verified: np,
            });
        }

        let validation = ValidationSummary {
            structural_passed: self.structural_passed,
            numeric_passed: self.numeric_passed,
            roundtrip_level: self.roundtrip_level.clone(),
            max_divergence: self.max_divergence,
            warnings: self.warnings,
        };

        let body = serde_json::json!({
            "umc_version": UMC_VERSION,
            "timestamp": timestamp,
            "source": source,
            "target": target,
            "validation": validation,
            "guarantees": guarantees,
        });
        let body_hash = hex::encode(Sha256::digest(body.to_string().as_bytes()));

        Some(ConversionCertificate {
            umc_version: UMC_VERSION.to_string(),
            timestamp,
            source,
            target,
            validation,
            guarantees,
            body_hash: body_hash.clone(),
            signature: format!("umc-self-certified:{}", &body_hash[..16]),
        })
    }
}

/// Compute SHA256 of a file.
pub fn sha256_file(path: &std::path::Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cert_info(format: &str) -> CertFileInfo {
        CertFileInfo {
            format: format.into(),
            sha256: "abc123".into(),
            file_size_bytes: 1024,
            num_tensors: 10,
            num_parameters: 1_000_000,
        }
    }

    #[test]
    fn test_certificate_built_when_all_pass() {
        let cert = CertificateBuilder::new()
            .source(make_cert_info("GGUF"))
            .target(make_cert_info("SafeTensors"))
            .structural_passed(true)
            .numeric_passed(true, 1.2e-7)
            .build();
        assert!(cert.is_some());
        let c = cert.unwrap();
        assert_eq!(c.source.format, "GGUF");
        assert_eq!(c.target.format, "SafeTensors");
        assert!(!c.body_hash.is_empty());
    }

    #[test]
    fn test_no_certificate_when_structural_fails() {
        let cert = CertificateBuilder::new()
            .source(make_cert_info("GGUF"))
            .target(make_cert_info("SafeTensors"))
            .structural_passed(false)
            .build();
        assert!(cert.is_none());
    }

    #[test]
    fn test_no_certificate_when_numeric_fails() {
        let cert = CertificateBuilder::new()
            .source(make_cert_info("GGUF"))
            .target(make_cert_info("SafeTensors"))
            .structural_passed(true)
            .numeric_passed(false, 0.5)
            .build();
        assert!(cert.is_none());
    }

    #[test]
    fn test_certificate_json_serialization() {
        let cert = CertificateBuilder::new()
            .source(make_cert_info("GGUF"))
            .target(make_cert_info("ONNX"))
            .structural_passed(true)
            .build()
            .unwrap();
        let json = serde_json::to_string_pretty(&cert).unwrap();
        assert!(json.contains("GGUF"));
        assert!(json.contains("ONNX"));
        assert!(json.contains("body_hash"));
    }
}
