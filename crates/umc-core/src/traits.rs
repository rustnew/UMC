use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::{UniversalIR, UmcError};

// ── ProgressCallback ──────────────────────────────────────────────────────────

/// Thread-safe progress reporting.
#[derive(Clone)]
pub struct ProgressCallback {
    tensors_done: Arc<AtomicU64>,
    tensors_total: Arc<AtomicU64>,
    handler: Arc<dyn Fn(u64, u64, &str) + Send + Sync>,
}

impl ProgressCallback {
    /// Create a no-op progress callback.
    pub fn noop() -> Self {
        Self {
            tensors_done: Arc::new(AtomicU64::new(0)),
            tensors_total: Arc::new(AtomicU64::new(0)),
            handler: Arc::new(|_, _, _| {}),
        }
    }

    /// Create a callback that prints to stderr.
    pub fn stderr() -> Self {
        Self {
            tensors_done: Arc::new(AtomicU64::new(0)),
            tensors_total: Arc::new(AtomicU64::new(0)),
            handler: Arc::new(|done, total, msg| {
                if total > 0 {
                    eprint!("\r  [{done}/{total}] {msg}   ");
                } else {
                    eprint!("\r  {msg}   ");
                }
            }),
        }
    }

    pub fn with_handler(handler: impl Fn(u64, u64, &str) + Send + Sync + 'static) -> Self {
        Self {
            tensors_done: Arc::new(AtomicU64::new(0)),
            tensors_total: Arc::new(AtomicU64::new(0)),
            handler: Arc::new(handler),
        }
    }

    pub fn set_total(&self, total: u64) {
        self.tensors_total.store(total, Ordering::Relaxed);
    }

    pub fn increment(&self, msg: &str) {
        let done = self.tensors_done.fetch_add(1, Ordering::Relaxed) + 1;
        let total = self.tensors_total.load(Ordering::Relaxed);
        (self.handler)(done, total, msg);
    }

    pub fn report(&self, msg: &str) {
        let done = self.tensors_done.load(Ordering::Relaxed);
        let total = self.tensors_total.load(Ordering::Relaxed);
        (self.handler)(done, total, msg);
    }
}

// ── LoadOptions ───────────────────────────────────────────────────────────────

/// Options controlling how a format is loaded.
#[derive(Debug, Clone, Default)]
pub struct LoadOptions {
    /// Override format detection (use this format regardless of magic bytes).
    pub format_override: Option<String>,
    /// Shard directory (for multi-file models like SafeTensors with index.json).
    pub shard_dir: Option<std::path::PathBuf>,
    /// Load only metadata, skip tensor data.
    pub metadata_only: bool,
    /// Number of worker threads (0 = auto).
    pub threads: usize,
}

// ── SaveOptions ───────────────────────────────────────────────────────────────

/// Options controlling how a format is saved.
#[derive(Debug, Clone, Default)]
pub struct SaveOptions {
    /// Target dtype override (e.g. convert to F16 on save).
    pub dtype: Option<crate::DType>,
    /// Number of worker threads (0 = auto).
    pub threads: usize,
    /// Write to this path (overrides the path given to `save()`).
    pub output_path_override: Option<std::path::PathBuf>,
}

// ── FormatLoader ──────────────────────────────────────────────────────────────

/// Trait for loading a file format into the Universal IR.
///
/// # Contract
/// - All tensor data > 64 MiB MUST use `TensorData::MmapView` (zero-copy).
/// - Fields not representable in the IR MUST be stored in `extensions`.
/// - `unwrap()` / `expect()` are FORBIDDEN in implementations.
pub trait FormatLoader: Send + Sync {
    /// Human-readable format name (e.g. `"GGUF"`, `"SafeTensors"`).
    fn format_name(&self) -> &'static str;

    /// Load the file at `path` into a `UniversalIR`.
    fn load(
        &self,
        path: &Path,
        options: &LoadOptions,
        progress: &ProgressCallback,
    ) -> Result<UniversalIR, UmcError>;

    /// Return true if this loader can handle the given path (quick check).
    fn can_load(&self, path: &Path) -> bool;
}

// ── FormatSaver ───────────────────────────────────────────────────────────────

/// Trait for saving a Universal IR to a file format.
///
/// # Contract
/// - Write to a temp file first, then atomic rename on success.
/// - Validate the output file after writing (structural check at minimum).
/// - All warnings MUST be reported via `progress`, never silenced.
pub trait FormatSaver: Send + Sync {
    fn format_name(&self) -> &'static str;

    fn save(
        &self,
        ir: &UniversalIR,
        path: &Path,
        options: &SaveOptions,
        progress: &ProgressCallback,
    ) -> Result<(), UmcError>;

    /// Return the standard file extension (without dot).
    fn default_extension(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_callback_noop() {
        let p = ProgressCallback::noop();
        p.set_total(100);
        p.increment("loading tensor 1");
        p.report("done");
        // No panic = success
    }

    #[test]
    fn test_progress_counts() {
        let done_count = Arc::new(AtomicU64::new(0));
        let dc = done_count.clone();
        let p = ProgressCallback::with_handler(move |done, _total, _msg| {
            dc.store(done, Ordering::Relaxed);
        });
        p.increment("step 1");
        p.increment("step 2");
        assert_eq!(done_count.load(Ordering::Relaxed), 2);
    }
}
