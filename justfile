# justfile for AuroraView development
# Run `just --list` to see all available commands
#
# Quick Start:
#   just rebuild-core        - Rebuild Rust core with maturin (release mode)
#   just rebuild-core-verbose - Same as above with verbose output
#   just test                - Run all tests
#   just format              - Format code
#   just lint                - Run linting
#
# Note: This justfile uses vx for tool management.
#       Run `vx setup` to install all required tools.
#       Or use `vx just <command>` to run commands with vx-managed tools.

# Set shell for Windows compatibility
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set shell := ["sh", "-c"]

# Default recipe to display help
default:
    @just --list

# Install dependencies
install:
    @echo "Installing dependencies..."
    vx uv sync --group dev

# Build the extension module
build:
    @echo "Building extension module..."
    vx uv run maturin develop --features "ext-module,python-bindings,abi3-py38,win-webview2"

# Build with release optimizations
build-release:
    @echo "Building release version..."
    vx uv run maturin develop --release --features "ext-module,python-bindings,abi3-py38,win-webview2"

# Build Python library (PyO3 bindings)
rebuild-pylib:
    @echo "Building Python library with maturin..."
    vx uv run maturin develop --release --features "ext-module,python-bindings,abi3-py38,win-webview2"
    @echo "[OK] Python library rebuilt and installed successfully!"

# Build Python library with verbose output
rebuild-pylib-verbose:
    @echo "Building Python library with maturin (verbose)..."
    vx uv run maturin develop --release --features "ext-module,python-bindings,abi3-py38,win-webview2" --verbose
    @echo "[OK] Python library rebuilt and installed successfully!"

# Build CLI binary
build-cli:
    @echo "Building CLI binary..."
    vx cargo build -p auroraview-cli --release
    @echo "[OK] CLI built: target/release/auroraview.exe"

# Build all workspace crates (including SDK assets)
build-all: sdk-build-all
    @echo "Building all workspace crates..."
    vx cargo build -p auroraview-core
    vx cargo build -p auroraview-pack
    vx cargo build -p auroraview-cli --release
    vx uv run maturin develop --release --features "ext-module,python-bindings,abi3-py38,win-webview2"
    @echo "[OK] All crates built successfully!"

# Run all tests
[unix]
test:
    @echo "Running workspace crate tests..."
    vx cargo test -p auroraview-core
    vx cargo test -p auroraview-pack
    vx cargo test -p auroraview-cli
    @echo "Running Rust integration tests (with rstest)..."
    # Note: names are aligned with Cargo.toml [[test]] targets.
    vx cargo test --test mdns_integration --features "test-helpers"
    vx cargo test --test protocol_handlers_integration --features "test-helpers"
    vx cargo test --test protocol_integration --features "test-helpers"
    vx cargo test --test timer_integration --features "test-helpers"
    vx cargo test --test ipc_message_queue_integration --features "test-helpers"
    vx cargo test --test http_discovery_integration --features "test-helpers"
    vx cargo test --test standalone_integration --features "test-helpers"
    vx cargo test --test config_integration --features "test-helpers"
    vx cargo test --test ipc_json_integration --features "test-helpers"
    vx cargo test --test file_protocol_integration --features "test-helpers"
    vx cargo test --test port_allocator_integration --features "test-helpers"

    @echo "Running Rust doc tests..."
    vx cargo test --doc
    @echo "Running Python tests..."
    vx uv run pytest -q -rA tests/python/unit tests/python/integration


[windows]
test:
    @echo "Running workspace crate tests..."
    vx cargo test -p auroraview-core
    vx cargo test -p auroraview-pack
    vx cargo test -p auroraview-cli
    @echo ""
    @echo "Note: Rust integration tests are skipped on Windows due to STATUS_DLL_NOT_FOUND (abi3/PyO3 linking) on some machines."
    @echo "These tests run successfully in CI on Linux."
    @echo "Running Rust doc tests..."
    vx cargo test --doc
    @echo "Running Python tests..."
    vx uv run pytest -q -rA tests/python/unit tests/python/integration



# Run tests with coverage
test-cov:
    @echo "Running tests with coverage..."
    vx uv run pytest -v --cov=auroraview --cov-report=html --cov-report=term-missing tests/python/unit tests/python/integration


# Run only fast tests (exclude slow tests)
test-fast:
    @echo "Running fast tests..."
    vx uvx pytest tests/python/ -v -m "not slow"


# Test with Python 3.7
test-py37:
    @echo "Testing with Python 3.7..."
    vx uv venv --python 3.7 .venv-py37
    vx uv pip install -e . pytest pytest-cov --python .venv-py37\Scripts\python.exe
    .venv-py37\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.8
test-py38:
    @echo "Testing with Python 3.8..."
    vx uv venv --python 3.8 .venv-py38
    vx uv pip install -e . pytest pytest-cov --python .venv-py38\Scripts\python.exe
    .venv-py38\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.9
test-py39:
    @echo "Testing with Python 3.9..."
    vx uv venv --python 3.9 .venv-py39
    vx uv pip install -e . pytest pytest-cov --python .venv-py39\Scripts\python.exe
    .venv-py39\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.10
