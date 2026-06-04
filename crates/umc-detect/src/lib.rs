/// UMC automatic format detection.
///
/// Uses a three-level cascade:
/// 1. Magic bytes (most reliable, confidence ≥ 0.95)
/// 2. File extension (reliable when magic absent, confidence 0.70-0.85)
/// 3. Content analysis (least reliable, confidence 0.50-0.70)

pub mod registry;

pub use registry::{FormatRegistry, DetectionResult, DetectionMethod};
