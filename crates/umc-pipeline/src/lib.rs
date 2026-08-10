pub mod cancel;
/// UMC conversion pipeline — orchestrates detection, loading, transformation,
/// validation, and saving into a robust single-call API.
pub mod pipeline;

pub use cancel::CancellationToken;
pub use pipeline::{ConversionPipeline, ConversionRequest, ConversionResult};
