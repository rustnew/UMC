# UMC Documentation

Welcome to the UMC documentation. UMC is the **Universal Model Converter** —
the ffmpeg of AI models.

## Getting Started

- [README](../README.md) — Overview, features, quick start
- [LAUNCH](../LAUNCH.md) — How to launch the project
- [Installation](../README.md#installation) — Install UMC

## Project Documentation

The following documents describe the project in depth:

| Document | Description |
|----------|-------------|
| [design.md](../design.md) | Architecture and design decisions |
| [backend.md](../backend.md) | Backend implementation details |
| [implemente.md](../implemente.md) | Implementation guide |
| [probleme.md](../probleme.md) | Known problems and solutions |
| [regles.md](../regles.md) | Project rules and conventions |

## Repository

- [Contributing](../CONTRIBUTING.md) — How to contribute
- [Changelog](../CHANGELOG.md) — Release history
- [Security](../SECURITY.md) — Security policy
- [Code of Conduct](../CODE_OF_CONDUCT.md) — Community standards
- [License](../LICENSE) — Apache 2.0

## Development

```bash
# Build the whole workspace
cargo build --workspace

# Run all tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Start the API
cargo run -p umc-api

# Start the frontend
cd umc-frontend && bun run dev
```

## Docker

```bash
# Full stack (db + api + ui)
docker compose up -d --build

# Health check
./healthcheck.sh
```

## API

The REST API is served by `umc-api` (Actix-Web). See
[API Reference](../README.md#api-reference) in the README.

- `GET /health` — liveness probe
- `GET /ready` — readiness probe
- `POST /v1/convert` — start an async conversion
- `GET /v1/jobs/:id` — job status and progress