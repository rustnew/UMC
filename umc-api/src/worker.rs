use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use uuid::Uuid;

use umc_core::ProgressCallback;
use umc_pipeline::{ConversionPipeline, ConversionRequest};
use umc_validate::ValidationMode;

use crate::{models::ProgressEvent, state::AppState};

/// Map lowercase API slug → uppercase pipeline format name.
fn slug_to_format(slug: &str) -> String {
    match slug {
        "gguf" => "GGUF",
        "safetensors" => "SafeTensors",
        "onnx" => "ONNX",
        "pytorch" => "PyTorch",
        "awq" => "AWQ",
        "gptq" => "GPTQ",
        "tflite" => "TFLite",
        "coreml" => "CoreML",
        "tensorrt" => "TensorRT",
        "openvino" => "OpenVINO",
        "executorch" => "ExecuTorch",
        "lora" => "LoRA",
        other => other, // pass through if already uppercase
    }
    .to_string()
}

/// Spawns a Tokio task to run the UMC conversion pipeline for the given job.
pub fn spawn_conversion_worker(state: AppState, job_id: Uuid) {
    tokio::spawn(async move {
        if let Err(e) = run_conversion(state, job_id).await {
            tracing::error!(job_id = %job_id, error = %e, "Conversion worker panicked");
        }
    });
}

async fn run_conversion(state: AppState, job_id: Uuid) -> anyhow::Result<()> {
    // Acquire semaphore permit
    let _permit = state.conversion_semaphore.acquire().await?;

    // Mark running
    sqlx::query("UPDATE conversion_jobs SET status='running', started_at=NOW() WHERE id=$1")
        .bind(job_id)
        .execute(&state.db)
        .await?;

    emit_progress(
        &state,
        job_id,
        "running",
        0.0,
        0,
        None,
        Some("Starting conversion".into()),
    );

    // Fetch job details
    let row = sqlx::query(
        "SELECT source_format, target_format, validate_mode, generate_cert,
                source_file_path, output_file_path
         FROM conversion_jobs WHERE id=$1",
    )
    .bind(job_id)
    .fetch_one(&state.db)
    .await?;

    let source_slug: String = sqlx::Row::get(&row, "source_format");
    let target_slug: String = sqlx::Row::get(&row, "target_format");
    let validate_mode_str: String = sqlx::Row::get(&row, "validate_mode");
    let source_file: Option<String> = sqlx::Row::get(&row, "source_file_path");
    let output_file: Option<String> = sqlx::Row::get(&row, "output_file_path");

    let source_format = slug_to_format(&source_slug);
    let target_format = slug_to_format(&target_slug);

    let input_path = PathBuf::from(source_file.ok_or_else(|| anyhow::anyhow!("No source file"))?);
    let output_path = PathBuf::from(output_file.ok_or_else(|| anyhow::anyhow!("No output path"))?);

    let validation_mode = match validate_mode_str.as_str() {
        "strict" | "numeric" => ValidationMode::Numeric,
        "structural" => ValidationMode::Structural,
        _ => ValidationMode::Structural,
    };

    // Build progress callback: broadcasts SSE events
    let state_clone = state.clone();
    let tensors_done_counter = Arc::new(AtomicU64::new(0));
    let tdc = tensors_done_counter.clone();

    let progress_cb = ProgressCallback::with_handler(move |done, total, msg| {
        let _ = tdc.fetch_add(0, Ordering::Relaxed); // keep compiler happy
        let pct = if total > 0 {
            done as f32 / total as f32 * 0.95
        } else {
            0.0
        };
        emit_progress_sync(
            &state_clone,
            job_id,
            "running",
            pct,
            done as i64,
            if total > 0 { Some(total as i64) } else { None },
            Some(msg.to_string()),
        );
    });

    // Run conversion in a blocking thread (pipeline is CPU-bound + blocking I/O)
    let target_format_clone = target_format.clone();
    let source_format_clone = source_format.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut req = ConversionRequest::new(&input_path, &output_path);
        req.source_format = Some(source_format_clone);
        req.target_format = Some(target_format_clone);
        req.validation_mode = validation_mode;

        let pipeline = ConversionPipeline::new();
        pipeline.convert(req, &progress_cb)
    })
    .await?;

    match result {
        Ok(conversion_result) => {
            let output_size = tokio::fs::metadata(&conversion_result.output_path)
                .await
                .map(|m| m.len() as i64)
                .ok();

            let warnings_json = if conversion_result.warnings.is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::json!(conversion_result.warnings)
            };

            sqlx::query(
                "UPDATE conversion_jobs SET
                    status='done',
                    progress=1.0,
                    tensors_done=$2,
                    output_file_size=$3,
                    warnings=$4,
                    finished_at=NOW()
                 WHERE id=$1",
            )
            .bind(job_id)
            .bind(conversion_result.tensor_count as i64)
            .bind(output_size)
            .bind(&warnings_json)
            .execute(&state.db)
            .await?;

            emit_progress(
                &state,
                job_id,
                "done",
                1.0,
                conversion_result.tensor_count as i64,
                None,
                Some(format!(
                    "Conversion complete: {} tensors in {:.1}s",
                    conversion_result.tensor_count,
                    conversion_result.elapsed_ms as f64 / 1000.0
                )),
            );

            tracing::info!(
                job_id = %job_id,
                summary = %conversion_result.summary(),
                "Conversion done"
            );
        }
        Err(e) => {
            let msg = e.to_string();
            sqlx::query(
                "UPDATE conversion_jobs SET status='failed', error_message=$2, finished_at=NOW() WHERE id=$1"
            )
            .bind(job_id)
            .bind(&msg)
            .execute(&state.db)
            .await?;

            emit_progress(&state, job_id, "failed", 0.0, 0, None, Some(msg.clone()));
            tracing::error!(job_id = %job_id, error = %msg, "Conversion failed");
        }
    }

    Ok(())
}

fn emit_progress(
    state: &AppState,
    job_id: Uuid,
    status: &str,
    progress: f32,
    tensors_done: i64,
    tensors_total: Option<i64>,
    message: Option<String>,
) {
    let ev = ProgressEvent {
        job_id,
        status: status.to_string(),
        progress,
        tensors_done,
        tensors_total,
        last_tensor: None,
        message,
    };
    let _ = state.progress_tx.send(ev);
}

fn emit_progress_sync(
    state: &AppState,
    job_id: Uuid,
    status: &str,
    progress: f32,
    tensors_done: i64,
    tensors_total: Option<i64>,
    message: Option<String>,
) {
    emit_progress(
        state,
        job_id,
        status,
        progress,
        tensors_done,
        tensors_total,
        message,
    );
}
