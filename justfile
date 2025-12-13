# justfile for AuroraView development
# Run `just --list` to see all available commands
#
# Quick Start:
#   just rebuild-core        - Rebuild Rust core with maturin (release mode)
#   just rebuild-core-verbose - Same as above with verbose output
#   just test                - Run all tests
#   just format              - Format code
#   just lint                - Run linting

# Set shell for Windows compatibility
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set shell := ["sh", "-c"]

# Default recipe to display help
default:
    @just --list

# Install dependencies
install:
    @echo "Installing dependencies..."
    uv sync --group dev

# Build the extension module
build:
    @echo "Building extension module..."
    uv run maturin develop --features "ext-module,python-bindings,abi3-py38,win-webview2"

# Build with release optimizations
build-release:
    @echo "Building release version..."
    uv run maturin develop --release --features "ext-module,python-bindings,abi3-py38,win-webview2"

# Build Python library (PyO3 bindings)
rebuild-pylib:
    @echo "Building Python library with maturin..."
    uv run maturin develop --release --features "ext-module,python-bindings,abi3-py38,win-webview2"
    @echo "[OK] Python library rebuilt and installed successfully!"

# Build Python library with verbose output
rebuild-pylib-verbose:
    @echo "Building Python library with maturin (verbose)..."
    uv run maturin develop --release --features "ext-module,python-bindings,abi3-py38,win-webview2" --verbose
    @echo "[OK] Python library rebuilt and installed successfully!"

# Build CLI binary
build-cli:
    @echo "Building CLI binary..."
    cargo build -p auroraview-cli --release
    @echo "[OK] CLI built: target/release/auroraview.exe"

# Build all workspace crates
build-all:
    @echo "Building all workspace crates..."
    cargo build -p auroraview-core
    cargo build -p auroraview-pack
    cargo build -p auroraview-cli --release
    uv run maturin develop --release --features "ext-module,python-bindings,abi3-py38,win-webview2"
    @echo "[OK] All crates built successfully!"

# Run all tests
test:
    @echo "Running workspace crate tests..."
    cargo test -p auroraview-core
    cargo test -p auroraview-pack
    cargo test -p auroraview-cli
    @echo "Running Rust integration tests (with rstest)..."
    cargo test --test mdns_integration_tests --features "test-helpers"
    cargo test --test parent_monitor_integration_tests --features "test-helpers"
    cargo test --test protocol_handlers_integration_tests --features "test-helpers"
    cargo test --test protocol_integration_tests --features "test-helpers"
    cargo test --test timer_integration_tests --features "test-helpers"
    cargo test --test ipc_message_queue_integration_tests --features "test-helpers"
    cargo test --test http_discovery_integration_tests --features "test-helpers"
    cargo test --test standalone_integration_tests --features "test-helpers"
    @echo "Running Rust doc tests..."
    cargo test --doc
    @echo "Running Python tests..."
    pytest -q -rA tests/python/unit tests/python/integration
    @echo ""
    @echo "Note: Rust unit tests (cargo test --lib), window_utils_integration_tests, and ipc_batch_integration_tests are skipped on Windows due to PyO3 abi3 DLL linking issues."
    @echo "These tests run successfully in CI on Linux."

# Run tests with coverage
test-cov:
    @echo "Running tests with coverage..."
    pytest -v --cov=auroraview --cov-report=html --cov-report=term-missing tests/python/unit tests/python/integration

# Run only fast tests (exclude slow tests)
test-fast:
    @echo "Running fast tests..."
    pytest tests/python/ -v -m "not slow"

# Test with Python 3.7
test-py37:
    @echo "Testing with Python 3.7..."
    uv venv --python 3.7 .venv-py37
    uv pip install -e . pytest pytest-cov --python .venv-py37\Scripts\python.exe
    .venv-py37\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.8
