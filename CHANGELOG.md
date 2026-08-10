# Changelog

All notable changes to **UMC** — The Universal Model Converter are documented
in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- Repository customization: Apache-2.0 `LICENSE`, CI/CD workflows
  (`.github/workflows/`), issue/PR templates, community docs
  (`CONTRIBUTING.md`, `CHANGELOG.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`),
  Docker deployment files, and `.zenodo.json` citation metadata.
- Overhauled `README.md` with badges, Mermaid architecture diagrams and
  corrected repository URLs.

---

## [1.0.0] — 2026-06

### Added

- Universal Intermediate Representation (IR): TensorStore, GraphStore,
  QuantStore, AdapterStore, ExtensionStore, TokenizerStore, ProvenanceChain.
- 31 format loaders/savers across 3 tiers (GGUF, ONNX, SafeTensors, PyTorch,
  TensorRT, CoreML, TFLite, AWQ, GPTQ, ExecuTorch, ...).
- Dijkstra-based automatic conversion path finding.
- 4-level validation: structural, numeric, functional, round-trip certificate
  (ed25519 signed).
- Zero information loss guarantee via ExtensionStore (SHA256 bit-identical
  round-trips).
- Quantization support: GGUF Q2K-Q8, AWQ, GPTQ, NF4/FP4, FP8, INT8.
- Adapter support: LoRA, QLoRA, PEFT (merge or keep separate).
- Large model support (400 GB+) via mmap + zero-copy pipeline.
- CLI: `convert`, `inspect`, `dry-run`, `diff`, `doctor`, `benchmark`,
  `watch`, `lineage`, `formats`.
- REST API (Actix-Web + SQLx/Postgres): async conversions, jobs, certificates.
- Web dashboard (TanStack Start + React 19 + Vite).
- Plugin system for custom formats in any language.

---

## [0.1.0] — 2026-05

### Added

- Initial workspace scaffolding and core IR types.
- GGUF + ONNX + SafeTensors loaders/savers.
- Conversion pipeline with mmap and rayon parallelism.
- Round-trip test harness.