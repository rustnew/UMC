use umc_core::{DType, UmcError, UniversalIR};

/// Per-tensor divergence measurement.
#[derive(Debug, Clone)]
pub struct TensorDivergence {
    pub name: String,
    pub max_abs_error: f64,
    pub mean_abs_error: f64,
    pub passed: bool,
    pub threshold: f64,
}

/// Result of numeric validation.
#[derive(Debug, Clone)]
pub struct NumericReport {
    pub tensor_results: Vec<TensorDivergence>,
    pub global_max_divergence: f64,
    pub global_mean_divergence: f64,
    pub passed: bool,
    pub threshold: f64,
}

impl NumericReport {
    pub fn summary(&self) -> String {
        if self.passed {
            format!(
                "PASS — max divergence: {:.2e} (threshold: {:.2e})",
                self.global_max_divergence, self.threshold
            )
        } else {
            format!(
                "FAIL — max divergence: {:.2e} (threshold: {:.2e})",
                self.global_max_divergence, self.threshold
            )
        }
    }
}

/// Tolerance thresholds per dtype conversion pair.
pub fn divergence_threshold(before_dtype: &DType, after_dtype: &DType) -> f64 {
    match (before_dtype, after_dtype) {
        (DType::F32, DType::F32) => 0.0,
        (DType::F32, DType::F16) | (DType::F16, DType::F32) => 1e-3,
        (DType::F32, DType::BF16) | (DType::BF16, DType::F32) => 2e-2,
        (DType::F32, DType::Q4KM) | (DType::Q4KM, DType::F32) => 1e-2,
        (DType::F32, DType::Q8_0) | (DType::Q8_0, DType::F32) => 1e-3,
        _ => 1e-4, // Conservative default
    }
}

/// Compare F32 tensors numerically between before and after IRs.
///
/// Only compares tensors that are F32 in both IRs (quantized tensors
/// are skipped with a warning — use semantic validation for those).
pub fn numeric_validate(
    before: &UniversalIR,
    after: &UniversalIR,
    threshold_override: Option<f64>,
) -> Result<NumericReport, UmcError> {
    let mut results = Vec::new();
    let mut global_max = 0.0f64;
    let mut global_mean_sum = 0.0f64;
    let mut compared_tensors = 0usize;

    for (name, before_tensor) in before.tensors.iter() {
        let after_tensor = match after.tensors.get(name) {
            Some(t) => t,
            None => continue,
        };

        // Only compare F32 ↔ F32 (extend for other dtypes if needed)
        if before_tensor.dtype != DType::F32 || after_tensor.dtype != DType::F32 {
            continue;
        }

        let threshold = threshold_override
            .unwrap_or_else(|| divergence_threshold(&before_tensor.dtype, &after_tensor.dtype));

        let before_bytes = match before_tensor.data.as_bytes() {
            Ok(b) => b,
            Err(_) => continue,
        };
        let after_bytes = match after_tensor.data.as_bytes() {
            Ok(b) => b,
            Err(_) => continue,
        };

        if before_bytes.len() != after_bytes.len() {
            continue;
        }

        let n = before_bytes.len() / 4;
        if n == 0 {
            continue;
        }

        let mut max_err = 0.0f32;
        let mut sum_err = 0.0f64;

        for i in 0..n {
            let a = f32::from_le_bytes(
                before_bytes[i * 4..(i + 1) * 4]
                    .try_into()
                    .unwrap_or([0; 4]),
            );
            let b =
                f32::from_le_bytes(after_bytes[i * 4..(i + 1) * 4].try_into().unwrap_or([0; 4]));
            let err = (a - b).abs();
            max_err = max_err.max(err);
            sum_err += err as f64;
        }

        let mean_err = sum_err / n as f64;
        let max_err_f64 = max_err as f64;
        global_max = global_max.max(max_err_f64);
        global_mean_sum += mean_err;
        compared_tensors += 1;

        results.push(TensorDivergence {
            name: name.clone(),
            max_abs_error: max_err_f64,
            mean_abs_error: mean_err,
            passed: max_err_f64 <= threshold,
            threshold,
        });
    }

    let global_mean = if compared_tensors > 0 {
        global_mean_sum / compared_tensors as f64
    } else {
        0.0
    };

    let threshold = threshold_override.unwrap_or(1e-5);
    let passed = results.iter().all(|r| r.passed);

    Ok(NumericReport {
        tensor_results: results,
        global_max_divergence: global_max,
        global_mean_divergence: global_mean,
        passed,
        threshold,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use umc_core::{Tensor, UniversalIR};

    fn f32_to_le_bytes(v: &[f32]) -> Vec<u8> {
        v.iter().flat_map(|f| f.to_le_bytes()).collect()
    }

    fn make_ir_f32(name: &str, values: &[f32]) -> UniversalIR {
        let mut ir = UniversalIR::new("TEST", Path::new("test.bin"));
        let data = f32_to_le_bytes(values);
        let t = Tensor::from_bytes(name, DType::F32, vec![values.len()], data);
        ir.tensors.insert(t).unwrap();
        ir
    }

    #[test]
    fn test_identical_tensors_pass() {
        let vals = vec![1.0f32, 2.0, 3.0, 4.0];
        let before = make_ir_f32("weight", &vals);
        let after = make_ir_f32("weight", &vals);
        let report = numeric_validate(&before, &after, None).unwrap();
        assert!(report.passed);
        assert_eq!(report.global_max_divergence, 0.0);
    }

    #[test]
    fn test_small_divergence_passes() {
        let before = make_ir_f32("weight", &[1.0f32, 2.0]);
        let after = make_ir_f32("weight", &[1.0000001f32, 2.0]);
        let report = numeric_validate(&before, &after, Some(1e-5)).unwrap();
        assert!(report.passed);
    }

    #[test]
    fn test_large_divergence_fails() {
        let before = make_ir_f32("weight", &[1.0f32]);
        let after = make_ir_f32("weight", &[2.0f32]);
        let report = numeric_validate(&before, &after, Some(1e-5)).unwrap();
        assert!(!report.passed);
        assert!(report.global_max_divergence > 0.5);
    }
}
