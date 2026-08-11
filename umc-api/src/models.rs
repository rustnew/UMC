use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── Auth DTOs ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserPublic,
}

#[derive(Debug, Serialize)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub plan: String,
    pub created_at: DateTime<Utc>,
}

// ── Job ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ConversionJob {
    pub id: Uuid,
    pub user_id: Uuid,
    pub source_format: String,
    pub target_format: String,
    pub validate_mode: String,
    pub generate_cert: bool,
    pub extra_options: serde_json::Value,
    pub source_file_path: Option<String>,
    pub source_file_size: Option<i64>,
    pub source_file_hash: Option<String>,
    pub output_file_path: Option<String>,
    pub output_file_size: Option<i64>,
    pub output_file_hash: Option<String>,
    pub status: String,
    pub progress: f32,
    pub tensors_done: i64,
    pub tensors_total: Option<i64>,
    pub last_tensor: Option<String>,
    pub error_message: Option<String>,
    pub warnings: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub source_format: String,
    pub target_format: String,
    #[serde(default = "default_validate_mode")]
    pub validate_mode: String,
    #[serde(default)]
    pub generate_cert: bool,
    #[serde(default)]
    pub extra_options: serde_json::Value,
    pub upload_id: String,
}

fn default_validate_mode() -> String {
    "structural".into()
}

#[derive(Debug, Deserialize, Default)]
pub struct JobListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub status: Option<String>,
}

fn default_limit() -> i64 {
    20
}

// ── Upload ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub upload_id: String,
    pub filename: String,
    pub size: u64,
    pub hash: String,
    pub detected_format: Option<String>,
}

// ── Progress ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub job_id: Uuid,
    pub status: String,
    pub progress: f32,
    pub tensors_done: i64,
    pub tensors_total: Option<i64>,
    pub last_tensor: Option<String>,
    pub message: Option<String>,
}

// ── Format ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FormatInfo {
    pub slug: String,
    pub name: String,
    pub can_read: bool,
    pub can_write: bool,
    pub native: bool,
    pub extensions: Vec<String>,
    pub description: String,
}
