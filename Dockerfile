# ─── UMC API — Multi-stage Docker Build ────────────────────────────
# Build:  docker build -t umc-api .
# Run:    docker run -p 8080:8080 --env-file umc-api/.env umc-api

# ── Stage 1: Build ──────────────────────────────────────────────────
FROM rust:1.80-bookworm AS builder

WORKDIR /app

# Copy workspace manifests first (cache dependencies)
COPY Cargo.toml Cargo.lock ./
COPY crates/umc-core/Cargo.toml crates/umc-core/Cargo.toml
COPY crates/umc-detect/Cargo.toml crates/umc-detect/Cargo.toml
COPY crates/umc-graph/Cargo.toml crates/umc-graph/Cargo.toml
COPY crates/umc-pipeline/Cargo.toml crates/umc-pipeline/Cargo.toml
COPY crates/umc-validate/Cargo.toml crates/umc-validate/Cargo.toml
COPY crates/umc-formats/Cargo.toml crates/umc-formats/Cargo.toml
COPY crates/umc-cli/Cargo.toml crates/umc-cli/Cargo.toml
COPY crates/umc-tests/Cargo.toml crates/umc-tests/Cargo.toml
COPY umc-api/Cargo.toml umc-api/Cargo.toml

# Create dummy src files so cargo can resolve the workspace
RUN mkdir -p crates/umc-core/src && echo "fn main(){}" > crates/umc-core/src/lib.rs \
    && mkdir -p crates/umc-detect/src && echo "fn main(){}" > crates/umc-detect/src/lib.rs \
    && mkdir -p crates/umc-graph/src && echo "fn main(){}" > crates/umc-graph/src/lib.rs \
    && mkdir -p crates/umc-pipeline/src && echo "fn main(){}" > crates/umc-pipeline/src/lib.rs \
    && mkdir -p crates/umc-validate/src && echo "fn main(){}" > crates/umc-validate/src/lib.rs \
    && mkdir -p crates/umc-formats/src && echo "fn main(){}" > crates/umc-formats/src/lib.rs \
    && mkdir -p crates/umc-cli/src && echo "fn main(){}" > crates/umc-cli/src/main.rs \
    && mkdir -p crates/umc-tests/src && echo "fn main(){}" > crates/umc-tests/src/lib.rs \
    && mkdir -p umc-api/src && echo "fn main(){}" > umc-api/src/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release -p umc-api 2>/dev/null || true

# Copy actual source code
COPY crates/ crates/
COPY umc-api/ umc-api/

# Touch source files to invalidate cache
RUN find crates umc-api -name "*.rs" -exec touch {} +

# Build the API binary
RUN cargo build --release -p umc-api

# ── Stage 2: Runtime ────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash umc

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/umc-api /app/umc-api

# Copy migrations
COPY --from=builder /app/umc-api/migrations/ /app/migrations/

RUN mkdir -p /tmp/umc/uploads /tmp/umc/outputs \
    && chown -R umc:umc /app /tmp/umc

USER umc

# Default env vars (override with --env-file or -e)
ENV UMC_HOST=0.0.0.0
ENV UMC_PORT=8080
ENV RUST_LOG=info
ENV UPLOAD_DIR=/tmp/umc/uploads
ENV OUTPUT_DIR=/tmp/umc/outputs

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["/app/umc-api"]