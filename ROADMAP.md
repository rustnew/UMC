# UMC Roadmap

**Universal Model Converter** — the ffmpeg of AI models.

This document describes the planned development for the next versions of UMC.
It is a living document: priorities are reviewed with each release.

Legend:

- ✅ done
- 🚧 in progress
- 📋 planned
- 💡 idea (not yet scheduled)

---

## Current state (v1.0.0)

| Feature | Status |
|---|---|
| Universal Intermediate Representation (IR) | ✅ |
| Format detection (magic bytes, extension, content) | ✅ |
| GGUF loader | ✅ |
| SafeTensors loader / saver | ✅ |
| SentencePiece loader | ✅ |
| ONNX loader / saver | 🚧 (opset 13–21) |
| PyTorch loader / saver | 🚧 |
| TFLite loader / saver | 🚧 |
| AWQ / GPTQ / LoRA support | 📋 |
| Conversion pipeline (mmap, rayon, cancel) | ✅ |
| Structural & numeric validation, certificates | ✅ |
| CLI (`convert`, `inspect`, `formats`, `path`) | ✅ |
| Desktop app (egui/eframe) | ✅ |
| REST API (`umc-api`, Actix-Web) | ✅ |

---

## v1.1 — Hardening & format completeness

Target: reliability of the core before adding surface area.

### Formats

- 📋 Finish ONNX loader/saver: full opset 13–21 tensor support, batch
  dimensions, external data files (`.onnx.data`).
- 📋 Finish PyTorch saver: quantized tensors, non-contiguous storages,
  proper `stride` handling on load.
- 📋 Finish TFLite saver: quantized (int8/uint8) tensors, buffer dedup.
- 📋 GGUF saver (currently loader only).
- 📋 Quantization support in the pipeline: `--q8`, `--q4` (GGUF-style
  block quant) as a first-class IR transformation.

### Reliability

- 📋 Fuzz harness for every loader (cargo-fuzz) — malformed files must
  never panic or OOM.
- 📋 Property-based tests for round-trips across dtypes and shapes.
- 📋 Memory budget: cap peak RSS during huge-model conversions, add
  streaming path for tensors > 2 GiB.
- 📋 Error taxonomy: machine-readable error codes in CLI/API.

### UX

- 📋 Desktop: batch conversion queue (multiple files in, multiple out).
- 📋 Desktop: per-format options panel (quantization, dtype cast).
- 💡 Desktop: conversion profiles (presets).

---

## v1.2 — Quantization & compression

Target: make UMC the reference tool for shrinking models losslessly.

- 📋 Block-quantization library (Q4_0, Q4_K, Q5_K, Q6_K, Q8_0) matching
  llama.cpp bit layouts exactly, verified against ggml.
- 📋 Dtype cast with round-trip certificates (F32 → F16 → F32
  bit-identical checks).
- 📋 LoRA merge (adapter + base → full weights) in the pipeline.
- 📋 AWQ / GPTQ dequantize → requantize flows.
- 💡 Weight clustering / pruning passes (experimental, opt-in).

---

## v1.3 — More formats

Target: broad coverage of the AI ecosystem.

- 📋 Keras H5 (read-only) via native HDF5 parser.
- 📋 GGML legacy (read-only).
- 📋 CoreML (export), TensorRT engine (via plugin), OpenVINO (export).
- 📋 MLIR / StableHLO import (research-grade).
- 💡 vLLM / tensor-parallel checkpoint sharding (split & merge).

---

## v2.0 — Ecosystem & automation

Target: embeddable, scriptable, verified at scale.

- 📋 `umc-core` as a stable public API (`libumc`) with docs.rs coverage.
- 📋 Library API for embeddings: `UMCPlugin` trait, metadata schemas.
- 📋 WASM target for the detection + core pipeline (browser demos).
- 📋 API: webhook notifications, job priority, quotas.
- 💡 Model cards: read/write HuggingFace model card metadata.
- 💡 Diff/summary of two models (`umc diff a.bin b.bin`) — structural +
  numeric report.

---

## v2.1 — Speed

Target: "ffmpeg-fast" claim.

- 📋 Threaded loader overlap (read-ahead + parse on multiple threads).
- 📋 SIMD kernels for dtype conversion (AVX2/NEON via `std::simd`).
- 💡 GPU-accelerated conversion (WGSL/WebGPU backend for quant ops).
- 💡 Incremental re-conversion cache keyed by content hash.

---

## Non-goals

- Training or fine-tuning models.
- Runtime/inference engines (UMC converts, it does not run models).
- Web frontend (desktop + CLI are the interfaces; API serves
  automation).

---

## How to contribute

See [CONTRIBUTING.md](CONTRIBUTING.md). Roadmap items are tracked as
GitHub issues tagged with their target milestone.
