# Changelog

All notable changes to **UMC** — The Universal Model Converter are documented
in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- Desktop app (`umc-desktop`) : interface native egui/eframe avec
  glisser-déposer, détection automatique du format, barre de progression,
  annulation, historique persistant et réglages.
- Linux packaging (`packaging/install.sh` + entrée `.desktop`).

### Changed

- UMC devient un outil de bureau local : suppression du frontend web
  (`umc-frontend`) et de l'interface web.
- README refondu, documentation nettoyée.

---

## [0.1.0] — 2026-05

### Added

- Workspace Rust : `umc-core`, `umc-detect`, `umc-graph`, `umc-pipeline`,
  `umc-validate`, `umc-formats`, `umc-cli`, `umc-tests`.
- Universal Intermediate Representation (IR) : TensorStore, GraphStore,
  ExtensionStore, ProvenanceChain.
- Format detection : 13 détecteurs (magic bytes, extension, analyse de
  contenu) — GGUF, GGML, SafeTensors, TFLite, HDF5, ONNX, PyTorch,
  SentencePiece, AWQ, GPTQ...
- Conversion pipeline : reader / transformer / writer, mmap, parallélisme
  rayon.
- Validation structurelle et numérique (round-trip F32 bit-identique).
- CLI : `convert`, `inspect`, `formats`, `path`.
- REST API (`umc-api`, Actix-Web + SQLx/Postgres) : jobs asynchrones,
  progression SSE, certificats.
- Docker Compose (db + api), scripts de démarrage, CI GitHub Actions.