test-py310:
    @echo "Testing with Python 3.10..."
    vx uv venv --python 3.10 .venv-py310
    vx uv pip install -e . pytest pytest-cov --python .venv-py310\Scripts\python.exe
    .venv-py310\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.11
test-py311:
    @echo "Testing with Python 3.11..."
    vx uv venv --python 3.11 .venv-py311
    vx uv pip install -e . pytest pytest-cov --python .venv-py311\Scripts\python.exe
    .venv-py311\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.12
test-py312:
    @echo "Testing with Python 3.12..."
    vx uv venv --python 3.12 .venv-py312
    vx uv pip install -e . pytest pytest-cov --python .venv-py312\Scripts\python.exe
    .venv-py312\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with all supported Python versions
test-all-python:
    @echo "Testing with all supported Python versions..."
    just test-py37
    just test-py38
    just test-py39
    just test-py310
    just test-py311
    just test-py312
    @echo "[OK] All Python versions tested successfully!"
# nox wrappers for multi-Python testing
nox:
    @echo "Running nox session: pytest (multi-Python)"
    vx uvx nox -s pytest

nox-qt:
    @echo "Running nox session: pytest-qt (multi-Python with Qt)"
    vx uvx nox -s pytest-qt


nox-all:
    @echo "Running nox session: pytest-all (full suite)"
    vx uvx nox -s pytest-all


# ═══════════════════════════════════════════════════════════════════════════════
# Package-Level Test Commands (for isolated CI)
# ═══════════════════════════════════════════════════════════════════════════════

# Test aurora-signals crate (standalone, no deps)
test-signals:
    @echo "Testing aurora-signals..."
    vx cargo test -p aurora-signals
    vx cargo clippy -p aurora-signals -- -D warnings

# Test aurora-protect crate (standalone, no deps)
test-protect:
    @echo "Testing aurora-protect..."
    vx cargo test -p aurora-protect
    vx cargo clippy -p aurora-protect -- -D warnings

# Test auroraview-plugin-core crate (standalone, no deps)
test-plugin-core:
    @echo "Testing auroraview-plugin-core..."
    vx cargo test -p auroraview-plugin-core
    vx cargo clippy -p auroraview-plugin-core -- -D warnings

# Test auroraview-plugin-fs crate (depends on plugin-core)
test-plugin-fs:
    @echo "Testing auroraview-plugin-fs..."
    vx cargo test -p auroraview-plugin-fs
    vx cargo clippy -p auroraview-plugin-fs -- -D warnings

# Test auroraview-extensions crate (standalone, no deps)
test-extensions:
    @echo "Testing auroraview-extensions..."
    vx cargo test -p auroraview-extensions
    vx cargo clippy -p auroraview-extensions -- -D warnings

# Test auroraview-plugins crate (depends on plugin-core, plugin-fs, extensions)
test-plugins:
    @echo "Testing auroraview-plugins..."
    vx cargo test -p auroraview-plugins
    vx cargo clippy -p auroraview-plugins -- -D warnings

# Test auroraview-core crate (depends on signals, plugins)
test-core:
    @echo "Testing auroraview-core..."
    vx cargo test -p auroraview-core
    vx cargo clippy -p auroraview-core -- -D warnings

# Test auroraview-pack crate (depends on protect)
test-pack:
    @echo "Testing auroraview-pack..."
    vx cargo test -p auroraview-pack
    vx cargo clippy -p auroraview-pack -- -D warnings

# Test auroraview-cli crate (depends on core, pack)
test-cli:
    @echo "Testing auroraview-cli..."
    vx cargo test -p auroraview-cli
    vx cargo clippy -p auroraview-cli -- -D warnings

# Test all standalone crates (no internal dependencies)
test-standalone:
    @echo "Testing standalone crates..."
    just test-signals
    just test-protect
    just test-plugin-core
    just test-extensions

# Test Python package only (no Rust rebuild)
test-python:
    @echo "Running Python tests..."
    vx uvx pytest tests/python/unit tests/python/integration -v --tb=short

# Test Python unit tests only
test-python-unit:
    @echo "Running Python unit tests..."
    vx uvx pytest tests/python/unit -v --tb=short

# Test Python integration tests only
test-python-integration:
    @echo "Running Python integration tests..."
    vx uvx pytest tests/python/integration -v --tb=short \
        --ignore=tests/python/integration/test_gallery_e2e.py \
        --ignore=tests/python/integration/test_gallery_real_e2e.py

# ═══════════════════════════════════════════════════════════════════════════════
# Legacy Test Commands (for backward compatibility)
# ═══════════════════════════════════════════════════════════════════════════════

# Run only Rust unit tests
test-unit:
    @echo "Running Rust unit tests..."
    vx cargo test --lib
    vx cargo test -p auroraview-core
    vx cargo test -p auroraview-pack
    vx cargo test -p auroraview-cli
    @echo "Running Python unit tests..."
    vx uvx pytest tests/python/unit -v


# Run only Rust integration tests
test-integration:
    @echo "Running Rust integration tests (with rstest)..."
    vx cargo test --test '*' --features "test-helpers"
    @echo "Running Python integration tests..."
    vx uvx pytest tests/python/integration -v


# Watch mode for continuous testing
test-watch:
    @echo "Running tests in watch mode..."
    vx cargo watch -x test

