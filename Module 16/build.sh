#!/bin/bash
# Build script for Module 16: Multi-Step Form State Fragmentation

set -e  # Exit on error

echo "🔨 Building Module 16: Multi-Step Form State Fragmentation"
echo "=================================================="

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null
then
    echo "❌ wasm-pack not found!"
    echo "📦 Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null
then
    echo "❌ Cargo (Rust) not found!"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo ""
echo "✅ Dependencies check passed"
echo ""

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf pkg/
rm -rf target/

# Run tests first
echo ""
echo "🧪 Running tests..."
cargo test

# Build the WASM module
echo ""
echo "🏗️  Building WASM module..."
wasm-pack build --target web --release

echo ""
echo "✅ Build complete!"
echo ""
echo "📦 Output directory: pkg/"
echo ""
echo "📝 Next steps:"
echo "  1. Open demo.html in a browser"
echo "  2. Or serve with: python3 -m http.server 8000"
echo "  3. Or integrate pkg/ into your web application"
echo ""
echo "📚 Documentation:"
echo "  - README.md      : Full documentation"
echo "  - QUICKSTART.md  : Quick setup guide"
echo "  - SECURITY.md    : Security implications"
echo "  - INDEX.md       : Module overview"
echo ""
echo "🐛 Remember: This module is intentionally buggy for educational purposes!"
