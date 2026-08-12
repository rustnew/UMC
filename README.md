<div align="center">

# UMC

**Universal Model Converter — convert AI models between formats. Fast. Lossless. Verifiable.**

[![CI](https://github.com/rustnew/UMC/actions/workflows/ci.yml/badge.svg)](https://github.com/rustnew/UMC/actions)
[![Release](https://img.shields.io/github/v/release/rustnew/UMC)](https://github.com/rustnew/UMC/releases)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80-orange.svg?logo=rust)](https://www.rust-lang.org/)

</div>

## The problem

Production AI is fragmented. GGUF for llama.cpp, SafeTensors for HuggingFace, ONNX for
inference engines, PyTorch for training — every tool speaks its own format. Converting a model
means juggling incompatible Python scripts, with silent quality loss and **no way to prove the
result is correct**.

## The vision

UMC is **the ffmpeg of AI models**: one tool that converts any model to any format, losslessly,
in a verifiable way — without Python, without a server, without guesswork.

The key idea is a **Universal Intermediate Representation**. Converting A → B is always
`load(A) → IR → save(B)`, never a fragile A → B converter. Formats are detected automatically
(magic bytes, extension, content), conversions are validated structurally and numerically, and
F32 round-trips are bit-identical. It is written in Rust: fast, memory-safe, single binary.

## Install

### Linux — binary (recommended)

```bash
curl -LO https://github.com/rustnew/UMC/releases/download/v1.0.0/umc-1.0.0-linux-x86_64.tar.gz
tar xzf umc-1.0.0-linux-x86_64.tar.gz
./umc-1.0.0-linux-x86_64/install.sh
```

Then launch **UMC** from your applications menu, or run `umc-desktop`.

### From source

```bash
git clone https://github.com/rustnew/UMC.git
cd UMC
cargo run -p umc-desktop    # desktop app
cargo run -p umc-cli -- --help   # CLI
```

## Usage

```bash
umc convert model.gguf model.onnx              # auto-detect formats
umc convert model.bin out.safetensors --from gguf --to safetensors
umc convert model.gguf model.onnx --dtype f16  # cast dtype
umc convert model.gguf model.onnx --validate strict
umc inspect model.gguf                         # metadata
umc formats                                    # supported formats
```

## Desktop app

Drag & drop, format auto-detection, progress + cancel, history, settings.

| Screen | Purpose |
|--------|---------|
| Convert | Drop a model, pick target, convert |
| History | Past conversions (persisted) |
| Formats | Supported formats catalogue |
| Settings | Theme, threads, validation |

## Formats

| Format | Load | Save |
|--------|:----:|:----:|
| GGUF | ✅ | ✅ |
| SafeTensors | ✅ | ✅ |
| SentencePiece | ✅ | — |
| ONNX | 🚧 | 🚧 |
| PyTorch | 🚧 | 🚧 |
| TFLite | 🚧 | 🚧 |

## Documentation

- [Docs](docs/README.md) · [Changelog](CHANGELOG.md) · [Contributing](CONTRIBUTING.md) · [Roadmap](ROADMAP.md)
- REST API: `umc-api` (Actix-Web) — [endpoints](docs/README.md#api)

## License

Apache 2.0 — see [LICENSE](LICENSE).