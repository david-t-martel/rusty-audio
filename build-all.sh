#!/bin/bash
# Build all workspace members
#
# This script builds the core library, desktop app, and WASM app

set -e

echo "Building Rusty Audio Workspace"
echo "=============================="
echo ""

# Build core library
echo "1/3 Building rusty-audio-core..."
cd rusty-audio-core
cargo build --release --features native
cd ..
echo "✓ Core library built"
echo ""

# Build desktop application
echo "2/3 Building rusty-audio-desktop..."
cd rusty-audio-desktop
cargo build --release
cd ..
echo "✓ Desktop application built"
echo ""

# Build WASM application
echo "3/3 Building rusty-audio-web..."
cd rusty-audio-web
if command -v wasm-pack &> /dev/null; then
    wasm-pack build --target web --release
    echo "✓ WASM application built"
else
    echo "⚠ wasm-pack not found. Install with: cargo install wasm-pack"
    echo "  Skipping WASM build"
fi
cd ..

echo ""
echo "Build complete!"
echo ""
echo "Binaries:"
echo "  Desktop: rusty-audio-desktop/target/release/rusty-audio"
echo "  WASM:    rusty-audio-web/pkg/"
echo ""
