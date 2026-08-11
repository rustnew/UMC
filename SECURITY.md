# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.0.x   | ✅ Active development |
| 0.x     | ❌ Not supported |

## Reporting a Vulnerability

**Please do not open public issues for security vulnerabilities.**

Instead, report privately via GitHub's **Security Advisories**:

1. Go to https://github.com/rustnew/Universal_Model_Convert/security/advisories
2. Click **"New draft security advisory"**
3. Describe the vulnerability, affected versions, and impact

You can also email the maintainer directly (address available on the
repository profile page).

We aim to acknowledge reports within **48 hours** and publish a fix within
**7 days** for confirmed vulnerabilities.

## Security Notes

- UMC parses **untrusted model files** from many formats — treat files from
  unknown sources with care. The parsers are fuzz-tested, but malformed
  inputs are always a risk.
- External tools (TensorRT `trtexec`, CoreML tools, ...) are invoked as
  subprocesses; keep them pinned to trusted versions.
- Secrets (`DATABASE_URL`, `JWT_SECRET`, API keys) must only be set via
  environment variables, never committed.