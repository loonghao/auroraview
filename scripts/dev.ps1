# Development script for DCC WebView
# Usage: .\scripts\dev.ps1 [command]
# Commands:
#   build    - Build the package
#   test     - Run tests
#   fmt      - Format code
#   lint     - Run linters
#   clean    - Clean build artifacts

param(
    [Parameter(Position=0)]
    [string]$Command = "build"
)

function Show-Help {
    Write-Host "DCC WebView Development Script" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Usage: .\scripts\dev.ps1 [command]" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Commands:" -ForegroundColor Cyan
    Write-Host "  build    - Build the package (default)"
    Write-Host "  test     - Run tests"
    Write-Host "  fmt      - Format code (Rust + Python)"
    Write-Host "  lint     - Run linters"
    Write-Host "  clean    - Clean build artifacts"
    Write-Host "  help     - Show this help message"
}

function Build-Package {
    Write-Host " Building package..." -ForegroundColor Cyan
    maturin develop --release
}

function Run-Tests {
    Write-Host "Running tests..." -ForegroundColor Cyan

    Write-Host ""
    Write-Host "Running Rust tests..." -ForegroundColor Yellow
    cargo test

    Write-Host ""
    Write-Host "Running Python tests..." -ForegroundColor Yellow
    uv run pytest tests/ -v

    Write-Host ""
    Write-Host "Running standalone test..." -ForegroundColor Yellow
    uv run python examples/standalone_test.py
}

function Format-Code {
    Write-Host "Formatting code..." -ForegroundColor Cyan

    Write-Host ""
    Write-Host "Formatting Rust code..." -ForegroundColor Yellow
    cargo fmt --all

    Write-Host ""
    Write-Host "Formatting Python code..." -ForegroundColor Yellow
    uv run ruff format python/ tests/ examples/
}

function Run-Linters {
    Write-Host "Running linters..." -ForegroundColor Cyan

    Write-Host ""
    Write-Host "Running Rust clippy..." -ForegroundColor Yellow
    cargo clippy --all-targets --all-features

    Write-Host ""
    Write-Host "Running Python ruff..." -ForegroundColor Yellow
    uv run ruff check python/ tests/ examples/
}

function Clean-Artifacts {
    Write-Host "Cleaning build artifacts..." -ForegroundColor Cyan

    if (Test-Path "target") {
        Remove-Item -Recurse -Force "target"
        Write-Host "[OK] Removed target/" -ForegroundColor Green
    }

    if (Test-Path "dist") {
        Remove-Item -Recurse -Force "dist"
        Write-Host "[OK] Removed dist/" -ForegroundColor Green
    }

    Get-ChildItem -Recurse -Filter "__pycache__" | Remove-Item -Recurse -Force
    Write-Host "[OK] Removed __pycache__/" -ForegroundColor Green

    Get-ChildItem -Recurse -Filter "*.pyc" | Remove-Item -Force
    Write-Host "[OK] Removed *.pyc files" -ForegroundColor Green

    Write-Host ""
    Write-Host "Clean completed!" -ForegroundColor Green
}

# Main script logic
switch ($Command.ToLower()) {
    "build" { Build-Package }
    "test" { Run-Tests }
    "fmt" { Format-Code }
    "lint" { Run-Linters }
    "clean" { Clean-Artifacts }
    "help" { Show-Help }
    default {
        Write-Host "[ERROR] Unknown command: $Command" -ForegroundColor Red
        Write-Host ""
        Show-Help
        exit 1
    }
}