# Run specific test file
test-file FILE:
    @echo "Running tests in {{FILE}}..."
    vx uv run pytest {{FILE}} -v


# Run tests with specific marker
test-marker MARKER:
    @echo "Running tests with marker {{MARKER}}..."
    vx uvx pytest tests/ -v -m {{MARKER}}


# Format code
format:
    @echo "Formatting Rust code..."
    vx cargo fmt --all
    @echo "Formatting Python code..."
    vx uv run ruff format python/ tests/ examples/

# Run linting
lint:
    @echo "Linting Rust code..."
    vx cargo clippy --all-targets --all-features -- -D warnings
    @echo "Linting Python code..."
    vx uv run ruff check python/ tests/ examples/

# Fix linting issues automatically
fix:
    @echo "Fixing linting issues..."
    vx cargo clippy --fix --allow-dirty --allow-staged
    vx uv run ruff check --fix python/ tests/ examples/

# Run all checks (format, lint, test)
check: format lint test
    @echo "All checks passed!"

# CI-specific commands
ci-install:
    @echo "Installing CI dependencies (including Qt)..."
    vx uv sync --group dev --group test
    vx uv pip install qtpy PySide6 pytest-qt

# CI build command - consistent across all platforms
# Uses ext-module for proper Python extension module compilation
[unix]
ci-build:
    @echo "Building extension for CI (Unix)..."
    vx uv pip install maturin
    vx uv run maturin develop --features "ext-module,python-bindings,abi3-py38"

[windows]
ci-build:
    @echo "Building extension for CI (Windows)..."
    vx uv pip install maturin
    vx uv run maturin develop --features "ext-module,python-bindings,abi3-py38,win-webview2"

ci-test-rust:
    @echo "Running Rust doc tests..."
    @echo "Note: lib tests are skipped due to abi3 linking issues with PyO3"
    @echo "      Python tests provide comprehensive coverage instead"
    vx cargo test --doc

ci-test-python:
    @echo "Running Python unit tests with coverage..."
    vx uvx pytest tests/ -v --tb=short -m "not slow" \
        --cov=auroraview \
        --cov-report=term-missing \
        --cov-report=html \
        --cov-report=xml \
        --cov-fail-under=0 \
        --timeout=60

ci-test-basic:
    @echo "Running basic import tests..."
    vx uv run python -c "import auroraview; print('AuroraView imported successfully')"

ci-lint:
    @echo "Running CI linting..."
    vx cargo fmt --all -- --check
    vx cargo clippy --all-targets --all-features -- -D warnings
    vx uvx ruff check python/ tests/
    vx uvx ruff format --check python/ tests/

# Coverage commands
coverage-python:
    @echo "Running Python tests with coverage..."
    vx uv run pytest -v --cov=auroraview --cov-report=html --cov-report=term-missing --cov-report=xml tests/python/unit tests/python/integration
# Shortcut alias for Python coverage
pycov:
    @echo "[Alias] Running Python coverage via coverage-python..."
    @just coverage-python


coverage-rust:
	@echo "Running Rust tests with coverage (preferring cargo-llvm-cov) in headless mode..."
	if (Get-Command py -ErrorAction SilentlyContinue) { $pyBase = py -c "import sys; print(sys.base_prefix)" } else { $pyBase = python -c "import sys; print(sys.base_prefix)" }; $env:Path = "$pyBase;$pyBase\DLLs;$pyBase\bin;$env:Path"; if (Get-Command cargo-llvm-cov -ErrorAction SilentlyContinue) { $ignore = "(src[/\\]webview[/\\](aurora_view\\.rs|embedded\\.rs|standalone\\.rs|protocol\\.rs|timer_bindings\\.rs|webview_inner\\.rs|backend[/\\].*|platform[/\\].*)|src[/\\]service_discovery[/\\]mdns_service\\.rs)"; if ($env:CI -eq "true") { vx rustup component add llvm-tools-preview; vx cargo llvm-cov --workspace --html --tests --no-default-features --features "python-bindings threaded-ipc test-helpers" --ignore-filename-regex $ignore --fail-under-lines 50 } else { vx cargo llvm-cov --workspace --html --tests --no-default-features --features "python-bindings threaded-ipc test-helpers" --ignore-filename-regex $ignore; $json = vx cargo llvm-cov --summary-only --json --workspace --tests --no-default-features --features "python-bindings threaded-ipc test-helpers" --ignore-filename-regex $ignore | Out-String | ConvertFrom-Json; $covered = [double]$json.data[0].totals.lines.covered; $count = [double]$json.data[0].totals.lines.count; if ($count -gt 0) { $lines = [math]::Round((100.0 * $covered / $count), 2) } else { $lines = 0 }; if ($lines -ge 50) { echo ("[OK] Rust coverage lines: {0}% (>=50) report: target/llvm-cov/html/index.html" -f $lines) } else { echo ("[WARN] Rust coverage lines: {0}% (<50)" -f $lines) } } } elseif (Get-Command cargo-tarpaulin -ErrorAction SilentlyContinue) { vx cargo tarpaulin --no-default-features --features "python-bindings threaded-ipc test-helpers" --out Html --out Xml --output-dir target/tarpaulin; if ($LASTEXITCODE -eq 0) { echo "[OK] Rust coverage report: target/tarpaulin/tarpaulin-report.html" } else { echo "[WARN] cargo-tarpaulin exited with code $LASTEXITCODE" } } else { echo "[INFO] No Rust coverage tool found."; echo "      Install recommended: cargo install cargo-llvm-cov"; echo "      Also run: rustup component add llvm-tools-preview" }

