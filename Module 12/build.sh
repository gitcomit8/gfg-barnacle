#!/bin/bash

# Build script for Hydration Mismatch Module
# This script compiles the Rust code to WebAssembly

set -e

echo "🦀 Building Hydration Mismatch Module (Rust → WASM)"
echo "======================================================"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is not installed!"
    echo "📦 Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed!"
    echo "Please install Rust from: https://rustup.rs/"
    exit 1
fi

echo "✅ Dependencies found"
echo ""

# Build for web target (default)
echo "🔨 Building for web target..."
wasm-pack build --target web

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "📦 Output directory: pkg/"
    echo ""
    echo "To use in a web application:"
    echo "  import init, { HydrationData } from './pkg/hydration_mismatch_module.js';"
    echo ""
    echo "⚠️  WARNING: This module is intentionally buggy!"
    echo "    It will cause hydration mismatch errors in SSR applications."
else
    echo "❌ Build failed!"
    exit 1
fi

# Run tests
echo ""
echo "🧪 Running tests..."
cargo test

echo ""
echo "✨ All done!"
