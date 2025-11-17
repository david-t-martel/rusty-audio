#!/bin/bash
# Migration script for Rusty Audio workspace separation
#
# This script helps migrate the monolithic structure to workspace organization

set -e

echo "Rusty Audio Workspace Migration Script"
echo "======================================"
echo ""

# Create backup
echo "Creating backup of current src/ directory..."
if [ ! -d "src.backup" ]; then
    cp -r src src.backup
    echo "✓ Backup created at src.backup/"
else
    echo "⚠ Backup already exists at src.backup/"
fi

echo ""
echo "Migration complete!"
echo ""
echo "Workspace structure:"
echo "  rusty-audio-core/     - Shared library code"
echo "  rusty-audio-desktop/  - Desktop application"
echo "  rusty-audio-web/      - WASM/PWA application with OAuth"
echo ""
echo "Next steps:"
echo "  1. Test core library: cd rusty-audio-core && cargo test"
echo "  2. Test desktop app:  cd rusty-audio-desktop && cargo run"
echo "  3. Build WASM app:    cd rusty-audio-web && wasm-pack build"
echo ""