# Run coverage for individual crates
coverage-crate CRATE:
    @echo "Running coverage for crate: {{CRATE}}..."
    vx cargo llvm-cov --package {{CRATE}} --html --tests --output-dir target/llvm-cov/{{CRATE}}
    @echo "[OK] Coverage report: target/llvm-cov/{{CRATE}}/html/index.html"

# Run coverage for auroraview-core crate
coverage-core:
    @echo "Running coverage for auroraview-core..."
    vx cargo llvm-cov --package auroraview-core --html --tests --output-dir target/llvm-cov/auroraview-core
    @echo "[OK] Coverage report: target/llvm-cov/auroraview-core/html/index.html"

# Run coverage for auroraview-pack crate
coverage-pack:
    @echo "Running coverage for auroraview-pack..."
    vx cargo llvm-cov --package auroraview-pack --html --tests --output-dir target/llvm-cov/auroraview-pack
    @echo "[OK] Coverage report: target/llvm-cov/auroraview-pack/html/index.html"

# Run coverage for auroraview-cli crate
coverage-cli:
    @echo "Running coverage for auroraview-cli..."
    vx cargo llvm-cov --package auroraview-cli --html --tests --output-dir target/llvm-cov/auroraview-cli
    @echo "[OK] Coverage report: target/llvm-cov/auroraview-cli/html/index.html"

# Run coverage for all crates with lcov output (for CI)
coverage-rust-lcov:
    @echo "Running Rust coverage with lcov output..."
    vx cargo llvm-cov --workspace --lcov --output-path rust-coverage.lcov --tests --no-default-features --features "python-bindings threaded-ipc test-helpers"
    @echo "[OK] Coverage report: rust-coverage.lcov"

coverage-all: coverage-rust coverage-python
    @echo "All coverage reports generated!"

# Run benchmarks
bench:
    @echo "Running benchmarks..."
    vx cargo bench --bench ipc_bench

# Run benchmarks and save baseline
bench-save BASELINE="main":
    @echo "Running benchmarks and saving baseline: {{BASELINE}}..."
    vx cargo bench --bench ipc_bench -- --save-baseline {{BASELINE}}

# Compare benchmarks against baseline
bench-compare BASELINE="main":
    @echo "Comparing benchmarks against baseline: {{BASELINE}}..."
    vx cargo bench --bench ipc_bench -- --baseline {{BASELINE}}

# Clean build artifacts
clean:
    @echo "Cleaning build artifacts..."
    vx cargo clean
    rm -rf dist/ build/ htmlcov/
    find . -type d -name "__pycache__" -exec rm -rf {} +
    find . -type f -name "*.pyc" -delete
    find . -type f -name "*.pyo" -delete
    find . -type f -name "*.so" -delete
    find . -type f -name "*.pyd" -delete

# Setup development environment
dev: install build
    @echo "Development environment ready!"
    @echo "Try: just test"

# Build release wheels
release:
    @echo "Building release wheels..."
    vx uv run maturin build --release --features "ext-module,python-bindings,win-webview2"
    @echo "Wheels built in target/wheels/"

# Run examples
example EXAMPLE:
    @echo "Running example: {{EXAMPLE}}"
    vx uv run python examples/{{EXAMPLE}}.py

# Show project info
info:
    @echo "Project Information:"
    @echo "  Rust version: $(vx rustc --version)"
    @echo "  Python version: $(vx uv run python --version)"
    @echo "  UV version: $(vx uv --version)"
    @echo "  Node version: $(vx node --version)"

# Run security audit
audit:
    @echo "Running security audit..."
    vx cargo audit

# Documentation
docs:
    @echo "Building documentation..."
    vx cargo doc --no-deps --document-private-items --open

# Comprehensive checks
check-all: format lint test coverage-all
    @echo "All checks completed!"

