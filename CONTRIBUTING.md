# Contributing to UMC

First off, thanks for taking the time to contribute! 🔄

UMC is the Universal Model Converter — the ffmpeg of AI models. It converts
between 31 model formats without quality loss, at maximum speed, with
mathematical proof.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Project Layout](#project-layout)
- [Prerequisites](#prerequisites)
- [Development Workflow](#development-workflow)
- [Running Tests](#running-tests)
- [Adding a New Format](#adding-a-new-format)
- [The Web UI](#the-web-ui)
- [The API](#the-api)
- [Commit Conventions](#commit-conventions)
- [Publishing](#publishing)

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md) in all
interactions.

## Project Layout

UMC is a Cargo workspace plus a REST API and a desktop app:

| Directory | Description |
|-----------|-------------|
| `crates/umc-core` | Universal IR, TensorStore, GraphStore, ExtensionStore |
| `crates/umc-detect` | Format detection & registry |
| `crates/umc-graph` | Conversion graph + Dijkstra path finding |
| `crates/umc-pipeline` | Reader/Transformer/Writer pipeline, mmap, parallelism |
| `crates/umc-validate` | 4-level validation (structural, numeric, functional, certificate) |
| `crates/umc-formats` | 31 format loaders/savers |
| `crates/umc-cli` | CLI: `convert`, `inspect`, `dry-run`, `diff`, `doctor`... |
| `crates/umc-tests` | Integration & round-trip test suite |
| `umc-api` | REST API (Actix-Web + SQLx/Postgres) |
| `umc-desktop` | Desktop app (egui/eframe, cross-platform) |

## Prerequisites

- **Rust** (stable, edition 2021, rust-version 1.80+)
- **PostgreSQL** (for `umc-api` — optional for CLI/desktop development)

## Development Workflow

1. **Fork** the repository and create your branch from `main`:

   ```bash
   git checkout -b feat/my-change
   ```

2. **Make your changes** with tests.

3. **Run checks locally** (see below).

4. **Open a Pull Request** with a clear description of what and why.

## Running Tests

```bash
# Rust workspace (all crates)
cargo build --workspace
cargo test --workspace

# Just one crate
cargo test -p umc-core
cargo test -p umc-formats

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

## Adding a New Format

Adding a format takes roughly **200-500 lines of code** and immediately
enables it in all conversion paths via the Dijkstra graph.

1. Create `crates/umc-formats/src/my_format/` with `mod.rs`, `loader.rs`
   and/or `saver.rs`.
2. Implement the `FormatLoader` and/or `FormatSaver` traits from `umc-core`.
3. Register the format in `FormatRegistry` and `ConversionGraph`.
4. Add tests — **a round-trip test is required** (A → B → A must be
   bit-identical).
5. Update the format table in `README.md`.
6. Submit the PR.

## The Desktop App

```bash
cargo run -p umc-desktop
```

## The API

```bash
cd umc-api
cp .env.example .env   # configure DATABASE_URL, JWT_SECRET...
cargo run -p umc-api
```

The API listens on port `8085` by default. See `umc-api/.env` for options.

## Commit Conventions

Use conventional commit prefixes:

- `feat:` — new feature
- `fix:` — bug fix
- `docs:` — documentation only
- `chore:` — maintenance (bumps, metadata, tooling)
- `refactor:` — code change that neither fixes a bug nor adds a feature
- `test:` — adding or updating tests
- `perf:` — performance improvement

Example: `feat(umc-formats): add MyFormat loader + round-trip tests`

## Publishing

Releases are versioned with SemVer and tagged `vX.Y.Z`. See
[docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) for the full release and deployment process.