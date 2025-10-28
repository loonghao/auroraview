# justfile for AuroraView development
# Run `just --list` to see all available commands

# Set shell for Windows compatibility
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set shell := ["sh", "-c"]

# Default recipe to display help
default:
    @just --list

# Install dependencies
install:
    @echo "📦 Installing dependencies..."
    uv sync --group dev

# Build the extension module
build:
    @echo "🔧 Building extension module..."
    uv run maturin develop

# Build with release optimizations
build-release:
    @echo "🚀 Building release version..."
    uv run maturin develop --release

# Run all tests
test:
    @echo "🧪 Running Rust doc tests..."
    @echo "⚠️  Note: lib tests are skipped due to abi3 linking issues with PyO3"
    cargo test --doc
    @echo "🧪 Running Python tests..."
    uv run pytest tests/ -v

# Run tests with coverage
test-cov:
    @echo "🧪 Running tests with coverage..."
    uv run pytest tests/ -v --cov=auroraview --cov-report=html --cov-report=term-missing

# Run only fast tests (exclude slow tests)
test-fast:
    @echo "🧪 Running fast tests..."
    uv run pytest tests/ -v -m "not slow"

# Test with Python 3.7
test-py37:
    @echo "🧪 Testing with Python 3.7..."
    uv venv --python 3.7 .venv-py37
    uv pip install -e . pytest pytest-cov --python .venv-py37\Scripts\python.exe
    .venv-py37\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.8
test-py38:
    @echo "🧪 Testing with Python 3.8..."
    uv venv --python 3.8 .venv-py38
    uv pip install -e . pytest pytest-cov --python .venv-py38\Scripts\python.exe
    .venv-py38\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.9
test-py39:
    @echo "🧪 Testing with Python 3.9..."
    uv venv --python 3.9 .venv-py39
    uv pip install -e . pytest pytest-cov --python .venv-py39\Scripts\python.exe
    .venv-py39\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.10
test-py310:
    @echo "🧪 Testing with Python 3.10..."
    uv venv --python 3.10 .venv-py310
    uv pip install -e . pytest pytest-cov --python .venv-py310\Scripts\python.exe
    .venv-py310\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.11
test-py311:
    @echo "🧪 Testing with Python 3.11..."
    uv venv --python 3.11 .venv-py311
    uv pip install -e . pytest pytest-cov --python .venv-py311\Scripts\python.exe
    .venv-py311\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with Python 3.12
test-py312:
    @echo "🧪 Testing with Python 3.12..."
    uv venv --python 3.12 .venv-py312
    uv pip install -e . pytest pytest-cov --python .venv-py312\Scripts\python.exe
    .venv-py312\Scripts\python.exe -m pytest tests/ -v -o addopts=""

# Test with all supported Python versions
test-all-python:
    @echo "🧪 Testing with all supported Python versions..."
    just test-py37
    just test-py38
    just test-py39
    just test-py310
    just test-py311
    just test-py312
    @echo "✅ All Python versions tested successfully!"

# Run only unit tests
test-unit:
    @echo "🧪 Running unit tests..."
    uv run pytest tests/ -v -m "unit"

# Run only integration tests
test-integration:
    @echo "🧪 Running integration tests..."
    uv run pytest tests/ -v -m "integration"

# Run specific test file
test-file FILE:
    @echo "🧪 Running tests in {{FILE}}..."
    uv run pytest {{FILE}} -v

# Run tests with specific marker
test-marker MARKER:
    @echo "🧪 Running tests with marker {{MARKER}}..."
    uv run pytest tests/ -v -m {{MARKER}}

# Format code
format:
    @echo "🎨 Formatting Rust code..."
    cargo fmt --all
    @echo "🎨 Formatting Python code..."
    uv run ruff format python/ tests/ examples/

# Run linting
lint:
    @echo "🔍 Linting Rust code..."
    cargo clippy --all-targets --all-features -- -D warnings
    @echo "🔍 Linting Python code..."
    uv run ruff check python/ tests/ examples/

# Fix linting issues automatically
fix:
    @echo "🔧 Fixing linting issues..."
    cargo clippy --fix --allow-dirty --allow-staged
    uv run ruff check --fix python/ tests/ examples/

# Run all checks (format, lint, test)
check: format lint test
    @echo "✅ All checks passed!"

# CI-specific commands
ci-install:
    @echo "📦 Installing CI dependencies..."
    uv sync --group dev --group test

ci-build:
    @echo "🔧 Building extension for CI..."
    uv pip install maturin
    uv run maturin develop

ci-test-rust:
    @echo "🧪 Running Rust doc tests..."
    @echo "⚠️  Note: lib tests are skipped due to abi3 linking issues with PyO3"
    @echo "     Python tests provide comprehensive coverage instead"
    cargo test --doc

ci-test-python:
    @echo "🧪 Running Python unit tests..."
    uv run pytest tests/ -v --tb=short -m "not slow"

ci-test-basic:
    @echo "🧪 Running basic import tests..."
    uv run python -c "import auroraview; print('AuroraView imported successfully')"

ci-lint:
    @echo "🔍 Running CI linting..."
    cargo fmt --all -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    uv run ruff check python/ tests/
    uv run ruff format --check python/ tests/

# Coverage commands
coverage-python:
    @echo "📊 Running Python tests with coverage..."
    uv run pytest tests/ -v --cov=auroraview --cov-report=html --cov-report=term-missing --cov-report=xml

coverage-rust:
    @echo "📊 Running Rust tests with coverage..."
    @if command -v cargo-tarpaulin >/dev/null 2>&1; then \
        cargo tarpaulin --out Html --out Xml --output-dir target/tarpaulin; \
    else \
        echo "⚠️  cargo-tarpaulin not installed. Installing..."; \
        cargo install cargo-tarpaulin; \
        cargo tarpaulin --out Html --out Xml --output-dir target/tarpaulin; \
    fi

coverage-all: coverage-rust coverage-python
    @echo "📊 All coverage reports generated!"

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean
    rm -rf dist/ build/ htmlcov/
    find . -type d -name "__pycache__" -exec rm -rf {} +
    find . -type f -name "*.pyc" -delete
    find . -type f -name "*.pyo" -delete
    find . -type f -name "*.so" -delete
    find . -type f -name "*.pyd" -delete

# Setup development environment
dev: install build
    @echo "🚀 Development environment ready!"
    @echo "💡 Try: just test"

# Build release wheels
release:
    @echo "📦 Building release wheels..."
    uv run maturin build --release
    @echo "✅ Wheels built in target/wheels/"

# Run examples
example EXAMPLE:
    @echo "🚀 Running example: {{EXAMPLE}}"
    uv run python examples/{{EXAMPLE}}.py

# Show project info
info:
    @echo "📊 Project Information:"
    @echo "  Rust version: $(rustc --version)"
    @echo "  Python version: $(python --version)"
    @echo "  UV version: $(uv --version)"

# Run security audit
audit:
    @echo "🔒 Running security audit..."
    cargo audit

# Documentation
docs:
    @echo "📚 Building documentation..."
    cargo doc --no-deps --document-private-items --open

# Comprehensive checks
check-all: format lint test coverage-all
    @echo "🎉 All checks completed!"

