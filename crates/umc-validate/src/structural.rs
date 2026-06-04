use umc_core::{UniversalIR, UmcError};

/// Result of structural validation.
#[derive(Debug, Clone)]
pub struct StructuralReport {
    pub tensor_count_before: usize,
    pub tensor_count_after: usize,
    pub tensor_count_match: bool,
    pub shape_mismatches: Vec<String>,
    pub dtype_changes: Vec<String>,
    pub passed: bool,
    pub warnings: Vec<String>,
}

impl StructuralReport {
    pub fn summary(&self) -> String {
        if self.passed {
            format!(
                "PASS — {} tensors, {} shape mismatches, {} dtype changes",
                self.tensor_count_after,
                self.shape_mismatches.len(),
                self.dtype_changes.len()
            )
        } else {
            format!("FAIL — {}", self.shape_mismatches.first().cloned().unwrap_or_default())
        }
    }
}

/// Validate that `after` structurally matches `before`.
///
/// Checks:
/// - Tensor count preserved
/// - All expected tensor names are present
/// - Shape changes are only those expected (e.g., transpose)
/// - Dtype changes are documented
pub fn structural_validate(
    before: &UniversalIR,
    after: &UniversalIR,
) -> Result<StructuralReport, UmcError> {
    let before_count = before.tensors.len();
    let after_count = after.tensors.len();
    let count_match = before_count == after_count;
    let mut shape_mismatches = Vec::new();
    let mut dtype_changes = Vec::new();
    let mut warnings = Vec::new();

    if !count_match {
        warnings.push(format!(
            "Tensor count changed: {} → {}",
            before_count, after_count
        ));
    }

    // Check that all tensors from 'before' are present in 'after' (by name)
    for (name, before_tensor) in before.tensors.iter() {
        match after.tensors.get(name) {
            None => {
                shape_mismatches.push(format!("Tensor '{}' missing from output", name));
            }
            Some(after_tensor) => {
                // Shape check (allow transpose)
                let before_elems: usize = before_tensor.shape.iter().product();
                let after_elems: usize = after_tensor.shape.iter().product();
                if before_elems != after_elems {
                    shape_mismatches.push(format!(
                        "Tensor '{}': element count changed {} → {}",
                        name, before_elems, after_elems
                    ));
                }
                // DType change
                if before_tensor.dtype != after_tensor.dtype {
                    dtype_changes.push(format!(
                        "Tensor '{}': {} → {}",
                        name, before_tensor.dtype, after_tensor.dtype
                    ));
                }
            }
        }
    }

    let passed = shape_mismatches.is_empty();

    Ok(StructuralReport {
        tensor_count_before: before_count,
        tensor_count_after: after_count,
        tensor_count_match: count_match,
        shape_mismatches,
        dtype_changes,
        passed,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use umc_core::{UniversalIR, Tensor, DType};
    use std::path::Path;

    fn make_ir_with_tensor(name: &str, shape: Vec<usize>) -> UniversalIR {
        let mut ir = UniversalIR::new("TEST", Path::new("test.bin"));
        let t = Tensor::from_bytes(name, DType::F32, shape, vec![0u8; 16]);
        ir.tensors.insert(t).unwrap();
        ir
    }

    #[test]
    fn test_identical_irs_pass() {
        let before = make_ir_with_tensor("weight", vec![4, 4]);
        let after = make_ir_with_tensor("weight", vec![4, 4]);
        let report = structural_validate(&before, &after).unwrap();
        assert!(report.passed);
        assert!(report.shape_mismatches.is_empty());
    }

    #[test]
    fn test_missing_tensor_fails() {
        let before = make_ir_with_tensor("weight", vec![4, 4]);
        let after = UniversalIR::new("TEST", Path::new("test.bin"));
        let report = structural_validate(&before, &after).unwrap();
        assert!(!report.passed);
        assert!(!report.shape_mismatches.is_empty());
    }

    #[test]
    fn test_dtype_change_recorded() {
        let mut before = UniversalIR::new("TEST", Path::new("test.bin"));
        before.tensors.insert(Tensor::from_bytes("w", DType::F32, vec![4], vec![0u8; 16])).unwrap();
        let mut after = UniversalIR::new("TEST", Path::new("test.bin"));
        after.tensors.insert(Tensor::from_bytes("w", DType::F16, vec![4], vec![0u8; 8])).unwrap();
        let report = structural_validate(&before, &after).unwrap();
        assert!(report.passed); // element count matches (4 elements each)
        assert!(!report.dtype_changes.is_empty());
    }
}
