# Sentry Telemetry Configuration

AuroraView supports optional telemetry via Sentry for error tracking and performance monitoring.

## Overview

- **Frontend (Browser)**: Uses `@sentry/react` + OpenTelemetry
- **Backend (Rust/Packed Client)**: Uses `sentry` crate + `sentry-tracing`
- **Python Backend**: Uses `sentry-sdk` (optional, for packed mode)

## Local Development

### 1. Create Local Sentry Configuration

Create `gallery/.env.sentry` (already in `.gitignore`):

```bash
# Sentry DSNs for local development
# Copy from your Sentry project settings

# Rust/Packed Client (backend telemetry)
AURORAVIEW_GALLERY_RUST_SENTRY_DSN=https://xxx@xxx.ingest.sentry.io/xxx
AURORAVIEW_GALLERY_RUST_OTLP_ENDPOINT=https://xxx.ingest.sentry.io/api/xxx/integration/otlp
AURORAVIEW_GALLERY_RUST_SERVICE_NAME=auroraview-gallery-packed-client
AURORAVIEW_GALLERY_RUST_SENTRY_ENV=development
AURORAVIEW_GALLERY_RUST_SENTRY_SAMPLE_RATE=1.0
AURORAVIEW_GALLERY_RUST_SENTRY_TRACES_SAMPLE_RATE=0.2

# Frontend (browser telemetry)
VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_DSN=https://xxx@xxx.ingest.sentry.io/xxx
VITE_AURORAVIEW_GALLERY_FRONTEND_OTLP_ENDPOINT=https://xxx.ingest.sentry.io/api/xxx/integration/otlp
VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_SAMPLE_RATE=1.0
VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_TRACES_SAMPLE_RATE=0.2
```

### 2. Build with Local Configuration

```bash
# Build frontend (loads .env.sentry automatically via Vite)
just gallery-build

# Pack executable (loads .env.sentry via justfile)
just gallery-pack-local
```

### 3. Run Packed Application

```bash
just gallery-run-packed
```

## CI/CD Configuration (GitHub Actions)

### Setting Up GitHub Secrets

Use the provided script to upload your Sentry DSNs to GitHub Secrets:

```powershell
# From project root
./scripts/setup-sentry-secrets.ps1

# Or dry-run first
./scripts/setup-sentry-secrets.ps1 -DryRun
```

This will set the following GitHub Secrets:

| Secret Name | Description |
|-------------|-------------|
| `SENTRY_RUST_DSN` | DSN for Rust/packed client backend |
| `SENTRY_RUST_OTLP_ENDPOINT` | OTLP endpoint for Rust backend |
| `SENTRY_FRONTEND_DSN` | DSN for browser frontend |
| `SENTRY_FRONTEND_OTLP_ENDPOINT` | OTLP endpoint for frontend |

And GitHub Variables:

| Variable Name | Default | Description |
|---------------|---------|-------------|
| `SENTRY_RUST_SAMPLE_RATE` | 1.0 | Error sample rate for Rust |
| `SENTRY_RUST_TRACES_SAMPLE_RATE` | 0.2 | Trace sample rate for Rust |
| `SENTRY_FRONTEND_SAMPLE_RATE` | 1.0 | Error sample rate for frontend |
| `SENTRY_FRONTEND_TRACES_SAMPLE_RATE` | 0.2 | Trace sample rate for frontend |

### Manual Setup via `gh` CLI

```bash
# Set secrets
gh secret set SENTRY_RUST_DSN --body "https://xxx@xxx.ingest.sentry.io/xxx"
gh secret set SENTRY_RUST_OTLP_ENDPOINT --body "https://xxx.ingest.sentry.io/api/xxx/integration/otlp"
gh secret set SENTRY_FRONTEND_DSN --body "https://xxx@xxx.ingest.sentry.io/xxx"
gh secret set SENTRY_FRONTEND_OTLP_ENDPOINT --body "https://xxx.ingest.sentry.io/api/xxx/integration/otlp"

# Set variables (non-sensitive)
gh variable set SENTRY_RUST_SAMPLE_RATE --body "1.0"
gh variable set SENTRY_RUST_TRACES_SAMPLE_RATE --body "0.2"
gh variable set SENTRY_FRONTEND_SAMPLE_RATE --body "1.0"
gh variable set SENTRY_FRONTEND_TRACES_SAMPLE_RATE --body "0.2"
```

### How CI/CD Uses Secrets

In `.github/workflows/build-gallery.yml`:

1. **Frontend build** - Vite reads `VITE_*` environment variables from GitHub Secrets
2. **Pack command** - Rust CLI reads `AURORAVIEW_GALLERY_*` environment variables

Both are injected at build time and embedded into the final executable.

## Disabling Telemetry

### Local Development

Add to `gallery/.env.sentry`:

```bash
AURORAVIEW_GALLERY_FRONTEND_SENTRY_DISABLED=1
AURORAVIEW_GALLERY_FRONTEND_OTEL_DISABLED=1
```

### Runtime (Packed Application)

Set environment variables:

```bash
# Disable frontend telemetry
AURORAVIEW_GALLERY_FRONTEND_SENTRY_DISABLED=1

# Disable backend telemetry
AURORAVIEW_GALLERY_RUST_SENTRY_DISABLED=1
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    AuroraView Gallery                           │
├─────────────────────────────────────────────────────────────────┤
│  Frontend (React)              │  Backend (Rust/Python)         │
│  ─────────────────             │  ──────────────────            │
│  @sentry/react                 │  sentry crate                  │
│  @opentelemetry/*              │  sentry-tracing                │
│                                │  tracing-subscriber            │
├─────────────────────────────────────────────────────────────────┤
│  VITE_* env vars (build time)  │  AURORAVIEW_* env vars         │
│                                │  (runtime/pack time)           │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
                        ┌───────────────┐
                        │ Sentry.io     │
                        │ (SaaS/On-prem)│
                        └───────────────┘
```

## Security Considerations

1. **DSN is not a secret** - Sentry DSNs are designed to be public (like API keys for client-side apps)
2. **Source Maps** - Upload source maps to Sentry for readable stack traces
3. **PII** - Be careful not to log personal identifiable information
4. **Rate Limits** - Configure sample rates to avoid quota issues

## Troubleshooting

### Frontend telemetry not working

1. Check if `VITE_AURORAVIEW_GALLERY_FRONTEND_SENTRY_DSN` is set
2. Check browser console for Sentry initialization logs
3. Verify DSN format: `https://<key>@<host>/<project_id>`

### Backend telemetry not working

1. Check if `AURORAVIEW_GALLERY_RUST_SENTRY_DSN` is set
2. Verify `sentry` feature is enabled in Cargo.toml
3. Check for "Sentry initialized" log in stdout

### CI build fails with missing DSN

This is expected if no secrets are configured - telemetry will be disabled.
To enable, configure GitHub Secrets as described above.