test-py38:
    @echo "Testing with Python 3.8..."
    uv venv --python 3.8 .venv-py38
    uv pip install -e . pytest pytest-cov --python .venv-py38\Scripts\python.exe
    .venv-py38\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.9
test-py39:
    @echo "Testing with Python 3.9..."
    uv venv --python 3.9 .venv-py39
    uv pip install -e . pytest pytest-cov --python .venv-py39\Scripts\python.exe
    .venv-py39\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.10
test-py310:
    @echo "Testing with Python 3.10..."
    uv venv --python 3.10 .venv-py310
    uv pip install -e . pytest pytest-cov --python .venv-py310\Scripts\python.exe
    .venv-py310\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.11
test-py311:
    @echo "Testing with Python 3.11..."
    uv venv --python 3.11 .venv-py311
    uv pip install -e . pytest pytest-cov --python .venv-py311\Scripts\python.exe
    .venv-py311\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.12
test-py312:
    @echo "Testing with Python 3.12..."
    uv venv --python 3.12 .venv-py312
    uv pip install -e . pytest pytest-cov --python .venv-py312\Scripts\python.exe
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
    uvx nox -s pytest

nox-qt:
    @echo "Running nox session: pytest-qt (multi-Python with Qt)"
    uvx nox -s pytest-qt

nox-all:
    @echo "Running nox session: pytest-all (full suite)"
    uvx nox -s pytest-all


# Run only Rust unit tests
test-unit:
    @echo "Running Rust unit tests..."
    cargo test --lib
    cargo test -p auroraview-core
    cargo test -p auroraview-pack
    cargo test -p auroraview-cli
    @echo "Running Python unit tests..."
    pytest tests/python/unit -v

# Run only Rust integration tests
test-integration:
    @echo "Running Rust integration tests (with rstest)..."
    cargo test --test '*' --features "test-helpers"
    @echo "Running Python integration tests..."
    pytest tests/python/integration -v

# Watch mode for continuous testing
test-watch:
    @echo "Running tests in watch mode..."
    cargo watch -x test

# Run specific test file
test-file FILE:
    @echo "Running tests in {{FILE}}..."
    pytest {{FILE}} -v

# Run tests with specific marker
test-marker MARKER:
    @echo "Running tests with marker {{MARKER}}..."
    pytest tests/ -v -m {{MARKER}}

# Format code
format:
    @echo "Formatting Rust code..."
    cargo fmt --all
    @echo "Formatting Python code..."
    uv run ruff format python/ tests/ examples/

# Run linting
lint:
    @echo "Linting Rust code..."
    cargo clippy --all-targets --all-features -- -D warnings
    @echo "Linting Python code..."
    uv run ruff check python/ tests/ examples/

# Fix linting issues automatically
fix:
    @echo "Fixing linting issues..."
    cargo clippy --fix --allow-dirty --allow-staged
    uv run ruff check --fix python/ tests/ examples/

# Run all checks (format, lint, test)
check: format lint test
    @echo "All checks passed!"

# CI-specific commands
ci-install:
    @echo "Installing CI dependencies (including Qt)..."
    uv sync --group dev --group test
    uv pip install qtpy PySide6 pytest-qt

# CI build command - consistent across all platforms
# Uses ext-module for proper Python extension module compilation
[unix]
ci-build:
    @echo "Building extension for CI (Unix)..."
    uv pip install maturin
    uv run maturin develop --features "ext-module,python-bindings,abi3-py38"

[windows]
ci-build:
    @echo "Building extension for CI (Windows)..."
    uv pip install maturin
    uv run maturin develop --features "ext-module,python-bindings,abi3-py38,win-webview2"

ci-test-rust:
    @echo "Running Rust doc tests..."
    @echo "Note: lib tests are skipped due to abi3 linking issues with PyO3"
    @echo "      Python tests provide comprehensive coverage instead"
    cargo test --doc

