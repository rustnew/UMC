use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures_util::TryStreamExt;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;
use xxhash_rust::xxh64::xxh64;

use crate::{auth::AuthUser, errors::ApiError, models::UploadResponse, state::AppState};

pub async fn upload_file(
    state: web::Data<AppState>,
    user: AuthUser,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    let upload_id = Uuid::new_v4().to_string();
    let upload_dir = PathBuf::from(&state.config.upload_dir);

    let mut file_path: Option<PathBuf> = None;
    let mut original_name = "upload".to_string();
    let mut total_size: u64 = 0;
    let mut all_bytes: Vec<u8> = Vec::new();

    if let Some(mut field) = payload
        .try_next()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Multipart error: {e}")))?
    {
        let filename = field
            .content_disposition()
            .and_then(|cd| cd.get_filename())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "upload".to_string());

        original_name = sanitize_filename(&filename);
        let dest = upload_dir.join(format!("{}_{}", upload_id, original_name));

        let mut f = std::fs::File::create(&dest)
            .map_err(|e| ApiError::Internal(format!("Cannot create upload file: {e}")))?;

        while let Some(chunk) = field
            .try_next()
            .await
            .map_err(|e| ApiError::Internal(format!("Read chunk: {e}")))?
        {
            total_size += chunk.len() as u64;
            if total_size > state.config.max_upload_bytes {
                let _ = std::fs::remove_file(&dest);
                return Err(ApiError::BadRequest("File too large".into()));
            }
            all_bytes.extend_from_slice(&chunk);
            f.write_all(&chunk)
                .map_err(|e| ApiError::Internal(format!("Write chunk: {e}")))?;
        }

        file_path = Some(dest);
    }

    let _path = file_path.ok_or_else(|| ApiError::BadRequest("No file in upload".into()))?;
    let hash = format!("{:016x}", xxh64(&all_bytes, 0));

    let detected_format = detect_format_from_name(&original_name);

    tracing::info!(
        user_id = %user.0.sub,
        upload_id = %upload_id,
        filename = %original_name,
        size = total_size,
        "File uploaded"
    );

    Ok(HttpResponse::Ok().json(UploadResponse {
        upload_id,
        filename: original_name,
        size: total_size,
        hash,
        detected_format,
    }))
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .take(128)
        .collect::<String>()
}

fn detect_format_from_name(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    if lower.ends_with(".gguf") {
        return Some("gguf".into());
    }
    if lower.ends_with(".safetensors") {
        return Some("safetensors".into());
    }
    if lower.ends_with(".onnx") {
        return Some("onnx".into());
    }
    if lower.ends_with(".pt") || lower.ends_with(".pth") || lower.ends_with(".bin") {
        return Some("pytorch".into());
    }
    if lower.ends_with(".tflite") {
        return Some("tflite".into());
    }
    None
}
