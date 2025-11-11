# AuroraView WebSocket Server Launcher
# PowerShell script for Windows

Write-Host "🚀 Starting AuroraView WebSocket Server..." -ForegroundColor Cyan
Write-Host ""

# Check if Rust is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Error: Rust is not installed!" -ForegroundColor Red
    Write-Host "Please install Rust from: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Check if we're in the correct directory
if (-not (Test-Path "Cargo.toml")) {
    Write-Host "❌ Error: Cargo.toml not found!" -ForegroundColor Red
    Write-Host "Please run this script from the examples/photoshop_examples directory" -ForegroundColor Yellow
    exit 1
}

# Build the project
Write-Host "📦 Building project..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Build successful!" -ForegroundColor Green
Write-Host ""

# Run the server
Write-Host "🌐 Starting WebSocket server on ws://localhost:9001" -ForegroundColor Cyan
Write-Host "📡 Waiting for Photoshop UXP plugin to connect..." -ForegroundColor Cyan
Write-Host ""
Write-Host "Press Ctrl+C to stop the server" -ForegroundColor Yellow
Write-Host ""

$env:RUST_LOG = "info"
cargo run --bin websocket_server

