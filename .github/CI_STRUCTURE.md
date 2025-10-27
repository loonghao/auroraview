# AuroraView CI/CD Configuration

This document describes the CI/CD structure for AuroraView, which follows the PyRustor project pattern.

## Directory Structure

```
.github/
├── actions/
│   ├── setup-auroraview/          # Setup environment (Python, Rust, uv, just)
│   │   └── action.yml
│   ├── cache-setup/               # Advanced caching strategy
│   │   └── action.yml
│   ├── build-and-test/            # Build and run tests
│   │   └── action.yml
│   └── build-wheel/               # Build wheels with maturin
│       └── action.yml
└── workflows/
    ├── ci.yml                     # Main CI workflow (runs on main branch)
    ├── pr-checks.yml              # PR validation workflow
    └── build-wheels.yml           # Wheel building workflow
```

## Workflows

### 1. CI Workflow (`ci.yml`)

**Triggers:**
- Push to `main` branch
- Pull requests with `full-ci` or `test-wheels` labels
- Manual workflow dispatch

**Jobs:**
- `quick-test`: Basic tests on Python 3.8, 3.10, 3.12 (runs on every PR)
- `lint`: Code quality checks (runs on every PR)
- `full-test`: Comprehensive tests on all platforms (main branch only)
- `wheel-test`: Wheel building and testing (main branch only)
- `docs`: Documentation tests
- `ci-success`: Final status check

### 2. PR Checks Workflow (`pr-checks.yml`)

**Triggers:**
- Pull request opened, synchronized, reopened, or ready for review
- Manual workflow dispatch

**Jobs:**
- `essential-tests`: Full tests on Python 3.8, 3.10, 3.12
- `quality-checks`: Code quality and linting
- `pr-ready`: Final approval gate

### 3. Build Wheels Workflow (`build-wheels.yml`)

**Triggers:**
- Push to `main` branch
- Push to tags
- Pull requests with changes to source files
- Manual workflow dispatch

**Jobs:**
- `linux`: Build wheels for x86_64 and i686
- `windows`: Build wheels for x64 and x86
- `macos`: Build wheels for x86_64 and universal2

## Custom Actions

### setup-auroraview

Sets up the development environment with:
- Python (specified version)
- Rust toolchain with optional components
- uv package manager
- just command runner
- System dependencies (Linux only)
- Project dependencies

**Inputs:**
- `python-version`: Python version (default: 3.11)
- `rust-components`: Additional Rust components (e.g., 'clippy', 'rustfmt')
- `cache-key-suffix`: Cache key suffix for better cache management
- `install-dependencies`: Whether to install project dependencies (default: true)

### cache-setup

Implements intelligent caching for:
- Rust dependencies and build artifacts
- Python dependencies and virtual environments
- Build outputs
- Test artifacts

**Inputs:**
- `cache-type`: Type of cache (rust, python, build, test)
- `cache-key-suffix`: Additional cache key suffix
- `python-version`: Python version for cache key

### build-and-test

Builds the extension and runs tests with configurable test types:
- `basic`: Basic import tests
- `rust`: Rust unit tests only
- `python`: Python unit tests only
- `full`: Complete test suite (Rust + Python)

**Inputs:**
- `test-type`: Type of tests to run
- `generate-stubs`: Whether to generate type stubs
- `upload-artifacts`: Whether to upload test artifacts
- `artifact-name`: Name for uploaded artifacts

### build-wheel

Builds wheels using maturin with platform-specific support:
- Linux (x86_64, i686)
- Windows (x64, x86)
- macOS (x86_64, universal2)

**Inputs:**
- `target`: Target platform
- `python-version`: Python version for wheel
- `maturin-args`: Additional maturin arguments
- `test-wheel`: Whether to test the built wheel
- `upload-wheel`: Whether to upload as artifact
- `artifact-name`: Name for wheel artifact

## Just Commands

The CI workflows use the following `just` commands:

- `ci-install`: Install CI dependencies
- `ci-build`: Build extension for CI
- `ci-test-rust`: Run Rust tests
- `ci-test-python`: Run Python tests
- `ci-test-basic`: Run basic import tests
- `ci-lint`: Run linting and formatting checks

## System Dependencies

On Linux, the following system packages are automatically installed:
- `libwebkit2gtk-4.1-dev`: WebKitGTK development files
- `libgtk-3-dev`: GTK 3 development files
- `libglib2.0-dev`: GLib development files
- `libpango1.0-dev`: Pango text rendering
- `libcairo2-dev`: Cairo graphics library

These are required for building Wry (the WebView library) on Linux.

## Caching Strategy

The CI uses a multi-level caching strategy:

1. **Rust Cache**: Caches cargo registry, git dependencies, and build artifacts
2. **Python Cache**: Caches uv cache and virtual environments
3. **Build Cache**: Caches compiled binaries and build outputs
4. **Test Cache**: Caches pytest cache and coverage reports

Cache keys include:
- OS (runner.os)
- Dependency hashes (Cargo.lock, pyproject.toml, uv.lock)
- Python version (for Python-specific caches)
- Git SHA (for build cache freshness)
- Custom suffix (for fine-grained control)

## PR Approval Requirements

For a PR to be approved and merged, the following must pass:
1. ✅ Essential tests (Python 3.8, 3.10, 3.12)
2. ✅ Code quality checks (Rust formatting, clippy, Python linting)
3. ✅ All required status checks

## Labels for Extended Testing

- `full-ci`: Trigger full test suite on all platforms
- `test-wheels`: Trigger wheel building and testing

## Performance Optimizations

1. **Parallel Testing**: Tests run in parallel across Python versions
2. **Fail-Fast Disabled**: All test combinations run even if one fails
3. **Selective Caching**: Different cache strategies for different job types
4. **Artifact Retention**: Test artifacts retained for 7 days, wheels for 30 days
5. **Early Exit**: Quick tests run first to catch issues early

## References

This CI configuration is based on the PyRustor project structure:
https://github.com/loonghao/PyRustor/tree/main/.github

