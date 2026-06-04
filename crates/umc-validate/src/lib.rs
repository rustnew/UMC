/// UMC validation and certification.

pub mod structural;
pub mod numeric;
pub mod certificate;

pub use structural::{structural_validate, StructuralReport};
pub use numeric::{numeric_validate, NumericReport};
pub use certificate::{ConversionCertificate, CertificateBuilder};

use serde::{Deserialize, Serialize};

/// Validation mode — controls which checks are performed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationMode {
    /// No validation (fastest, not recommended).
    None,
    /// Structural check only (tensor count, shapes, dtype consistency).
    Structural,
    /// Structural + numeric divergence check.
    Numeric,
    /// All checks + round-trip verification (default).
    Strict,
}

impl Default for ValidationMode {
    fn default() -> Self { Self::Strict }
}
