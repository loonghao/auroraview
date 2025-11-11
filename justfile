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
    uv run maturin develop --features win-webview2

# Build with release optimizations
build-release:
    @echo "Building release version..."
    uv run maturin develop --release --features win-webview2

# Rebuild and install in development mode (recommended)
rebuild-core:
    @echo "Rebuilding Rust core with maturin..."
    uv run maturin develop --release --features win-webview2
    @echo "[OK] Core module rebuilt and installed successfully!"

# Rebuild with verbose output
rebuild-core-verbose:
    @echo "Rebuilding Rust core with maturin (verbose)..."
    uv run maturin develop --release --features win-webview2 --verbose
    @echo "[OK] Core module rebuilt and installed successfully!"

# Run all tests
test:
    @echo "Running Rust unit tests..."
    cargo test --lib
    @echo "Running Rust integration tests..."
    cargo test --test '*'
    @echo "Running Rust doc tests..."
    cargo test --doc
    @echo "Running Python tests with coverage..."
    pytest -q -rA -s --cov=auroraview --cov-report=term-missing tests/test_package_init.py tests/test_testing_framework.py tests/test_event_timer.py

# Run tests with coverage
test-cov:
    @echo "Running tests with coverage..."
    pytest -v --cov=auroraview --cov-report=html --cov-report=term-missing tests/test_package_init.py tests/test_testing_framework.py tests/test_event_timer.py

# Run only fast tests (exclude slow tests)
test-fast:
    @echo "Running fast tests..."
    pytest tests/ -v -m "not slow"

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
    @echo "Running Python unit tests..."
    pytest tests/ -v -m "unit" --ignore=tests/unit --ignore=tests/integration --ignore=tests/common

# Run only Rust integration tests
test-integration:
    @echo "Running Rust integration tests..."
    cargo test --test '*'
    @echo "Running Python integration tests..."
    pytest tests/ -v -m "integration" --ignore=tests/unit --ignore=tests/integration --ignore=tests/common

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
    @echo "Installing CI dependencies..."
    uv sync --group dev --group test

ci-build:
    @echo "Building extension for CI..."
    uv pip install maturin
    uv run maturin develop --features win-webview2

ci-test-rust:
    @echo "Running Rust doc tests..."
    @echo "Note: lib tests are skipped due to abi3 linking issues with PyO3"
    @echo "      Python tests provide comprehensive coverage instead"
    cargo test --doc

ci-test-python:
    @echo "Running Python unit tests..."
    uv run pytest tests/ -v --tb=short -m "not slow"

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
    @echo "Running Python tests with coverage (headless subset)..."
    pytest -v --cov=auroraview --cov-report=html --cov-report=term-missing --cov-report=xml tests/test_package_init.py tests/test_testing_framework.py tests/test_event_timer.py
# Shortcut alias for Python coverage
pycov:
    @echo "[Alias] Running Python coverage via coverage-python..."
    @just coverage-python


coverage-rust:
	@echo "Running Rust tests with coverage (preferring cargo-llvm-cov) in headless mode..."
	if (Get-Command py -ErrorAction SilentlyContinue) { $pyBase = py -c "import sys; print(sys.base_prefix)" } else { $pyBase = python -c "import sys; print(sys.base_prefix)" }; $env:Path = "$pyBase;$pyBase\DLLs;$pyBase\bin;$env:Path"; if (Get-Command cargo-llvm-cov -ErrorAction SilentlyContinue) { $ignore = "(src[/\\]webview[/\\](aurora_view\\.rs|embedded\\.rs|standalone\\.rs|protocol\\.rs|timer_bindings\\.rs|webview_inner\\.rs|backend[/\\].*|platform[/\\].*)|src[/\\]service_discovery[/\\]mdns_service\\.rs)"; if ($env:CI -eq "true") { rustup component add llvm-tools-preview; cargo llvm-cov --workspace --html --tests --no-default-features --features "python-bindings threaded-ipc test-helpers" --ignore-filename-regex $ignore --fail-under-lines 50 } else { cargo llvm-cov --workspace --html --tests --no-default-features --features "python-bindings threaded-ipc test-helpers" --ignore-filename-regex $ignore; $json = cargo llvm-cov --summary-only --json --workspace --tests --no-default-features --features "python-bindings threaded-ipc test-helpers" --ignore-filename-regex $ignore | Out-String | ConvertFrom-Json; $covered = [double]$json.data[0].totals.lines.covered; $count = [double]$json.data[0].totals.lines.count; if ($count -gt 0) { $lines = [math]::Round((100.0 * $covered / $count), 2) } else { $lines = 0 }; if ($lines -ge 50) { echo ("[OK] Rust coverage lines: {0}% (>=50) report: target/llvm-cov/html/index.html" -f $lines) } else { echo ("[WARN] Rust coverage lines: {0}% (<50)" -f $lines) } } } elseif (Get-Command cargo-tarpaulin -ErrorAction SilentlyContinue) { cargo tarpaulin --no-default-features --features "python-bindings threaded-ipc test-helpers" --out Html --out Xml --output-dir target/tarpaulin; if ($LASTEXITCODE -eq 0) { echo "[OK] Rust coverage report: target/tarpaulin/tarpaulin-report.html" } else { echo "[WARN] cargo-tarpaulin exited with code $LASTEXITCODE" } } else { echo "[INFO] No Rust coverage tool found."; echo "      Install recommended: cargo install cargo-llvm-cov"; echo "      Also run: rustup component add llvm-tools-preview" }

coverage-all: coverage-rust coverage-python
    @echo "All coverage reports generated!"

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
    uv run maturin build --release --features win-webview2
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

