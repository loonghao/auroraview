#!/bin/bash
# Photoshop Layers Demo - Quick Start Script

echo "🚀 Starting Photoshop Layers Demo..."
echo ""

# Check Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Python not found. Please install Python 3.7+"
    exit 1
fi

echo "✅ Python found: $(python3 --version)"

# Check if websockets is installed
if ! python3 -c "import websockets" 2>/dev/null; then
    echo "📦 Installing websockets..."
    python3 -m pip install websockets
fi

echo ""
echo "================================================================================"
echo "Photoshop Layers Demo"
echo "================================================================================"
echo ""
echo "📝 Instructions:"
echo "1. Open Photoshop"
echo "2. Load UXP plugin from: examples/photoshop_auroraview/uxp_plugin"
echo "3. Click 'Connect to Python' in the plugin"
echo "4. Use the WebView window to create/manage layers"
echo ""
echo "================================================================================"
echo ""

# Run the tool
python3 examples/photoshop_layers_demo/photoshop_layers_tool.py