# Setup development module for Maya
maya-setup-dev:
    @echo "=========================================="
    @echo "Setting up Maya Development Environment"
    @echo "=========================================="
    @echo ""
    @echo "[1/3] Creating symlink to project root..."
    @powershell -Command "New-Item -ItemType Directory -Force -Path '$env:USERPROFILE\Documents\maya\modules' | Out-Null; if (Test-Path '$env:USERPROFILE\Documents\maya\modules\auroraview') { Remove-Item -Recurse -Force '$env:USERPROFILE\Documents\maya\modules\auroraview' }; New-Item -ItemType SymbolicLink -Path '$env:USERPROFILE\Documents\maya\modules\auroraview' -Target '{{justfile_directory()}}' -Force | Out-Null"
    @echo "[OK] Symlink created: ~/Documents/maya/modules/auroraview -> {{justfile_directory()}}"
    @echo ""
    @echo "[2/3] Installing Maya module file..."
    @powershell -Command "Copy-Item -Path '{{justfile_directory()}}\examples\maya-outliner\auroraview.mod' -Destination '$env:USERPROFILE\Documents\maya\modules\auroraview.mod' -Force"
    @echo "[OK] Module file installed: ~/Documents/maya/modules/auroraview.mod"
    @echo ""
    @echo "[3/3] Installing userSetup.py..."
    @powershell -Command "New-Item -ItemType Directory -Force -Path '$env:USERPROFILE\Documents\maya\2024\scripts' | Out-Null; Copy-Item -Path '{{justfile_directory()}}\examples\maya-outliner\userSetup_dev.py' -Destination '$env:USERPROFILE\Documents\maya\2024\scripts\userSetup.py' -Force"
    @echo "[OK] userSetup.py installed for Maya 2024"
    @echo ""
    @echo "=========================================="
    @echo "Development environment ready!"
    @echo "=========================================="
    @echo ""
    @echo "Module configuration:"
    @echo "  Symlink: ~/Documents/maya/modules/auroraview -> {{justfile_directory()}}"
    @echo "  Module file: ~/Documents/maya/modules/auroraview.mod"
    @echo "  PYTHONPATH: {{justfile_directory()}}/python"
    @echo "  PYTHONPATH: {{justfile_directory()}}/examples/maya-outliner"
    @echo ""
    @echo "Next steps:"
    @echo "  1. Run: just maya-dev (rebuild + launch Maya)"
    @echo "  2. Click 'Outliner' button on AuroraView shelf"
    @echo ""

# Complete Maya development workflow (setup + rebuild + launch)
maya-dev:
    @echo "=========================================="
    @echo "Maya Development Workflow"
    @echo "=========================================="
    @echo ""
    @echo "[1/3] Killing all Maya processes..."
    -@powershell -Command "try { Get-Process maya -ErrorAction Stop | Stop-Process -Force; Write-Host '[OK] Maya processes terminated' } catch { Write-Host '[OK] No Maya processes running' }"
    @echo ""
    @echo "[2/3] Rebuilding Rust core..."
    @just rebuild-core
    @echo ""
    @echo "[3/3] Launching Maya 2024..."
    @powershell -Command "Start-Process -FilePath 'C:\Program Files\Autodesk\Maya2024\bin\maya.exe'"
    @echo "[OK] Maya launched"
    @echo ""
    @echo "=========================================="
    @echo "Maya Development Mode Active"
    @echo "=========================================="
    @echo ""
    @echo "✓ Symlinks are active (changes reflect immediately)"
    @echo "✓ Click 'Outliner' button on AuroraView shelf"
    @echo "✓ Or run in Script Editor:"
    @echo "    from maya_integration import maya_outliner"
    @echo "    maya_outliner.main()"
    @echo ""
    @echo "To rebuild after code changes:"
    @echo "  just maya-dev"
    @echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# Maya Development Commands
# ═══════════════════════════════════════════════════════════════════════════════

# Maya debugging workflow (legacy - use maya-dev instead)
maya-debug:
    @echo "=========================================="
    @echo "Maya Debug Workflow"
    @echo "=========================================="
    @echo ""
    @echo "[1/4] Killing all Maya processes..."
    -@powershell -Command "try { Get-Process maya -ErrorAction Stop | Stop-Process -Force; Write-Host '[OK] Maya processes terminated' } catch { Write-Host '[OK] No Maya processes running' }"
    @echo ""
    @echo "[2/4] Rebuilding Rust core..."
    @just rebuild-core
    @echo ""
    @echo "[3/4] Creating launch script..."
    @echo @echo off > launch_maya_temp.bat
    @echo set PYTHONPATH={{justfile_directory()}}\python >> launch_maya_temp.bat
    @echo "C:\Program Files\Autodesk\Maya2024\bin\maya.exe" >> launch_maya_temp.bat
    @echo "[OK] Launch script created"
    @echo ""
    @echo "[4/4] Launching Maya 2024..."
    @start launch_maya_temp.bat
    @echo "[OK] Maya launched"
    @echo ""
    @echo "=========================================="
    @echo "Maya launched with AuroraView in PYTHONPATH"
    @echo "=========================================="
    @echo ""
    @echo "In Maya Script Editor, run:"
    @echo "  import sys"
    @echo "  sys.path.append(r'{{justfile_directory()}}\examples\maya-outliner')"
    @echo "  from maya_integration import maya_outliner"
    @echo "  maya_outliner.main()"
    @echo ""



# ═══════════════════════════════════════════════════════════════════════════════
# SDK Commands (TypeScript SDK for frontend)
# ═══════════════════════════════════════════════════════════════════════════════

# Install SDK dependencies
sdk-install:
    @echo "Installing SDK dependencies..."
    cd packages/auroraview-sdk; vx npm install
    @echo "[OK] SDK dependencies installed!"

# Build SDK npm package
sdk-build:
    @echo "Building SDK npm package..."
    cd packages/auroraview-sdk; vx npm run build
    @echo "[OK] SDK built in packages/auroraview-sdk/dist/"

# Build SDK assets (inject scripts for Rust)
sdk-build-assets:
    @echo "Building SDK assets (inject scripts)..."
    cd packages/auroraview-sdk; vx npm run build:assets
    @echo "[OK] Assets built in crates/auroraview-core/src/assets/js/"

