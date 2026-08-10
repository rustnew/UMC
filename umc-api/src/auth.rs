use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::ApiError;

// ── JWT Claims ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub plan: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn encode_jwt(
    user_id: Uuid,
    email: &str,
    plan: &str,
    secret: &str,
    expiry_secs: u64,
) -> Result<String, ApiError> {
    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        plan: plan.to_string(),
        exp: now + expiry_secs as usize,
        iat: now,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(format!("JWT encode: {e}")))
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, ApiError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|e| {
        tracing::debug!("JWT decode failed: {e}");
        ApiError::Unauthorized
    })
}

// ── Password ──────────────────────────────────────────────────────────────────

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| ApiError::Internal(format!("Password hash: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, ApiError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| ApiError::Internal(format!("Password parse: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// ── Refresh Token ─────────────────────────────────────────────────────────────

pub fn generate_refresh_token() -> String {
    use rand::Rng;
    let bytes: Vec<u8> = (0..48).map(|_| rand::thread_rng().gen::<u8>()).collect();
    hex::encode(bytes)
}

pub fn hash_refresh_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(token.as_bytes());
    hex::encode(h.finalize())
}

// ── Extractor ─────────────────────────────────────────────────────────────────

use actix_web::{FromRequest, HttpRequest};
use std::future::{ready, Ready};

/// Extract authenticated user from Authorization: Bearer <token>
#[derive(Clone)]
pub struct AuthUser(pub Claims);

impl FromRequest for AuthUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, ApiError>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let result = extract_claims(req);
        ready(result.map(AuthUser))
    }
}

fn extract_claims(req: &HttpRequest) -> Result<Claims, ApiError> {
    let header = req
        .headers()
        .get("Authorization")
        .ok_or(ApiError::Unauthorized)?
        .to_str()
        .map_err(|_| ApiError::Unauthorized)?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or(ApiError::Unauthorized)?;

    let secret = req
        .app_data::<actix_web::web::Data<crate::state::AppState>>()
        .ok_or_else(|| ApiError::Internal("no app state".into()))?
        .config
        .jwt_secret
        .as_str();

    decode_jwt(token, secret)
}
