# UMC — Launch Guide

## Prerequisites

- Rust (stable) + Cargo
- PostgreSQL 16+ running on port 5432
- Node.js 20+ with npm

---

## 1. Database setup (first time only)

```bash
sudo -u postgres psql <<'SQL'
CREATE USER umc WITH PASSWORD 'umc_password';
CREATE DATABASE umc_db OWNER umc;
GRANT ALL PRIVILEGES ON DATABASE umc_db TO umc;
SQL
```

---

## 2. Backend (`umc-api`)

```bash
cd /home/fossouomartial/UMC

# Dev (hot-reloads not supported, just run directly)
DATABASE_URL=postgres://umc:umc_password@localhost:5432/umc_db \
JWT_SECRET=umc_super_secret_jwt_key_change_in_production_minimum_32_chars \
RUST_LOG=info \
cargo run -p umc-api

# Release
cargo build --release -p umc-api
DATABASE_URL=postgres://umc:umc_password@localhost:5432/umc_db \
JWT_SECRET=umc_super_secret_jwt_key_change_in_production_minimum_32_chars \
RUST_LOG=info \
./target/release/umc-api
```

Backend starts on **http://localhost:8080**

Alternatively, create `umc-api/.env` (already committed):

```
DATABASE_URL=postgres://umc:umc_password@localhost:5432/umc_db
JWT_SECRET=umc_super_secret_jwt_key_change_in_production_minimum_32_chars
UMC_HOST=0.0.0.0
UMC_PORT=8080
RUST_LOG=info,umc_api=debug
```

Then just:

```bash
cargo run -p umc-api
```

---

## 3. Frontend (`umc-frontend`)

```bash
cd /home/fossouomartial/UMC/umc-frontend
npm install        # first time only
npm run dev        # dev server (typically port 5173 or 8081)
```

Frontend starts on **http://localhost:5173** (or 8081 if 5173 is taken).

`umc-frontend/.env` is already configured:
```
VITE_API_URL=http://localhost:8080
```

---

## 4. Quick smoke tests

```bash
# Health
curl http://localhost:8080/health

# Register
curl -s -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"you@example.com","password":"YourPass123!","display_name":"Your Name"}' | jq .

# Login
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"you@example.com","password":"YourPass123!"}' | jq -r .access_token)

# List formats
curl -s -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/v1/formats | jq '.formats[].slug'

# Conversion graph
curl -s -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/v1/formats/graph | jq '.edges | length'
```

---

## 5. API summary

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET  | `/health` | — | Health + DB check |
| POST | `/auth/register` | — | Create account |
| POST | `/auth/login` | — | Login → JWT |
| POST | `/auth/refresh` | — | Refresh token |
| POST | `/auth/logout` | JWT | Revoke token |
| GET  | `/auth/me` | JWT | Current user |
| GET  | `/v1/formats` | JWT | List all formats |
| GET  | `/v1/formats/graph` | JWT | Conversion edges |
| POST | `/v1/upload` | JWT | Upload model file |
| POST | `/v1/jobs` | JWT | Create conversion job |
| GET  | `/v1/jobs` | JWT | List jobs |
| GET  | `/v1/jobs/:id` | JWT | Job details |
| DELETE | `/v1/jobs/:id` | JWT | Cancel job |
| GET  | `/v1/jobs/:id/download` | JWT | Download output |
| GET  | `/v1/jobs/:id/progress` | JWT | SSE progress stream |

---

## 6. Supported formats

All 12 formats from `umc-formats` are registered:

| Slug | Name | Read | Write |
|------|------|------|-------|
| `gguf` | GGUF | ✓ | ✓ |
| `safetensors` | SafeTensors | ✓ | ✓ |
| `onnx` | ONNX | ✓ | ✓ |
| `pytorch` | PyTorch | ✓ | ✓ |
| `awq` | AWQ | ✓ | ✓ |
| `gptq` | GPTQ | ✓ | ✓ |
| `tflite` | TFLite | ✓ | ✓ |
| `coreml` | CoreML | — | ✓ |
| `tensorrt` | TensorRT | — | ✓ |
| `openvino` | OpenVINO | — | ✓ |
| `executorch` | ExecuTorch | — | ✓ |
| `lora` | LoRA | ✓ | — |
