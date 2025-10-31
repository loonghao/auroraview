# Build script for DCC WebView on Windows
# Usage: .\scripts\build.ps1

Write-Host "[LAUNCH] Building DCC WebView..." -ForegroundColor Cyan
Write-Host ""

# Check if uv is installed
if (-not (Get-Command uv -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] Error: uv is not installed" -ForegroundColor Red
    Write-Host "Please install uv: https://github.com/astral-sh/uv" -ForegroundColor Yellow
    exit 1
}

# Check if Rust is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] Error: Rust is not installed" -ForegroundColor Red
    Write-Host "Please install Rust: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

Write-Host "[OK] Prerequisites check passed" -ForegroundColor Green
Write-Host ""

# Install maturin if not already installed
Write-Host "Installing maturin..." -ForegroundColor Cyan
uv pip install maturin

# Build the Rust extension
Write-Host ""
Write-Host "Building Rust extension..." -ForegroundColor Cyan
uv run maturin develop --release

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "Build completed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "You can now test the package:" -ForegroundColor Cyan
    Write-Host "  uv run python examples/standalone_test.py" -ForegroundColor Yellow
} else {
    Write-Host ""
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