ci-test-python:
    @echo "Running Python unit tests with coverage..."
    uv run pytest tests/ -v --tb=short -m "not slow" \
        --cov=auroraview \
        --cov-report=term-missing \
        --cov-report=html \
        --cov-report=xml \
        --cov-fail-under=0

ci-test-basic:
    @echo "Running basic import tests..."
    uv run python -c "import auroraview; print('AuroraView imported successfully')"

ci-lint:
    @echo "Running CI linting..."
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    uvx ruff check python/ tests/
    uvx ruff format --check python/ tests/

# Coverage commands
coverage-python:
    @echo "Running Python tests with coverage..."
    pytest -v --cov=auroraview --cov-report=html --cov-report=term-missing --cov-report=xml tests/python/unit tests/python/integration
# Shortcut alias for Python coverage
pycov:
    @echo "[Alias] Running Python coverage via coverage-python..."
    @just coverage-python


coverage-rust:
	@echo "Running Rust tests with coverage (preferring cargo-llvm-cov) in headless mode..."
	if (Get-Command py -ErrorAction SilentlyContinue) { $pyBase = py -c "import sys; print(sys.base_prefix)" } else { $pyBase = python -c "import sys; print(sys.base_prefix)" }; $env:Path = "$pyBase;$pyBase\DLLs;$pyBase\bin;$env:Path"; if (Get-Command cargo-llvm-cov -ErrorAction SilentlyContinue) { $ignore = "(src[/\\]webview[/\\](aurora_view\\.rs|embedded\\.rs|standalone\\.rs|protocol\\.rs|timer_bindings\\.rs|webview_inner\\.rs|backend[/\\].*|platform[/\\].*)|src[/\\]service_discovery[/\\]mdns_service\\.rs)"; if ($env:CI -eq "true") { rustup component add llvm-tools-preview; cargo llvm-cov --workspace --html --tests --no-default-features --features "python-bindings threaded-ipc test-helpers" --ignore-filename-regex $ignore --fail-under-lines 50 } else { cargo llvm-cov --workspace --html --tests --no-default-features --features "python-bindings threaded-ipc test-helpers" --ignore-filename-regex $ignore; $json = cargo llvm-cov --summary-only --json --workspace --tests --no-default-features --features "python-bindings threaded-ipc test-helpers" --ignore-filename-regex $ignore | Out-String | ConvertFrom-Json; $covered = [double]$json.data[0].totals.lines.covered; $count = [double]$json.data[0].totals.lines.count; if ($count -gt 0) { $lines = [math]::Round((100.0 * $covered / $count), 2) } else { $lines = 0 }; if ($lines -ge 50) { echo ("[OK] Rust coverage lines: {0}% (>=50) report: target/llvm-cov/html/index.html" -f $lines) } else { echo ("[WARN] Rust coverage lines: {0}% (<50)" -f $lines) } } } elseif (Get-Command cargo-tarpaulin -ErrorAction SilentlyContinue) { cargo tarpaulin --no-default-features --features "python-bindings threaded-ipc test-helpers" --out Html --out Xml --output-dir target/tarpaulin; if ($LASTEXITCODE -eq 0) { echo "[OK] Rust coverage report: target/tarpaulin/tarpaulin-report.html" } else { echo "[WARN] cargo-tarpaulin exited with code $LASTEXITCODE" } } else { echo "[INFO] No Rust coverage tool found."; echo "      Install recommended: cargo install cargo-llvm-cov"; echo "      Also run: rustup component add llvm-tools-preview" }

# Run coverage for individual crates
coverage-crate CRATE:
    @echo "Running coverage for crate: {{CRATE}}..."
    cargo llvm-cov --package {{CRATE}} --html --tests --output-dir target/llvm-cov/{{CRATE}}
    @echo "[OK] Coverage report: target/llvm-cov/{{CRATE}}/html/index.html"

