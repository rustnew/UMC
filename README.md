<div align="center">

# UMC

**Universal Model Converter — convert AI models between formats. Fast. Lossless. Verifiable.**

[![CI](https://github.com/rustnew/UMC/actions/workflows/ci.yml/badge.svg)](https://github.com/rustnew/UMC/actions)
[![Release](https://img.shields.io/github/v/release/rustnew/UMC)](https://github.com/rustnew/UMC/releases)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80-orange.svg?logo=rust)](https://www.rust-lang.org/)

</div>

UMC converts models between formats (GGUF, SafeTensors, ONNX, PyTorch, TFLite, ...) through a
single Universal Intermediate Representation. Written in Rust — no Python, no GIL, no server.

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
| Convertir | Drop a model, pick target, convert |
| Historique | Past conversions (persisted) |
| Formats | Supported formats catalogue |
| Réglages | Theme, threads, validation |

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

- [Docs](docs/README.md) · [Changelog](CHANGELOG.md) · [Contributing](CONTRIBUTING.md)
- REST API: `umc-api` (Actix-Web) — [endpoints](docs/README.md#api)

## License

Apache 2.0 — see [LICENSE](LICENSE).