# Build SDK all (npm package + assets)
sdk-build-all: sdk-install
    @echo "Building SDK (all)..."
    cd packages/auroraview-sdk; vx npm run build:all
    @echo "[OK] SDK and assets built!"

# Run SDK unit tests
sdk-test:
    @echo "Running SDK unit tests..."
    cd packages/auroraview-sdk; vx npm run test
    @echo "[OK] SDK tests passed!"

# Run SDK tests with coverage
sdk-test-cov:
    @echo "Running SDK tests with coverage..."
    cd packages/auroraview-sdk; vx npm run test:coverage
    @echo "[OK] SDK coverage report: packages/auroraview-sdk/coverage/"

# Run SDK E2E tests (requires Playwright)
sdk-test-e2e:
    @echo "Running SDK E2E tests..."
    cd packages/auroraview-sdk; vx npm run test:e2e
    @echo "[OK] SDK E2E tests passed!"

# Run all SDK tests (unit + E2E)
sdk-test-all:
    @echo "Running all SDK tests..."
    cd packages/auroraview-sdk; vx npm run test:all
    @echo "[OK] All SDK tests passed!"

# Run SDK type check
sdk-typecheck:
    @echo "Running SDK type check..."
    cd packages/auroraview-sdk; vx npm run typecheck
    @echo "[OK] SDK type check passed!"

# Install Playwright for SDK E2E tests
sdk-playwright-install:
    @echo "Installing Playwright for SDK E2E tests..."
    cd packages/auroraview-sdk; vx npx playwright install chromium --with-deps
    @echo "[OK] Playwright installed!"

# Full SDK CI check (typecheck + test + coverage + build)
sdk-ci: sdk-install sdk-typecheck sdk-test-cov sdk-build-all
    @echo "[OK] SDK CI check passed!"

# ═══════════════════════════════════════════════════════════════════════════════
# Gallery Commands
# ═══════════════════════════════════════════════════════════════════════════════

# Build gallery frontend (builds SDK first)
gallery-build: sdk-build
    @echo "Building gallery frontend..."
    cd gallery; vx npm install; vx npm run build
    @echo "[OK] Gallery built in gallery/dist/"

# Run gallery (build frontend first, then launch with AuroraView)
gallery: gallery-build
    @echo "Starting AuroraView Gallery..."
    vx uv run python gallery/main.py

# Run gallery dev server (for frontend development)
gallery-dev:
    @echo "Starting gallery dev server..."
    cd gallery; vx npm run dev

# Run Gallery E2E tests
gallery-test:
    @echo "Running Gallery E2E tests..."
    vx uvx pytest tests/python/integration/test_gallery_e2e.py tests/python/integration/test_gallery_contract.py tests/python/integration/test_gallery_plugin_api.py -v --tb=short

# Run Gallery Playwright E2E tests (frontend only, with mock API)
gallery-test-playwright:
    @echo "Running Gallery Playwright E2E tests..."
    vx uv run python scripts/test_gallery_e2e.py

# Run Gallery real E2E tests (requires gallery-build first)
gallery-test-real: gallery-build
    @echo "Running Gallery real E2E tests..."
    vx uvx pytest tests/python/integration/test_gallery_real_e2e.py -v --tb=short

# Run Gallery test loop (continuous testing)
gallery-test-loop:
    @echo "Running Gallery test loop..."
    vx uv run python scripts/test_gallery_loop.py

# Run Gallery test loop in watch mode
gallery-test-watch:
    @echo "Running Gallery test loop in watch mode..."
    vx uv run python scripts/test_gallery_loop.py --watch

# Generate Gallery screenshots for documentation
gallery-screenshots:
    @echo "Generating Gallery screenshots for documentation..."
    vx uv run python scripts/test_gallery_e2e.py --screenshots-only
    @echo "[OK] Screenshots saved to docs/public/gallery/"

# Generate example screenshots for documentation
example-screenshots:
    @echo "Generating example screenshots for documentation..."
    vx uv run python scripts/screenshot_examples.py
    @echo "[OK] Screenshots saved to docs/public/examples/"

# Generate example screenshots (specific example)
example-screenshot EXAMPLE:
    @echo "Generating screenshot for: {{EXAMPLE}}..."
    vx uv run python scripts/screenshot_examples.py --example {{EXAMPLE}}

# List available examples for screenshots
example-list:
    @echo "Available examples:"
    vx uv run python scripts/screenshot_examples.py --list

# Generate all documentation screenshots (gallery + examples)
docs-screenshots: gallery-screenshots example-screenshots
    @echo "[OK] All documentation screenshots generated!"

# ═══════════════════════════════════════════════════════════════════════════════
# Packaging Commands
# ═══════════════════════════════════════════════════════════════════════════════

# Build wheel
build-wheel:
    @echo "Building Python wheel..."
    vx uv run maturin build --release --features "ext-module,python-bindings,win-webview2"
    @echo "[OK] Wheel built in target/wheels/"

# Pack Gallery into standalone executable
gallery-pack: gallery-build
    @echo "Packing Gallery into standalone executable..."
    vx cargo run -p auroraview-cli --release -- pack --config gallery/auroraview.pack.toml --build
    @echo "[OK] Gallery packed successfully!"