# Run coverage for auroraview-core crate
coverage-core:
    @echo "Running coverage for auroraview-core..."
    cargo llvm-cov --package auroraview-core --html --tests --output-dir target/llvm-cov/auroraview-core
    @echo "[OK] Coverage report: target/llvm-cov/auroraview-core/html/index.html"

# Run coverage for auroraview-pack crate
coverage-pack:
    @echo "Running coverage for auroraview-pack..."
    cargo llvm-cov --package auroraview-pack --html --tests --output-dir target/llvm-cov/auroraview-pack
    @echo "[OK] Coverage report: target/llvm-cov/auroraview-pack/html/index.html"

# Run coverage for auroraview-cli crate
coverage-cli:
    @echo "Running coverage for auroraview-cli..."
    cargo llvm-cov --package auroraview-cli --html --tests --output-dir target/llvm-cov/auroraview-cli
    @echo "[OK] Coverage report: target/llvm-cov/auroraview-cli/html/index.html"

# Run coverage for all crates with lcov output (for CI)
coverage-rust-lcov:
    @echo "Running Rust coverage with lcov output..."
    cargo llvm-cov --workspace --lcov --output-path rust-coverage.lcov --tests --no-default-features --features "python-bindings threaded-ipc test-helpers"
    @echo "[OK] Coverage report: rust-coverage.lcov"

coverage-all: coverage-rust coverage-python
    @echo "All coverage reports generated!"

# Run benchmarks
bench:
    @echo "Running benchmarks..."
    cargo bench --bench ipc_bench

# Run benchmarks and save baseline
bench-save BASELINE="main":
    @echo "Running benchmarks and saving baseline: {{BASELINE}}..."
    cargo bench --bench ipc_bench -- --save-baseline {{BASELINE}}

# Compare benchmarks against baseline
bench-compare BASELINE="main":
    @echo "Comparing benchmarks against baseline: {{BASELINE}}..."
    cargo bench --bench ipc_bench -- --baseline {{BASELINE}}

# Clean build artifacts
clean:
    @echo "Cleaning build artifacts..."
    cargo clean
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
    uv run maturin build --release --features "ext-module,python-bindings,win-webview2"
    @echo "Wheels built in target/wheels/"

# Run examples
example EXAMPLE:
    @echo "Running example: {{EXAMPLE}}"
    uv run python examples/{{EXAMPLE}}.py

# Show project info
info:
    @echo "Project Information:"
    @echo "  Rust version: $(rustc --version)"
    @echo "  Python version: $(python --version)"
    @echo "  UV version: $(uv --version)"

# Run security audit
audit:
    @echo "Running security audit..."
    cargo audit

# Documentation
docs:
    @echo "Building documentation..."
    cargo doc --no-deps --document-private-items --open

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
# Gallery Commands
# ═══════════════════════════════════════════════════════════════════════════════

# Build gallery frontend
gallery-build:
    @echo "Building gallery frontend..."
    cd gallery; npm install; npm run build
    @echo "[OK] Gallery built in gallery/dist/"

# Run gallery (build frontend first, then launch with AuroraView)
gallery: gallery-build
    @echo "Starting AuroraView Gallery..."
    uv run python gallery/main.py

# Run gallery dev server (for frontend development)
gallery-dev:
    @echo "Starting gallery dev server..."
    cd gallery; npm run dev

# Run Gallery E2E tests
gallery-test:
    @echo "Running Gallery E2E tests..."
    uv run pytest tests/python/integration/test_gallery_e2e.py tests/python/integration/test_gallery_contract.py tests/python/integration/test_gallery_plugin_api.py -v --tb=short

# Run Gallery Playwright E2E tests (frontend only, with mock API)
gallery-test-playwright:
    @echo "Running Gallery Playwright E2E tests..."
    uv run python scripts/test_gallery_e2e.py

# Run Gallery real E2E tests (requires gallery-build first)
gallery-test-real: gallery-build
    @echo "Running Gallery real E2E tests..."
    uv run pytest tests/python/integration/test_gallery_real_e2e.py -v --tb=short

