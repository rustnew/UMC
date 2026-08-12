//! Background conversion: the pipeline runs on a dedicated thread,
//! the UI stays responsive and receives progress events.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

use umc_core::{DType, LoadOptions, SaveOptions, UMC_VERSION};
use umc_pipeline::{CancellationToken, ConversionPipeline, ConversionRequest};
use umc_validate::ValidationMode;

/// Conversion options chosen in the UI.
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    pub input: PathBuf,
    pub output: PathBuf,
    pub source_format: Option<String>,
    pub target_format: Option<String>,
    pub dtype: Option<DType>,
    pub validation_mode: ValidationMode,
    pub threads: usize,
}

/// Events sent from the worker to the UI.
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    /// Progress: (tensors_done, tensors_total, message).
    Progress(u64, u64, String),
    /// Conversion completed successfully.
    Done {
        source_format: String,
        target_format: String,
        elapsed_ms: u64,
        tensor_count: usize,
        output: PathBuf,
        warnings: Vec<String>,
    },
    /// Conversion failure.
    Error(String),
}

/// Final result returned by the worker.
pub struct WorkerResult {
    pub source_format: String,
    pub target_format: String,
    pub elapsed_ms: u64,
    pub tensor_count: usize,
    pub output: PathBuf,
    pub warnings: Vec<String>,
}

/// Launches a conversion in the background.
/// Returns (sender to cancel, receiver for events).
pub fn spawn_conversion(opts: ConvertOptions) -> (Arc<CancellationToken>, Receiver<WorkerEvent>) {
    let token = Arc::new(CancellationToken::new());
    let (tx, rx): (Sender<WorkerEvent>, Receiver<WorkerEvent>) = mpsc::channel();

    let token_clone = token.clone();
    std::thread::spawn(move || {
        let result = run_conversion(opts, tx.clone(), token_clone);
        match result {
            Ok(res) => {
                let _ = tx.send(WorkerEvent::Done {
                    source_format: res.source_format,
                    target_format: res.target_format,
                    elapsed_ms: res.elapsed_ms,
                    tensor_count: res.tensor_count,
                    output: res.output,
                    warnings: res.warnings,
                });
            }
            Err(e) => {
                let _ = tx.send(WorkerEvent::Error(e));
            }
        }
    });

    (token, rx)
}

fn run_conversion(
    opts: ConvertOptions,
    tx: Sender<WorkerEvent>,
    token: Arc<CancellationToken>,
) -> Result<WorkerResult, String> {
    let _ = tx.send(WorkerEvent::Progress(0, 0, "Detecting format…".into()));

    let pipeline = ConversionPipeline::new();

    // Progress callback → channel.
    let progress = umc_core::ProgressCallback::with_handler(move |done, total, msg| {
        let _ = tx.send(WorkerEvent::Progress(done, total, msg.to_string()));
    });

    let mut req = ConversionRequest::new(&opts.input, &opts.output);
    req.source_format = opts.source_format.clone();
    req.target_format = opts.target_format.clone();
    req.validation_mode = opts.validation_mode;
    req.cancellation = (*token).clone();
    req.load_options = LoadOptions {
        format_override: opts.source_format.clone(),
        shard_dir: None,
        metadata_only: false,
        threads: opts.threads,
    };
    req.save_options = SaveOptions {
        dtype: opts.dtype.clone(),
        threads: opts.threads,
        output_path_override: None,
    };

    let started = std::time::Instant::now();
    let result = pipeline
        .convert(req, &progress)
        .map_err(|e| format!("{e}"))?;

    Ok(WorkerResult {
        source_format: result.source_format,
        target_format: result.target_format,
        elapsed_ms: result.elapsed_ms.max(started.elapsed().as_millis() as u64),
        tensor_count: result.tensor_count,
        output: result.output_path,
        warnings: result.warnings,
    })
}

/// UMC version exposed to the UI.
pub fn version() -> &'static str {
    UMC_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    /// End-to-end GGUF → SafeTensors conversion via the worker.
    #[test]
    fn test_worker_converts_gguf_to_safetensors() {
        let src = umc_tests::write_gguf_v3_with_f32_tensors(&[
            ("weight", vec![2, 2], vec![1.0f32, 2.0, 3.0, 4.0]),
            ("bias", vec![2], vec![0.5f32, -0.5]),
        ]);
        let out = tempfile::NamedTempFile::new().unwrap();
        let out_path = out.path().with_extension("safetensors");

        let opts = ConvertOptions {
            input: src.path().to_path_buf(),
            output: out_path.clone(),
            source_format: Some("GGUF".into()),
            target_format: Some("SafeTensors".into()),
            dtype: None,
            validation_mode: ValidationMode::Structural,
            threads: 0,
        };

        let (_token, rx) = spawn_conversion(opts);
        let events = drain_with_timeout(rx, 10);

        // The last event must be Done with 2 tensors.
        let done = events.iter().rev().find_map(|e| match e {
            WorkerEvent::Done { tensor_count, .. } => Some(*tensor_count),
            _ => None,
        });
        assert_eq!(done, Some(2), "events: {events:?}");
        assert!(!events.iter().any(|e| matches!(e, WorkerEvent::Error(_))));
        assert!(
            out_path.exists(),
            "the output file must exist after conversion"
        );
    }

    /// The worker properly sends progress events.
    #[test]
    fn test_worker_reports_progress() {
        let src = umc_tests::write_minimal_gguf();
        let out = tempfile::NamedTempFile::new().unwrap();

        let opts = ConvertOptions {
            input: src.path().to_path_buf(),
            output: out.path().with_extension("safetensors"),
            source_format: Some("GGUF".into()),
            target_format: Some("SafeTensors".into()),
            dtype: None,
            validation_mode: ValidationMode::None,
            threads: 0,
        };

        let (_token, rx) = spawn_conversion(opts);
        let events = drain_with_timeout(rx, 10);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WorkerEvent::Progress(_, _, _))),
            "at least one Progress event expected, received: {events:?}"
        );
        assert!(
            events.iter().any(|e| matches!(e, WorkerEvent::Done { .. })),
            "the conversion must succeed, received: {events:?}"
        );
    }

    /// Cooperative cancellation: cancel() flips the flag.
    #[test]
    fn test_cancel_token() {
        let token = umc_pipeline::CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    /// The channel is drained even with no fast consumer.
    #[test]
    fn test_channel_drains() {
        let (_tx, rx): (mpsc::Sender<WorkerEvent>, _) = mpsc::channel();
        let _ = rx.try_recv(); // must not panic
    }

    /// Drains the channel blocking until Done/Error or until timeout.
    fn drain_with_timeout(rx: Receiver<WorkerEvent>, secs: u64) -> Vec<WorkerEvent> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(secs);
        let mut events = Vec::new();
        loop {
            match rx.recv_timeout(std::time::Duration::from_millis(100)) {
                Ok(ev) => {
                    let is_terminal =
                        matches!(ev, WorkerEvent::Done { .. } | WorkerEvent::Error(_));
                    events.push(ev);
                    if is_terminal {
                        return events;
                    }
                }
                Err(_) if std::time::Instant::now() > deadline => return events,
                Err(mpsc::RecvTimeoutError::Disconnected) => return events,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
            }
        }
    }
}
