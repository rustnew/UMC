# Changelog

All notable changes to **UMC** — The Universal Model Converter are documented
in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0] — 2026-08

### Added

- Desktop app (`umc-desktop`): native egui/eframe UI with drag-and-drop,
  automatic format detection, progress bar, cancellation, persistent
  history and settings.
- Linux packaging (`packaging/install.sh` + `.desktop` entry).
- End-to-end tests for the conversion worker (GGUF → SafeTensors).

### Changed

- UMC is now a local desktop tool: removed the web frontend
  (`umc-frontend`) and the web UI.
- Reworked README, cleaned up documentation, removed obsolete files.
- Unified all workspace crates to version 1.0.0.

---

## [0.1.0] — 2026-05

### Added

- Rust workspace: `umc-core`, `umc-detect`, `umc-graph`, `umc-pipeline`,
  `umc-validate`, `umc-formats`, `umc-cli`, `umc-tests`.
- Universal Intermediate Representation (IR): TensorStore, GraphStore,
  ExtensionStore, ProvenanceChain.
- Format detection: 13 detectors (magic bytes, extension, content
  analysis) — GGUF, GGML, SafeTensors, TFLite, HDF5, ONNX, PyTorch,
  SentencePiece, AWQ, GPTQ...
- Conversion pipeline: reader / transformer / writer, mmap, rayon
  parallelism.
- Structural and numeric validation (bit-identical F32 round-trip).
- CLI: `convert`, `inspect`, `formats`, `path`.
- REST API (`umc-api`, Actix-Web + SQLx/Postgres): async jobs,
  SSE progress, certificates.
- Docker Compose (db + api), startup scripts, GitHub Actions CI.