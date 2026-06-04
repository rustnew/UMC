use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::{
        encode_jwt, generate_refresh_token, hash_password, hash_refresh_token,
        verify_password, AuthUser,
    },
    errors::ApiError,
    models::{AuthResponse, LoginRequest, RegisterRequest, UserPublic},
    state::AppState,
};

// ── POST /auth/register ──────────────────────────────────────────────────────

pub async fn register(
    state: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, ApiError> {
    let body = body.into_inner();

    if body.email.is_empty() || !body.email.contains('@') {
        return Err(ApiError::BadRequest("Invalid email".into()));
    }
    if body.password.len() < 8 {
        return Err(ApiError::BadRequest("Password must be at least 8 characters".into()));
    }

    // Check duplicate email
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
    )
    .bind(&body.email)
    .fetch_one(&state.db)
    .await?;

    if exists {
        return Err(ApiError::Conflict("Email already registered".into()));
    }

    let password_hash = hash_password(&body.password)?;
    let user_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, display_name) VALUES ($1, $2, $3, $4)"
    )
    .bind(user_id)
    .bind(&body.email)
    .bind(&password_hash)
    .bind(body.display_name.as_deref())
    .execute(&state.db)
    .await?;

    let (access_token, refresh_token, expiry) =
        issue_tokens(&state, user_id, &body.email, "free").await?;

    Ok(HttpResponse::Created().json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        expires_in: expiry,
        user: UserPublic {
            id: user_id,
            email: body.email.clone(),
            display_name: body.display_name.clone(),
            plan: "free".into(),
            created_at: Utc::now(),
        },
    }))
}

// ── POST /auth/login ─────────────────────────────────────────────────────────

pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    let body = body.into_inner();

    let row = sqlx::query(
        "SELECT id, email, password_hash, display_name, plan, is_active, created_at
         FROM users WHERE email = $1"
    )
    .bind(&body.email)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::Unauthorized)?;

    let password_hash: String = row.get("password_hash");
    let is_active: bool = row.get("is_active");

    if !is_active {
        return Err(ApiError::Forbidden);
    }

    if !verify_password(&body.password, &password_hash)? {
        return Err(ApiError::Unauthorized);
    }

    let user_id: Uuid = row.get("id");
    let email: String = row.get("email");
    let display_name: Option<String> = row.get("display_name");
    let plan: String = row.get("plan");
    let created_at: chrono::DateTime<Utc> = row.get("created_at");

    // Update last_login_at
    let _ = sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await;

    let (access_token, refresh_token, expiry) =
        issue_tokens(&state, user_id, &email, &plan).await?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        expires_in: expiry,
        user: UserPublic {
            id: user_id,
            email,
            display_name,
            plan,
            created_at,
        },
    }))
}

// ── POST /auth/refresh ────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct RefreshRequest { pub refresh_token: String }

pub async fn refresh(
    state: web::Data<AppState>,
    body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, ApiError> {
    let token_hash = hash_refresh_token(&body.refresh_token);

    let row = sqlx::query(
        "SELECT rt.user_id, u.email, u.plan
         FROM refresh_tokens rt JOIN users u ON u.id = rt.user_id
         WHERE rt.token_hash = $1
           AND rt.revoked = FALSE
           AND rt.expires_at > NOW()"
    )
    .bind(&token_hash)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::Unauthorized)?;

    let user_id: Uuid = row.get("user_id");
    let email: String = row.get("email");
    let plan: String = row.get("plan");

    // Revoke old token
    sqlx::query("UPDATE refresh_tokens SET revoked = TRUE WHERE token_hash = $1")
        .bind(&token_hash)
        .execute(&state.db)
        .await?;

    let (access_token, new_refresh_token, expiry) =
        issue_tokens(&state, user_id, &email, &plan).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": new_refresh_token,
        "token_type": "Bearer",
        "expires_in": expiry,
    })))
}

// ── POST /auth/logout ─────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct LogoutRequest { pub refresh_token: Option<String> }

pub async fn logout(
    state: web::Data<AppState>,
    _user: AuthUser,
    body: web::Json<LogoutRequest>,
) -> Result<HttpResponse, ApiError> {
    if let Some(rt) = &body.refresh_token {
        let hash = hash_refresh_token(rt);
        sqlx::query("UPDATE refresh_tokens SET revoked = TRUE WHERE token_hash = $1")
            .bind(&hash)
            .execute(&state.db)
            .await?;
    }
    Ok(HttpResponse::NoContent().finish())
}

// ── GET /auth/me ─────────────────────────────────────────────────────────────

pub async fn me(
    state: web::Data<AppState>,
    user: AuthUser,
) -> Result<HttpResponse, ApiError> {
    let uid: Uuid = user.0.sub.parse().map_err(|_| ApiError::Unauthorized)?;
    let row = sqlx::query(
        "SELECT id, email, display_name, plan, created_at FROM users WHERE id = $1"
    )
    .bind(uid)
    .fetch_one(&state.db)
    .await?;

    Ok(HttpResponse::Ok().json(UserPublic {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        plan: row.get("plan"),
        created_at: row.get("created_at"),
    }))
}

// ── Helper ───────────────────────────────────────────────────────────────────

async fn issue_tokens(
    state: &AppState,
    user_id: Uuid,
    email: &str,
    plan: &str,
) -> Result<(String, String, u64), ApiError> {
    let expiry = state.config.jwt_access_expiry_secs;
    let access_token = encode_jwt(user_id, email, plan, &state.config.jwt_secret, expiry)?;

    let refresh_token = generate_refresh_token();
    let refresh_hash = hash_refresh_token(&refresh_token);
    let refresh_expiry = chrono::Duration::seconds(
        state.config.jwt_refresh_expiry_secs as i64
    );
    let expires_at = Utc::now() + refresh_expiry;

    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)"
    )
    .bind(user_id)
    .bind(&refresh_hash)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    Ok((access_token, refresh_token, expiry))
}