# Run Gallery CDP tests (build, start, test, cleanup)
gallery-cdp: gallery-pack
    @echo "=========================================="
    @echo "Gallery CDP Test Suite"
    @echo "=========================================="
    @echo ""
    @echo "[1/4] Starting Gallery with CDP enabled..."
    @powershell -File scripts/gallery_cdp_start.ps1 -ExePath "{{justfile_directory()}}\gallery\pack-output\auroraview-gallery.exe" -WorkDir "{{justfile_directory()}}\gallery\pack-output" -PidFile "{{justfile_directory()}}\.gallery-pid.tmp"
    @echo ""
    @echo "[2/4] Waiting for CDP port (9222)..."
    @powershell -File scripts/gallery_cdp_wait.ps1
    @echo ""
    @echo "[3/4] Running CDP tests..."
    -vx uvx pytest tests/test_gallery_cdp.py -v --tb=short
    @echo ""
    @echo "[4/4] Cleaning up..."
    @powershell -File scripts/gallery_cdp_stop.ps1 -PidFile "{{justfile_directory()}}\.gallery-pid.tmp"
    @echo ""
    @echo "=========================================="
    @echo "[OK] Gallery CDP tests completed!"
    @echo "=========================================="

# Run Gallery CDP tests without rebuilding (assumes gallery-pack already run)
gallery-cdp-only:
    @echo "=========================================="
    @echo "Gallery CDP Test Suite (no rebuild)"
    @echo "=========================================="
    @echo ""
    @echo "[1/3] Starting Gallery with CDP enabled..."
    @powershell -File scripts/gallery_cdp_start.ps1 -ExePath "{{justfile_directory()}}\gallery\pack-output\auroraview-gallery.exe" -WorkDir "{{justfile_directory()}}\gallery\pack-output" -PidFile "{{justfile_directory()}}\.gallery-pid.tmp"
    @echo ""
    @echo "[2/3] Waiting for CDP port (9222)..."
    @powershell -File scripts/gallery_cdp_wait.ps1
    @echo ""
    @echo "[3/3] Running CDP tests..."
    -vx uvx pytest tests/test_gallery_cdp.py -v --tb=short
    @echo ""
    @echo "Cleaning up..."
    @powershell -File scripts/gallery_cdp_stop.ps1 -PidFile "{{justfile_directory()}}\.gallery-pid.tmp"
    @echo ""
    @echo "=========================================="
    @echo "[OK] Gallery CDP tests completed!"
    @echo "=========================================="

# Pack Gallery for release (generates project without building)
gallery-pack-project: gallery-build
    @echo "Generating Gallery pack project..."
    vx cargo run -p auroraview-cli --release -- pack --config gallery/auroraview.pack.toml --output-dir target/pack
    @echo "[OK] Gallery pack project generated in target/pack/auroraview-gallery/"

# ═══════════════════════════════════════════════════════════════════════════════
# Pack Commands (Application Packaging)
# ═══════════════════════════════════════════════════════════════════════════════

# Test auroraview-pack with verbose output
test-pack-verbose:
    @echo "Testing auroraview-pack crate (verbose)..."
    vx cargo test -p auroraview-pack --lib -- --nocapture
    @echo "[OK] Pack tests passed!"

# Run clippy on pack crate
lint-pack:
    @echo "Linting auroraview-pack crate..."
    vx cargo clippy -p auroraview-pack --all-targets -- -D warnings
    @echo "[OK] Pack lint passed!"

# Pack a URL into standalone executable
pack-url URL OUTPUT="myapp":
    @echo "Packing URL: {{URL}} -> {{OUTPUT}}.exe"
    vx cargo run -p auroraview-cli --release -- pack --url "{{URL}}" --output "{{OUTPUT}}"
    @echo "[OK] Packed to target/pack/{{OUTPUT}}/"

# Pack a frontend directory into standalone executable
pack-frontend FRONTEND OUTPUT="myapp":
    @echo "Packing frontend: {{FRONTEND}} -> {{OUTPUT}}.exe"
    vx cargo run -p auroraview-cli --release -- pack --frontend "{{FRONTEND}}" --output "{{OUTPUT}}"
    @echo "[OK] Packed to target/pack/{{OUTPUT}}/"

# Pack using a config file
pack-config CONFIG:
    @echo "Packing with config: {{CONFIG}}"
    vx cargo run -p auroraview-cli --release -- pack --config "{{CONFIG}}"
    @echo "[OK] Pack completed!"

# Pack and build in one step
pack-build CONFIG:
    @echo "Packing and building with config: {{CONFIG}}"
    vx cargo run -p auroraview-cli --release -- pack --config "{{CONFIG}}" --build
    @echo "[OK] Pack and build completed!"

# Show pack info for a config file
pack-info CONFIG:
    @echo "Pack info for: {{CONFIG}}"
    vx cargo run -p auroraview-cli --release -- info --config "{{CONFIG}}"

# Clean pack output directory
pack-clean:
    @echo "Cleaning pack output..."
    rm -rf target/pack
    @echo "[OK] Pack output cleaned!"

# Full pack workflow: test, lint, then pack gallery
pack-all: test-pack lint-pack gallery-pack
    @echo "[OK] Full pack workflow completed!"

