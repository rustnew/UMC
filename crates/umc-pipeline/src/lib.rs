/// UMC conversion pipeline — orchestrates detection, loading, transformation,
/// validation, and saving into a robust single-call API.

pub mod pipeline;
pub mod cancel;

pub use pipeline::{ConversionPipeline, ConversionRequest, ConversionResult};
pub use cancel::CancellationToken;
