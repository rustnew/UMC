use actix_web::{web, HttpResponse};
use chrono::Utc;

use crate::{errors::ApiError, state::AppState};

pub async fn health(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    // Check DB connectivity
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();

    let status = if db_ok { "ok" } else { "degraded" };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": status,
        "timestamp": Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "services": {
            "database": if db_ok { "ok" } else { "error" }
        }
    })))
}

pub async fn readiness(state: web::Data<AppState>) -> HttpResponse {
    match sqlx::query("SELECT 1").fetch_one(&state.db).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"ready": true})),
        Err(_) => HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({"ready": false, "reason": "database"})),
    }
}
