# Photoshop Layers Demo - Quick Start Script
# This script starts the Photoshop Layers Demo

Write-Host "🚀 Starting Photoshop Layers Demo..." -ForegroundColor Cyan
Write-Host ""

# Check Python
if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Python not found. Please install Python 3.7+" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Python found: $(python --version)" -ForegroundColor Green

# Check if websockets is installed
$websocketsInstalled = python -c "import websockets; print('ok')" 2>$null
if ($websocketsInstalled -ne "ok") {
    Write-Host "📦 Installing websockets..." -ForegroundColor Yellow
    python -m pip install websockets
}

Write-Host ""
Write-Host "=" * 80 -ForegroundColor Cyan
Write-Host "Photoshop Layers Demo" -ForegroundColor Cyan
Write-Host "=" * 80 -ForegroundColor Cyan
Write-Host ""
Write-Host "📝 Instructions:" -ForegroundColor Yellow
Write-Host "1. Open Photoshop" -ForegroundColor White
Write-Host "2. Load UXP plugin from: examples/photoshop_auroraview/uxp_plugin" -ForegroundColor White
Write-Host "3. Click 'Connect to Python' in the plugin" -ForegroundColor White
Write-Host "4. Use the WebView window to create/manage layers" -ForegroundColor White
Write-Host ""
Write-Host "=" * 80 -ForegroundColor Cyan
Write-Host ""

# Run the tool
python examples/photoshop_layers_demo/photoshop_layers_tool.py

