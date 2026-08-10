use actix_web::{web, HttpResponse};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    errors::ApiError,
    models::{ConversionJob, CreateJobRequest, JobListQuery},
    state::AppState,
    worker::spawn_conversion_worker,
};

// ── POST /v1/jobs ─────────────────────────────────────────────────────────────

pub async fn create_job(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<CreateJobRequest>,
) -> Result<HttpResponse, ApiError> {
    let body = body.into_inner();
    let user_id: Uuid = user.0.sub.parse().map_err(|_| ApiError::Unauthorized)?;

    // Resolve upload path
    let upload_dir = std::path::PathBuf::from(&state.config.upload_dir);
    let entries = std::fs::read_dir(&upload_dir)
        .map_err(|e| ApiError::Internal(format!("Read upload dir: {e}")))?;

    let mut source_path = None;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(&body.upload_id) {
            source_path = Some(entry.path());
            break;
        }
    }

    let source_path = source_path
        .ok_or_else(|| ApiError::NotFound(format!("Upload {} not found", body.upload_id)))?;

    let source_size = source_path.metadata().map(|m| m.len() as i64).ok();
    let job_id = Uuid::new_v4();

    let output_dir = std::path::PathBuf::from(&state.config.output_dir);
    let ext = extension_for_format(&body.target_format);
    let output_path = output_dir.join(format!("{}.{}", job_id, ext));

    sqlx::query(
        "INSERT INTO conversion_jobs
         (id, user_id, source_format, target_format, validate_mode, generate_cert,
          extra_options, source_file_path, source_file_size, output_file_path, status)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'queued')",
    )
    .bind(job_id)
    .bind(user_id)
    .bind(&body.source_format)
    .bind(&body.target_format)
    .bind(&body.validate_mode)
    .bind(body.generate_cert)
    .bind(&body.extra_options)
    .bind(source_path.to_str())
    .bind(source_size)
    .bind(output_path.to_str())
    .execute(&state.db)
    .await?;

    // Spawn background worker
    spawn_conversion_worker(state.get_ref().clone(), job_id);

    let job = fetch_job(&state, job_id).await?;
    Ok(HttpResponse::Created().json(job))
}

// ── GET /v1/jobs ──────────────────────────────────────────────────────────────

pub async fn list_jobs(
    state: web::Data<AppState>,
    user: AuthUser,
    query: web::Query<JobListQuery>,
) -> Result<HttpResponse, ApiError> {
    let user_id: Uuid = user.0.sub.parse().map_err(|_| ApiError::Unauthorized)?;

    let jobs: Vec<ConversionJob> = if let Some(status) = &query.status {
        sqlx::query_as(
            "SELECT * FROM conversion_jobs WHERE user_id=$1 AND status=$2
             ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(user_id)
        .bind(status)
        .bind(query.limit)
        .bind(query.offset)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            "SELECT * FROM conversion_jobs WHERE user_id=$1
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(query.limit)
        .bind(query.offset)
        .fetch_all(&state.db)
        .await?
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "jobs": jobs,
        "limit": query.limit,
        "offset": query.offset,
    })))
}

// ── GET /v1/jobs/:id ──────────────────────────────────────────────────────────

pub async fn get_job(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let user_id: Uuid = user.0.sub.parse().map_err(|_| ApiError::Unauthorized)?;
    let job_id = path.into_inner();
    let job = fetch_job(&state, job_id).await?;

    if job.user_id != user_id {
        return Err(ApiError::Forbidden);
    }
    Ok(HttpResponse::Ok().json(job))
}

// ── DELETE /v1/jobs/:id ───────────────────────────────────────────────────────

pub async fn cancel_job(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let user_id: Uuid = user.0.sub.parse().map_err(|_| ApiError::Unauthorized)?;
    let job_id = path.into_inner();
    let job = fetch_job(&state, job_id).await?;

    if job.user_id != user_id {
        return Err(ApiError::Forbidden);
    }

    if matches!(job.status.as_str(), "done" | "failed" | "cancelled") {
        return Err(ApiError::BadRequest("Job is already terminal".into()));
    }

    sqlx::query("UPDATE conversion_jobs SET status='cancelled', finished_at=NOW() WHERE id=$1")
        .bind(job_id)
        .execute(&state.db)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

// ── GET /v1/jobs/:id/download ─────────────────────────────────────────────────

pub async fn download_job(
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let user_id: Uuid = user.0.sub.parse().map_err(|_| ApiError::Unauthorized)?;
    let job_id = path.into_inner();
    let job = fetch_job(&state, job_id).await?;

    if job.user_id != user_id {
        return Err(ApiError::Forbidden);
    }
    if job.status != "done" {
        return Err(ApiError::BadRequest("Job is not complete".into()));
    }

    let output_path = job
        .output_file_path
        .ok_or_else(|| ApiError::NotFound("No output file".into()))?;

    let data = tokio::fs::read(&output_path)
        .await
        .map_err(|e| ApiError::Internal(format!("Read output: {e}")))?;

    let ext = std::path::Path::new(&output_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");

    let content_type = mime_for_ext(ext);
    let disposition = format!("attachment; filename=\"converted_{}.{}\"", job_id, ext);

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .insert_header(("Content-Disposition", disposition))
        .body(data))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn fetch_job(state: &AppState, job_id: Uuid) -> Result<ConversionJob, ApiError> {
    sqlx::query_as("SELECT * FROM conversion_jobs WHERE id=$1")
        .bind(job_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ApiError::NotFound(format!("Job {job_id} not found")),
            other => ApiError::from(other),
        })
}

fn extension_for_format(fmt: &str) -> &'static str {
    match fmt {
        "gguf" => "gguf",
        "safetensors" => "safetensors",
        "onnx" => "onnx",
        "pytorch" => "pt",
        "awq" => "safetensors",
        "gptq" => "safetensors",
        "tflite" => "tflite",
        "coreml" => "mlpackage",
        "tensorrt" => "engine",
        "openvino" => "xml",
        "executorch" => "pte",
        _ => "bin",
    }
}

fn mime_for_ext(ext: &str) -> &'static str {
    match ext {
        "onnx" | "gguf" | "safetensors" | "pt" | "pth" | "tflite" | "pte" | "engine" => {
            "application/octet-stream"
        }
        "json" => "application/json",
        "xml" => "application/xml",
        _ => "application/octet-stream",
    }
}
