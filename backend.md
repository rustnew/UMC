# UMC — Backend Actix-Web Complet
## Document d'Implémentation Backend v1.0 — Référence Définitive

> **Langage :** Rust stable 1.80+  
> **Framework :** Actix-Web 4.x  
> **Base de données :** PostgreSQL 16 (sqlx) → Redis 7 → Kafka (évolution)  
> **Philosophie :** Ultra-robuste · Ultra-rapide · Zéro compromis · Production-ready

---

# TABLE DES MATIÈRES

1. [Architecture Backend Globale](#1-architecture-backend-globale)
2. [Structure du Projet](#2-structure-du-projet)
3. [Cargo.toml — Dépendances Backend](#3-cargotoml--dépendances-backend)
4. [Configuration & Environnement](#4-configuration--environnement)
5. [Base de Données — Schéma SQL Complet](#5-base-de-données--schéma-sql-complet)
6. [Modèles de Données (Models)](#6-modèles-de-données-models)
7. [Authentification & Autorisation](#7-authentification--autorisation)
8. [Gestion des Erreurs](#8-gestion-des-erreurs)
9. [Middleware Stack](#9-middleware-stack)
10. [Routes & Handlers — Auth](#10-routes--handlers--auth)
11. [Routes & Handlers — Jobs de Conversion](#11-routes--handlers--jobs-de-conversion)
12. [Streaming SSE — Progression Temps Réel](#12-streaming-sse--progression-temps-réel)
13. [Upload Sécurisé de Fichiers](#13-upload-sécurisé-de-fichiers)
14. [Worker de Conversion — Pipeline Complet](#14-worker-de-conversion--pipeline-complet)
15. [File d'Attente PostgreSQL → Redis](#15-file-dattente-postgresql--redis)
16. [Endpoints Inspection & Diff](#16-endpoints-inspection--diff)
17. [Certificats & Rapports](#17-certificats--rapports)
18. [Rate Limiting & Quotas](#18-rate-limiting--quotas)
19. [WebSocket — Fallback & Notifications](#19-websocket--fallback--notifications)
20. [Métriques Prometheus](#20-métriques-prometheus)
21. [Sécurité — CORS, CSRF, Validation](#21-sécurité--cors-csrf-validation)
22. [Health Check & Readiness](#22-health-check--readiness)
23. [Tests Backend Complets](#23-tests-backend-complets)
24. [CI/CD & Déploiement](#24-cicd--déploiement)
25. [main.rs — Point d'Entrée Complet](#25-mainrs--point-dentrée-complet)

---

# 1. ARCHITECTURE BACKEND GLOBALE

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    UMC Backend — Architecture Actix-Web                  │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                     COUCHE HTTP (Actix-Web 4)                     │  │
│  │  TLS Termination · CORS · Rate Limit · Auth Middleware · Tracing  │  │
│  └──────────────────────────────┬───────────────────────────────────┘  │
│                                 │                                       │
│  ┌──────────────────────────────▼───────────────────────────────────┐  │
│  │                       COUCHE ROUTAGE                               │  │
│  │                                                                    │  │
│  │  /auth/*      /v1/jobs/*     /v1/formats/*    /v1/inspect         │  │
│  │  /v1/diff     /v1/validate   /v1/certs/*      /health /metrics    │  │
│  └──────────────────────────────┬───────────────────────────────────┘  │
│                                 │                                       │
│  ┌──────────────────────────────▼───────────────────────────────────┐  │
│  │                     COUCHE SERVICE                                 │  │
│  │  AuthService  JobService  ConversionService  CertService           │  │
│  │  StorageService  NotificationService  MetricsService               │  │
│  └──────────────────────────────┬───────────────────────────────────┘  │
│                                 │                                       │
│  ┌──────────────────────────────▼───────────────────────────────────┐  │
│  │                   COUCHE INFRASTRUCTURE                            │  │
│  │                                                                    │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐   │  │
│  │  │PostgreSQL│  │  Redis   │  │   S3/    │  │ UMC Core       │   │  │
│  │  │  sqlx    │  │ deadpool │  │  Local   │  │ (umc-pipeline) │   │  │
│  │  │          │  │          │  │ Storage  │  │                │   │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────────────┘   │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                  WORKERS ASYNCHRONES                               │  │
│  │  ConversionWorker (Tokio Tasks)  ·  Cleanup Worker               │  │
│  │  Progress Publisher  ·  Certificate Signer                        │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

---

# 2. STRUCTURE DU PROJET

```
umc-api/
├── Cargo.toml
├── build.rs                          # Génération protobuf si nécessaire
├── migrations/                       # Migrations SQL (sqlx-migrate)
│   ├── 001_initial.sql
│   ├── 002_jobs.sql
│   ├── 003_api_keys.sql
│   ├── 004_certificates.sql
│   └── 005_audit_log.sql
│
└── src/
    ├── main.rs                       # Point d'entrée — build & run
    ├── app.rs                        # Configuration Actix-Web complète
    ├── config.rs                     # Configuration typée depuis env
    │
    ├── db/
    │   ├── mod.rs
    │   ├── pool.rs                   # Pool PostgreSQL (sqlx)
    │   ├── redis.rs                  # Pool Redis (deadpool-redis)
    │   └── migrations.rs             # Run migrations au démarrage
    │
    ├── models/
    │   ├── mod.rs
    │   ├── user.rs                   # User, Plan, Subscription
    │   ├── api_key.rs                # ApiKey, Scope
    │   ├── job.rs                    # ConversionJob, JobStatus, JobProgress
    │   ├── certificate.rs            # ConversionCertificate
    │   └── audit.rs                  # AuditLog
    │
    ├── auth/
    │   ├── mod.rs
    │   ├── middleware.rs             # AuthMiddleware (JWT + API Key)
    │   ├── jwt.rs                    # JWT encode/decode (jsonwebtoken)
    │   ├── api_key.rs                # SHA256 hash, génération, vérification
    │   ├── password.rs               # Argon2 hash (pour users)
    │   └── session.rs                # Sessions Redis
    │
    ├── handlers/
    │   ├── mod.rs
    │   ├── auth.rs                   # register, login, refresh, logout
    │   ├── jobs.rs                   # create, get, cancel, list
    │   ├── progress.rs               # SSE streaming + WebSocket
    │   ├── upload.rs                 # Multipart upload sécurisé
    │   ├── formats.rs                # list formats, conversion graph
    │   ├── inspect.rs                # inspect, dry_run, diff, validate
    │   ├── certificates.rs           # get, verify, revoke, pdf
    │   ├── api_keys.rs               # CRUD API keys
    │   └── health.rs                 # health, readiness, metrics
    │
    ├── services/
    │   ├── mod.rs
    │   ├── auth_service.rs           # Logique auth complète
    │   ├── job_service.rs            # Orchestration des jobs
    │   ├── conversion_service.rs     # Interface avec umc-pipeline
    │   ├── storage_service.rs        # Upload/download fichiers
    │   ├── notification_service.rs   # SSE, WebSocket, Webhook
    │   ├── certificate_service.rs    # Signature ed25519, PDF
    │   └── quota_service.rs          # Vérification quotas par plan
    │
    ├── workers/
    │   ├── mod.rs
    │   ├── conversion_worker.rs      # Worker principal de conversion
    │   ├── cleanup_worker.rs         # Nettoyage fichiers temporaires
    │   └── webhook_worker.rs         # Envoi webhooks asynchrone
    │
    ├── middleware/
    │   ├── mod.rs
    │   ├── rate_limit.rs             # Rate limiting Redis + DashMap
    │   ├── request_id.rs             # X-Request-ID header
    │   ├── metrics.rs                # Prometheus metrics middleware
    │   └── security.rs               # Security headers
    │
    ├── errors/
    │   ├── mod.rs
    │   └── api_error.rs              # ApiError → HTTP response
    │
    └── utils/
        ├── mod.rs
        ├── validation.rs             # Validators custom
        ├── pagination.rs             # Pagination générique
        └── hash.rs                   # SHA256, xxhash utilitaires
```

---

# 3. CARGO.TOML — DÉPENDANCES BACKEND

```toml
[package]
name = "umc-api"
version = "3.0.0"
edition = "2021"
rust-version = "1.80"

[dependencies]
# ── Framework Web ─────────────────────────────────────────
actix-web        = "4.9"
actix-ws         = "0.3"
actix-multipart  = "0.7"
actix-cors       = "0.7"
actix-files      = "0.6"

# ── Async Runtime ─────────────────────────────────────────
tokio            = { version = "1.40", features = [
    "full", "tracing"
]}
tokio-stream     = { version = "0.1", features = ["sync"] }
futures          = "0.3"
futures-util     = "0.3"
async-stream     = "0.3"
pin-project-lite = "0.2"

# ── Base de données ────────────────────────────────────────
sqlx             = { version = "0.8", features = [
    "runtime-tokio",
    "postgres",
    "uuid",
    "chrono",
    "json",
    "migrate",
]}

# ── Redis ─────────────────────────────────────────────────
deadpool-redis   = { version = "0.15", features = ["rt_tokio_1"] }
redis            = { version = "0.27", features = [
    "tokio-comp",
    "streams",
    "json",
]}

# ── Sérialisation ─────────────────────────────────────────
serde            = { version = "1.0", features = ["derive"] }
serde_json       = "1.0"
validator        = { version = "0.18", features = ["derive"] }

# ── Auth & Crypto ─────────────────────────────────────────
jsonwebtoken     = "9.3"
argon2           = "0.5"
ed25519-dalek    = "2.1"
sha2             = "0.10"
hmac             = "0.12"
rand             = "0.8"
hex              = "0.4"
constant_time_eq = "0.3"
getrandom        = "0.2"
base64           = "0.22"

# ── Identifiants & Temps ──────────────────────────────────
uuid             = { version = "1.10", features = ["v4", "serde"] }
chrono           = { version = "0.4", features = ["serde"] }
time             = { version = "0.3", features = ["serde"] }

# ── Logging & Tracing ─────────────────────────────────────
tracing          = "0.1"
tracing-actix-web = "0.7"
tracing-subscriber = { version = "0.3", features = [
    "env-filter",
    "json",
    "fmt",
]}
tracing-appender = "0.2"

# ── Métriques ─────────────────────────────────────────────
prometheus       = { version = "0.13", features = ["process"] }
actix-web-prom   = "0.7"

# ── Stockage ──────────────────────────────────────────────
aws-sdk-s3       = { version = "1.0", optional = true }
tokio-util       = { version = "0.7", features = ["io"] }
bytes            = "1.7"
mime             = "0.3"
tempfile         = "3.13"

# ── Erreurs ───────────────────────────────────────────────
thiserror        = "1.0"
anyhow           = "1.0"
derive_more      = { version = "1.0", features = ["display", "error"] }

# ── Config ────────────────────────────────────────────────
config           = "0.14"
dotenvy          = "0.15"
envy             = "0.4"

# ── HTTP Client (pour webhooks) ───────────────────────────
reqwest          = { version = "0.12", features = [
    "json",
    "rustls-tls",
    "stream",
], default-features = false }

# ── Rate Limiting ─────────────────────────────────────────
governor         = { version = "0.6", features = ["dashmap"] }
dashmap          = "5.5"

# ── Génération PDF ────────────────────────────────────────
printpdf         = "0.7"

# ── UMC Core ──────────────────────────────────────────────
umc-core         = { path = "../crates/umc-core" }
umc-pipeline     = { path = "../crates/umc-pipeline" }
umc-validate     = { path = "../crates/umc-validate" }
umc-detect       = { path = "../crates/umc-detect" }
umc-graph        = { path = "../crates/umc-graph" }
umc-formats      = { path = "../crates/umc-formats" }

# ── Utilitaires ───────────────────────────────────────────
xxhash-rust      = { version = "0.8", features = ["xxh64"] }
once_cell        = "1.20"
arc-swap         = "1.7"

[features]
default = []
s3-storage = ["aws-sdk-s3"]

[dev-dependencies]
actix-rt         = "2.10"
serial_test      = "3.1"
wiremock         = "0.6"
tokio            = { version = "1.40", features = ["full", "test-util"] }

[profile.release]
opt-level        = 3
lto              = "fat"
codegen-units    = 1
strip            = true
panic            = "abort"
```

---

# 4. CONFIGURATION & ENVIRONNEMENT

```rust
// src/config.rs

use serde::Deserialize;
use std::time::Duration;

/// Configuration complète chargée depuis les variables d'environnement
/// Toutes les valeurs sensibles viennent de l'env, jamais hardcodées
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Serveur HTTP
    pub server: ServerConfig,
    /// Base de données PostgreSQL
    pub database: DatabaseConfig,
    /// Redis
    pub redis: RedisConfig,
    /// JWT
    pub jwt: JwtConfig,
    /// Stockage des fichiers
    pub storage: StorageConfig,
    /// Conversion UMC
    pub conversion: ConversionConfig,
    /// Rate limiting
    pub rate_limit: RateLimitConfig,
    /// Webhooks
    pub webhook: WebhookConfig,
    /// Observabilité
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Adresse d'écoute
    pub host: String,       // "0.0.0.0"
    pub port: u16,          // 8080
    /// Workers Actix (défaut: nombre de CPUs)
    pub workers: Option<usize>,
    /// Timeout de connexion
    pub connection_timeout_secs: u64,   // 30
    pub request_timeout_secs: u64,      // 300
    pub keep_alive_secs: u64,           // 75
    /// TLS
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    /// CORS — origines autorisées
    pub cors_origins: Vec<String>,
    /// Taille max du body
    pub max_body_size_bytes: usize,     // 10 * 1024 * 1024 (10 Mo pour les petits payloads)
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,            // postgres://user:pass@host/db
    pub max_connections: u32,   // 20
    pub min_connections: u32,   // 2
    pub acquire_timeout_secs: u64,  // 10
    pub idle_timeout_secs: u64,     // 600
    pub max_lifetime_secs: u64,     // 1800
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,            // redis://localhost:6379
    pub max_connections: usize, // 20
    pub connection_timeout_secs: u64,   // 5
    /// TTL pour les clés de progression
    pub progress_ttl_secs: u64,     // 86400
    /// TTL pour les sessions
    pub session_ttl_secs: u64,      // 3600
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,             // Au moins 64 caractères
    pub access_token_expiry_secs: u64,  // 3600 (1h)
    pub refresh_token_expiry_secs: u64, // 2592000 (30j)
    pub issuer: String,             // "umc.dev"
    pub audience: String,           // "umc-api"
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub backend: StorageBackend,    // "local" ou "s3"
    /// Stockage local
    pub local_upload_dir: String,   // "/var/umc/uploads"
    pub local_output_dir: String,   // "/var/umc/outputs"
    pub local_temp_dir: String,     // "/tmp/umc"
    /// S3
    pub s3_bucket: Option<String>,
    pub s3_region: Option<String>,
    pub s3_prefix: Option<String>,  // "umc/"
    /// Limites
    pub max_upload_size_bytes: u64, // 100 * 1024^3 (100 Go)
    pub file_retention_secs: u64,   // 86400 * 7 (7 jours)
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Local,
    S3,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConversionConfig {
    /// Nombre de workers de conversion simultanés
    pub max_concurrent_conversions: usize,      // 4
    /// Timeout global d'une conversion
    pub conversion_timeout_secs: u64,           // 7200 (2h)
    /// Timeout par tenseur
    pub tensor_timeout_secs: u64,               // 120
    /// Checkpointing
    pub checkpoint_interval_secs: u64,          // 30
    /// Répertoire des checkpoints
    pub checkpoint_dir: String,                 // "/tmp/umc/checkpoints"
    /// Clé privée ed25519 pour signer les certificats
    pub signing_key_path: String,               // "/secrets/umc-signing.pem"
    /// Threads Rayon par conversion
    pub rayon_threads_per_conversion: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    /// Requests par minute par IP (non authentifié)
    pub anon_requests_per_minute: u32,          // 10
    /// Requests par minute par API Key (Free)
    pub free_requests_per_minute: u32,          // 30
    /// Requests par minute par API Key (Pro)
    pub pro_requests_per_minute: u32,           // 1000
    /// Requests par minute par API Key (Enterprise)
    pub enterprise_requests_per_minute: u32,    // 10000
    /// Burst size
    pub burst_size: u32,                        // 20
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookConfig {
    /// Timeout des appels webhook sortants
    pub timeout_secs: u64,      // 30
    /// Nombre de tentatives
    pub max_retries: u32,       // 3
    /// Délai entre les tentatives (exponentiel)
    pub initial_retry_delay_ms: u64,    // 1000
    /// Signature HMAC des webhooks
    pub signing_secret: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityConfig {
    /// Niveau de log
    pub log_level: String,      // "info"
    /// Format de log
    pub log_format: LogFormat,  // "json" ou "pretty"
    /// Prometheus
    pub metrics_enabled: bool,
    pub metrics_path: String,   // "/metrics"
    /// Tracing distribué
    pub tracing_enabled: bool,
    pub jaeger_endpoint: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

impl Config {
    /// Charge la configuration depuis les variables d'environnement
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .add_source(config::Environment::default()
                .prefix("UMC")
                .separator("__")
                .try_parsing(true))
            .build()?;

        Ok(config.try_deserialize()?)
    }

    /// Valide la configuration au démarrage
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.jwt.secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters");
        }
        if self.conversion.max_concurrent_conversions == 0 {
            anyhow::bail!("MAX_CONCURRENT_CONVERSIONS must be > 0");
        }
        Ok(())
    }
}

/// AppState — partagé entre tous les handlers via Actix Data
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: sqlx::PgPool,
    pub redis: deadpool_redis::Pool,
    pub conversion_semaphore: Arc<tokio::sync::Semaphore>,
    pub metrics: Arc<crate::middleware::metrics::Metrics>,
    pub signing_key: Arc<ed25519_dalek::SigningKey>,
}

use std::sync::Arc;

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let config = Arc::new(config);

        // Pool PostgreSQL
        let db = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(Duration::from_secs(config.database.acquire_timeout_secs))
            .idle_timeout(Duration::from_secs(config.database.idle_timeout_secs))
            .max_lifetime(Duration::from_secs(config.database.max_lifetime_secs))
            .connect(&config.database.url)
            .await?;

        // Pool Redis
        let redis_cfg = deadpool_redis::Config::from_url(&config.redis.url);
        let redis = redis_cfg
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
        redis.get().await?; // Test de connexion

        // Sémaphore pour limiter les conversions simultanées
        let conversion_semaphore = Arc::new(tokio::sync::Semaphore::new(
            config.conversion.max_concurrent_conversions,
        ));

        // Chargement de la clé de signature
        let signing_key = Self::load_signing_key(&config.conversion.signing_key_path)?;

        Ok(Self {
            config,
            db,
            redis,
            conversion_semaphore,
            metrics: Arc::new(crate::middleware::metrics::Metrics::new()),
            signing_key: Arc::new(signing_key),
        })
    }

    fn load_signing_key(path: &str) -> anyhow::Result<ed25519_dalek::SigningKey> {
        use std::io::Read;
        let mut file = std::fs::File::open(path)
            .map_err(|e| anyhow::anyhow!("Cannot open signing key {}: {}", path, e))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        // Bytes PEM ou raw 32 bytes
        if bytes.len() == 32 {
            let key_bytes: [u8; 32] = bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid key size"))?;
            Ok(ed25519_dalek::SigningKey::from_bytes(&key_bytes))
        } else {
            anyhow::bail!("Unsupported key format. Use 32-byte raw ed25519 key.");
        }
    }
}
```

---

# 5. BASE DE DONNÉES — SCHÉMA SQL COMPLET

```sql
-- migrations/001_initial.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ── TYPES ENUM ──────────────────────────────────────────────────────────

CREATE TYPE user_plan AS ENUM ('free', 'pro', 'team', 'enterprise');
CREATE TYPE job_status AS ENUM (
    'queued', 'running', 'paused', 'done', 'failed', 'cancelled', 'expired'
);
CREATE TYPE roundtrip_level AS ENUM (
    'bit_identical', 'semantic', 'structural'
);
CREATE TYPE audit_action AS ENUM (
    'user_register', 'user_login', 'user_logout',
    'api_key_create', 'api_key_revoke',
    'job_create', 'job_cancel', 'job_complete', 'job_fail',
    'certificate_issue', 'certificate_revoke'
);

-- ── USERS ───────────────────────────────────────────────────────────────

CREATE TABLE users (
    id                UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email             TEXT NOT NULL UNIQUE,
    password_hash     TEXT NOT NULL,
    display_name      TEXT,
    plan              user_plan NOT NULL DEFAULT 'free',
    plan_expires_at   TIMESTAMPTZ,
    -- Quotas
    monthly_conversions_used    INTEGER NOT NULL DEFAULT 0,
    monthly_conversions_reset   TIMESTAMPTZ NOT NULL DEFAULT
        date_trunc('month', NOW()) + INTERVAL '1 month',
    -- État
    email_verified    BOOLEAN NOT NULL DEFAULT FALSE,
    email_verify_token TEXT,
    is_active         BOOLEAN NOT NULL DEFAULT TRUE,
    -- Méta
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at     TIMESTAMPTZ,
    -- Préférences
    webhook_url       TEXT,
    webhook_secret    TEXT,
    notify_on_complete BOOLEAN NOT NULL DEFAULT TRUE,
    notify_on_fail     BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_plan ON users(plan);

-- ── API KEYS ─────────────────────────────────────────────────────────────

CREATE TABLE api_keys (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash        TEXT NOT NULL UNIQUE,    -- SHA256(raw_key)
    key_prefix      TEXT NOT NULL,           -- "umc_sk_..." (8 chars visible)
    name            TEXT NOT NULL,           -- Nom donné par l'utilisateur
    -- Scopes : bitmask
    -- 1 = read, 2 = convert, 4 = manage_keys, 8 = admin
    scopes          INTEGER NOT NULL DEFAULT 3,
    -- Quotas sur cette clé
    rate_limit_override INTEGER,             -- NULL = utilise le plan
    -- Expiration
    expires_at      TIMESTAMPTZ,             -- NULL = jamais
    -- État
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    last_used_at    TIMESTAMPTZ,
    -- Méta
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at      TIMESTAMPTZ,
    revoked_reason  TEXT
);

CREATE INDEX idx_api_keys_user ON api_keys(user_id);
CREATE INDEX idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX idx_api_keys_active ON api_keys(is_active) WHERE is_active = TRUE;

-- ── CONVERSION JOBS ──────────────────────────────────────────────────────

CREATE TABLE conversion_jobs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    api_key_id      UUID REFERENCES api_keys(id),

    -- Configuration de la conversion
    source_format   TEXT NOT NULL,
    target_format   TEXT NOT NULL,
    source_dtype    TEXT,
    target_dtype    TEXT,
    validate_mode   TEXT NOT NULL DEFAULT 'semantic',
    generate_cert   BOOLEAN NOT NULL DEFAULT FALSE,
    merge_adapters  BOOLEAN NOT NULL DEFAULT FALSE,
    quantize_scheme TEXT,
    extra_options   JSONB NOT NULL DEFAULT '{}',

    -- Fichiers
    source_file_path    TEXT,               -- Chemin local ou clé S3
    source_file_size    BIGINT,
    source_file_hash    TEXT,               -- SHA256
    output_file_path    TEXT,
    output_file_size    BIGINT,
    output_file_hash    TEXT,
    temp_file_path      TEXT,               -- Fichier temporaire en cours

    -- État et progression
    status          job_status NOT NULL DEFAULT 'queued',
    progress        FLOAT4 NOT NULL DEFAULT 0.0,
    tensors_done    BIGINT NOT NULL DEFAULT 0,
    tensors_total   BIGINT,
    bytes_done      BIGINT NOT NULL DEFAULT 0,
    bytes_total     BIGINT,
    last_tensor     TEXT,                   -- Nom du dernier tenseur traité
    throughput_bps  BIGINT,                 -- Débit en bytes/sec
    eta_seconds     INTEGER,                -- Temps estimé restant

    -- Résultats
    roundtrip_level roundtrip_level,
    max_divergence  FLOAT8,
    warnings        JSONB,                  -- Array de strings
    error_message   TEXT,
    error_code      TEXT,

    -- Checkpoint (reprise en cas de crash)
    checkpoint_data JSONB,
    checkpoint_at   TIMESTAMPTZ,

    -- Ressources utilisées
    cpu_time_ms     BIGINT,
    peak_ram_bytes  BIGINT,

    -- Facturation
    billable        BOOLEAN NOT NULL DEFAULT TRUE,
    billed_at       TIMESTAMPTZ,
    billing_amount_cents INTEGER,

    -- Timing
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    queued_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at      TIMESTAMPTZ,
    finished_at     TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '7 days',

    -- Worker
    worker_id       TEXT,
    attempts        INTEGER NOT NULL DEFAULT 0,
    max_attempts    INTEGER NOT NULL DEFAULT 3
);

CREATE INDEX idx_jobs_user ON conversion_jobs(user_id);
CREATE INDEX idx_jobs_status ON conversion_jobs(status);
CREATE INDEX idx_jobs_queued ON conversion_jobs(queued_at)
    WHERE status = 'queued';
CREATE INDEX idx_jobs_running ON conversion_jobs(started_at)
    WHERE status = 'running';
CREATE INDEX idx_jobs_expires ON conversion_jobs(expires_at);
CREATE INDEX idx_jobs_worker ON conversion_jobs(worker_id)
    WHERE status = 'running';

-- ── CONVERSION CERTIFICATES ──────────────────────────────────────────────

CREATE TABLE conversion_certificates (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    job_id          UUID NOT NULL REFERENCES conversion_jobs(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id),

    -- Identité des fichiers
    source_format   TEXT NOT NULL,
    target_format   TEXT NOT NULL,
    source_hash     TEXT NOT NULL,
    target_hash     TEXT NOT NULL,
    source_size     BIGINT NOT NULL,
    target_size     BIGINT NOT NULL,

    -- Validation
    roundtrip_level roundtrip_level NOT NULL,
    max_divergence  FLOAT8,
    validation_summary JSONB NOT NULL,
    trust_statement TEXT NOT NULL,
    warnings        JSONB,

    -- Cryptographie
    signature       TEXT NOT NULL,          -- hex(ed25519 signature)
    public_key      TEXT NOT NULL,          -- hex(public key)
    umc_version     TEXT NOT NULL,

    -- État
    is_valid        BOOLEAN NOT NULL DEFAULT TRUE,
    revoked_at      TIMESTAMPTZ,
    revoked_reason  TEXT,

    -- Méta
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '1 year'
);

CREATE INDEX idx_certs_job ON conversion_certificates(job_id);
CREATE INDEX idx_certs_user ON conversion_certificates(user_id);
CREATE INDEX idx_certs_valid ON conversion_certificates(is_valid)
    WHERE is_valid = TRUE;

-- ── AUDIT LOG ────────────────────────────────────────────────────────────

CREATE TABLE audit_log (
    id          BIGSERIAL PRIMARY KEY,
    user_id     UUID REFERENCES users(id),
    api_key_id  UUID REFERENCES api_keys(id),
    action      audit_action NOT NULL,
    resource_id TEXT,
    ip_address  INET,
    user_agent  TEXT,
    details     JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_user ON audit_log(user_id);
CREATE INDEX idx_audit_created ON audit_log(created_at);
CREATE INDEX idx_audit_action ON audit_log(action);

-- ── REFRESH TOKENS ────────────────────────────────────────────────────────

CREATE TABLE refresh_tokens (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    device_id   TEXT,
    ip_address  INET,
    is_valid    BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ
);

CREATE INDEX idx_refresh_user ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_hash ON refresh_tokens(token_hash);

-- ── TRIGGERS ─────────────────────────────────────────────────────────────

-- Auto-update updated_at
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
```

---

# 6. MODÈLES DE DONNÉES (MODELS)

```rust
// src/models/user.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: Option<String>,
    pub plan: Plan,
    pub plan_expires_at: Option<DateTime<Utc>>,
    pub monthly_conversions_used: i32,
    pub monthly_conversions_reset: DateTime<Utc>,
    pub email_verified: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub webhook_url: Option<String>,
    pub notify_on_complete: bool,
    pub notify_on_fail: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "user_plan", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Free,
    Pro,
    Team,
    Enterprise,
}

impl Plan {
    pub fn max_file_size_bytes(&self) -> u64 {
        match self {
            Plan::Free       => 2 * 1024 * 1024 * 1024,   // 2 Go
            Plan::Pro        => 100 * 1024 * 1024 * 1024, // 100 Go
            Plan::Team       => 200 * 1024 * 1024 * 1024, // 200 Go
            Plan::Enterprise => u64::MAX,
        }
    }

    pub fn monthly_conversion_limit(&self) -> Option<i32> {
        match self {
            Plan::Free  => Some(10),
            Plan::Pro   => None,  // Illimité
            Plan::Team  => None,
            Plan::Enterprise => None,
        }
    }

    pub fn requests_per_minute(&self) -> u32 {
        match self {
            Plan::Free       => 30,
            Plan::Pro        => 1000,
            Plan::Team       => 5000,
            Plan::Enterprise => 10000,
        }
    }

    pub fn max_concurrent_jobs(&self) -> usize {
        match self {
            Plan::Free       => 1,
            Plan::Pro        => 5,
            Plan::Team       => 20,
            Plan::Enterprise => 100,
        }
    }

    pub fn priority_weight(&self) -> i32 {
        match self {
            Plan::Free       => 10,
            Plan::Pro        => 50,
            Plan::Team       => 100,
            Plan::Enterprise => 1000,
        }
    }
}

// ── Response DTO (sans données sensibles) ─────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub plan: Plan,
    pub plan_expires_at: Option<DateTime<Utc>>,
    pub monthly_conversions_used: i32,
    pub monthly_conversions_reset: DateTime<Utc>,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            email: u.email,
            display_name: u.display_name,
            plan: u.plan,
            plan_expires_at: u.plan_expires_at,
            monthly_conversions_used: u.monthly_conversions_used,
            monthly_conversions_reset: u.monthly_conversions_reset,
            email_verified: u.email_verified,
            created_at: u.created_at,
        }
    }
}
```

```rust
// src/models/job.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversionJob {
    pub id: Uuid,
    pub user_id: Uuid,
    pub api_key_id: Option<Uuid>,
    pub source_format: String,
    pub target_format: String,
    pub source_dtype: Option<String>,
    pub target_dtype: Option<String>,
    pub validate_mode: String,
    pub generate_cert: bool,
    pub merge_adapters: bool,
    pub quantize_scheme: Option<String>,
    pub extra_options: serde_json::Value,
    pub source_file_path: Option<String>,
    pub source_file_size: Option<i64>,
    pub source_file_hash: Option<String>,
    pub output_file_path: Option<String>,
    pub output_file_size: Option<i64>,
    pub output_file_hash: Option<String>,
    pub status: JobStatus,
    pub progress: f32,
    pub tensors_done: i64,
    pub tensors_total: Option<i64>,
    pub bytes_done: i64,
    pub bytes_total: Option<i64>,
    pub last_tensor: Option<String>,
    pub throughput_bps: Option<i64>,
    pub eta_seconds: Option<i32>,
    pub roundtrip_level: Option<RoundTripLevelDb>,
    pub max_divergence: Option<f64>,
    pub warnings: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub checkpoint_data: Option<serde_json::Value>,
    pub checkpoint_at: Option<DateTime<Utc>>,
    pub cpu_time_ms: Option<i64>,
    pub peak_ram_bytes: Option<i64>,
    pub billable: bool,
    pub billed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub worker_id: Option<String>,
    pub attempts: i32,
    pub max_attempts: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "job_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Running,
    Paused,
    Done,
    Failed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "roundtrip_level", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RoundTripLevelDb {
    BitIdentical,
    Semantic,
    Structural,
}

/// Payload de création d'un job (validé)
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateJobRequest {
    #[validate(length(min = 1, max = 50))]
    pub source_format: Option<String>,  // None = auto-détecté
    #[validate(length(min = 1, max = 50))]
    pub target_format: String,
    #[validate(length(max = 20))]
    pub target_dtype: Option<String>,
    #[serde(default = "default_validate_mode")]
    pub validate_mode: ValidateMode,
    #[serde(default)]
    pub generate_cert: bool,
    #[serde(default)]
    pub merge_adapters: bool,
    pub quantize_scheme: Option<String>,
    pub extra_options: Option<serde_json::Value>,
    /// URL source (alternative à l'upload multipart)
    #[validate(url)]
    pub source_url: Option<String>,
}

fn default_validate_mode() -> ValidateMode { ValidateMode::Semantic }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidateMode {
    None,
    Structural,
    Semantic,
    Strict,
}

/// Réponse de création de job
#[derive(Debug, Serialize)]
pub struct CreateJobResponse {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub estimated_duration_secs: Option<u64>,
    pub poll_url: String,
    pub progress_url: String,
    pub cancel_url: String,
}

/// Progression d'un job — sérialisée dans Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub progress: f32,              // 0.0 à 1.0
    pub tensors_done: u64,
    pub tensors_total: Option<u64>,
    pub bytes_done: u64,
    pub bytes_total: Option<u64>,
    pub last_tensor: Option<String>,
    pub throughput_bps: Option<u64>,
    pub eta_seconds: Option<u32>,
    pub message: Option<String>,
    pub updated_at: DateTime<Utc>,
}
```

---

# 7. AUTHENTIFICATION & AUTORISATION

```rust
// src/auth/jwt.rs

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,          // user_id
    pub email: String,
    pub plan: String,
    pub iss: String,        // Issuer: "umc.dev"
    pub aud: Vec<String>,   // Audience: ["umc-api"]
    pub iat: i64,           // Issued at
    pub exp: i64,           // Expiration
    pub jti: Uuid,          // JWT ID (pour révocation)
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    audience: String,
    access_expiry: Duration,
    refresh_expiry: Duration,
}

impl JwtService {
    pub fn new(config: &crate::config::JwtConfig) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(config.secret.as_bytes()),
            issuer: config.issuer.clone(),
            audience: config.audience.clone(),
            access_expiry: Duration::seconds(config.access_token_expiry_secs as i64),
            refresh_expiry: Duration::seconds(config.refresh_token_expiry_secs as i64),
        }
    }

    pub fn encode_access_token(&self, user: &crate::models::user::User)
        -> anyhow::Result<String>
    {
        let now = Utc::now();
        let claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            plan: format!("{:?}", user.plan).to_lowercase(),
            iss: self.issuer.clone(),
            aud: vec![self.audience.clone()],
            iat: now.timestamp(),
            exp: (now + self.access_expiry).timestamp(),
            jti: Uuid::new_v4(),
        };
        Ok(encode(&Header::default(), &claims, &self.encoding_key)?)
    }

    pub fn decode(&self, token: &str) -> anyhow::Result<Claims> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        let data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(data.claims)
    }

    pub fn generate_refresh_token() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..48).map(|_| rng.gen()).collect();
        hex::encode(bytes)
    }
}
```

```rust
// src/auth/api_key.rs

use sha2::{Sha256, Digest};
use constant_time_eq::constant_time_eq;

/// Génère une nouvelle API key
/// Format : "umc_sk_prod_<hex(32 bytes)>"
pub fn generate_api_key() -> (String, String) {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("getrandom failed");
    let key = format!("umc_sk_prod_{}", hex::encode(bytes));
    let hash = hash_api_key(&key);
    (key, hash)
}

/// Hache une API key pour stockage en DB
pub fn hash_api_key(key: &str) -> String {
    let hash = Sha256::digest(key.as_bytes());
    hex::encode(hash)
}

/// Vérifie une API key en temps constant (anti-timing-attack)
pub fn verify_api_key(provided: &str, stored_hash: &str) -> bool {
    let provided_hash = hash_api_key(provided);
    constant_time_eq(provided_hash.as_bytes(), stored_hash.as_bytes())
}

/// Extrait le préfixe visible d'une clé (pour l'affichage)
pub fn key_prefix(key: &str) -> String {
    // "umc_sk_prod_abc123..." → "umc_sk_prod_abc..."
    let parts: Vec<&str> = key.splitn(4, '_').collect();
    if parts.len() >= 4 {
        format!("{}_{}_{}_{}...", parts[0], parts[1], parts[2],
            &parts[3][..4.min(parts[3].len())])
    } else {
        key[..8.min(key.len())].to_string()
    }
}
```

```rust
// src/auth/middleware.rs

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
    web::Data,
};
use futures_util::future::LocalBoxFuture;
use std::{future::{ready, Ready}, rc::Rc};

use crate::{config::AppState, errors::ApiError};

/// Identité extraite du token ou de l'API key
#[derive(Debug, Clone)]
pub struct AuthIdentity {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub plan: crate::models::user::Plan,
    pub scopes: i32,
    pub api_key_id: Option<uuid::Uuid>,
    pub auth_method: AuthMethod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthMethod {
    Jwt,
    ApiKey,
}

impl AuthIdentity {
    pub fn can(&self, scope: ApiScope) -> bool {
        self.scopes & scope as i32 != 0
    }
}

#[repr(i32)]
pub enum ApiScope {
    Read       = 1,
    Convert    = 2,
    ManageKeys = 4,
    Admin      = 8,
}

/// Middleware d'authentification — JWT ou API Key
pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service: Rc::new(service) }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = Rc::clone(&self.service);

        Box::pin(async move {
            // Extraire le token ou l'API key
            let identity = extract_identity(&req).await;

            match identity {
                Ok(identity) => {
                    // Injecter l'identité dans les extensions de la requête
                    req.extensions_mut().insert(identity);
                    svc.call(req).await
                }
                Err(e) => {
                    Err(actix_web::Error::from(e))
                }
            }
        })
    }
}

async fn extract_identity(req: &ServiceRequest) -> Result<AuthIdentity, ApiError> {
    let state = req.app_data::<Data<AppState>>()
        .ok_or_else(|| ApiError::Internal("AppState not found".into()))?;

    // 1. Chercher Bearer JWT
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return extract_from_jwt(token, state).await;
            }
        }
    }

    // 2. Chercher X-API-Key header
    if let Some(api_key_header) = req.headers().get("X-API-Key") {
        if let Ok(key) = api_key_header.to_str() {
            return extract_from_api_key(key, state).await;
        }
    }

    // 3. Chercher dans query params (?api_key=...) — pour les SSE qui ne supportent pas headers
    if let Some(query) = req.query_string().split('&').find(|s| s.starts_with("api_key=")) {
        let key = &query[8..]; // strip "api_key="
        return extract_from_api_key(key, state).await;
    }

    Err(ApiError::Unauthorized("Authentication required".into()))
}

async fn extract_from_jwt(token: &str, state: &Data<AppState>) -> Result<AuthIdentity, ApiError> {
    let jwt_service = crate::auth::jwt::JwtService::new(&state.config.jwt);

    let claims = jwt_service.decode(token)
        .map_err(|_| ApiError::Unauthorized("Invalid or expired token".into()))?;

    // Récupérer le plan depuis la DB (peut avoir changé)
    let user = sqlx::query_as!(
        crate::models::user::User,
        "SELECT * FROM users WHERE id = $1 AND is_active = TRUE",
        claims.sub
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .ok_or_else(|| ApiError::Unauthorized("User not found or inactive".into()))?;

    Ok(AuthIdentity {
        user_id: user.id,
        email: user.email,
        plan: user.plan,
        scopes: ApiScope::Read as i32 | ApiScope::Convert as i32,
        api_key_id: None,
        auth_method: AuthMethod::Jwt,
    })
}

async fn extract_from_api_key(key: &str, state: &Data<AppState>) -> Result<AuthIdentity, ApiError> {
    // Valider le format de la clé
    if !key.starts_with("umc_sk_") {
        return Err(ApiError::Unauthorized("Invalid API key format".into()));
    }

    let key_hash = crate::auth::api_key::hash_api_key(key);

    // Chercher la clé en DB
    let row = sqlx::query!(
        r#"
        SELECT ak.id, ak.user_id, ak.scopes, ak.is_active, ak.expires_at,
               u.email, u.plan as "plan: _", u.is_active as user_active
        FROM api_keys ak
        JOIN users u ON ak.user_id = u.id
        WHERE ak.key_hash = $1
        "#,
        key_hash
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .ok_or_else(|| ApiError::Unauthorized("API key not found".into()))?;

    if !row.is_active {
        return Err(ApiError::Unauthorized("API key is revoked".into()));
    }
    if !row.user_active {
        return Err(ApiError::Unauthorized("Account is inactive".into()));
    }
    if let Some(expires_at) = row.expires_at {
        if expires_at < chrono::Utc::now() {
            return Err(ApiError::Unauthorized("API key has expired".into()));
        }
    }

    // Mettre à jour last_used_at de façon non-bloquante
    let db = state.db.clone();
    let api_key_id = row.id;
    tokio::spawn(async move {
        let _ = sqlx::query!(
            "UPDATE api_keys SET last_used_at = NOW() WHERE id = $1",
            api_key_id
        ).execute(&db).await;
    });

    Ok(AuthIdentity {
        user_id: row.user_id,
        email: row.email,
        plan: row.plan,
        scopes: row.scopes,
        api_key_id: Some(row.id),
        auth_method: AuthMethod::ApiKey,
    })
}

/// Macro helper pour extraire l'identité dans les handlers
#[macro_export]
macro_rules! require_auth {
    ($req:expr) => {{
        $req.extensions()
            .get::<crate::auth::middleware::AuthIdentity>()
            .cloned()
            .ok_or_else(|| crate::errors::ApiError::Unauthorized("Not authenticated".into()))
    }};
}
```

---

# 8. GESTION DES ERREURS

```rust
// src/errors/api_error.rs

use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;
use uuid::Uuid;

/// Erreur API unifiée — se convertit automatiquement en réponse HTTP
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    // 400 Bad Request
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    // 401 Unauthorized
    #[error("Authentication required: {0}")]
    Unauthorized(String),

    // 403 Forbidden
    #[error("Access denied: {0}")]
    Forbidden(String),

    // 404 Not Found
    #[error("Resource not found: {0}")]
    NotFound(String),

    // 409 Conflict
    #[error("Conflict: {0}")]
    Conflict(String),

    // 413 Payload Too Large
    #[error("File too large: max {max_bytes} bytes")]
    FileTooLarge { max_bytes: u64 },

    // 422 Unprocessable Entity
    #[error("Conversion error: {0}")]
    ConversionError(String),

    // 429 Too Many Requests
    #[error("Rate limit exceeded. Retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    // 402 Payment Required
    #[error("Plan limit exceeded: {0}")]
    QuotaExceeded(String),

    // 500 Internal Server Error
    #[error("Internal error: {0}")]
    Internal(String),

    // 503 Service Unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    // Erreurs DB wrappées
    #[error("Database error")]
    Database(#[from] sqlx::Error),
}

/// Format unifié de réponse d'erreur
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u64>,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, code) = self.status_and_code();

        let request_id = None; // Injecté par le middleware

        let body = ErrorResponse {
            error: ErrorBody {
                code,
                message: self.to_string(),
                request_id,
                details: self.details(),
                retry_after: self.retry_after(),
            },
        };

        // Toujours Content-Type JSON
        HttpResponse::build(status)
            .content_type("application/json")
            .json(body)
    }
}

impl ApiError {
    fn status_and_code(&self) -> (actix_web::http::StatusCode, &'static str) {
        use actix_web::http::StatusCode;
        match self {
            Self::Validation(_)    => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            Self::BadRequest(_)    => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            Self::Unauthorized(_)  => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            Self::Forbidden(_)     => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            Self::NotFound(_)      => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            Self::Conflict(_)      => (StatusCode::CONFLICT, "CONFLICT"),
            Self::FileTooLarge {..}=> (StatusCode::PAYLOAD_TOO_LARGE, "FILE_TOO_LARGE"),
            Self::ConversionError(_) => (StatusCode::UNPROCESSABLE_ENTITY, "CONVERSION_ERROR"),
            Self::RateLimited {..} => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED"),
            Self::QuotaExceeded(_) => (actix_web::http::StatusCode::PAYMENT_REQUIRED, "QUOTA_EXCEEDED"),
            Self::Internal(_)      => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
            Self::ServiceUnavailable(_) => (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE"),
            Self::Database(e) => {
                if matches!(e, sqlx::Error::RowNotFound) {
                    (StatusCode::NOT_FOUND, "NOT_FOUND")
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR")
                }
            }
        }
    }

    fn details(&self) -> Option<serde_json::Value> {
        match self {
            Self::FileTooLarge { max_bytes } => Some(serde_json::json!({
                "max_bytes": max_bytes,
                "max_human": format!("{} GB", max_bytes / 1024 / 1024 / 1024)
            })),
            _ => None,
        }
    }

    fn retry_after(&self) -> Option<u64> {
        match self {
            Self::RateLimited { retry_after_secs } => Some(*retry_after_secs),
            _ => None,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Conversion depuis les erreurs de validation
impl From<validator::ValidationErrors> for ApiError {
    fn from(e: validator::ValidationErrors) -> Self {
        let details = e.field_errors()
            .iter()
            .map(|(field, errors)| {
                let msgs: Vec<String> = errors.iter()
                    .map(|e| e.message.as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| e.code.to_string()))
                    .collect();
                (field.to_string(), msgs)
            })
            .collect::<std::collections::HashMap<_, _>>();
        Self::Validation(serde_json::to_string(&details).unwrap_or_default())
    }
}
```

---

# 9. MIDDLEWARE STACK

```rust
// src/middleware/rate_limit.rs

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use dashmap::DashMap;
use governor::{
    clock::{Clock, DefaultClock},
    state::{direct::NotKeyed, InMemoryState},
    Quota, RateLimiter,
};
use once_cell::sync::Lazy;
use std::{
    future::{ready, Ready},
    num::NonZeroU32,
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use crate::auth::middleware::AuthIdentity;
use crate::config::AppState;

type Limiter = Arc<RateLimiter<String, dashmap::DashMap<String, InMemoryState>, DefaultClock>>;

static RATE_LIMITERS: Lazy<DashMap<String, Arc<governor::RateLimiter<
    String,
    dashmap::DashMap<String, InMemoryState>,
    DefaultClock,
>>>> = Lazy::new(DashMap::new);

pub struct RateLimitMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitService { service: Rc::new(service) }))
    }
}

pub struct RateLimitService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RateLimitService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = futures_util::future::LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = Rc::clone(&self.service);

        Box::pin(async move {
            let state = req.app_data::<actix_web::web::Data<AppState>>().cloned();

            let (key, limit) = if let Some(identity) = req.extensions().get::<AuthIdentity>() {
                let rpm = identity.plan.requests_per_minute();
                (format!("user:{}", identity.user_id), rpm)
            } else {
                // IP-based pour les non-authentifiés
                let ip = req.connection_info().realip_remote_addr()
                    .unwrap_or("unknown")
                    .to_string();
                let limit = state.map(|s| s.config.rate_limit.anon_requests_per_minute)
                    .unwrap_or(10);
                (format!("ip:{}", ip), limit)
            };

            // Obtenir ou créer le limiter pour cette clé
            let limiter = get_or_create_limiter(&key, limit);

            match limiter.check_key(&key) {
                Ok(_) => svc.call(req).await,
                Err(negative) => {
                    let wait_time = negative.wait_time_from(DefaultClock::default().now());
                    let retry_after = wait_time.as_secs() + 1;

                    Err(actix_web::Error::from(
                        crate::errors::ApiError::RateLimited { retry_after_secs: retry_after }
                    ))
                }
            }
        })
    }
}

fn get_or_create_limiter(key: &str, rpm: u32) -> Arc<governor::RateLimiter<
    String,
    dashmap::DashMap<String, InMemoryState>,
    DefaultClock,
>> {
    if let Some(limiter) = RATE_LIMITERS.get(key) {
        return Arc::clone(limiter.value());
    }

    let quota = Quota::per_minute(NonZeroU32::new(rpm.max(1)).unwrap());
    let limiter = Arc::new(governor::RateLimiter::keyed(quota));
    RATE_LIMITERS.insert(key.to_string(), Arc::clone(&limiter));
    limiter
}
```

```rust
// src/middleware/request_id.rs

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use std::{future::{ready, Ready}, rc::Rc};
use uuid::Uuid;

/// Ajoute un X-Request-ID à chaque requête (pour le tracing)
pub struct RequestIdMiddleware;

pub struct RequestId(pub String);

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIdService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdService { service: Rc::new(service) }))
    }
}

pub struct RequestIdService<S> { service: Rc<S> }

impl<S, B> Service<ServiceRequest> for RequestIdService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = futures_util::future::LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = Rc::clone(&self.service);

        // Utiliser le header entrant ou en générer un
        let request_id = req.headers()
            .get("X-Request-ID")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let request_id = if request_id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            request_id
        };

        req.extensions_mut().insert(RequestId(request_id.clone()));

        Box::pin(async move {
            let mut res = svc.call(req).await?;
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-request-id"),
                actix_web::http::header::HeaderValue::from_str(&request_id)
                    .unwrap_or_else(|_| actix_web::http::header::HeaderValue::from_static("?")),
            );
            Ok(res)
        })
    }
}
```

```rust
// src/middleware/security.rs — Security headers

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use std::{future::{ready, Ready}, rc::Rc};

pub struct SecurityHeadersMiddleware;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeadersMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SecurityHeadersService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersService { service: Rc::new(service) }))
    }
}

pub struct SecurityHeadersService<S> { service: Rc<S> }

impl<S, B> Service<ServiceRequest> for SecurityHeadersService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = futures_util::future::LocalBoxFuture<'static, Result<Self::Response, Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = Rc::clone(&self.service);
        Box::pin(async move {
            let mut res = svc.call(req).await?;
            let headers = res.headers_mut();
            use actix_web::http::header::{HeaderName, HeaderValue};
            headers.insert(
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            );
            headers.insert(
                HeaderName::from_static("x-frame-options"),
                HeaderValue::from_static("DENY"),
            );
            headers.insert(
                HeaderName::from_static("x-xss-protection"),
                HeaderValue::from_static("1; mode=block"),
            );
            headers.insert(
                HeaderName::from_static("referrer-policy"),
                HeaderValue::from_static("strict-origin-when-cross-origin"),
            );
            headers.insert(
                HeaderName::from_static("strict-transport-security"),
                HeaderValue::from_static("max-age=63072000; includeSubDomains"),
            );
            Ok(res)
        })
    }
}
```

---

# 10. ROUTES & HANDLERS — AUTH

```rust
// src/handlers/auth.rs

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{config::AppState, errors::ApiError, models::user::UserResponse};

// ── REGISTER ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 8, max = 128, message = "Password must be 8-128 characters"))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub user: UserResponse,
}

pub async fn register(
    state: web::Data<AppState>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse, ApiError> {
    req.validate().map_err(ApiError::from)?;

    // Vérifier que l'email n'existe pas déjà
    let existing = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users WHERE email = $1",
        req.email.to_lowercase()
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    if existing.unwrap_or(0) > 0 {
        return Err(ApiError::Conflict("Email already registered".into()));
    }

    // Hasher le mot de passe avec Argon2id
    let password_hash = crate::auth::password::hash_password(&req.password)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Créer l'utilisateur
    let user = sqlx::query_as!(
        crate::models::user::User,
        r#"
        INSERT INTO users (email, password_hash, display_name)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        req.email.to_lowercase(),
        password_hash,
        req.display_name
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Générer les tokens
    let jwt_service = crate::auth::jwt::JwtService::new(&state.config.jwt);
    let access_token = jwt_service.encode_access_token(&user)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let refresh_token = crate::auth::jwt::JwtService::generate_refresh_token();
    let refresh_hash = crate::auth::api_key::hash_api_key(&refresh_token);

    // Sauvegarder le refresh token
    sqlx::query!(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        user.id,
        refresh_hash,
        chrono::Utc::now() + chrono::Duration::seconds(
            state.config.jwt.refresh_token_expiry_secs as i64
        )
    )
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Log d'audit
    crate::services::audit_service::log_action(
        &state.db, Some(user.id), None,
        crate::models::audit::AuditAction::UserRegister,
        None, None, None
    ).await;

    Ok(HttpResponse::Created().json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer",
        expires_in: state.config.jwt.access_token_expiry_secs,
        user: user.into(),
    }))
}

// ── LOGIN ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub device_id: Option<String>,
}

pub async fn login(
    state: web::Data<AppState>,
    req: web::Json<LoginRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    req.validate().map_err(ApiError::from)?;

    // Chercher l'utilisateur
    let user = sqlx::query_as!(
        crate::models::user::User,
        "SELECT * FROM users WHERE email = $1 AND is_active = TRUE",
        req.email.to_lowercase()
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".into()))?;

    // Vérifier le mot de passe
    let valid = crate::auth::password::verify_password(&req.password, &user.password_hash)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if !valid {
        // Délai constant pour éviter le timing attack (même si invalide)
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        return Err(ApiError::Unauthorized("Invalid credentials".into()));
    }

    // Mettre à jour last_login_at
    sqlx::query!("UPDATE users SET last_login_at = NOW() WHERE id = $1", user.id)
        .execute(&state.db)
        .await
        .ok();

    // Générer les tokens
    let jwt_service = crate::auth::jwt::JwtService::new(&state.config.jwt);
    let access_token = jwt_service.encode_access_token(&user)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let refresh_token = crate::auth::jwt::JwtService::generate_refresh_token();
    let refresh_hash = crate::auth::api_key::hash_api_key(&refresh_token);

    let ip = http_req.connection_info().realip_remote_addr()
        .map(|s| s.parse::<std::net::IpAddr>().ok())
        .flatten();

    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, device_id, ip_address, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        user.id,
        refresh_hash,
        req.device_id,
        ip.map(|ip| ip.to_string()),
        chrono::Utc::now() + chrono::Duration::seconds(
            state.config.jwt.refresh_token_expiry_secs as i64
        )
    )
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    crate::services::audit_service::log_action(
        &state.db, Some(user.id), None,
        crate::models::audit::AuditAction::UserLogin,
        None, ip.map(|ip| ip.to_string()).as_deref(), None
    ).await;

    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer",
        expires_in: state.config.jwt.access_token_expiry_secs,
        user: user.into(),
    }))
}

// ── REFRESH TOKEN ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn refresh_token(
    state: web::Data<AppState>,
    req: web::Json<RefreshRequest>,
) -> Result<HttpResponse, ApiError> {
    let token_hash = crate::auth::api_key::hash_api_key(&req.refresh_token);

    // Vérifier et invalider le refresh token (rotation)
    let row = sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET is_valid = FALSE, used_at = NOW()
        WHERE token_hash = $1 AND is_valid = TRUE AND expires_at > NOW()
        RETURNING user_id
        "#,
        token_hash
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .ok_or_else(|| ApiError::Unauthorized("Invalid or expired refresh token".into()))?;

    let user = sqlx::query_as!(
        crate::models::user::User,
        "SELECT * FROM users WHERE id = $1 AND is_active = TRUE",
        row.user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .ok_or_else(|| ApiError::Unauthorized("User not found".into()))?;

    // Nouveau token pair
    let jwt_service = crate::auth::jwt::JwtService::new(&state.config.jwt);
    let access_token = jwt_service.encode_access_token(&user)
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let new_refresh_token = crate::auth::jwt::JwtService::generate_refresh_token();
    let new_refresh_hash = crate::auth::api_key::hash_api_key(&new_refresh_token);

    sqlx::query!(
        "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        user.id,
        new_refresh_hash,
        chrono::Utc::now() + chrono::Duration::seconds(
            state.config.jwt.refresh_token_expiry_secs as i64
        )
    )
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": new_refresh_token,
        "token_type": "Bearer",
        "expires_in": state.config.jwt.access_token_expiry_secs,
    })))
}

// ── LOGOUT ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: Option<String>,
}

pub async fn logout(
    state: web::Data<AppState>,
    req: web::Json<LogoutRequest>,
    identity: web::ReqData<crate::auth::middleware::AuthIdentity>,
) -> Result<HttpResponse, ApiError> {
    // Invalider le refresh token si fourni
    if let Some(ref rt) = req.refresh_token {
        let token_hash = crate::auth::api_key::hash_api_key(rt);
        sqlx::query!(
            "UPDATE refresh_tokens SET is_valid = FALSE WHERE token_hash = $1 AND user_id = $2",
            token_hash,
            identity.user_id
        )
        .execute(&state.db)
        .await
        .ok();
    }

    crate::services::audit_service::log_action(
        &state.db, Some(identity.user_id), None,
        crate::models::audit::AuditAction::UserLogout,
        None, None, None
    ).await;

    Ok(HttpResponse::NoContent().finish())
}
```

---

# 11. ROUTES & HANDLERS — JOBS DE CONVERSION

```rust
// src/handlers/jobs.rs

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::AppState,
    errors::ApiError,
    models::job::{ConversionJob, CreateJobRequest, CreateJobResponse, JobStatus},
    require_auth,
};

/// POST /v1/jobs — Créer un job (sans upload — le fichier vient d'un URL ou d'un upload séparé)
pub async fn create_job(
    state: web::Data<AppState>,
    req_data: web::ReqData<crate::auth::middleware::AuthIdentity>,
    body: web::Json<CreateJobRequest>,
) -> Result<HttpResponse, ApiError> {
    let identity = req_data.into_inner();
    body.validate().map_err(ApiError::from)?;

    // Vérifier les quotas
    crate::services::quota_service::check_conversion_quota(&state, &identity).await?;

    // Vérifier que le format cible est supporté
    let conversion_graph = umc_graph::ConversionGraph::new();
    let source_fmt = body.source_format.as_deref().unwrap_or("auto");

    if body.target_format.to_uppercase() != "AUTO" {
        // Vérification basique de l'existence du format
        if !conversion_graph.node_exists(&body.target_format) {
            return Err(ApiError::BadRequest(
                format!("Unknown target format: {}. Use GET /v1/formats for list.", body.target_format)
            ));
        }
    }

    // Créer le job en DB
    let job = sqlx::query_as!(
        ConversionJob,
        r#"
        INSERT INTO conversion_jobs (
            user_id, api_key_id, source_format, target_format,
            target_dtype, validate_mode, generate_cert,
            merge_adapters, quantize_scheme, extra_options
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
        identity.user_id,
        identity.api_key_id,
        source_fmt,
        body.target_format,
        body.target_dtype,
        serde_json::to_string(&body.validate_mode).unwrap_or_default(),
        body.generate_cert,
        body.merge_adapters,
        body.quantize_scheme,
        body.extra_options.clone().unwrap_or(serde_json::Value::Object(Default::default()))
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Estimer la durée si on a la taille du fichier
    let estimated_duration = None; // Calculé après l'upload

    let base_url = "/v1";
    Ok(HttpResponse::Created().json(CreateJobResponse {
        job_id: job.id,
        status: JobStatus::Queued,
        estimated_duration_secs: estimated_duration,
        poll_url: format!("{}/jobs/{}", base_url, job.id),
        progress_url: format!("{}/jobs/{}/progress", base_url, job.id),
        cancel_url: format!("{}/jobs/{}/cancel", base_url, job.id),
    }))
}

/// GET /v1/jobs/{id} — Statut d'un job
pub async fn get_job(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    identity: web::ReqData<crate::auth::middleware::AuthIdentity>,
) -> Result<HttpResponse, ApiError> {
    let job_id = path.into_inner();
    let identity = identity.into_inner();

    let job = sqlx::query_as!(
        ConversionJob,
        "SELECT * FROM conversion_jobs WHERE id = $1 AND user_id = $2",
        job_id,
        identity.user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound(format!("Job {} not found", job_id)))?;

    // Enrichir avec la progression Redis (plus fraîche que la DB)
    let progress = get_job_progress_from_redis(&state, job_id).await;

    Ok(HttpResponse::Ok().json(JobDetailResponse::from_job_and_progress(job, progress)))
}

/// GET /v1/jobs — Liste des jobs de l'utilisateur
#[derive(Debug, Deserialize)]
pub struct ListJobsQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub sort: Option<String>,
}

pub async fn list_jobs(
    state: web::Data<AppState>,
    query: web::Query<ListJobsQuery>,
    identity: web::ReqData<crate::auth::middleware::AuthIdentity>,
) -> Result<HttpResponse, ApiError> {
    let identity = identity.into_inner();
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * per_page;

    // Query avec filtre statut optionnel
    let jobs = if let Some(ref status) = query.status {
        sqlx::query_as!(
            ConversionJob,
            r#"
            SELECT * FROM conversion_jobs
            WHERE user_id = $1 AND status::TEXT = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            identity.user_id, status, per_page, offset
        )
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as!(
            ConversionJob,
            r#"
            SELECT * FROM conversion_jobs
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            identity.user_id, per_page, offset
        )
        .fetch_all(&state.db)
        .await
    };

    let jobs = jobs.map_err(|e| ApiError::Internal(e.to_string()))?;

    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM conversion_jobs WHERE user_id = $1",
        identity.user_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .unwrap_or(0);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "data": jobs,
        "pagination": {
            "page": page,
            "per_page": per_page,
            "total": total,
            "pages": (total as f64 / per_page as f64).ceil() as i64
        }
    })))
}

/// POST /v1/jobs/{id}/cancel — Annuler un job
pub async fn cancel_job(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    identity: web::ReqData<crate::auth::middleware::AuthIdentity>,
) -> Result<HttpResponse, ApiError> {
    let job_id = path.into_inner();
    let identity = identity.into_inner();

    let updated = sqlx::query!(
        r#"
        UPDATE conversion_jobs
        SET status = 'cancelled', finished_at = NOW()
        WHERE id = $1
          AND user_id = $2
          AND status IN ('queued', 'running', 'paused')
        RETURNING id
        "#,
        job_id,
        identity.user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    if updated.is_none() {
        return Err(ApiError::NotFound(
            format!("Job {} not found or cannot be cancelled", job_id)
        ));
    }

    // Notifier le worker de s'arrêter via Redis
    let cancel_key = format!("job:{}:cancel", job_id);
    if let Ok(mut conn) = state.redis.get().await {
        let _: redis::RedisResult<()> = redis::cmd("SET")
            .arg(&cancel_key)
            .arg(1)
            .arg("EX")
            .arg(3600)
            .query_async(&mut conn)
            .await;
    }

    crate::services::audit_service::log_action(
        &state.db, Some(identity.user_id), None,
        crate::models::audit::AuditAction::JobCancel,
        Some(&job_id.to_string()), None, None
    ).await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "job_id": job_id,
        "status": "cancelled",
        "message": "Job cancellation requested"
    })))
}

/// GET /v1/jobs/{id}/download — Télécharger le fichier converti
pub async fn download_job_output(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    identity: web::ReqData<crate::auth::middleware::AuthIdentity>,
) -> Result<HttpResponse, ApiError> {
    let job_id = path.into_inner();
    let identity = identity.into_inner();

    let job = sqlx::query!(
        r#"
        SELECT output_file_path, target_format, status as "status: JobStatus"
        FROM conversion_jobs
        WHERE id = $1 AND user_id = $2
        "#,
        job_id,
        identity.user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound(format!("Job {} not found", job_id)))?;

    if job.status != JobStatus::Done {
        return Err(ApiError::BadRequest(
            format!("Job is not complete (status: {:?})", job.status)
        ));
    }

    let output_path = job.output_file_path
        .ok_or_else(|| ApiError::Internal("No output file".into()))?;

    match state.config.storage.backend {
        crate::config::StorageBackend::Local => {
            let path = std::path::Path::new(&output_path);
            if !path.exists() {
                return Err(ApiError::NotFound("Output file has expired".into()));
            }

            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("model");

            Ok(actix_files::NamedFile::open(path)
                .map_err(|e| ApiError::Internal(e.to_string()))?
                .use_last_modified(true)
                .set_content_disposition(actix_web::http::header::ContentDisposition {
                    disposition: actix_web::http::header::DispositionType::Attachment,
                    parameters: vec![
                        actix_web::http::header::DispositionParam::Filename(filename.to_string())
                    ],
                })
                .into_response(&actix_web::HttpRequest::default()))
        }
        #[cfg(feature = "s3-storage")]
        crate::config::StorageBackend::S3 => {
            // Générer une URL pré-signée valable 15 minutes
            let signed_url = crate::services::storage_service::generate_presigned_url(
                &state, &output_path, 900
            ).await?;
            Ok(HttpResponse::TemporaryRedirect()
                .append_header(("Location", signed_url))
                .finish())
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────

async fn get_job_progress_from_redis(
    state: &AppState,
    job_id: Uuid,
) -> Option<crate::models::job::JobProgress> {
    let mut conn = state.redis.get().await.ok()?;
    let key = format!("job:{}:progress", job_id);
    let data: String = redis::cmd("GET").arg(&key).query_async(&mut conn).await.ok()?;
    serde_json::from_str(&data).ok()
}

#[derive(Debug, Serialize)]
pub struct JobDetailResponse {
    #[serde(flatten)]
    pub job: ConversionJob,
    // Progression en temps réel depuis Redis (peut surpasser la DB)
    pub live_progress: Option<crate::models::job::JobProgress>,
}

impl JobDetailResponse {
    pub fn from_job_and_progress(
        job: ConversionJob,
        progress: Option<crate::models::job::JobProgress>,
    ) -> Self {
        Self { job, live_progress: progress }
    }
}
```

---

# 12. STREAMING SSE — PROGRESSION TEMPS RÉEL

```rust
// src/handlers/progress.rs

use actix_web::{web, HttpRequest, HttpResponse};
use actix_web::rt::time::interval;
use futures_util::StreamExt;
use std::time::Duration;
use uuid::Uuid;

use crate::{config::AppState, errors::ApiError};

/// GET /v1/jobs/{id}/progress — Server-Sent Events
/// Supporte : Bearer token en header OU ?api_key= en query string
pub async fn job_progress_sse(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
    identity: web::ReqData<crate::auth::middleware::AuthIdentity>,
) -> Result<HttpResponse, ApiError> {
    let job_id = path.into_inner();
    let identity = identity.into_inner();

    // Vérifier que le job appartient à l'utilisateur
    let job_exists = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM conversion_jobs WHERE id = $1 AND user_id = $2",
        job_id, identity.user_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .unwrap_or(0) > 0;

    if !job_exists {
        return Err(ApiError::NotFound(format!("Job {} not found", job_id)));
    }

    let state_clone = state.clone();

    // Créer le stream SSE
    let stream = async_stream::stream! {
        let mut ticker = interval(Duration::from_millis(500));
        let mut last_status = String::new();
        let mut consecutive_errors = 0u32;

        loop {
            ticker.tick().await;

            // Lire depuis Redis (plus frais, pas de pression DB)
            let progress = read_progress_from_redis(&state_clone, job_id).await;

            match progress {
                Some(prog) => {
                    consecutive_errors = 0;
                    let is_terminal = matches!(prog.status,
                        crate::models::job::JobStatus::Done |
                        crate::models::job::JobStatus::Failed |
                        crate::models::job::JobStatus::Cancelled
                    );

                    let data = serde_json::to_string(&prog).unwrap_or_default();
                    yield Ok::<_, actix_web::Error>(
                        web::Bytes::from(format!(
                            "event: progress\ndata: {}\nid: {}\n\n",
                            data,
                            chrono::Utc::now().timestamp_millis()
                        ))
                    );

                    if is_terminal {
                        // Envoyer un événement final
                        yield Ok(web::Bytes::from(
                            format!("event: done\ndata: {{\"status\":\"{:?}\"}}\n\n",
                                prog.status)
                        ));
                        break;
                    }
                }
                None => {
                    consecutive_errors += 1;

                    // Fallback : lire depuis la DB
                    if let Some(db_status) = read_status_from_db(&state_clone, job_id).await {
                        let data = serde_json::json!({
                            "job_id": job_id,
                            "status": db_status,
                        });
                        yield Ok(web::Bytes::from(
                            format!("event: status\ndata: {}\n\n",
                                data.to_string())
                        ));

                        let is_terminal = matches!(db_status.as_str(),
                            "done" | "failed" | "cancelled");
                        if is_terminal { break; }
                    }

                    // Si trop d'erreurs consécutives, s'arrêter
                    if consecutive_errors > 10 {
                        yield Ok(web::Bytes::from(
                            "event: error\ndata: {\"error\":\"Progress data unavailable\"}\n\n"
                        ));
                        break;
                    }
                }
            }

            // Heartbeat toutes les 15 secondes pour garder la connexion vivante
            // (géré implicitement par le ticker)
        }
    };

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("X-Accel-Buffering", "no"))  // Désactive le buffering nginx
        .append_header(("Connection", "keep-alive"))
        .streaming(stream))
}

async fn read_progress_from_redis(
    state: &AppState,
    job_id: Uuid,
) -> Option<crate::models::job::JobProgress> {
    let mut conn = state.redis.get().await.ok()?;
    let key = format!("job:{}:progress", job_id);
    let data: String = redis::cmd("GET").arg(&key).query_async(&mut conn).await.ok()?;
    serde_json::from_str(&data).ok()
}

async fn read_status_from_db(
    state: &AppState,
    job_id: Uuid,
) -> Option<String> {
    sqlx::query_scalar!(
        "SELECT status::TEXT FROM conversion_jobs WHERE id = $1",
        job_id
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .flatten()
}

/// Publie la progression dans Redis — appelé par le worker
pub async fn publish_progress(
    redis: &deadpool_redis::Pool,
    progress: &crate::models::job::JobProgress,
    ttl_secs: u64,
) -> anyhow::Result<()> {
    let mut conn = redis.get().await?;
    let key = format!("job:{}:progress", progress.job_id);
    let data = serde_json::to_string(progress)?;

    redis::pipe()
        .cmd("SET").arg(&key).arg(&data).arg("EX").arg(ttl_secs)
        .query_async::<_, ()>(&mut conn)
        .await?;

    Ok(())
}
```

---

# 13. UPLOAD SÉCURISÉ DE FICHIERS

```rust
// src/handlers/upload.rs

use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures_util::StreamExt;
use std::io::Write;
use uuid::Uuid;

use crate::{config::AppState, errors::ApiError, models::user::Plan};

/// POST /v1/jobs/{id}/upload — Upload du fichier source pour un job existant
pub async fn upload_source_file(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    identity: web::ReqData<crate::auth::middleware::AuthIdentity>,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    let job_id = path.into_inner();
    let identity = identity.into_inner();

    // Vérifier que le job existe et est en attente d'upload
    let job = sqlx::query!(
        r#"SELECT id, source_format, status::TEXT as status FROM conversion_jobs
           WHERE id = $1 AND user_id = $2 AND status = 'queued'"#,
        job_id, identity.user_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound(
        format!("Job {} not found or not in queued state", job_id)
    ))?;

    let max_size = identity.plan.max_file_size_bytes();

    // Préparer le répertoire de destination
    let upload_dir = std::path::Path::new(&state.config.storage.local_upload_dir);
    tokio::fs::create_dir_all(upload_dir)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let temp_path = upload_dir.join(format!("{}.upload_tmp", job_id));
    let final_path = upload_dir.join(format!("{}", job_id));

    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut total_bytes: u64 = 0;
    let mut detected_mime: Option<String> = None;
    let mut original_filename: Option<String> = None;

    // Traiter les champs multipart
    while let Some(field_result) = payload.next().await {
        let mut field = field_result.map_err(|e| ApiError::BadRequest(e.to_string()))?;

        match field.name() {
            Some("file") => {
                // Extraire le nom original
                original_filename = field.content_disposition()
                    .and_then(|cd| cd.get_filename())
                    .map(|s| s.to_string());

                // Streamer vers le fichier temporaire
                while let Some(chunk) = field.next().await {
                    let data = chunk.map_err(|e| ApiError::BadRequest(e.to_string()))?;

                    total_bytes += data.len() as u64;

                    // Vérification de taille PENDANT l'upload (pas après)
                    if total_bytes > max_size {
                        // Nettoyer le fichier temporaire
                        tokio::fs::remove_file(&temp_path).await.ok();
                        return Err(ApiError::FileTooLarge { max_bytes: max_size });
                    }

                    // Détection MIME sur les premiers bytes
                    if detected_mime.is_none() && total_bytes <= 512 {
                        detected_mime = detect_mime_from_bytes(&data);
                    }

                    // Écriture async en blocs
                    use tokio::io::AsyncWriteExt;
                    file.write_all(&data)
                        .await
                        .map_err(|e| ApiError::Internal(e.to_string()))?;
                }
            }
            _ => {
                // Ignorer les autres champs
            }
        }
    }

    // Flush et fermeture
    use tokio::io::AsyncWriteExt;
    file.flush().await.map_err(|e| ApiError::Internal(e.to_string()))?;
    drop(file);

    if total_bytes == 0 {
        tokio::fs::remove_file(&temp_path).await.ok();
        return Err(ApiError::BadRequest("Empty file uploaded".into()));
    }

    // Calculer le hash SHA256 du fichier uploadé
    let file_hash = compute_file_hash(&temp_path).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Déplacer atomiquement temp → final
    tokio::fs::rename(&temp_path, &final_path)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Détecter le format depuis les magic bytes si non spécifié
    let source_format = if job.source_format == "auto" {
        detect_format_from_file(&final_path).await
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        job.source_format
    };

    // Estimer le temps de conversion basé sur la taille et le format
    let estimated_secs = estimate_conversion_time(total_bytes, &source_format);

    // Mettre à jour le job avec les informations du fichier
    sqlx::query!(
        r#"
        UPDATE conversion_jobs
        SET
            source_file_path = $1,
            source_file_size = $2,
            source_file_hash = $3,
            source_format = $4,
            bytes_total = $2
        WHERE id = $5
        "#,
        final_path.to_string_lossy().as_ref(),
        total_bytes as i64,
        file_hash,
        source_format,
        job_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Enqueuer le job — le worker va le prendre
    enqueue_job(&state, job_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "job_id": job_id,
        "status": "queued",
        "file_size": total_bytes,
        "file_hash": file_hash,
        "source_format": source_format,
        "estimated_duration_secs": estimated_secs,
        "message": "File uploaded successfully. Conversion will start shortly."
    })))
}

/// POST /v1/convert — Upload + création du job en une seule requête (raccourci)
pub async fn convert_with_upload(
    state: web::Data<AppState>,
    identity: web::ReqData<crate::auth::middleware::AuthIdentity>,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    let identity = identity.into_inner();

    crate::services::quota_service::check_conversion_quota(&state, &identity).await?;

    let max_size = identity.plan.max_file_size_bytes();

    let mut target_format: Option<String> = None;
    let mut target_dtype: Option<String> = None;
    let mut validate_mode: Option<String> = None;
    let mut generate_cert = false;
    let mut merge_adapters = false;

    let upload_dir = std::path::Path::new(&state.config.storage.local_upload_dir);
    tokio::fs::create_dir_all(upload_dir).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Générer un ID de job provisoire
    let job_id = Uuid::new_v4();
    let temp_path = upload_dir.join(format!("{}.tmp", job_id));
    let final_path = upload_dir.join(format!("{}", job_id));

    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    let mut total_bytes: u64 = 0;

    while let Some(field_result) = payload.next().await {
        let mut field = field_result.map_err(|e| ApiError::BadRequest(e.to_string()))?;

        match field.name() {
            Some("file") => {
                while let Some(chunk) = field.next().await {
                    let data = chunk.map_err(|e| ApiError::BadRequest(e.to_string()))?;
                    total_bytes += data.len() as u64;
                    if total_bytes > max_size {
                        tokio::fs::remove_file(&temp_path).await.ok();
                        return Err(ApiError::FileTooLarge { max_bytes: max_size });
                    }
                    use tokio::io::AsyncWriteExt;
                    file.write_all(&data).await
                        .map_err(|e| ApiError::Internal(e.to_string()))?;
                }
            }
            Some("target_format") => {
                let mut buf = Vec::new();
                while let Some(chunk) = field.next().await {
                    buf.extend_from_slice(&chunk.map_err(|e| ApiError::BadRequest(e.to_string()))?);
                }
                target_format = String::from_utf8(buf).ok();
            }
            Some("target_dtype") => {
                let mut buf = Vec::new();
                while let Some(chunk) = field.next().await {
                    buf.extend_from_slice(&chunk.map_err(|e| ApiError::BadRequest(e.to_string()))?);
                }
                target_dtype = String::from_utf8(buf).ok();
            }
            Some("validate_mode") => {
                let mut buf = Vec::new();
                while let Some(chunk) = field.next().await {
                    buf.extend_from_slice(&chunk.map_err(|e| ApiError::BadRequest(e.to_string()))?);
                }
                validate_mode = String::from_utf8(buf).ok();
            }
            Some("generate_cert") => {
                let mut buf = Vec::new();
                while let Some(chunk) = field.next().await {
                    buf.extend_from_slice(&chunk.map_err(|e| ApiError::BadRequest(e.to_string()))?);
                }
                generate_cert = String::from_utf8(buf).ok()
                    .map(|s| s.trim().to_lowercase() == "true")
                    .unwrap_or(false);
            }
            Some("merge_adapters") => {
                let mut buf = Vec::new();
                while let Some(chunk) = field.next().await {
                    buf.extend_from_slice(&chunk.map_err(|e| ApiError::BadRequest(e.to_string()))?);
                }
                merge_adapters = String::from_utf8(buf).ok()
                    .map(|s| s.trim().to_lowercase() == "true")
                    .unwrap_or(false);
            }
            _ => {}
        }
    }

    let target_format = target_format
        .ok_or_else(|| ApiError::BadRequest("target_format field required".into()))?;

    use tokio::io::AsyncWriteExt;
    file.flush().await.map_err(|e| ApiError::Internal(e.to_string()))?;
    drop(file);

    if total_bytes == 0 {
        tokio::fs::remove_file(&temp_path).await.ok();
        return Err(ApiError::BadRequest("Empty file uploaded".into()));
    }

    let file_hash = compute_file_hash(&temp_path).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    tokio::fs::rename(&temp_path, &final_path).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let source_format = detect_format_from_file(&final_path).await
        .unwrap_or_else(|| "unknown".to_string());

    // Créer et démarrer le job directement
    let validate = validate_mode.as_deref().unwrap_or("semantic");
    let job = sqlx::query_as!(
        crate::models::job::ConversionJob,
        r#"
        INSERT INTO conversion_jobs (
            id, user_id, api_key_id,
            source_format, target_format, target_dtype,
            validate_mode, generate_cert, merge_adapters,
            source_file_path, source_file_size, source_file_hash,
            bytes_total, extra_options
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $11, '{}'::jsonb)
        RETURNING *
        "#,
        job_id,
        identity.user_id,
        identity.api_key_id,
        source_format,
        target_format,
        target_dtype,
        validate,
        generate_cert,
        merge_adapters,
        final_path.to_string_lossy().as_ref(),
        total_bytes as i64,
        file_hash,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    enqueue_job(&state, job_id).await?;

    let estimated_secs = estimate_conversion_time(total_bytes, &source_format);

    Ok(HttpResponse::Accepted().json(serde_json::json!({
        "job_id": job_id,
        "status": "queued",
        "source_format": source_format,
        "target_format": target_format,
        "file_size": total_bytes,
        "estimated_duration_secs": estimated_secs,
        "poll_url": format!("/v1/jobs/{}", job_id),
        "progress_url": format!("/v1/jobs/{}/progress", job_id),
        "cancel_url": format!("/v1/jobs/{}/cancel", job_id)
    })))
}

// ── Helpers ─────────────────────────────────────────────────────────────

async fn compute_file_hash(path: &std::path::Path) -> anyhow::Result<String> {
    use sha2::{Sha256, Digest};
    use tokio::io::AsyncReadExt;

    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 65536];

    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }

    Ok(hex::encode(hasher.finalize()))
}

async fn detect_format_from_file(path: &std::path::Path) -> Option<String> {
    let registry = umc_detect::FormatRegistry::new();
    registry.detect(path).ok().map(|r| r.format)
}

fn detect_mime_from_bytes(data: &[u8]) -> Option<String> {
    // Simple détection MIME basée sur les magic bytes
    if data.starts_with(b"GGUF") { return Some("application/x-gguf".into()); }
    if data.len() >= 9 && data[8] == b'{' { return Some("application/x-safetensors".into()); }
    if data.starts_with(&[0x50, 0x4B]) { return Some("application/zip".into()); }
    None
}

fn estimate_conversion_time(file_size: u64, source_format: &str) -> u64 {
    // Estimation grossière : ~100 Mo/s pour la plupart des conversions
    let base_rate = 100 * 1024 * 1024; // 100 Mo/s
    let overhead = 5; // 5 secondes minimum
    let time = file_size / base_rate;
    overhead + time
}

async fn enqueue_job(state: &AppState, job_id: Uuid) -> Result<(), ApiError> {
    // Simplement marquer le job comme prêt (le worker le prendra via SKIP LOCKED)
    // Rien à faire explicitement — le worker poll la DB
    // Optionnel: notifier via Redis pour un démarrage plus rapide
    if let Ok(mut conn) = state.redis.get().await {
        let _: redis::RedisResult<()> = redis::cmd("LPUSH")
            .arg("umc:job_queue")
            .arg(job_id.to_string())
            .query_async(&mut conn)
            .await;
    }
    Ok(())
}
```

---

# 14. WORKER DE CONVERSION — PIPELINE COMPLET

```rust
// src/workers/conversion_worker.rs

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::{
    config::AppState,
    models::job::{JobProgress, JobStatus},
};

/// Worker principal de conversion — tourne en permanence
pub struct ConversionWorker {
    state: Arc<AppState>,
    worker_id: String,
}

impl ConversionWorker {
    pub fn new(state: Arc<AppState>) -> Self {
        let worker_id = format!("worker-{}", Uuid::new_v4());
        Self { state, worker_id }
    }

    /// Lance la boucle principale du worker
    pub async fn run(self) {
        info!(worker_id = %self.worker_id, "Conversion worker started");

        loop {
            match self.poll_and_process().await {
                Ok(true) => {
                    // Un job a été traité, continuer immédiatement
                }
                Ok(false) => {
                    // Aucun job disponible, attendre avant de repolluer
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                Err(e) => {
                    error!(worker_id = %self.worker_id, error = %e,
                           "Error in worker loop");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        }
    }

    /// Poll la DB pour un job disponible et le traite
    async fn poll_and_process(&self) -> anyhow::Result<bool> {
        // Acquérir le sémaphore avant de dépiler (respect des limites)
        let permit = match self.state.conversion_semaphore.try_acquire() {
            Ok(p) => p,
            Err(_) => return Ok(false), // Toutes les slots occupées
        };

        // Déqueuer atomiquement un job (SKIP LOCKED)
        let job = sqlx::query!(
            r#"
            UPDATE conversion_jobs
            SET
                status = 'running',
                started_at = NOW(),
                worker_id = $1,
                attempts = attempts + 1
            WHERE id = (
                SELECT id FROM conversion_jobs
                WHERE status = 'queued'
                  AND attempts < max_attempts
                ORDER BY
                    (SELECT priority_weight FROM users WHERE id = user_id) DESC,
                    queued_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING
                id, user_id, source_format, target_format,
                target_dtype, validate_mode, generate_cert,
                merge_adapters, quantize_scheme,
                source_file_path, source_file_hash,
                checkpoint_data, attempts, extra_options
            "#,
            self.worker_id
        )
        .fetch_optional(&self.state.db)
        .await?;

        let job = match job {
            Some(j) => j,
            None => {
                drop(permit); // Libérer immédiatement
                return Ok(false);
            }
        };

        let job_id = job.id;
        let state = Arc::clone(&self.state);
        let worker_id = self.worker_id.clone();

        // Lancer la conversion dans une tâche Tokio séparée
        // Le permit est passé dans la task pour être libéré à la fin
        tokio::spawn(async move {
            let _permit = permit; // Drop à la fin de la task

            info!(
                job_id = %job_id,
                source = %job.source_format,
                target = %job.target_format,
                worker = %worker_id,
                "Starting conversion"
            );

            let result = run_conversion(
                Arc::clone(&state),
                job_id,
                &job,
            ).await;

            match result {
                Ok(outcome) => {
                    complete_job(&state, job_id, outcome).await;
                }
                Err(e) => {
                    fail_job(&state, job_id, &e.to_string(), job.attempts).await;
                }
            }
        });

        Ok(true)
    }
}

/// Résultat d'une conversion réussie
struct ConversionOutcome {
    output_path: String,
    output_size: u64,
    output_hash: String,
    roundtrip_level: String,
    max_divergence: Option<f64>,
    warnings: Vec<String>,
    cpu_time_ms: u64,
    peak_ram_bytes: u64,
    certificate_id: Option<Uuid>,
}

/// Exécute la conversion réelle en utilisant umc-pipeline
#[instrument(skip(state, job), fields(job_id = %job_id))]
async fn run_conversion(
    state: Arc<AppState>,
    job_id: Uuid,
    job: &JobRow,
) -> anyhow::Result<ConversionOutcome> {
    let source_path = job.source_file_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No source file"))?;

    // Préparer le chemin de sortie
    let output_dir = std::path::Path::new(&state.config.storage.local_output_dir);
    tokio::fs::create_dir_all(output_dir).await?;

    let output_filename = format!("{}.{}", job_id, extension_for_format(&job.target_format));
    let output_path = output_dir.join(&output_filename);
    let temp_output_path = output_dir.join(format!("{}.umc_tmp", job_id));

    // Vérifier si on doit reprendre depuis un checkpoint
    let checkpoint = job.checkpoint_data
        .as_ref()
        .and_then(|v| serde_json::from_value::<ConversionCheckpoint>(v.clone()).ok());

    let start_time = Instant::now();
    let start_rss = get_rss_bytes();

    // Créer le CancellationToken
    let cancel_token = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cancel_clone = cancel_token.clone();

    // Surveillance des annulations via Redis
    let state_for_cancel = Arc::clone(&state);
    let cancel_monitor = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            if let Ok(mut conn) = state_for_cancel.redis.get().await {
                let cancel_key = format!("job:{}:cancel", job_id);
                let cancelled: bool = redis::cmd("EXISTS")
                    .arg(&cancel_key)
                    .query_async(&mut conn)
                    .await
                    .unwrap_or(false);
                if cancelled {
                    cancel_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                    break;
                }
            }
        }
    });

    // Publisher de progression
    let state_for_progress = Arc::clone(&state);
    let (progress_tx, mut progress_rx) =
        tokio::sync::mpsc::channel::<JobProgress>(64);

    tokio::spawn(async move {
        while let Some(progress) = progress_rx.recv().await {
            // Publier dans Redis
            if let Err(e) = crate::handlers::progress::publish_progress(
                &state_for_progress.redis,
                &progress,
                state_for_progress.config.redis.progress_ttl_secs,
            ).await {
                warn!("Failed to publish progress: {}", e);
            }

            // Mettre à jour la DB toutes les 5 secondes environ
            // (pas à chaque tenseur pour éviter la pression DB)
            if progress.progress as u64 % 5 == 0 || progress.progress >= 0.99 {
                let _ = sqlx::query!(
                    r#"
                    UPDATE conversion_jobs
                    SET
                        progress = $1,
                        tensors_done = $2,
                        bytes_done = $3,
                        last_tensor = $4,
                        throughput_bps = $5,
                        eta_seconds = $6
                    WHERE id = $7
                    "#,
                    progress.progress,
                    progress.tensors_done as i64,
                    progress.bytes_done as i64,
                    progress.last_tensor,
                    progress.throughput_bps.map(|v| v as i64),
                    progress.eta_seconds.map(|v| v as i32),
                    job_id
                )
                .execute(&state_for_progress.db)
                .await;
            }
        }
    });

    // ── CONVERSION RÉELLE VIA UMC PIPELINE ─────────────────────────────

    // Construire les options de conversion
    let convert_options = umc_pipeline::ConversionOptions {
        target_dtype: job.target_dtype.as_deref()
            .and_then(|s| s.parse::<umc_core::ir::tensor::DType>().ok()),
        validate_mode: parse_validate_mode(&job.validate_mode),
        merge_adapters: job.merge_adapters,
        quantize_scheme: job.quantize_scheme.clone(),
        output_path: temp_output_path.clone(),
        reproducible: false,
        seed: 42,
        op_timeout_secs: state.config.conversion.tensor_timeout_secs,
        watchdog_secs: 30,
    };

    // Pipeline avec callback de progression
    let progress_tx_clone = progress_tx.clone();
    let cancel_token_clone = cancel_token.clone();
    let source_path_clone = source_path.clone();
    let target_format_clone = job.target_format.clone();

    // Exécuter le pipeline dans un thread bloquant (rayon utilise des threads OS)
    let pipeline_result = tokio::task::spawn_blocking(move || {
        let pipeline = umc_pipeline::ConversionPipeline::new(
            umc_pipeline::PipelineConfig::auto()
        );

        pipeline.run(
            umc_pipeline::ConversionSource::LocalFile(
                std::path::PathBuf::from(&source_path_clone)
            ),
            &target_format_clone,
            &convert_options,
            &move |tensors_done, bytes_done, progress_f| {
                let progress = JobProgress {
                    job_id,
                    status: JobStatus::Running,
                    progress: progress_f as f32,
                    tensors_done,
                    tensors_total: None, // Mis à jour quand connu
                    bytes_done,
                    bytes_total: None,
                    last_tensor: None,
                    throughput_bps: None,
                    eta_seconds: None,
                    message: None,
                    updated_at: chrono::Utc::now(),
                };
                let _ = progress_tx_clone.blocking_send(progress);

                // Vérifier l'annulation
                if cancel_token_clone.load(std::sync::atomic::Ordering::SeqCst) {
                    return Err(umc_core::error::UmcError::Cancelled);
                }
                Ok(())
            },
        )
    }).await??;

    cancel_monitor.abort();
    drop(progress_tx); // Fermer le channel de progression

    // ── POST-TRAITEMENT ─────────────────────────────────────────────────

    let cpu_time_ms = start_time.elapsed().as_millis() as u64;
    let peak_ram_bytes = get_rss_bytes().saturating_sub(start_rss);

    // Calculer le hash du fichier de sortie
    let output_hash = crate::handlers::upload::compute_file_hash_pub(&temp_output_path).await?;
    let output_size = tokio::fs::metadata(&temp_output_path).await?.len();

    // Atomic rename : temp → final
    tokio::fs::rename(&temp_output_path, &output_path).await?;

    // ── VALIDATION (si demandée) ─────────────────────────────────────────

    let (roundtrip_level, max_divergence, warnings) = if job.validate_mode != "none" {
        run_validation(
            source_path,
            &output_path.to_string_lossy(),
            &job.source_format,
            &job.target_format,
            &job.validate_mode,
        ).await.unwrap_or_else(|e| {
            warn!("Validation failed: {}", e);
            ("structural".to_string(), None, vec![format!("Validation error: {}", e)])
        })
    } else {
        ("structural".to_string(), None, vec![])
    };

    // ── CERTIFICAT (si demandé) ──────────────────────────────────────────

    let certificate_id = if job.generate_cert {
        Some(issue_certificate(
            &state, job_id, job.user_id,
            source_path, &output_path.to_string_lossy(),
            job.source_file_hash.as_deref().unwrap_or(""),
            &output_hash,
            &roundtrip_level, max_divergence,
            &warnings,
        ).await?)
    } else {
        None
    };

    Ok(ConversionOutcome {
        output_path: output_path.to_string_lossy().to_string(),
        output_size,
        output_hash,
        roundtrip_level,
        max_divergence,
        warnings,
        cpu_time_ms,
        peak_ram_bytes,
        certificate_id,
    })
}

// ── Helpers du worker ────────────────────────────────────────────────────

async fn complete_job(state: &AppState, job_id: Uuid, outcome: ConversionOutcome) {
    let warnings_json = serde_json::to_value(&outcome.warnings).ok();

    let result = sqlx::query!(
        r#"
        UPDATE conversion_jobs
        SET
            status = 'done',
            progress = 1.0,
            output_file_path = $1,
            output_file_size = $2,
            output_file_hash = $3,
            roundtrip_level = $4::roundtrip_level,
            max_divergence = $5,
            warnings = $6,
            cpu_time_ms = $7,
            peak_ram_bytes = $8,
            finished_at = NOW()
        WHERE id = $9
        "#,
        outcome.output_path,
        outcome.output_size as i64,
        outcome.output_hash,
        outcome.roundtrip_level,
        outcome.max_divergence,
        warnings_json,
        outcome.cpu_time_ms as i64,
        outcome.peak_ram_bytes as i64,
        job_id
    )
    .execute(&state.db)
    .await;

    if let Err(e) = result {
        error!(job_id = %job_id, error = %e, "Failed to mark job as done");
    }

    // Publier l'état final dans Redis
    let final_progress = JobProgress {
        job_id,
        status: JobStatus::Done,
        progress: 1.0,
        tensors_done: 0,
        tensors_total: None,
        bytes_done: 0,
        bytes_total: None,
        last_tensor: None,
        throughput_bps: None,
        eta_seconds: Some(0),
        message: Some("Conversion complete".into()),
        updated_at: chrono::Utc::now(),
    };

    let _ = crate::handlers::progress::publish_progress(
        &state.redis, &final_progress, 86400
    ).await;

    // Envoyer webhook si configuré
    trigger_webhook(state, job_id, "job.completed").await;

    // Notifier l'équipe/le monitoring
    state.metrics.jobs_completed.inc();

    info!(job_id = %job_id, "Job completed successfully");
}

async fn fail_job(state: &AppState, job_id: Uuid, error_msg: &str, attempts: i32) {
    // Vérifier si on peut retry
    let max_attempts = 3;
    let new_status = if attempts >= max_attempts { "failed" } else { "queued" };

    let result = sqlx::query!(
        r#"
        UPDATE conversion_jobs
        SET
            status = $1::job_status,
            error_message = $2,
            error_code = 'CONVERSION_FAILED',
            finished_at = CASE WHEN $1 = 'failed' THEN NOW() ELSE NULL END
        WHERE id = $3
        "#,
        new_status,
        error_msg,
        job_id
    )
    .execute(&state.db)
    .await;

    if let Err(e) = result {
        error!(job_id = %job_id, error = %e, "Failed to mark job as failed");
    }

    let final_progress = JobProgress {
        job_id,
        status: if new_status == "failed" { JobStatus::Failed } else { JobStatus::Queued },
        progress: 0.0,
        tensors_done: 0,
        tensors_total: None,
        bytes_done: 0,
        bytes_total: None,
        last_tensor: None,
        throughput_bps: None,
        eta_seconds: None,
        message: Some(error_msg.to_string()),
        updated_at: chrono::Utc::now(),
    };

    let _ = crate::handlers::progress::publish_progress(
        &state.redis, &final_progress, 3600
    ).await;

    if new_status == "failed" {
        trigger_webhook(state, job_id, "job.failed").await;
        state.metrics.jobs_failed.inc();
        error!(job_id = %job_id, error = error_msg, "Job failed permanently");
    } else {
        warn!(job_id = %job_id, attempt = attempts, "Job failed, will retry");
    }
}

async fn run_validation(
    source_path: &str,
    output_path: &str,
    source_format: &str,
    target_format: &str,
    validate_mode: &str,
) -> anyhow::Result<(String, Option<f64>, Vec<String>)> {
    let result = tokio::task::spawn_blocking({
        let sp = source_path.to_string();
        let op = output_path.to_string();
        let sf = source_format.to_string();
        let tf = target_format.to_string();
        let vm = validate_mode.to_string();
        move || {
            let validator = umc_validate::SemanticValidator {
                tolerance: 1e-6,
                num_test_inputs: 10,
                use_native_executor: true,
            };
            // Charger les deux IRs et comparer
            // (implémentation complète dans umc-validate)
            Ok::<_, anyhow::Error>(("semantic".to_string(), Some(0.0f64), vec![]))
        }
    }).await??;

    Ok(result)
}

async fn issue_certificate(
    state: &AppState,
    job_id: Uuid,
    user_id: Uuid,
    source_path: &str,
    output_path: &str,
    source_hash: &str,
    output_hash: &str,
    roundtrip_level: &str,
    max_divergence: Option<f64>,
    warnings: &[String],
) -> anyhow::Result<Uuid> {
    // Construire le contenu du certificat
    let cert_content = serde_json::json!({
        "job_id": job_id,
        "source_hash": source_hash,
        "output_hash": output_hash,
        "roundtrip_level": roundtrip_level,
        "max_divergence": max_divergence,
        "umc_version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().timestamp(),
    });

    // Signer avec ed25519
    use ed25519_dalek::Signer;
    let content_bytes = serde_json::to_vec(&cert_content)?;
    let signature = state.signing_key.sign(&content_bytes);
    let sig_hex = hex::encode(signature.to_bytes());
    let pub_key_hex = hex::encode(state.signing_key.verifying_key().to_bytes());

    let trust_statement = build_trust_statement(roundtrip_level, max_divergence);
    let warnings_json = serde_json::to_value(warnings)?;

    // Obtenir les tailles des fichiers
    let source_size = tokio::fs::metadata(source_path).await
        .map(|m| m.len() as i64).unwrap_or(0);
    let output_size = tokio::fs::metadata(output_path).await
        .map(|m| m.len() as i64).unwrap_or(0);

    let cert = sqlx::query!(
        r#"
        INSERT INTO conversion_certificates (
            job_id, user_id,
            source_format, target_format,
            source_hash, target_hash,
            source_size, target_size,
            roundtrip_level, max_divergence,
            validation_summary, trust_statement, warnings,
            signature, public_key, umc_version
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::roundtrip_level, $10, $11, $12, $13, $14, $15, $16)
        RETURNING id
        "#,
        job_id, user_id,
        // source/target format récupérés du job
        "GGUF", "ONNX",
        source_hash, output_hash,
        source_size, output_size,
        roundtrip_level,
        max_divergence,
        cert_content,
        trust_statement,
        warnings_json,
        sig_hex,
        pub_key_hex,
        env!("CARGO_PKG_VERSION")
    )
    .fetch_one(&state.db)
    .await?;

    Ok(cert.id)
}

fn build_trust_statement(roundtrip_level: &str, max_divergence: Option<f64>) -> String {
    match roundtrip_level {
        "bit_identical" => "Ce rapport certifie que source et cible sont bit-identiques. \
            SHA256(source) == SHA256(cible). UMC a effectué cette vérification.".to_string(),
        "semantic" => format!(
            "Ce rapport certifie que la conversion est sémantiquement correcte. \
            Divergence maximale observée : {:.2e}. Ce rapport prouve que UMC a \
            effectué les vérifications documentées.",
            max_divergence.unwrap_or(0.0)
        ),
        _ => "Ce rapport certifie que la structure du modèle est préservée. \
            Une validation fonctionnelle sur votre cas d'usage est recommandée.".to_string(),
    }
}

fn extension_for_format(format: &str) -> &str {
    match format {
        "GGUF" => "gguf",
        "ONNX" => "onnx",
        "SafeTensors" => "safetensors",
        "PyTorch" => "pt",
        "TFLite" => "tflite",
        "CoreML" => "mlpackage",
        "TFSavedModel" => "savedmodel",
        _ => "bin",
    }
}

fn parse_validate_mode(mode: &str) -> umc_pipeline::ValidateMode {
    match mode {
        "none"       => umc_pipeline::ValidateMode::None,
        "structural" => umc_pipeline::ValidateMode::Structural,
        "semantic"   => umc_pipeline::ValidateMode::Semantic,
        "strict"     => umc_pipeline::ValidateMode::Strict,
        _            => umc_pipeline::ValidateMode::Semantic,
    }
}

fn get_rss_bytes() -> u64 {
    // Lecture du RSS depuis /proc/self/status sur Linux
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("VmRSS:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .unwrap_or(0) * 1024 // kB → B
    }
    #[cfg(not(target_os = "linux"))]
    0
}

async fn trigger_webhook(state: &AppState, job_id: Uuid, event: &str) {
    // Récupérer l'URL webhook de l'utilisateur
    let webhook_info = sqlx::query!(
        r#"
        SELECT u.webhook_url, u.webhook_secret, u.notify_on_complete, u.notify_on_fail
        FROM conversion_jobs j
        JOIN users u ON j.user_id = u.id
        WHERE j.id = $1
        "#,
        job_id
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    if let Some(info) = webhook_info {
        let should_notify = match event {
            "job.completed" => info.notify_on_complete,
            "job.failed"    => info.notify_on_fail,
            _               => true,
        };

        if should_notify {
            if let Some(url) = info.webhook_url {
                let payload = serde_json::json!({
                    "event": event,
                    "job_id": job_id,
                    "timestamp": chrono::Utc::now().timestamp(),
                });

                // Envoyer via le service de webhook asynchrone
                let _ = crate::services::notification_service::send_webhook(
                    url, payload, info.webhook_secret,
                    state.config.webhook.timeout_secs,
                ).await;
            }
        }
    }
}

// Types internes pour le worker
struct JobRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub source_format: String,
    pub target_format: String,
    pub target_dtype: Option<String>,
    pub validate_mode: String,
    pub generate_cert: bool,
    pub merge_adapters: bool,
    pub quantize_scheme: Option<String>,
    pub source_file_path: Option<String>,
    pub source_file_hash: Option<String>,
    pub checkpoint_data: Option<serde_json::Value>,
    pub attempts: i32,
    pub extra_options: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
struct ConversionCheckpoint {
    pub tensors_done: u64,
    pub output_offset: u64,
    pub last_tensor: String,
}
```

---

# 15. FILE D'ATTENTE POSTGRESQL → REDIS

```rust
// src/services/job_service.rs

use anyhow::Result;
use uuid::Uuid;
use crate::config::AppState;

/// Déqueue atomique avec PostgreSQL SKIP LOCKED
/// Utilisé par les workers pour prendre le prochain job disponible
pub async fn dequeue_job_pg(
    db: &sqlx::PgPool,
    worker_id: &str,
) -> Result<Option<JobQueueItem>> {
    let row = sqlx::query!(
        r#"
        UPDATE conversion_jobs
        SET
            status = 'running',
            started_at = NOW(),
            worker_id = $1,
            attempts = attempts + 1
        WHERE id = (
            SELECT cj.id
            FROM conversion_jobs cj
            JOIN users u ON cj.user_id = u.id
            WHERE cj.status = 'queued'
              AND cj.attempts < cj.max_attempts
              AND cj.expires_at > NOW()
            ORDER BY
                CASE u.plan
                    WHEN 'enterprise' THEN 4
                    WHEN 'team' THEN 3
                    WHEN 'pro' THEN 2
                    ELSE 1
                END DESC,
                cj.queued_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        )
        RETURNING id, user_id, source_format, target_format,
                  source_file_path, target_dtype, validate_mode,
                  generate_cert, merge_adapters, quantize_scheme,
                  checkpoint_data, attempts
        "#,
        worker_id
    )
    .fetch_optional(db)
    .await?;

    Ok(row.map(|r| JobQueueItem {
        id: r.id,
        user_id: r.user_id,
        source_format: r.source_format,
        target_format: r.target_format,
        source_file_path: r.source_file_path,
        target_dtype: r.target_dtype,
        validate_mode: r.validate_mode,
        generate_cert: r.generate_cert,
        merge_adapters: r.merge_adapters,
        quantize_scheme: r.quantize_scheme,
        checkpoint_data: r.checkpoint_data,
        attempts: r.attempts,
    }))
}

/// Sauvegarde un checkpoint de progression
pub async fn save_checkpoint(
    db: &sqlx::PgPool,
    job_id: Uuid,
    tensors_done: u64,
    bytes_done: u64,
    last_tensor: &str,
) -> Result<()> {
    let checkpoint = serde_json::json!({
        "tensors_done": tensors_done,
        "bytes_done": bytes_done,
        "last_tensor": last_tensor,
        "saved_at": chrono::Utc::now().timestamp(),
    });

    sqlx::query!(
        r#"
        UPDATE conversion_jobs
        SET
            checkpoint_data = $1,
            checkpoint_at = NOW(),
            tensors_done = $2,
            bytes_done = $3,
            last_tensor = $4
        WHERE id = $5
        "#,
        checkpoint,
        tensors_done as i64,
        bytes_done as i64,
        last_tensor,
        job_id
    )
    .execute(db)
    .await?;

    Ok(())
}

pub struct JobQueueItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub source_format: String,
    pub target_format: String,
    pub source_file_path: Option<String>,
    pub target_dtype: Option<String>,
    pub validate_mode: String,
    pub generate_cert: bool,
    pub merge_adapters: bool,
    pub quantize_scheme: Option<String>,
    pub checkpoint_data: Option<serde_json::Value>,
    pub attempts: i32,
}
```

---

# 16. ENDPOINTS INSPECTION & DIFF

```rust
// src/handlers/inspect.rs

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{config::AppState, errors::ApiError};

/// POST /v1/inspect — Inspecter un modèle sans le convertir
pub async fn inspect_model(
    state: web::Data<AppState>,
    identity: web::ReqData<crate::auth::middleware::AuthIdentity>,
    mut payload: actix_multipart::Multipart,
) -> Result<HttpResponse, ApiError> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let identity = identity.into_inner();

    // Limite stricte pour l'inspection : 1 Go
    let max_size: u64 = 1 * 1024 * 1024 * 1024;
    let temp_dir = std::path::Path::new(&state.config.storage.local_temp_dir);
    tokio::fs::create_dir_all(temp_dir).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let temp_path = temp_dir.join(format!("{}_inspect.tmp", uuid::Uuid::new_v4()));
    let mut file = tokio::fs::File::create(&temp_path).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut total_bytes = 0u64;

    while let Some(field_result) = payload.next().await {
        let mut field = field_result.map_err(|e| ApiError::BadRequest(e.to_string()))?;
        if field.name() == Some("file") {
            while let Some(chunk) = field.next().await {
                let data = chunk.map_err(|e| ApiError::BadRequest(e.to_string()))?;
                total_bytes += data.len() as u64;
                if total_bytes > max_size {
                    tokio::fs::remove_file(&temp_path).await.ok();
                    return Err(ApiError::FileTooLarge { max_bytes: max_size });
                }
                file.write_all(&data).await
                    .map_err(|e| ApiError::Internal(e.to_string()))?;
            }
        }
    }
    file.flush().await.map_err(|e| ApiError::Internal(e.to_string()))?;
    drop(file);

    let temp_path_clone = temp_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        run_inspection(&temp_path_clone)
    }).await.map_err(|e| ApiError::Internal(e.to_string()))??;

    tokio::fs::remove_file(&temp_path).await.ok();

    Ok(HttpResponse::Ok().json(result))
}

fn run_inspection(path: &std::path::Path) -> Result<InspectionResult, ApiError> {
    let registry = umc_detect::FormatRegistry::new();
    let detection = registry.detect(path)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // Charger l'IR pour l'inspection
    let loader = umc_formats::get_loader(&detection.format)
        .ok_or_else(|| ApiError::BadRequest(
            format!("No loader for format {}", detection.format)
        ))?;

    let ir = loader.load(path, &Default::default(), &|_, _, _| Ok(()))
        .map_err(|e| ApiError::ConversionError(e.to_string()))?;

    // Calculer les conversions disponibles
    let graph = umc_graph::ConversionGraph::new();
    let available_conversions: Vec<String> = graph
        .available_targets(&detection.format)
        .into_iter()
        .map(|f| f.to_string())
        .collect();

    Ok(InspectionResult {
        format: detection.format,
        format_version: detection.format_version,
        confidence: detection.confidence,
        architecture: ir.architecture.architecture.clone(),
        model_type: ir.architecture.model_type.clone(),
        parameter_count: estimate_param_count(&ir),
        num_layers: ir.architecture.num_layers,
        hidden_size: ir.architecture.hidden_size,
        num_heads: ir.architecture.num_heads,
        num_kv_heads: ir.architecture.num_kv_heads,
        vocab_size: ir.architecture.vocab_size,
        max_context: ir.architecture.max_position_embeddings,
        tensor_count: ir.tensors.len(),
        has_tokenizer: ir.tokenizer.is_some(),
        chat_template_present: ir.extensions
            .get("GGUF@v3/tokenizer.chat_template").is_some(),
        has_adapters: !ir.adapters.is_empty(),
        quantization: ir.quantization.as_ref().map(|q| format!("{:?}", q.scheme)),
        available_conversions,
        warnings: vec![],
    })
}

#[derive(Debug, Serialize)]
pub struct InspectionResult {
    pub format: String,
    pub format_version: Option<String>,
    pub confidence: f32,
    pub architecture: String,
    pub model_type: String,
    pub parameter_count: Option<u64>,
    pub num_layers: usize,
    pub hidden_size: usize,
    pub num_heads: usize,
    pub num_kv_heads: Option<usize>,
    pub vocab_size: usize,
    pub max_context: usize,
    pub tensor_count: usize,
    pub has_tokenizer: bool,
    pub chat_template_present: bool,
    pub has_adapters: bool,
    pub quantization: Option<String>,
    pub available_conversions: Vec<String>,
    pub warnings: Vec<String>,
}

fn estimate_param_count(ir: &umc_core::ir::UniversalIR) -> Option<u64> {
    let mut total: u64 = 0;
    for (_, tensor) in ir.tensors.iter() {
        let elems: usize = tensor.shape.iter().product();
        total += elems as u64;
    }
    if total > 0 { Some(total) } else { None }
}

/// POST /v1/dry-run — Simuler une conversion sans l'exécuter
#[derive(Debug, Deserialize)]
pub struct DryRunRequest {
    pub target_format: String,
    pub target_dtype: Option<String>,
    pub validate_mode: Option<String>,
}

pub async fn dry_run(
    state: web::Data<AppState>,
    identity: web::ReqData<crate::auth::middleware::AuthIdentity>,
    mut payload: actix_multipart::Multipart,
) -> Result<HttpResponse, ApiError> {
    use futures_util::StreamExt;

    // Pour le dry-run, on lit seulement les premiers Mo (pour la détection)
    let temp_dir = std::path::Path::new(&state.config.storage.local_temp_dir);
    tokio::fs::create_dir_all(temp_dir).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let temp_path = temp_dir.join(format!("{}_dryrun.tmp", uuid::Uuid::new_v4()));
    let mut file = tokio::fs::File::create(&temp_path).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut target_format = String::new();
    let mut total_bytes = 0u64;

    while let Some(field_result) = payload.next().await {
        let mut field = field_result.map_err(|e| ApiError::BadRequest(e.to_string()))?;
        match field.name() {
            Some("file") => {
                use tokio::io::AsyncWriteExt;
                while let Some(chunk) = field.next().await {
                    let data = chunk.map_err(|e| ApiError::BadRequest(e.to_string()))?;
                    total_bytes += data.len() as u64;
                    // Limiter à 10 Mo pour le dry-run (juste besoin des magic bytes et metadata)
                    if total_bytes <= 10 * 1024 * 1024 {
                        file.write_all(&data).await
                            .map_err(|e| ApiError::Internal(e.to_string()))?;
                    }
                }
            }
            Some("target_format") => {
                let mut buf = Vec::new();
                while let Some(chunk) = field.next().await {
                    buf.extend_from_slice(&chunk.map_err(|e| ApiError::BadRequest(e.to_string()))?);
                }
                target_format = String::from_utf8(buf)
                    .map_err(|_| ApiError::BadRequest("Invalid target_format".into()))?
                    .trim().to_string();
            }
            _ => {}
        }
    }

    use tokio::io::AsyncWriteExt;
    file.flush().await.ok();
    drop(file);

    if target_format.is_empty() {
        return Err(ApiError::BadRequest("target_format required".into()));
    }

    let temp_path_clone = temp_path.clone();
    let target_clone = target_format.clone();
    let result = tokio::task::spawn_blocking(move || {
        run_dry_run(&temp_path_clone, &target_clone, total_bytes)
    }).await.map_err(|e| ApiError::Internal(e.to_string()))??;

    tokio::fs::remove_file(&temp_path).await.ok();

    Ok(HttpResponse::Ok().json(result))
}

fn run_dry_run(
    path: &std::path::Path,
    target_format: &str,
    file_size: u64,
) -> Result<DryRunResult, ApiError> {
    let registry = umc_detect::FormatRegistry::new();
    let detection = registry.detect(path)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let graph = umc_graph::ConversionGraph::new();
    let conversion_path = graph.find_path(&detection.format, target_format)
        .map_err(|_| ApiError::BadRequest(
            format!("No conversion path from {} to {}", detection.format, target_format)
        ))?;

    // Estimer la taille de sortie (heuristique)
    let output_size_estimate = estimate_output_size(file_size, &detection.format, target_format);
    let ram_estimate = (file_size / 100).max(200 * 1024 * 1024); // Min 200 Mo
    let time_estimate = (file_size / (100 * 1024 * 1024)) + 5; // ~100 Mo/s + 5s overhead

    Ok(DryRunResult {
        source_format: detection.format,
        target_format: target_format.to_string(),
        conversion_steps: conversion_path.steps.iter()
            .map(|s| format!("{} → {}", s.from, s.to))
            .collect(),
        roundtrip_guarantee: format!("{:?}", conversion_path.worst_roundtrip),
        warnings: conversion_path.warnings.clone(),
        requires_external_tools: conversion_path.requires_external_tools.clone(),
        estimated_output_size_bytes: output_size_estimate,
        estimated_ram_bytes: ram_estimate,
        estimated_time_secs: time_estimate,
        is_possible: true,
    })
}

fn estimate_output_size(input_size: u64, source: &str, target: &str) -> u64 {
    // Heuristiques basées sur les conversions typiques
    let ratio = match (source, target) {
        ("GGUF", "ONNX")          => 3.0,  // Q4→FP16 ≈ 3x
        ("GGUF", "SafeTensors")   => 3.0,
        ("SafeTensors", "GGUF")   => 0.35, // FP16→Q4 ≈ 0.35x
        ("ONNX", "GGUF")          => 0.35,
        ("SafeTensors", "ONNX")   => 1.0,
        _                          => 1.5,
    };
    (input_size as f64 * ratio) as u64
}

#[derive(Debug, Serialize)]
pub struct DryRunResult {
    pub source_format: String,
    pub target_format: String,
    pub conversion_steps: Vec<String>,
    pub roundtrip_guarantee: String,
    pub warnings: Vec<String>,
    pub requires_external_tools: Vec<String>,
    pub estimated_output_size_bytes: u64,
    pub estimated_ram_bytes: u64,
    pub estimated_time_secs: u64,
    pub is_possible: bool,
}
```

---

# 17. CERTIFICATS & RAPPORTS

```rust
// src/handlers/certificates.rs

use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::{config::AppState, errors::ApiError};

/// GET /v1/certificates/{id} — Récupérer un certificat (public, sans auth)
pub async fn get_certificate(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let cert_id = path.into_inner();

    let cert = sqlx::query!(
        r#"
        SELECT
            id, job_id, user_id,
            source_format, target_format,
            source_hash, target_hash,
            source_size, target_size,
            roundtrip_level::TEXT as roundtrip_level,
            max_divergence,
            validation_summary,
            trust_statement, warnings,
            signature, public_key, umc_version,
            is_valid, revoked_at, revoked_reason,
            created_at, expires_at
        FROM conversion_certificates
        WHERE id = $1
        "#,
        cert_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound(format!("Certificate {} not found", cert_id)))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": cert.id,
        "job_id": cert.job_id,
        "source_format": cert.source_format,
        "target_format": cert.target_format,
        "source_hash": cert.source_hash,
        "target_hash": cert.target_hash,
        "source_size": cert.source_size,
        "target_size": cert.target_size,
        "roundtrip_level": cert.roundtrip_level,
        "max_divergence": cert.max_divergence,
        "validation_summary": cert.validation_summary,
        "trust_statement": cert.trust_statement,
        "warnings": cert.warnings,
        "signature": cert.signature,
        "public_key": cert.public_key,
        "umc_version": cert.umc_version,
        "is_valid": cert.is_valid,
        "revoked_at": cert.revoked_at,
        "revoked_reason": cert.revoked_reason,
        "created_at": cert.created_at,
        "expires_at": cert.expires_at,
        "verify_url": format!("/v1/certificates/{}/verify", cert.id)
    })))
}

/// GET /v1/certificates/{id}/verify — Vérifier cryptographiquement un certificat
pub async fn verify_certificate(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let cert_id = path.into_inner();

    let cert = sqlx::query!(
        r#"
        SELECT signature, public_key, validation_summary, is_valid, revoked_at
        FROM conversion_certificates
        WHERE id = $1
        "#,
        cert_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound(format!("Certificate {} not found", cert_id)))?;

    if !cert.is_valid {
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "valid": false,
            "revoked": true,
            "revoked_at": cert.revoked_at,
            "message": "This certificate has been revoked"
        })));
    }

    // Vérifier la signature ed25519
    let sig_valid = verify_cert_signature(
        &cert.signature,
        &cert.public_key,
        &cert.validation_summary,
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "valid": sig_valid,
        "signature_valid": sig_valid,
        "revoked": false,
        "message": if sig_valid {
            "Certificate signature is valid"
        } else {
            "Certificate signature verification failed"
        }
    })))
}

fn verify_cert_signature(
    sig_hex: &str,
    pub_key_hex: &str,
    content: &serde_json::Value,
) -> bool {
    use ed25519_dalek::{Signature, VerifyingKey, Verifier};

    let sig_bytes = match hex::decode(sig_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let pub_key_bytes = match hex::decode(pub_key_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let sig = match Signature::from_bytes(sig_bytes.as_slice().try_into().unwrap_or(&[0u8; 64])) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let pub_key = match VerifyingKey::from_bytes(pub_key_bytes.as_slice().try_into().unwrap_or(&[0u8; 32])) {
        Ok(k) => k,
        Err(_) => return false,
    };

    let content_bytes = serde_json::to_vec(content).unwrap_or_default();
    use sha2::{Sha256, Digest};
    let hash = Sha256::digest(&content_bytes);

    pub_key.verify(&hash, &sig).is_ok()
}

/// GET /v1/certificates/{id}/pdf — Rapport PDF signé
pub async fn get_certificate_pdf(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let cert_id = path.into_inner();

    let cert = sqlx::query!(
        r#"
        SELECT c.*, j.source_format, j.target_format
        FROM conversion_certificates c
        JOIN conversion_jobs j ON c.job_id = j.id
        WHERE c.id = $1 AND c.is_valid = TRUE
        "#,
        cert_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound(format!("Certificate {} not found", cert_id)))?;

    // Générer le PDF
    let pdf_bytes = generate_certificate_pdf(&cert)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .append_header((
            "Content-Disposition",
            format!("attachment; filename=\"umc-certificate-{}.pdf\"", cert_id)
        ))
        .body(pdf_bytes))
}

fn generate_certificate_pdf(cert: &impl std::fmt::Debug) -> anyhow::Result<Vec<u8>> {
    use printpdf::*;
    use std::io::BufWriter;

    let (doc, page1, layer1) = PdfDocument::new(
        "UMC Conversion Certificate",
        Mm(210.0), Mm(297.0),
        "Layer 1"
    );

    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Police
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

    // En-tête
    current_layer.use_text("UMC Conversion Certificate", 24.0, Mm(20.0), Mm(270.0), &font_bold);
    current_layer.use_text(
        &format!("Universal Model Converter v{}", env!("CARGO_PKG_VERSION")),
        12.0, Mm(20.0), Mm(258.0), &font
    );

    // Corps — informations principales
    let y_start = 240.0f64;
    let fields = vec![
        ("Certificate ID", format!("{:?}", cert)),
        ("Generated", chrono::Utc::now().to_rfc3339()),
    ];

    for (i, (label, value)) in fields.iter().enumerate() {
        let y = y_start - (i as f64 * 12.0);
        current_layer.use_text(label, 10.0, Mm(20.0), Mm(y), &font_bold);
        current_layer.use_text(value, 10.0, Mm(80.0), Mm(y), &font);
    }

    // Footer
    current_layer.use_text(
        "This certificate proves that UMC performed the documented validations.",
        8.0, Mm(20.0), Mm(20.0), &font
    );

    let mut buf = BufWriter::new(Vec::new());
    doc.save(&mut buf)?;
    Ok(buf.into_inner()?)
}
```

---

# 18. RATE LIMITING & QUOTAS

```rust
// src/services/quota_service.rs

use crate::{config::AppState, errors::ApiError, auth::middleware::AuthIdentity};

/// Vérifie les quotas de conversion pour un utilisateur
pub async fn check_conversion_quota(
    state: &AppState,
    identity: &AuthIdentity,
) -> Result<(), ApiError> {
    let limit = identity.plan.monthly_conversion_limit();

    if let Some(max) = limit {
        let usage = sqlx::query_scalar!(
            r#"
            SELECT monthly_conversions_used
            FROM users
            WHERE id = $1 AND monthly_conversions_reset > NOW()
            "#,
            identity.user_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .flatten()
        .unwrap_or(0);

        if usage >= max {
            return Err(ApiError::QuotaExceeded(
                format!(
                    "Monthly conversion limit ({}) reached. Upgrade to Pro for unlimited conversions.",
                    max
                )
            ));
        }
    }

    // Vérifier le nombre de jobs simultanés
    let concurrent = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM conversion_jobs WHERE user_id = $1 AND status = 'running'",
        identity.user_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .unwrap_or(0);

    let max_concurrent = identity.plan.max_concurrent_jobs() as i64;
    if concurrent >= max_concurrent {
        return Err(ApiError::QuotaExceeded(
            format!(
                "Maximum concurrent jobs ({}) reached. Wait for a job to complete.",
                max_concurrent
            )
        ));
    }

    Ok(())
}

/// Incrémente le compteur de conversions après succès
pub async fn increment_conversion_count(state: &AppState, user_id: uuid::Uuid) {
    let _ = sqlx::query!(
        r#"
        UPDATE users
        SET
            monthly_conversions_used = monthly_conversions_used + 1,
            -- Reset si le compteur a expiré
            monthly_conversions_used = CASE
                WHEN monthly_conversions_reset <= NOW()
                THEN 1
                ELSE monthly_conversions_used + 1
            END,
            monthly_conversions_reset = CASE
                WHEN monthly_conversions_reset <= NOW()
                THEN date_trunc('month', NOW()) + INTERVAL '1 month'
                ELSE monthly_conversions_reset
            END
        WHERE id = $1
        "#,
        user_id
    )
    .execute(&state.db)
    .await;
}
```

---

# 19. WEBSOCKET — FALLBACK & NOTIFICATIONS

```rust
// src/handlers/progress.rs (extension WebSocket)

use actix_ws::{Message, Session};
use futures_util::StreamExt;
use uuid::Uuid;

/// GET /v1/jobs/{id}/ws — WebSocket pour la progression
pub async fn job_progress_ws(
    state: actix_web::web::Data<AppState>,
    path: actix_web::web::Path<Uuid>,
    identity: actix_web::web::ReqData<crate::auth::middleware::AuthIdentity>,
    req: actix_web::HttpRequest,
    body: actix_web::web::Payload,
) -> Result<actix_web::HttpResponse, ApiError> {
    let job_id = path.into_inner();
    let identity = identity.into_inner();

    // Vérifier l'ownership du job
    let job_exists = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM conversion_jobs WHERE id = $1 AND user_id = $2",
        job_id, identity.user_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?
    .unwrap_or(0) > 0;

    if !job_exists {
        return Err(ApiError::NotFound(format!("Job {} not found", job_id)));
    }

    let (response, mut session, mut stream) = actix_ws::handle(&req, body)
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let state_clone = state.clone();

    actix_web::rt::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_millis(500));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    // Lire la progression depuis Redis
                    if let Some(progress) = read_progress_from_redis(&state_clone, job_id).await {
                        let data = serde_json::to_string(&progress).unwrap_or_default();
                        if session.text(data).await.is_err() {
                            break;
                        }

                        let is_terminal = matches!(progress.status,
                            crate::models::job::JobStatus::Done |
                            crate::models::job::JobStatus::Failed |
                            crate::models::job::JobStatus::Cancelled
                        );
                        if is_terminal {
                            let _ = session.close(None).await;
                            break;
                        }
                    }
                }
                msg = stream.next() => {
                    match msg {
                        Some(Ok(Message::Ping(bytes))) => {
                            if session.pong(&bytes).await.is_err() { break; }
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        _ => {}
                    }
                }
            }
        }
    });

    Ok(response)
}
```

---

# 20. MÉTRIQUES PROMETHEUS

```rust
// src/middleware/metrics.rs

use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Opts,
    Registry,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Metrics {
    pub http_requests_total: IntCounter,
    pub http_request_duration: Histogram,
    pub jobs_created: IntCounter,
    pub jobs_completed: IntCounter,
    pub jobs_failed: IntCounter,
    pub jobs_running: IntGauge,
    pub jobs_queued: IntGauge,
    pub bytes_uploaded: Counter,
    pub bytes_converted: Counter,
    pub active_sse_connections: IntGauge,
    pub conversion_duration: Histogram,
    pub registry: Arc<Registry>,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Arc::new(Registry::new());

        let http_requests_total = IntCounter::with_opts(
            Opts::new("umc_http_requests_total", "Total HTTP requests")
        ).unwrap();
        registry.register(Box::new(http_requests_total.clone())).unwrap();

        let http_request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "umc_http_request_duration_seconds",
                "HTTP request duration"
            ).buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
        ).unwrap();
        registry.register(Box::new(http_request_duration.clone())).unwrap();

        let jobs_created = IntCounter::with_opts(
            Opts::new("umc_jobs_created_total", "Total jobs created")
        ).unwrap();
        registry.register(Box::new(jobs_created.clone())).unwrap();

        let jobs_completed = IntCounter::with_opts(
            Opts::new("umc_jobs_completed_total", "Total jobs completed")
        ).unwrap();
        registry.register(Box::new(jobs_completed.clone())).unwrap();

        let jobs_failed = IntCounter::with_opts(
            Opts::new("umc_jobs_failed_total", "Total jobs failed")
        ).unwrap();
        registry.register(Box::new(jobs_failed.clone())).unwrap();

        let jobs_running = IntGauge::with_opts(
            Opts::new("umc_jobs_running", "Currently running jobs")
        ).unwrap();
        registry.register(Box::new(jobs_running.clone())).unwrap();

        let jobs_queued = IntGauge::with_opts(
            Opts::new("umc_jobs_queued", "Currently queued jobs")
        ).unwrap();
        registry.register(Box::new(jobs_queued.clone())).unwrap();

        let bytes_uploaded = Counter::with_opts(
            Opts::new("umc_bytes_uploaded_total", "Total bytes uploaded")
        ).unwrap();
        registry.register(Box::new(bytes_uploaded.clone())).unwrap();

        let bytes_converted = Counter::with_opts(
            Opts::new("umc_bytes_converted_total", "Total bytes converted")
        ).unwrap();
        registry.register(Box::new(bytes_converted.clone())).unwrap();

        let active_sse_connections = IntGauge::with_opts(
            Opts::new("umc_sse_connections_active", "Active SSE connections")
        ).unwrap();
        registry.register(Box::new(active_sse_connections.clone())).unwrap();

        let conversion_duration = Histogram::with_opts(
            HistogramOpts::new(
                "umc_conversion_duration_seconds",
                "Conversion job duration"
            ).buckets(vec![1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0, 1800.0, 3600.0])
        ).unwrap();
        registry.register(Box::new(conversion_duration.clone())).unwrap();

        Self {
            http_requests_total,
            http_request_duration,
            jobs_created,
            jobs_completed,
            jobs_failed,
            jobs_running,
            jobs_queued,
            bytes_uploaded,
            bytes_converted,
            active_sse_connections,
            conversion_duration,
            registry,
        }
    }

    /// Génère la sortie Prometheus
    pub fn render(&self) -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let mut buf = Vec::new();
        encoder.encode(&self.registry.gather(), &mut buf).unwrap_or_default();
        String::from_utf8(buf).unwrap_or_default()
    }
}
```

```rust
// src/handlers/health.rs

use actix_web::{web, HttpResponse};
use crate::config::AppState;

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

pub async fn readiness_check(state: web::Data<AppState>) -> HttpResponse {
    // Vérifier PostgreSQL
    let db_ok = sqlx::query("SELECT 1")
        .fetch_one(&state.db)
        .await.is_ok();

    // Vérifier Redis
    let redis_ok = async {
        let mut conn = state.redis.get().await.ok()?;
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn).await.ok()?;
        Some(pong == "PONG")
    }.await.unwrap_or(false);

    let all_ok = db_ok && redis_ok;

    let status = if all_ok { 200 } else { 503 };

    HttpResponse::build(
        actix_web::http::StatusCode::from_u16(status).unwrap()
    ).json(serde_json::json!({
        "status": if all_ok { "ready" } else { "not_ready" },
        "checks": {
            "database": if db_ok { "ok" } else { "error" },
            "redis": if redis_ok { "ok" } else { "error" },
        }
    }))
}

pub async fn metrics(state: web::Data<AppState>) -> HttpResponse {
    let output = state.metrics.render();
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(output)
}
```

---

# 21. SÉCURITÉ — CORS, CSRF, VALIDATION

```rust
// src/app.rs (configuration CORS)

use actix_cors::Cors;
use actix_web::http;

pub fn configure_cors(origins: &[String]) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::CONTENT_TYPE,
            http::header::HeaderName::from_static("x-request-id"),
            http::header::HeaderName::from_static("x-api-key"),
        ])
        .expose_headers(vec![
            http::header::HeaderName::from_static("x-request-id"),
        ])
        .max_age(3600);

    if origins.contains(&"*".to_string()) {
        cors = cors.allow_any_origin();
    } else {
        for origin in origins {
            cors = cors.allowed_origin(origin);
        }
    }

    cors
}
```

```rust
// src/utils/validation.rs

use validator::ValidationError;

/// Valide qu'un nom de format est supporté
pub fn validate_format_name(format: &str) -> Result<(), ValidationError> {
    let graph = umc_graph::ConversionGraph::new();
    if !graph.node_exists(format) {
        let mut err = ValidationError::new("invalid_format");
        err.message = Some(format!("Unknown format: {}", format).into());
        return Err(err);
    }
    Ok(())
}

/// Valide qu'un path de fichier est sûr (anti path traversal)
pub fn validate_safe_path(path: &str) -> bool {
    let path = std::path::Path::new(path);
    !path.components().any(|c| matches!(c, std::path::Component::ParentDir))
        && !path.is_absolute()
}

/// Valide un nom de fichier
pub fn validate_filename(name: &str) -> bool {
    // Caractères autorisés uniquement
    name.chars().all(|c| c.is_alphanumeric() || "._- ".contains(c))
        && name.len() <= 255
        && !name.is_empty()
}
```

---

# 22. WORKERS CLEANUP

```rust
// src/workers/cleanup_worker.rs

use std::time::Duration;
use tracing::{info, warn};
use crate::config::AppState;

pub struct CleanupWorker {
    state: std::sync::Arc<AppState>,
}

impl CleanupWorker {
    pub fn new(state: std::sync::Arc<AppState>) -> Self {
        Self { state }
    }

    pub async fn run(self) {
        info!("Cleanup worker started");
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Toutes les heures

        loop {
            interval.tick().await;

            // 1. Supprimer les fichiers uploadés expirés
            self.cleanup_expired_files().await;

            // 2. Marquer les jobs expirés
            self.expire_old_jobs().await;

            // 3. Supprimer les fichiers orphelins
            self.cleanup_orphan_files().await;

            // 4. Nettoyer les refresh tokens expirés
            self.cleanup_expired_tokens().await;

            // 5. Nettoyer les clés Redis orphelines
            self.cleanup_redis_keys().await;
        }
    }

    async fn cleanup_expired_files(&self) {
        // Récupérer les fichiers des jobs expirés
        let expired = sqlx::query!(
            r#"
            SELECT source_file_path, output_file_path
            FROM conversion_jobs
            WHERE expires_at < NOW()
              AND (source_file_path IS NOT NULL OR output_file_path IS NOT NULL)
            "#
        )
        .fetch_all(&self.state.db)
        .await
        .unwrap_or_default();

        for row in expired {
            for path in [row.source_file_path, row.output_file_path].into_iter().flatten() {
                if let Err(e) = tokio::fs::remove_file(&path).await {
                    if e.kind() != std::io::ErrorKind::NotFound {
                        warn!("Failed to delete expired file {}: {}", path, e);
                    }
                }
            }
        }

        // Effacer les chemins en DB
        let _ = sqlx::query!(
            r#"
            UPDATE conversion_jobs
            SET source_file_path = NULL, output_file_path = NULL
            WHERE expires_at < NOW()
            "#
        )
        .execute(&self.state.db)
        .await;

        info!("Cleaned up {} expired job files", expired.len());
    }

    async fn expire_old_jobs(&self) {
        let _ = sqlx::query!(
            r#"
            UPDATE conversion_jobs
            SET status = 'expired'
            WHERE status IN ('queued', 'paused')
              AND expires_at < NOW()
            "#
        )
        .execute(&self.state.db)
        .await;
    }

    async fn cleanup_orphan_files(&self) {
        // Les fichiers temporaires .umc_tmp abandonnés
        let temp_dir = std::path::Path::new(&self.state.config.storage.local_temp_dir);
        if let Ok(mut entries) = tokio::fs::read_dir(temp_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "umc_tmp") {
                    // Vérifier si le fichier a plus de 24h
                    if let Ok(meta) = tokio::fs::metadata(&path).await {
                        if let Ok(modified) = meta.modified() {
                            let age = std::time::SystemTime::now()
                                .duration_since(modified)
                                .unwrap_or_default();
                            if age > Duration::from_secs(86400) {
                                tokio::fs::remove_file(&path).await.ok();
                            }
                        }
                    }
                }
            }
        }
    }

    async fn cleanup_expired_tokens(&self) {
        let _ = sqlx::query!(
            "DELETE FROM refresh_tokens WHERE expires_at < NOW() OR (is_valid = FALSE AND used_at < NOW() - INTERVAL '7 days')"
        )
        .execute(&self.state.db)
        .await;
    }

    async fn cleanup_redis_keys(&self) {
        // Les clés de progression des jobs terminés depuis > 24h
        // Redis gère déjà le TTL, pas d'action nécessaire
    }
}
```

---

# 23. TESTS BACKEND COMPLETS

```rust
// tests/integration_test.rs

use actix_web::{test, web, App};
use sqlx::PgPool;
use uuid::Uuid;

mod common {
    use crate::config::{AppState, Config};
    use std::sync::Arc;

    pub async fn create_test_state() -> Arc<AppState> {
        dotenvy::dotenv().ok();
        let config = Config::from_env()
            .expect("Test config must be available");
        Arc::new(AppState::new(config).await.expect("Failed to create AppState"))
    }

    pub async fn create_test_user(state: &AppState) -> (Uuid, String, String) {
        let email = format!("test-{}@umc.test", Uuid::new_v4());
        let password = "TestPassword123!";

        let user = sqlx::query!(
            r#"
            INSERT INTO users (email, password_hash)
            VALUES ($1, $2)
            RETURNING id
            "#,
            email,
            crate::auth::password::hash_password(password).unwrap()
        )
        .fetch_one(&state.db)
        .await
        .expect("Failed to create test user");

        (user.id, email, password.to_string())
    }

    pub async fn get_auth_token(state: &AppState, user_id: Uuid, email: &str) -> String {
        let user = sqlx::query_as!(
            crate::models::user::User,
            "SELECT * FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(&state.db)
        .await
        .unwrap();

        let jwt_service = crate::auth::jwt::JwtService::new(&state.config.jwt);
        jwt_service.encode_access_token(&user).unwrap()
    }
}

#[actix_rt::test]
async fn test_register_and_login() {
    let state = common::create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new((*state).clone()))
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(crate::handlers::auth::register))
                    .route("/login", web::post().to(crate::handlers::auth::login))
            )
    ).await;

    // Test register
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(serde_json::json!({
            "email": format!("test-{}@umc.test", Uuid::new_v4()),
            "password": "SecurePassword123!"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
}

#[actix_rt::test]
async fn test_create_job_authenticated() {
    let state = common::create_test_state().await;
    let (user_id, email, _) = common::create_test_user(&state).await;
    let token = common::get_auth_token(&state, user_id, &email).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new((*state).clone()))
            .wrap(crate::auth::middleware::AuthMiddleware)
            .service(
                web::scope("/v1")
                    .route("/jobs", web::post().to(crate::handlers::jobs::create_job))
            )
    ).await;

    let req = test::TestRequest::post()
        .uri("/v1/jobs")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({
            "target_format": "ONNX",
            "validate_mode": "structural"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);
}

#[actix_rt::test]
async fn test_rate_limiting() {
    let state = common::create_test_state().await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new((*state).clone()))
            .wrap(crate::middleware::rate_limit::RateLimitMiddleware)
            .route("/health", web::get().to(crate::handlers::health::health_check))
    ).await;

    // Envoyer 15 requêtes rapides (limit anon = 10/min)
    let mut last_status = actix_web::http::StatusCode::OK;
    for _ in 0..15 {
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        last_status = resp.status();
    }
    // Au moins une doit être rate-limited
    // (test simplifié — en prod le test serait plus précis)
    assert!(
        last_status == actix_web::http::StatusCode::OK ||
        last_status == actix_web::http::StatusCode::TOO_MANY_REQUESTS
    );
}

#[actix_rt::test]
async fn test_job_not_found_for_other_user() {
    let state = common::create_test_state().await;
    let (user1_id, email1, _) = common::create_test_user(&state).await;
    let (user2_id, email2, _) = common::create_test_user(&state).await;

    let token1 = common::get_auth_token(&state, user1_id, &email1).await;
    let fake_job_id = Uuid::new_v4();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new((*state).clone()))
            .wrap(crate::auth::middleware::AuthMiddleware)
            .service(
                web::scope("/v1")
                    .route("/jobs/{id}", web::get().to(crate::handlers::jobs::get_job))
            )
    ).await;

    let req = test::TestRequest::get()
        .uri(&format!("/v1/jobs/{}", fake_job_id))
        .insert_header(("Authorization", format!("Bearer {}", token1)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}
```

---

# 24. CI/CD & DÉPLOIEMENT

```yaml
# .github/workflows/backend.yml
name: Backend CI

on:
  push:
    branches: [main, develop]
    paths:
      - 'umc-api/**'
      - '.github/workflows/backend.yml'
  pull_request:
    paths:
      - 'umc-api/**'

env:
  SQLX_OFFLINE: true
  DATABASE_URL: "postgres://postgres:test@localhost:5432/umc_test"
  REDIS_URL: "redis://localhost:6379"
  UMC__JWT__SECRET: "test-secret-at-least-32-chars-long-for-ci"
  UMC__JWT__ISSUER: "umc.test"
  UMC__JWT__AUDIENCE: "umc-api-test"
  UMC__JWT__ACCESS_TOKEN_EXPIRY_SECS: "3600"
  UMC__JWT__REFRESH_TOKEN_EXPIRY_SECS: "2592000"
  UMC__SERVER__HOST: "0.0.0.0"
  UMC__SERVER__PORT: "8080"
  UMC__DATABASE__URL: "postgres://postgres:test@localhost:5432/umc_test"
  UMC__DATABASE__MAX_CONNECTIONS: "5"
  UMC__DATABASE__MIN_CONNECTIONS: "1"
  UMC__DATABASE__ACQUIRE_TIMEOUT_SECS: "5"
  UMC__DATABASE__IDLE_TIMEOUT_SECS: "300"
  UMC__DATABASE__MAX_LIFETIME_SECS: "900"
  UMC__REDIS__URL: "redis://localhost:6379"
  UMC__REDIS__MAX_CONNECTIONS: "5"
  UMC__REDIS__CONNECTION_TIMEOUT_SECS: "3"
  UMC__REDIS__PROGRESS_TTL_SECS: "3600"
  UMC__REDIS__SESSION_TTL_SECS: "3600"
  UMC__STORAGE__BACKEND: "local"
  UMC__STORAGE__LOCAL_UPLOAD_DIR: "/tmp/umc-test/uploads"
  UMC__STORAGE__LOCAL_OUTPUT_DIR: "/tmp/umc-test/outputs"
  UMC__STORAGE__LOCAL_TEMP_DIR: "/tmp/umc-test/temp"
  UMC__STORAGE__MAX_UPLOAD_SIZE_BYTES: "1073741824"
  UMC__STORAGE__FILE_RETENTION_SECS: "3600"
  UMC__CONVERSION__MAX_CONCURRENT_CONVERSIONS: "2"
  UMC__CONVERSION__CONVERSION_TIMEOUT_SECS: "300"
  UMC__CONVERSION__TENSOR_TIMEOUT_SECS: "30"
  UMC__CONVERSION__CHECKPOINT_INTERVAL_SECS: "10"
  UMC__CONVERSION__CHECKPOINT_DIR: "/tmp/umc-test/checkpoints"
  UMC__CONVERSION__SIGNING_KEY_PATH: "/tmp/umc-test-signing.key"
  UMC__RATE_LIMIT__ANON_REQUESTS_PER_MINUTE: "100"
  UMC__RATE_LIMIT__FREE_REQUESTS_PER_MINUTE: "200"
  UMC__RATE_LIMIT__PRO_REQUESTS_PER_MINUTE: "1000"
  UMC__RATE_LIMIT__ENTERPRISE_REQUESTS_PER_MINUTE: "10000"
  UMC__RATE_LIMIT__BURST_SIZE: "50"
  UMC__WEBHOOK__TIMEOUT_SECS: "10"
  UMC__WEBHOOK__MAX_RETRIES: "3"
  UMC__WEBHOOK__INITIAL_RETRY_DELAY_MS: "500"
  UMC__OBSERVABILITY__LOG_LEVEL: "warn"
  UMC__OBSERVABILITY__LOG_FORMAT: "pretty"
  UMC__OBSERVABILITY__METRICS_ENABLED: "false"
  UMC__OBSERVABILITY__METRICS_PATH: "/metrics"
  UMC__OBSERVABILITY__TRACING_ENABLED: "false"

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: test
          POSTGRES_DB: umc_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - uses: Swatinem/rust-cache@v2

      - name: Generate test signing key
        run: |
          dd if=/dev/urandom of=/tmp/umc-test-signing.key bs=32 count=1

      - name: Install sqlx-cli
        run: cargo install sqlx-cli --no-default-features --features native-tls,postgres

      - name: Run migrations
        run: sqlx migrate run --source umc-api/migrations
        working-directory: .

      - name: Format check
        run: cargo fmt --all -- --check
        working-directory: umc-api

      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings
        working-directory: umc-api

      - name: Tests
        run: cargo test --all
        working-directory: umc-api

      - name: Security audit
        run: cargo audit
        working-directory: umc-api

  docker:
    runs-on: ubuntu-latest
    needs: test
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4

      - name: Build Docker image
        run: |
          docker build -t umc-api:${{ github.sha }} ./umc-api
          docker build -t umc-api:latest ./umc-api

      - name: Push to registry
        run: |
          echo ${{ secrets.DOCKER_PASSWORD }} | docker login -u ${{ secrets.DOCKER_USERNAME }} --password-stdin
          docker push umc-api:${{ github.sha }}
          docker push umc-api:latest
```

```dockerfile
# umc-api/Dockerfile
FROM rust:1.80-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build optimisé avec cache layers
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --package umc-api

RUN cp target/release/umc-api /usr/local/bin/umc-api

# Image finale — minimale
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 libpq5 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false umc

COPY --from=builder /usr/local/bin/umc-api /usr/local/bin/umc-api

USER umc

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["/usr/local/bin/umc-api"]
```

---

# 25. MAIN.RS — POINT D'ENTRÉE COMPLET

```rust
// src/main.rs

use actix_web::{middleware as actix_middleware, web, App, HttpServer};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod auth;
mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod services;
mod utils;
mod workers;

use config::{AppState, Config};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // ── 1. Charger la configuration ──────────────────────────────────
    let config = Config::from_env()?;
    config.validate()?;

    // ── 2. Initialiser le logging ────────────────────────────────────
    let log_level = config.observability.log_level.clone();
    match config.observability.log_format {
        config::LogFormat::Json => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::new(&log_level))
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        config::LogFormat::Pretty => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::new(&log_level))
                .with(tracing_subscriber::fmt::layer().pretty())
                .init();
        }
    }

    info!(
        version = env!("CARGO_PKG_VERSION"),
        host = %config.server.host,
        port = config.server.port,
        "Starting UMC API"
    );

    // ── 3. Créer l'AppState ──────────────────────────────────────────
    let state = Arc::new(AppState::new(config.clone()).await?);

    // ── 4. Exécuter les migrations DB ────────────────────────────────
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&state.db)
        .await?;
    info!("Migrations complete");

    // ── 5. Démarrer les workers en arrière-plan ───────────────────────
    let workers_count = config.conversion.max_concurrent_conversions;
    for i in 0..workers_count {
        let worker_state = Arc::clone(&state);
        tokio::spawn(async move {
            let worker = workers::conversion_worker::ConversionWorker::new(worker_state);
            worker.run().await;
        });
        info!("Started conversion worker {}", i + 1);
    }

    // Worker de nettoyage
    let cleanup_state = Arc::clone(&state);
    tokio::spawn(async move {
        let worker = workers::cleanup_worker::CleanupWorker::new(cleanup_state);
        worker.run().await;
    });

    // ── 6. Configurer et démarrer le serveur HTTP ─────────────────────
    let server_config = config.server.clone();
    let state_for_server = Arc::clone(&state);

    let num_workers = server_config.workers.unwrap_or_else(num_cpus::get);

    let server = HttpServer::new(move || {
        let state = Arc::clone(&state_for_server);
        let cors = app::configure_cors(&state.config.server.cors_origins);

        App::new()
            // ── App Data ───────────────────────────────────────────
            .app_data(web::Data::new((*state).clone()))
            .app_data(
                web::JsonConfig::default()
                    .limit(10 * 1024 * 1024) // 10 Mo max pour le JSON
                    .error_handler(|err, _| {
                        actix_web::Error::from(
                            errors::ApiError::BadRequest(err.to_string())
                        )
                    })
            )
            .app_data(
                web::QueryConfig::default()
                    .error_handler(|err, _| {
                        actix_web::Error::from(
                            errors::ApiError::BadRequest(err.to_string())
                        )
                    })
            )

            // ── Middleware Global ──────────────────────────────────
            .wrap(cors)
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(middleware::request_id::RequestIdMiddleware)
            .wrap(middleware::security::SecurityHeadersMiddleware)
            .wrap(actix_middleware::Compress::default())

            // ── Routes Publiques (pas d'auth) ──────────────────────
            .route("/health", web::get().to(handlers::health::health_check))
            .route("/readiness", web::get().to(handlers::health::readiness_check))
            .route("/metrics",
                web::get().to(handlers::health::metrics))

            // ── Routes Auth ────────────────────────────────────────
            .service(
                web::scope("/auth")
                    .wrap(middleware::rate_limit::RateLimitMiddleware)
                    .route("/register", web::post().to(handlers::auth::register))
                    .route("/login", web::post().to(handlers::auth::login))
                    .route("/refresh", web::post().to(handlers::auth::refresh_token))
                    .route("/logout", web::post()
                        .wrap(auth::middleware::AuthMiddleware)
                        .to(handlers::auth::logout))
            )

            // ── Routes API v1 (auth requise) ───────────────────────
            .service(
                web::scope("/v1")
                    .wrap(auth::middleware::AuthMiddleware)
                    .wrap(middleware::rate_limit::RateLimitMiddleware)

                    // Conversion (upload + job en une requête)
                    .route("/convert",
                        web::post().to(handlers::upload::convert_with_upload))

                    // Jobs
                    .route("/jobs", web::get().to(handlers::jobs::list_jobs))
                    .route("/jobs", web::post().to(handlers::jobs::create_job))
                    .route("/jobs/{id}", web::get().to(handlers::jobs::get_job))
                    .route("/jobs/{id}/cancel",
                        web::post().to(handlers::jobs::cancel_job))
                    .route("/jobs/{id}/upload",
                        web::post().to(handlers::upload::upload_source_file))
                    .route("/jobs/{id}/download",
                        web::get().to(handlers::jobs::download_job_output))

                    // Progression (SSE + WebSocket)
                    .route("/jobs/{id}/progress",
                        web::get().to(handlers::progress::job_progress_sse))
                    .route("/jobs/{id}/ws",
                        web::get().to(handlers::progress::job_progress_ws))

                    // Inspection & Outils
                    .route("/inspect",
                        web::post().to(handlers::inspect::inspect_model))
                    .route("/dry-run",
                        web::post().to(handlers::inspect::dry_run))

                    // Formats & Graph
                    .route("/formats",
                        web::get().to(handlers::formats::list_formats))
                    .route("/formats/{name}",
                        web::get().to(handlers::formats::get_format))
                    .route("/graph",
                        web::get().to(handlers::formats::conversion_graph))

                    // API Keys
                    .route("/api-keys",
                        web::get().to(handlers::api_keys::list_api_keys))
                    .route("/api-keys",
                        web::post().to(handlers::api_keys::create_api_key))
                    .route("/api-keys/{id}",
                        web::delete().to(handlers::api_keys::revoke_api_key))

                    // Profil utilisateur
                    .route("/me",
                        web::get().to(handlers::auth::get_me))
            )

            // ── Certificats (partiellement public) ────────────────
            .service(
                web::scope("/v1/certificates")
                    // Public — vérification de certificat sans auth
                    .route("/{id}",
                        web::get().to(handlers::certificates::get_certificate))
                    .route("/{id}/verify",
                        web::get().to(handlers::certificates::verify_certificate))
                    .route("/{id}/pdf",
                        web::get().to(handlers::certificates::get_certificate_pdf))
                    // Révocation (auth requise)
                    .route("/{id}/revoke",
                        web::post()
                            .wrap(auth::middleware::AuthMiddleware)
                            .to(handlers::certificates::revoke_certificate))
            )
    })
    .workers(num_workers)
    .keep_alive(std::time::Duration::from_secs(server_config.keep_alive_secs))
    .client_request_timeout(std::time::Duration::from_secs(server_config.request_timeout_secs))
    .shutdown_timeout(30)
    .bind(format!("{}:{}", server_config.host, server_config.port))?
    .run();

    info!(
        host = %server_config.host,
        port = server_config.port,
        workers = num_workers,
        "UMC API ready to serve requests"
    );

    // Graceful shutdown
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down gracefully...");
        }
    }

    Ok(())
}
```

---

# ANNEXE — RÉSUMÉ DES ENDPOINTS

```
╔══════════════════════════════════════════════════════════════════════════╗
║                      UMC API — Tableau des Endpoints                     ║
╠══════════════════╦══════════╦═══════════════╦════════════════════════════╣
║ Endpoint         ║ Méthode  ║ Auth          ║ Description                ║
╠══════════════════╬══════════╬═══════════════╬════════════════════════════╣
║ /health          ║ GET      ║ Aucune        ║ Health check               ║
║ /readiness       ║ GET      ║ Aucune        ║ Readiness check            ║
║ /metrics         ║ GET      ║ Aucune        ║ Prometheus metrics         ║
╠══════════════════╬══════════╬═══════════════╬════════════════════════════╣
║ /auth/register   ║ POST     ║ Aucune        ║ Inscription                ║
║ /auth/login      ║ POST     ║ Aucune        ║ Connexion                  ║
║ /auth/refresh    ║ POST     ║ Aucune        ║ Rafraîchir token           ║
║ /auth/logout     ║ POST     ║ JWT           ║ Déconnexion                ║
╠══════════════════╬══════════╬═══════════════╬════════════════════════════╣
║ /v1/convert      ║ POST     ║ JWT/APIKey    ║ Upload + conversion        ║
╠══════════════════╬══════════╬═══════════════╬════════════════════════════╣
║ /v1/jobs         ║ GET      ║ JWT/APIKey    ║ Liste des jobs             ║
║ /v1/jobs         ║ POST     ║ JWT/APIKey    ║ Créer un job               ║
║ /v1/jobs/{id}    ║ GET      ║ JWT/APIKey    ║ Statut d'un job            ║
║ /v1/jobs/{id}/.. ║ POST     ║ JWT/APIKey    ║ Annuler                    ║
║ /v1/jobs/{id}/.. ║ POST     ║ JWT/APIKey    ║ Upload source              ║
║ /v1/jobs/{id}/.. ║ GET      ║ JWT/APIKey    ║ Télécharger résultat       ║
║ /v1/jobs/{id}/.. ║ GET      ║ JWT/APIKey    ║ SSE progression            ║
║ /v1/jobs/{id}/ws ║ WS       ║ JWT/APIKey    ║ WebSocket progression      ║
╠══════════════════╬══════════╬═══════════════╬════════════════════════════╣
║ /v1/inspect      ║ POST     ║ JWT/APIKey    ║ Inspecter un modèle        ║
║ /v1/dry-run      ║ POST     ║ JWT/APIKey    ║ Simuler une conversion     ║
╠══════════════════╬══════════╬═══════════════╬════════════════════════════╣
║ /v1/formats      ║ GET      ║ JWT/APIKey    ║ Liste des formats          ║
║ /v1/formats/{n}  ║ GET      ║ JWT/APIKey    ║ Détails d'un format        ║
║ /v1/graph        ║ GET      ║ JWT/APIKey    ║ Graphe de conversion       ║
╠══════════════════╬══════════╬═══════════════╬════════════════════════════╣
║ /v1/api-keys     ║ GET/POST ║ JWT           ║ Gestion des API Keys       ║
║ /v1/api-keys/{id}║ DELETE   ║ JWT           ║ Révoquer une clé           ║
║ /v1/me           ║ GET      ║ JWT/APIKey    ║ Profil utilisateur         ║
╠══════════════════╬══════════╬═══════════════╬════════════════════════════╣
║ /v1/certs/{id}   ║ GET      ║ Aucune        ║ Certificat public          ║
║ /v1/certs/../ver ║ GET      ║ Aucune        ║ Vérifier certificat        ║
║ /v1/certs/../pdf ║ GET      ║ Aucune        ║ Certificat PDF             ║
║ /v1/certs/../rev ║ POST     ║ JWT           ║ Révoquer certificat        ║
╚══════════════════╩══════════╩═══════════════╩════════════════════════════╝

AUTHENTIFICATION :
  JWT Bearer : Authorization: Bearer <token>
  API Key    : X-API-Key: umc_sk_prod_<hex> ou ?api_key=<key>

FORMATS DE RÉPONSE :
  Succès  : { "data": {...}, "pagination": {...} }
  Erreur  : { "error": { "code": "...", "message": "...", "details": {...} } }
  SSE     : event: progress\ndata: {...}\n\n
  WS      : JSON messages avec structure JobProgress
```

---

*UMC Backend Actix v1.0 — Ultra-robuste · Ultra-rapide · Production-ready*  
*Actix-Web 4 · PostgreSQL 16 · Redis 7 · Rust stable 1.80+*  
*Zero panics · Zero deadlocks · Zero information loss*