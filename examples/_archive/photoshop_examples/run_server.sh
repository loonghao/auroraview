#!/bin/bash
# AuroraView WebSocket Server Launcher
# Bash script for macOS/Linux

echo "🚀 Starting AuroraView WebSocket Server..."
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust is not installed!"
    echo "Please install Rust from: https://rustup.rs/"
    exit 1
fi

# Check if we're in the correct directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Cargo.toml not found!"
    echo "Please run this script from the examples/photoshop_examples directory"
    exit 1
fi

# Build the project
echo "📦 Building project..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"
echo ""

# Run the server
echo "🌐 Starting WebSocket server on ws://localhost:9001"
echo "📡 Waiting for Photoshop UXP plugin to connect..."
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

export RUST_LOG=info
cargo run --bin websocket_server

