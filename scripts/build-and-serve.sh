#!/usr/bin/env bash

##############################################################################
# Build and Serve Script for Rusty Audio WASM Application
#
# This script provides a unified workflow for:
# 1. Building the WASM binary with proper multithreading flags
# 2. Optimizing the WASM output
# 3. Compressing assets (Brotli + Gzip)
# 4. Validating the build
# 5. Starting the development server
#
# Usage:
#   ./scripts/build-and-serve.sh [options]
#
# Options:
#   --skip-build     Skip the WASM build step
#   --skip-optimize  Skip WASM optimization
#   --skip-compress  Skip asset compression
#   --port PORT      Set server port (default: 8080)
#   --verbose        Enable verbose logging
#   --prod           Production mode
#   --help           Show this help message
##############################################################################

set -euo pipefail

# Color output functions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
BOLD='\033[1m'
NC='\033[0m' # No Color

print_header() {
    echo -e "\n${BOLD}${BLUE}═══════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${CYAN}  $1${NC}"
    echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_info() {
    echo -e "${CYAN}→${NC} $1"
}

# Default options
SKIP_BUILD=false
SKIP_OPTIMIZE=false
SKIP_COMPRESS=false
PORT=8080
VERBOSE=false
PROD_MODE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --skip-optimize)
            SKIP_OPTIMIZE=true
            shift
            ;;
        --skip-compress)
            SKIP_COMPRESS=true
            shift
            ;;
        --port)
            PORT="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --prod)
            PROD_MODE=true
            shift
            ;;
        --help)
            grep '^#' "$0" | sed 's/^# //'
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DIST_DIR="$PROJECT_ROOT/dist"

cd "$PROJECT_ROOT"

print_header "Rusty Audio Build and Serve"

# Check prerequisites
print_info "Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo not found. Install from https://rustup.rs/"
    exit 1
fi

if ! command -v trunk &> /dev/null; then
    print_error "Trunk not found. Install with: cargo install trunk"
    exit 1
fi

if ! command -v node &> /dev/null; then
    print_error "Node.js not found. Install from https://nodejs.org/"
    exit 1
fi

if ! command -v wasm-opt &> /dev/null; then
    print_warning "wasm-opt not found. Install binaryen for optimization."
    SKIP_OPTIMIZE=true
fi

print_success "All prerequisites found"

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    print_info "Installing Node.js dependencies..."
    npm install
    print_success "Dependencies installed"
fi

# Build step
if [ "$SKIP_BUILD" = false ]; then
    print_header "Building WASM Binary"

    # Set Rust flags for multithreading
    export RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals"

    if [ "$PROD_MODE" = true ]; then
        print_info "Building in RELEASE mode..."
        trunk build --release
    else
        print_info "Building in DEBUG mode (faster build)..."
        trunk build
    fi

    if [ $? -eq 0 ]; then
        print_success "WASM build completed"

        # Display WASM file size
        if [ -f "$DIST_DIR/rusty_audio_bg.wasm" ]; then
            WASM_SIZE=$(du -h "$DIST_DIR/rusty_audio_bg.wasm" | cut -f1)
            print_info "WASM size: ${WASM_SIZE}"
        fi
    else
        print_error "WASM build failed"
        exit 1
    fi
else
    print_warning "Skipping build step"
fi

# Optimization step
if [ "$SKIP_OPTIMIZE" = false ] && [ "$PROD_MODE" = true ]; then
    print_header "Optimizing WASM Binary"

    WASM_FILE="$DIST_DIR/rusty_audio_bg.wasm"

    if [ -f "$WASM_FILE" ]; then
        ORIGINAL_SIZE=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null)

        print_info "Running wasm-opt with threading optimizations..."

        wasm-opt "$WASM_FILE" \
            -Oz \
            --enable-threads \
            --enable-bulk-memory \
            --enable-simd \
            -o "$WASM_FILE.opt"

        if [ $? -eq 0 ]; then
            mv "$WASM_FILE.opt" "$WASM_FILE"

            OPTIMIZED_SIZE=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null)
            REDUCTION=$(echo "scale=2; (1 - $OPTIMIZED_SIZE / $ORIGINAL_SIZE) * 100" | bc)

            print_success "Optimization complete"
            print_info "Original: $(numfmt --to=iec-i --suffix=B $ORIGINAL_SIZE 2>/dev/null || echo "${ORIGINAL_SIZE} bytes")"
            print_info "Optimized: $(numfmt --to=iec-i --suffix=B $OPTIMIZED_SIZE 2>/dev/null || echo "${OPTIMIZED_SIZE} bytes")"
            print_info "Reduction: ${REDUCTION}%"
        else
            print_error "Optimization failed, using unoptimized binary"
        fi
    else
        print_error "WASM file not found: $WASM_FILE"
    fi
else
    if [ "$SKIP_OPTIMIZE" = true ]; then
        print_warning "Skipping optimization step"
    else
        print_info "Optimization only runs in production mode"
    fi
fi

# Compression step
if [ "$SKIP_COMPRESS" = false ] && [ "$PROD_MODE" = true ]; then
    print_header "Compressing Assets"

    if [ -f "$DIST_DIR/rusty_audio_bg.wasm" ]; then
        print_info "Compressing WASM binary..."

        # Gzip compression
        gzip -9 -k -f "$DIST_DIR/rusty_audio_bg.wasm"
        print_success "Gzip: rusty_audio_bg.wasm.gz"

        # Brotli compression (if available)
        if command -v brotli &> /dev/null; then
            brotli -9 -k -f "$DIST_DIR/rusty_audio_bg.wasm"
            print_success "Brotli: rusty_audio_bg.wasm.br"
        else
            print_warning "Brotli not available, skipping Brotli compression"
        fi

        # Compress JavaScript files
        for js_file in "$DIST_DIR"/*.js; do
            if [ -f "$js_file" ] && [[ ! "$js_file" =~ service-worker ]]; then
                gzip -9 -k -f "$js_file"
                [ $VERBOSE = true ] && print_info "Compressed: $(basename "$js_file")"
            fi
        done

        print_success "Asset compression complete"
    fi
else
    if [ "$SKIP_COMPRESS" = true ]; then
        print_warning "Skipping compression step"
    else
        print_info "Compression only runs in production mode"
    fi
fi

# Validation step
print_header "Validating Build"

VALIDATION_FAILED=false

# Check for required files
REQUIRED_FILES=(
    "$DIST_DIR/index.html"
    "$DIST_DIR/rusty_audio_bg.wasm"
    "$DIST_DIR/rusty_audio.js"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        print_success "Found: $(basename "$file")"
    else
        print_error "Missing: $(basename "$file")"
        VALIDATION_FAILED=true
    fi
done

# Check for service worker
if [ -f "$DIST_DIR/service-worker.js" ]; then
    print_success "Service worker present"
else
    print_warning "Service worker not found (optional)"
fi

# Check for static assets
if [ -d "$DIST_DIR/static" ] || [ -d "$DIST_DIR/icons" ]; then
    print_success "Static assets found"
else
    print_warning "Static assets directory not found"
fi

if [ "$VALIDATION_FAILED" = true ]; then
    print_error "Build validation failed"
    exit 1
fi

print_success "Build validation passed"

# Start development server
print_header "Starting Development Server"

print_info "Server will start on port: $PORT"
print_info "Press Ctrl+C to stop the server"

if [ "$VERBOSE" = true ]; then
    node "$SCRIPT_DIR/dev-server.js" --port "$PORT" --verbose
else
    node "$SCRIPT_DIR/dev-server.js" --port "$PORT"
fi
