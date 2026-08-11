use actix_web::{web, HttpRequest, HttpResponse};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::{auth::AuthUser, errors::ApiError, state::AppState};

/// GET /v1/jobs/:id/progress  — SSE endpoint
pub async fn job_progress_sse(
    _req: HttpRequest,
    state: web::Data<AppState>,
    user: AuthUser,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let user_id: Uuid = user.0.sub.parse().map_err(|_| ApiError::Unauthorized)?;
    let job_id = path.into_inner();

    // Verify ownership
    let row = sqlx::query("SELECT user_id FROM conversion_jobs WHERE id=$1")
        .bind(job_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Job {job_id}")))?;

    let owner: Uuid = sqlx::Row::get(&row, "user_id");
    if owner != user_id {
        return Err(ApiError::Forbidden);
    }

    let rx = state.progress_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |res| {
        let ev = res.ok()?;
        if ev.job_id == job_id {
            let data = serde_json::to_string(&ev).ok()?;
            Some(Ok::<_, actix_web::Error>(actix_web::web::Bytes::from(
                format!("data: {data}\n\n"),
            )))
        } else {
            None
        }
    });

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(stream))
}
