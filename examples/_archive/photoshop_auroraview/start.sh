#!/bin/bash
# Quick Start Script for Photoshop + AuroraView Integration
# Bash script for macOS/Linux

echo "🚀 Starting Photoshop + AuroraView Integration..."
echo ""

# Check Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Python not found! Please install Python 3.8+"
    exit 1
fi

# Check Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js not found! Please install Node.js 18+"
    exit 1
fi

echo "✅ Python and Node.js found"
echo ""

# Install Python dependencies
echo "📦 Installing Python dependencies..."
cd python
python3 -m pip install -r requirements.txt
if [ $? -ne 0 ]; then
    echo "❌ Failed to install Python dependencies"
    exit 1
fi
cd ..

# Install UI dependencies
echo "📦 Installing UI dependencies..."
cd ui
npm install
if [ $? -ne 0 ]; then
    echo "❌ Failed to install UI dependencies"
    exit 1
fi
cd ..

echo ""
echo "✅ All dependencies installed!"
echo ""
echo "🌐 Starting services..."
echo ""

# Start UI dev server in background
echo "Starting Vite dev server..."
cd ui
npm run dev &
UI_PID=$!
cd ..

# Wait a bit for Vite to start
sleep 3

# Start Python backend
echo "Starting Python backend..."
cd python
python3 photoshop_tool.py

# Cleanup on exit
kill $UI_PID 2>/dev/null

