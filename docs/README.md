# UMC Documentation

Welcome to the UMC documentation. UMC is the **Universal Model Converter** —
the ffmpeg of AI models.

## Getting Started

- [README](../README.md) — Overview, features, quick start
- [CONTRIBUTING](../CONTRIBUTING.md) — How to contribute
- [CHANGELOG](../CHANGELOG.md) — Release history
- [ROADMAP](../ROADMAP.md) — Development plan for upcoming versions

## Repository Layout

| Directory | Description |
|-----------|-------------|
| `crates/umc-core` | Universal IR, tensor/graph/extension stores, traits |
| `crates/umc-detect` | Format detection (magic bytes, extension, content) |
| `crates/umc-graph` | Conversion graph & path finding |
| `crates/umc-pipeline` | Reader/Transformer/Writer pipeline, mmap, parallelism |
| `crates/umc-validate` | Structural, numeric validation & certificates |
| `crates/umc-formats` | Format loaders/savers |
| `crates/umc-cli` | CLI binary |
| `crates/umc-tests` | Integration & round-trip test suite |
| `umc-api` | REST API (Actix-Web + SQLx/Postgres) |
| `umc-desktop` | Desktop app (egui/eframe, cross-platform) |
| `packaging/` | Linux packaging (install script, .desktop entry) |

## Development

```bash
# Build the whole workspace
cargo build --workspace

# Run all tests
cargo test --workspace

# Lint & format
cargo clippy --workspace --all-targets
cargo fmt --check

# Run the CLI
cargo run -p umc-cli -- --help

# Run the desktop app
cargo run -p umc-desktop

# Run the API (needs PostgreSQL)
cargo run -p umc-api
```

## Docker

```bash
# Backend stack (db + api)
docker compose up -d --build

# Health check
./healthcheck.sh
```

> Note: UMC is now a local desktop tool. The user interface is the native
> `umc-desktop` app (egui/eframe) — there is no web frontend anymore.
> Docker is only needed for the API backend if you use it.

## API

The REST API is served by `umc-api` (Actix-Web).

- `GET /health` — liveness probe
- `GET /ready` — readiness probe
- `POST /v1/convert` — start an async conversion
- `GET /v1/jobs/:id` — job status and progress