# Run Gallery test loop (continuous testing)
gallery-test-loop:
    @echo "Running Gallery test loop..."
    uv run python scripts/test_gallery_loop.py

# Run Gallery test loop in watch mode
gallery-test-watch:
    @echo "Running Gallery test loop in watch mode..."
    uv run python scripts/test_gallery_loop.py --watch

# ═══════════════════════════════════════════════════════════════════════════════
# Packaging Commands
# ═══════════════════════════════════════════════════════════════════════════════

# Build wheel
build-wheel:
    @echo "Building Python wheel..."
    uv run maturin build --release --features "ext-module,python-bindings,win-webview2"
    @echo "[OK] Wheel built in target/wheels/"

# Pack Gallery into standalone executable
gallery-pack: gallery-build
    @echo "Packing Gallery into standalone executable..."
    cargo run -p auroraview-cli --release -- pack --config gallery/auroraview.pack.toml --build
    @echo "[OK] Gallery packed successfully!"

# Pack Gallery for release (generates project without building)
gallery-pack-project: gallery-build
    @echo "Generating Gallery pack project..."
    cargo run -p auroraview-cli --release -- pack --config gallery/auroraview.pack.toml --output-dir target/pack
    @echo "[OK] Gallery pack project generated in target/pack/auroraview-gallery/"

# ═══════════════════════════════════════════════════════════════════════════════
# Pack Commands (Application Packaging)
# ═══════════════════════════════════════════════════════════════════════════════

# Test auroraview-pack crate
test-pack:
    @echo "Testing auroraview-pack crate..."
    cargo test -p auroraview-pack --lib
    @echo "[OK] Pack tests passed!"

# Test auroraview-pack with verbose output
test-pack-verbose:
    @echo "Testing auroraview-pack crate (verbose)..."
    cargo test -p auroraview-pack --lib -- --nocapture
    @echo "[OK] Pack tests passed!"

# Run clippy on pack crate
lint-pack:
    @echo "Linting auroraview-pack crate..."
    cargo clippy -p auroraview-pack --all-targets -- -D warnings
    @echo "[OK] Pack lint passed!"

# Pack a URL into standalone executable
pack-url URL OUTPUT="myapp":
    @echo "Packing URL: {{URL}} -> {{OUTPUT}}.exe"
    cargo run -p auroraview-cli --release -- pack --url "{{URL}}" --output "{{OUTPUT}}"
    @echo "[OK] Packed to target/pack/{{OUTPUT}}/"

# Pack a frontend directory into standalone executable
pack-frontend FRONTEND OUTPUT="myapp":
    @echo "Packing frontend: {{FRONTEND}} -> {{OUTPUT}}.exe"
    cargo run -p auroraview-cli --release -- pack --frontend "{{FRONTEND}}" --output "{{OUTPUT}}"
    @echo "[OK] Packed to target/pack/{{OUTPUT}}/"

# Pack using a config file
pack-config CONFIG:
    @echo "Packing with config: {{CONFIG}}"
    cargo run -p auroraview-cli --release -- pack --config "{{CONFIG}}"
    @echo "[OK] Pack completed!"

# Pack and build in one step
pack-build CONFIG:
    @echo "Packing and building with config: {{CONFIG}}"
    cargo run -p auroraview-cli --release -- pack --config "{{CONFIG}}" --build
    @echo "[OK] Pack and build completed!"

# Show pack info for a config file
pack-info CONFIG:
    @echo "Pack info for: {{CONFIG}}"
    cargo run -p auroraview-cli --release -- info --config "{{CONFIG}}"

# Clean pack output directory
pack-clean:
    @echo "Cleaning pack output..."
    rm -rf target/pack
    @echo "[OK] Pack output cleaned!"

# Full pack workflow: test, lint, then pack gallery
pack-all: test-pack lint-pack gallery-pack
    @echo "[OK] Full pack workflow completed!"