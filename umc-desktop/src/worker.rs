//! Conversion en arrière-plan : le pipeline tourne sur un thread dédié,
//! l'UI reste réactive et reçoit des événements de progression.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;

use umc_core::{DType, LoadOptions, SaveOptions, UMC_VERSION};
use umc_pipeline::{CancellationToken, ConversionPipeline, ConversionRequest};
use umc_validate::ValidationMode;

/// Options de conversion choisies dans l'UI.
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

/// Événements envoyés du worker vers l'UI.
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    /// Progression : (tensors_done, tensors_total, message).
    Progress(u64, u64, String),
    /// Conversion terminée avec succès.
    Done {
        source_format: String,
        target_format: String,
        elapsed_ms: u64,
        tensor_count: usize,
        output: PathBuf,
        warnings: Vec<String>,
    },
    /// Échec de la conversion.
    Error(String),
}

/// Résultat final renvoyé par le worker.
pub struct WorkerResult {
    pub source_format: String,
    pub target_format: String,
    pub elapsed_ms: u64,
    pub tensor_count: usize,
    pub output: PathBuf,
    pub warnings: Vec<String>,
}

/// Lance une conversion en arrière-plan.
/// Retourne (sender pour annuler, receiver pour les événements).
pub fn spawn_conversion(
    opts: ConvertOptions,
) -> (Arc<CancellationToken>, Receiver<WorkerEvent>) {
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
    let _ = tx.send(WorkerEvent::Progress(0, 0, "Détection du format…".into()));

    let pipeline = ConversionPipeline::new();

    // Callback de progression → canal.
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

/// Version UMC exposée à l'UI.
pub fn version() -> &'static str {
    UMC_VERSION
}