#!/bin/bash

# Build script for Module 17: Shadowed Canvas Context
# This script compiles the Rust code to WebAssembly

set -e

echo "🔨 Building Module 17: Shadowed Canvas Context"
echo "================================================"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found!"
    echo "📦 Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build for web target
echo "🦀 Compiling Rust to WebAssembly..."
wasm-pack build --target web --release

echo ""
echo "✅ Build complete!"
echo "📁 Output files are in: ./pkg/"
echo ""
echo "🚀 To test the demo:"
echo "   1. Start a local web server:"
echo "      python3 -m http.server 8000"
echo "      or"
echo "      npx http-server"
echo ""
echo "   2. Open http://localhost:8000/demo.html in a browser"
echo ""
echo "🐛 Remember: This module is intentionally buggy!"