# ═══════════════════════════════════════════════════════════════════════════════
# Documentation Commands (VitePress)
# ═══════════════════════════════════════════════════════════════════════════════

# Install documentation dependencies
docs-install:
    @echo "Installing documentation dependencies..."
    cd docs; vx npm install
    @echo "[OK] Documentation dependencies installed!"

# Generate Python API documentation with pdoc
# Note: Only includes core modules that don't require third-party dependencies
# Excludes: integration/qt (requires qtpy), testing (requires pytest)
[unix]
docs-python-api:
    @echo "Generating Python API documentation with pdoc..."
    vx uv venv .docs-venv
    vx uv pip install --python .docs-venv/bin/python pdoc>=14.0.0 --quiet
    .docs-venv/bin/python -m pdoc --output-dir docs/api/python-gen --docformat google \
        python/auroraview/core \
        python/auroraview/ui \
        python/auroraview/utils \
        --no-show-source
    @echo "[OK] Python API docs generated in docs/api/python-gen/"

[windows]
docs-python-api:
    @echo "Generating Python API documentation with pdoc..."
    vx uv venv .docs-venv
    vx uv pip install --python .docs-venv\Scripts\python.exe pdoc>=14.0.0 --quiet
    .docs-venv\Scripts\python.exe -m pdoc --output-dir docs/api/python-gen --docformat google \
        python/auroraview/core \
        python/auroraview/ui \
        python/auroraview/utils \
        --no-show-source
    @echo "[OK] Python API docs generated in docs/api/python-gen/"



# Generate examples documentation from examples/ directory
docs-generate-examples:
    @echo "Generating examples documentation..."
    cd docs; vx npx tsx .vitepress/hooks/generate-examples.ts
    @echo "[OK] Examples documentation generated!"

# Start documentation dev server (auto-generates examples docs)
docs-dev: docs-install docs-generate-examples docs-python-api
    @echo "Starting documentation dev server..."
    cd docs; vx npx vitepress dev

# Alias for docs-dev
docs-serve: docs-dev

# Build documentation (auto-generates examples docs + Python API)
docs-build: docs-install docs-generate-examples docs-python-api
    @echo "Building documentation..."
    mkdir -p docs/public
    cp assets/icons/auroraview-logo-text.png docs/public/logo.png || echo "Logo not found, using placeholder"
    cd docs; vx npx vitepress build
    @echo "[OK] Documentation built in docs/.vitepress/dist/"

# Preview built documentation
docs-preview: docs-build
    @echo "Previewing documentation..."
    cd docs; vx npx vitepress preview

# Clean documentation build artifacts
docs-clean:
    @echo "Cleaning documentation build artifacts..."
    rm -rf docs/.vitepress/dist docs/.vitepress/cache docs/node_modules docs/api/python-gen .docs-venv
    @echo "[OK] Documentation artifacts cleaned!"


# ═══════════════════════════════════════════════════════════════════════════════
# MCP Server Commands
# ═══════════════════════════════════════════════════════════════════════════════

# Install MCP server dependencies
mcp-install:
    @echo "Installing MCP server dependencies..."
    cd packages/auroraview-mcp; vx uv pip install -e ".[dev]"
    @echo "[OK] MCP server dependencies installed!"

# Run MCP server in development mode
mcp-dev:
    @echo "Starting MCP server in development mode..."
    cd packages/auroraview-mcp; vx uv run auroraview-mcp

# Debug MCP server with built-in client
mcp-debug:
    @echo "Running MCP debug client..."
    cd packages/auroraview-mcp; vx uv run python scripts/debug_client.py

# Debug MCP server interactively
mcp-debug-interactive:
    @echo "Starting MCP interactive debug mode..."
    cd packages/auroraview-mcp; vx uv run python scripts/debug_client.py --test interactive

# Debug MCP server with MCP Inspector (requires Node.js)
mcp-inspector:
    @echo "Starting MCP Inspector..."
    @echo "Open http://localhost:5173 in your browser"
    vx npx @modelcontextprotocol/inspector vx uv --directory packages/auroraview-mcp run auroraview-mcp

# Run MCP server tests
mcp-test:
    @echo "Running MCP server tests..."
    cd packages/auroraview-mcp; vx uv sync --extra dev; vx uv run pytest -v

# Run MCP server tests with coverage
mcp-test-cov:
    @echo "Running MCP server tests with coverage..."
    cd packages/auroraview-mcp; vx uv sync --extra dev; vx uv run pytest --cov=auroraview_mcp --cov-report=html --cov-report=term-missing

# Lint MCP server code
mcp-lint:
    @echo "Linting MCP server code..."
    cd packages/auroraview-mcp; vx uvx ruff check src/ tests/
    cd packages/auroraview-mcp; vx uvx ruff format --check src/ tests/

# Format MCP server code
mcp-format:
    @echo "Formatting MCP server code..."
    cd packages/auroraview-mcp; vx uvx ruff format src/ tests/

# Build MCP server package
mcp-build:
    @echo "Building MCP server package..."
    cd packages/auroraview-mcp; vx uv build
    @echo "[OK] MCP server package built!"

# Full MCP CI check
mcp-ci: mcp-install mcp-lint mcp-test
    @echo "[OK] MCP CI check passed!"