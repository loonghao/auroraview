# Quick Start Script for Photoshop + AuroraView Integration
# PowerShell script for Windows

Write-Host "🚀 Starting Photoshop + AuroraView Integration..." -ForegroundColor Cyan
Write-Host ""

# Check Python
if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Python not found! Please install Python 3.8+" -ForegroundColor Red
    exit 1
}

# Check Node.js
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Node.js not found! Please install Node.js 18+" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Python and Node.js found" -ForegroundColor Green
Write-Host ""

# Install Python dependencies
Write-Host "📦 Installing Python dependencies..." -ForegroundColor Yellow
Set-Location python
python -m pip install -r requirements.txt
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Failed to install Python dependencies" -ForegroundColor Red
    exit 1
}
Set-Location ..

# Install UI dependencies
Write-Host "📦 Installing UI dependencies..." -ForegroundColor Yellow
Set-Location ui
npm install
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Failed to install UI dependencies" -ForegroundColor Red
    exit 1
}
Set-Location ..

Write-Host ""
Write-Host "✅ All dependencies installed!" -ForegroundColor Green
Write-Host ""
Write-Host "🌐 Starting services..." -ForegroundColor Cyan
Write-Host ""

# Start UI dev server in background
Write-Host "Starting Vite dev server..." -ForegroundColor Yellow
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd ui; npm run dev"

# Wait a bit for Vite to start
Start-Sleep -Seconds 3

# Start Python backend
Write-Host "Starting Python backend..." -ForegroundColor Yellow
Set-Location python
python photoshop_tool.py

