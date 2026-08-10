use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, code) = match self {
            ApiError::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, "not_found"),
            ApiError::Unauthorized => (actix_web::http::StatusCode::UNAUTHORIZED, "unauthorized"),
            ApiError::Forbidden => (actix_web::http::StatusCode::FORBIDDEN, "forbidden"),
            ApiError::BadRequest(_) => (actix_web::http::StatusCode::BAD_REQUEST, "bad_request"),
            ApiError::Conflict(_) => (actix_web::http::StatusCode::CONFLICT, "conflict"),
            ApiError::Internal(_) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
            ),
        };
        HttpResponse::build(status).json(json!({
            "error": { "code": code, "message": self.to_string() }
        }))
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => ApiError::NotFound("record not found".into()),
            _ => ApiError::Internal(e.to_string()),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        ApiError::Internal(e.to_string())
    }
}
