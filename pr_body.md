## Summary

Automated code quality scan found several issues across the codebase. This PR addresses low-risk, high-confidence fixes.

## Changes

### Metadata Fixes
- **Fix incorrect repository URLs** in 4 crates:
  - `auroraview-settings`: `aspect-build/auroraview` → `loonghao/auroraview`
  - `auroraview-notifications`: `aspect-build/auroraview` → `loonghao/auroraview`
  - `auroraview-desktop`: `AuroraView/auroraview` → `loonghao/auroraview`
  - `auroraview-dcc`: `AuroraView/auroraview` → `loonghao/auroraview`
- **Fix duplicate comment** in root `Cargo.toml` - workspace-hack had incorrect telemetry comment

### Dead Code Removal
- **Remove unused backward compatibility aliases** in `desktop.rs`:
  - `create_standalone()` - marked `#[deprecated]` and `#[allow(dead_code)]`, zero callers
  - `run_standalone()` - marked `#[deprecated]` and `#[allow(dead_code)]`, zero callers
  - Note: Python-level `run_standalone` alias in `desktop_runner.rs` is preserved

### Code Quality
- **Eliminate unnecessary `Vec<char>` allocation** in path validation - use direct byte indexing
- **Remove `unwrap()` in path normalization** - replace with safe `as_bytes` access
- **Clean up duplicate import** in `batch.rs`

## Verification
- `cargo check` ✅
- `cargo clippy` ✅
- `cargo fmt` ✅ (via pre-commit)
- All pre-commit hooks passed ✅
