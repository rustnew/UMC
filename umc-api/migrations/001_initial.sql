-- UMC API — Initial Schema
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ── ENUMS ────────────────────────────────────────────────────────────────────
CREATE TYPE user_plan AS ENUM ('free', 'pro', 'enterprise');
CREATE TYPE job_status AS ENUM ('queued','running','done','failed','cancelled');

-- ── USERS ────────────────────────────────────────────────────────────────────
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email           TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    display_name    TEXT,
    plan            user_plan NOT NULL DEFAULT 'free',
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at   TIMESTAMPTZ
);

CREATE INDEX idx_users_email ON users(email);

-- ── REFRESH TOKENS ───────────────────────────────────────────────────────────
CREATE TABLE refresh_tokens (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    expires_at  TIMESTAMPTZ NOT NULL,
    revoked     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);

-- ── API KEYS ─────────────────────────────────────────────────────────────────
CREATE TABLE api_keys (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash    TEXT NOT NULL UNIQUE,
    key_prefix  TEXT NOT NULL,
    name        TEXT NOT NULL,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    last_used_at TIMESTAMPTZ,
    expires_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_api_keys_user ON api_keys(user_id);
CREATE INDEX idx_api_keys_hash ON api_keys(key_hash);

-- ── CONVERSION JOBS ───────────────────────────────────────────────────────────
CREATE TABLE conversion_jobs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    source_format   TEXT NOT NULL,
    target_format   TEXT NOT NULL,
    validate_mode   TEXT NOT NULL DEFAULT 'structural',
    generate_cert   BOOLEAN NOT NULL DEFAULT FALSE,
    extra_options   JSONB NOT NULL DEFAULT '{}',

    source_file_path    TEXT,
    source_file_size    BIGINT,
    source_file_hash    TEXT,
    output_file_path    TEXT,
    output_file_size    BIGINT,
    output_file_hash    TEXT,

    status          job_status NOT NULL DEFAULT 'queued',
    progress        FLOAT4 NOT NULL DEFAULT 0.0,
    tensors_done    BIGINT NOT NULL DEFAULT 0,
    tensors_total   BIGINT,
    last_tensor     TEXT,
    error_message   TEXT,
    warnings        JSONB,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at      TIMESTAMPTZ,
    finished_at     TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '7 days'
);

CREATE INDEX idx_jobs_user ON conversion_jobs(user_id);
CREATE INDEX idx_jobs_status ON conversion_jobs(status);
CREATE INDEX idx_jobs_queued ON conversion_jobs(created_at) WHERE status = 'queued';
