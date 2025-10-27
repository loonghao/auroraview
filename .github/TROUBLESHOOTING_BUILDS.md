# Build Troubleshooting Guide

## Common Build Issues and Solutions

### Linux: glib-sys Build Failure

**Symptom:**
```
error: failed to run custom build command for `glib-sys v0.18.1`
The system library `glib-2.0` required by crate `glib-sys` was not found.
```

**Solution:**
Ensure the following packages are installed:
```bash
sudo apt-get update
sudo apt-get install -y \
  libglib2.0-dev \
  libglib2.0-0 \
  pkg-config
```

**Why:**
- `libglib2.0-dev`: Development headers for glib
- `libglib2.0-0`: Runtime library
- `pkg-config`: Tool to find library paths

---

### Windows: Architecture Mismatch

**Symptom:**
```
👽 'C:\...\python.exe' reports a platform 'win-amd64' (architecture 'x86_64'), 
while the Rust target is 'i686'. Skipping.
💥 maturin failed: Could not find any interpreters
```

**Solution:**
- Only build for x64 (x86_64) on Windows
- GitHub Actions Windows runners don't provide 32-bit Python
- Remove i686 from Windows build targets

**Configuration:**
```yaml
windows:
  strategy:
    matrix:
      target: [x64]  # Only x64, not x86
```

---

### Linux i686 Build: Testing Skipped

**Symptom:**
```
⚠️ Skipping wheel test: i686 wheel on x86_64 system (architecture mismatch)
This is expected and not an error - the wheel was built correctly for i686
```

**Explanation:**
- i686 wheels are built successfully on Linux
- Testing is skipped because the runner is x86_64
- This is expected and not an error
- The wheel is still valid for 32-bit systems

---

### macOS: Universal2 Builds

**Note:**
macOS builds support both x86_64 and ARM64 (Apple Silicon) via universal2 binaries.

**Configuration:**
```yaml
macos:
  strategy:
    matrix:
      target: [x86_64, universal2-apple-darwin]
```

---

## Platform Support Matrix

| Platform | Targets | Testing | Notes |
|----------|---------|---------|-------|
| Linux | x86_64, i686 | x86_64 only | i686 built but not tested |
| Windows | x64 | Yes | 32-bit not supported |
| macOS | x86_64, universal2 | Yes | Universal binaries |

---

## Debugging Build Issues

### 1. Check System Dependencies

**Linux:**
```bash
pkg-config --list-all | grep glib
dpkg -l | grep libglib
```

**Windows:**
```powershell
python -c "import sysconfig; print(sysconfig.get_platform())"
```

### 2. Check Wheel Compatibility

```bash
# List wheel contents
unzip -l dist/*.whl

# Check wheel metadata
python -m pip show -f auroraview

# Verify architecture
python -c "import platform; print(platform.machine())"
```

### 3. Enable Verbose Output

```bash
# Rust backtrace
export RUST_BACKTRACE=1

# Maturin verbose
maturin develop -vv

# Pip verbose
pip install -vv dist/*.whl
```

---

## CI/CD Workflow Files

- **Main workflow:** `.github/workflows/build-wheels.yml`
- **Build action:** `.github/actions/build-wheel/action.yml`
- **Setup action:** `.github/actions/setup-auroraview/action.yml`

---

## References

- [PyO3 maturin-action](https://github.com/PyO3/maturin-action)
- [glib-sys crate](https://crates.io/crates/glib-sys)
- [Python Packaging Guide](https://packaging.python.org/)
- [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)

