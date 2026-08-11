# UMC Organization Profile

This file is displayed on the GitHub organization profile page.

---

## 🔄 UMC

**The ffmpeg of AI models.**

Convert AI models between formats. Fast. Lossless. Verifiable.

### What We Do

UMC solves the most critical interoperability challenge in production AI: **format fragmentation**.

- 🔀 Convert between model formats (GGUF, SafeTensors, ONNX, PyTorch, TFLite...)
- 🧭 Automatic format detection (magic bytes, extension, content analysis)
- ✅ Verifiable conversions (structural + numeric validation)
- ⚡ Native Rust — no Python, no GIL, no runtime overhead
- 🖥️ Desktop app + CLI + REST API

### Key Features

- **Universal IR** - A single intermediate representation; converting A→B is always `load(A) → IR → save(B)`
- **Automatic detection** - 13 detectors using magic bytes, headers and content analysis
- **Conversion path finding** - `umc path` computes the optimal multi-hop route
- **Verifiable conversions** - structural and numeric validation, F32 round-trips bit-identical
- **Memory-mapped I/O** - models are streamed, never fully loaded in RAM
- **Parallel pipeline** - reader / transformer / writer with rayon data-parallel tensors
- **Desktop app** - native cross-platform GUI (egui/eframe) for local drag-and-drop conversions
- **Open source** - Apache 2.0 license

### Quick Links

- 💻 **GitHub:** [github.com/rustnew/UMC](https://github.com/rustnew/UMC)

### Tech Stack

| Component | Technology |
|-----------|------------|
| Core Engine | Rust |
| Desktop App | egui / eframe |
| API | Actix-Web + SQLx (Postgres) |
| Validation | Structural + numeric, ed25519 certificates |

### Get Started

```bash
# Clone and build
git clone https://github.com/rustnew/UMC.git
cd UMC
cargo build --workspace

# Convert a model (CLI)
cargo run -p umc-cli -- convert model.gguf model.onnx

# Desktop app
cargo run -p umc-desktop
```

### Sponsorship

UMC is open source and community-driven. Support development:

[![GitHub Sponsors](https://img.shields.io/badge/GitHub-Sponsors-blue)](https://github.com/sponsors/rustnew)

### Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

[![Good First Issues](https://img.shields.io/badge/GitHub-Good%20First%20Issues-green)](https://github.com/rustnew/UMC/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)

---

**Built by [Martial-Christian Fossouo](https://github.com/rustnew)**

[GitHub](https://github.com/rustnew/UMC)