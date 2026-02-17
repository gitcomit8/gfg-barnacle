#!/bin/bash

# Build script for Module 18: Re-entrancy Deadlock Module

echo "🔨 Building Module 18: Re-entrancy Deadlock Module"
echo "=================================================="
echo ""

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null
then
    echo "❌ wasm-pack not found!"
    echo "Please install it: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

echo "✓ wasm-pack found"
echo ""

# Build for web target
echo "📦 Building for web target..."
wasm-pack build --target web

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "📁 Output directory: pkg/"
    echo ""
    echo "🚀 To test the module:"
    echo "   1. Open demo.html in a web browser"
    echo "   2. Or serve with: python3 -m http.server 8000"
    echo "   3. Then visit: http://localhost:8000/demo.html"
    echo ""
else
    echo "❌ Build failed!"
    exit 1
fi
