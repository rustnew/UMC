# UMC Organization Profile

This file is displayed on the GitHub organization profile page.

---

## 🔄 UMC

**The ffmpeg of AI models.**

Convert any model format to any other format. Without quality loss. At maximum speed. With mathematical proof.

### What We Do

UMC solves the most critical interoperability challenge in production AI: **format fragmentation**.

- 🔀 Convert between 31 model formats (GGUF, ONNX, SafeTensors, CoreML, TensorRT...)
- 🧮 961 possible conversion paths, all covered by one binary
- ✅ Mathematically proven zero information loss (SHA256 round-trip)
- ⚡ 135x faster than competing tools
- 🔐 Signed conversion certificates (ed25519)

### Key Features

- **31 formats** - GGUF, ONNX, SafeTensors, PyTorch, TensorRT, CoreML, TFLite...
- **Universal IR** - A mathematical superset of all 31 formats
- **Dijkstra path finding** - Automatic optimal conversion routing
- **4-level validation** - Structural, numeric, functional, round-trip
- **Zero information loss** - ExtensionStore guarantees bit-identical round-trips
- **400 GB+ models** - mmap + zero-copy, constant RAM usage
- **Open source** - Apache 2.0 license

### Quick Links

- 💻 **GitHub:** [github.com/rustnew/Universal_Model_Convert](https://github.com/rustnew/Universal_Model_Convert)

### Tech Stack

| Component | Technology |
|-----------|------------|
| Core Engine | Rust |
| API | Actix-Web + SQLx (Postgres) |
| Frontend | TanStack Start + React 19 + Vite |
| Validation | SIMD (AVX2/NEON) + ed25519 certificates |

### Get Started

```bash
# Clone and build
git clone https://github.com/rustnew/Universal_Model_Convert.git
cd Universal_Model_Convert
cargo build --workspace

# Convert a model
cargo run -p umc-cli -- convert model.gguf model.onnx
```

### Sponsorship

UMC is open source and community-driven. Support development:

[![GitHub Sponsors](https://img.shields.io/badge/GitHub-Sponsors-blue)](https://github.com/sponsors/rustnew)

### Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

[![Good First Issues](https://img.shields.io/badge/GitHub-Good%20First%20Issues-green)](https://github.com/rustnew/Universal_Model_Convert/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)

---

**Built by [Martial-Christian Fossouo](https://github.com/rustnew)**

[GitHub](https://github.com/rustnew/Universal_Model_Convert)