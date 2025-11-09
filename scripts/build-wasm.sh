#!/usr/bin/env bash
# Build script for Rusty Audio WASM/PWA deployment
# Optimizes WASM bundle size and prepares for production deployment

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WWW_DIR="${PROJECT_ROOT}/www"
DIST_DIR="${PROJECT_ROOT}/dist"
TARGET_DIR="${PROJECT_ROOT}/target"
WASM_TARGET="wasm32-unknown-unknown"

# Build profiles
PROFILE="${PROFILE:-release}"
OPTIMIZE_LEVEL="${OPTIMIZE_LEVEL:-3}"

# Print functions
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Header
echo "======================================"
echo "  Rusty Audio - WASM/PWA Build"
echo "======================================"
echo ""

# Check prerequisites
print_info "Checking prerequisites..."

command -v rustc >/dev/null 2>&1 || {
    print_error "Rust is not installed. Please install from https://rustup.rs/"
    exit 1
}

command -v wasm-bindgen >/dev/null 2>&1 || {
    print_warning "wasm-bindgen not found. Installing..."
    cargo install wasm-bindgen-cli
}

command -v wasm-opt >/dev/null 2>&1 || {
    print_warning "wasm-opt not found. Installing binaryen for optimization..."
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        sudo apt-get install -y binaryen
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        brew install binaryen
    else
        print_warning "Please install binaryen manually for WASM optimization"
    fi
}

# Check if wasm32 target is installed
if ! rustup target list --installed | grep -q "$WASM_TARGET"; then
    print_warning "WASM target not installed. Installing..."
    rustup target add "$WASM_TARGET"
fi

print_success "All prerequisites met"
echo ""

# Clean previous builds
if [[ "${CLEAN:-false}" == "true" ]]; then
    print_info "Cleaning previous builds..."
    rm -rf "$DIST_DIR"
    cargo clean --target "$WASM_TARGET"
    print_success "Clean complete"
    echo ""
fi

# Create output directories
print_info "Creating output directories..."
mkdir -p "$DIST_DIR"
print_success "Directories created"
echo ""

# Build WASM binary
print_info "Building WASM binary (profile: $PROFILE)..."
RUSTFLAGS="-C opt-level=$OPTIMIZE_LEVEL -C lto=fat -C embed-bitcode=yes" \
    cargo build \
    --lib \
    --target "$WASM_TARGET" \
    --profile "$PROFILE" \
    --features "default"

WASM_BINARY="${TARGET_DIR}/${WASM_TARGET}/${PROFILE}/rusty_audio.wasm"

if [[ ! -f "$WASM_BINARY" ]]; then
    print_error "WASM binary not found at: $WASM_BINARY"
    exit 1
fi

WASM_SIZE_BEFORE=$(stat -f%z "$WASM_BINARY" 2>/dev/null || stat -c%s "$WASM_BINARY" 2>/dev/null)
print_success "WASM binary built: $(numfmt --to=iec-i --suffix=B "$WASM_SIZE_BEFORE" 2>/dev/null || echo "${WASM_SIZE_BEFORE} bytes")"
echo ""

# Run wasm-bindgen
print_info "Generating JavaScript bindings with wasm-bindgen..."
wasm-bindgen \
    --target web \
    --out-dir "$DIST_DIR" \
    --out-name rusty_audio \
    --no-typescript \
    --remove-name-section \
    --remove-producers-section \
    "$WASM_BINARY"

print_success "JavaScript bindings generated"
echo ""

# Optimize WASM with wasm-opt
if command -v wasm-opt >/dev/null 2>&1; then
    print_info "Optimizing WASM binary with wasm-opt..."

    WASM_DIST="${DIST_DIR}/rusty_audio_bg.wasm"

    # Create backup
    cp "$WASM_DIST" "${WASM_DIST}.bak"

    # Run optimization passes
    wasm-opt \
        -Oz \
        --enable-bulk-memory \
        --enable-sign-ext \
        --enable-mutable-globals \
        --enable-nontrapping-float-to-int \
        --enable-simd \
        --vacuum \
        --dce \
        --duplicate-function-elimination \
        --strip-debug \
        --strip-dwarf \
        --strip-producers \
        --output "$WASM_DIST" \
        "${WASM_DIST}.bak"

    # Remove backup
    rm "${WASM_DIST}.bak"

    WASM_SIZE_AFTER=$(stat -f%z "$WASM_DIST" 2>/dev/null || stat -c%s "$WASM_DIST" 2>/dev/null)
    REDUCTION=$((WASM_SIZE_BEFORE - WASM_SIZE_AFTER))
    PERCENTAGE=$(awk "BEGIN {printf \"%.2f\", ($REDUCTION / $WASM_SIZE_BEFORE) * 100}")

    print_success "WASM optimized: $(numfmt --to=iec-i --suffix=B "$WASM_SIZE_AFTER" 2>/dev/null || echo "${WASM_SIZE_AFTER} bytes")"
    print_success "Size reduction: $(numfmt --to=iec-i --suffix=B "$REDUCTION" 2>/dev/null || echo "${REDUCTION} bytes") (${PERCENTAGE}%)"
    echo ""
else
    print_warning "wasm-opt not available, skipping optimization"
    WASM_SIZE_AFTER=$WASM_SIZE_BEFORE
    echo ""
fi

# Copy static assets
print_info "Copying static assets..."
cp "${WWW_DIR}/index.html" "$DIST_DIR/"
cp "${WWW_DIR}/manifest.json" "$DIST_DIR/"
cp "${WWW_DIR}/sw.js" "$DIST_DIR/"

print_success "Static assets copied"
echo ""

# Generate placeholder icons (if not present)
print_info "Checking for PWA icons..."
ICON_SIZES=(16 32 72 96 128 144 152 192 384 512)
MISSING_ICONS=()

for size in "${ICON_SIZES[@]}"; do
    if [[ ! -f "${WWW_DIR}/icon-${size}.png" ]]; then
        MISSING_ICONS+=("$size")
    else
        cp "${WWW_DIR}/icon-${size}.png" "$DIST_DIR/"
    fi
done

if [[ ${#MISSING_ICONS[@]} -gt 0 ]]; then
    print_warning "Missing icon sizes: ${MISSING_ICONS[*]}"
    print_info "Generate icons with: https://realfavicongenerator.net/"
else
    print_success "All PWA icons present"
fi
echo ""

# Compress with gzip for CDN deployment
if command -v gzip >/dev/null 2>&1; then
    print_info "Creating gzip compressed versions for CDN..."
    gzip -9 -k "${DIST_DIR}/rusty_audio_bg.wasm"
    gzip -9 -k "${DIST_DIR}/rusty_audio.js"
    print_success "Gzip compression complete"
    echo ""
fi

# Compress with brotli if available
if command -v brotli >/dev/null 2>&1; then
    print_info "Creating brotli compressed versions for CDN..."
    brotli -9 -k "${DIST_DIR}/rusty_audio_bg.wasm"
    brotli -9 -k "${DIST_DIR}/rusty_audio.js"
    print_success "Brotli compression complete"
    echo ""
fi

# Generate build manifest
print_info "Generating build manifest..."
BUILD_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
BUILD_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

cat > "${DIST_DIR}/build-manifest.json" <<EOF
{
  "name": "rusty-audio",
  "version": "0.1.0",
  "build_date": "${BUILD_DATE}",
  "git_hash": "${BUILD_HASH}",
  "wasm_size": ${WASM_SIZE_AFTER},
  "profile": "${PROFILE}",
  "optimize_level": "${OPTIMIZE_LEVEL}"
}
EOF

print_success "Build manifest created"
echo ""

# Display bundle analysis
echo "======================================"
echo "  Bundle Analysis"
echo "======================================"
echo ""

printf "%-30s %15s\n" "File" "Size"
echo "----------------------------------------------"

for file in "$DIST_DIR"/*; do
    if [[ -f "$file" ]] && [[ ! "$file" =~ \.(gz|br)$ ]]; then
        filename=$(basename "$file")
        size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
        size_human=$(numfmt --to=iec-i --suffix=B "$size" 2>/dev/null || echo "${size} bytes")
        printf "%-30s %15s\n" "$filename" "$size_human"
    fi
done

echo ""

# Total size calculation
TOTAL_SIZE=0
for file in "$DIST_DIR"/*; do
    if [[ -f "$file" ]] && [[ ! "$file" =~ \.(gz|br)$ ]]; then
        size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
        TOTAL_SIZE=$((TOTAL_SIZE + size))
    fi
done

TOTAL_HUMAN=$(numfmt --to=iec-i --suffix=B "$TOTAL_SIZE" 2>/dev/null || echo "${TOTAL_SIZE} bytes")
echo "Total bundle size: $TOTAL_HUMAN"
echo ""

# Deployment instructions
echo "======================================"
echo "  Deployment Instructions"
echo "======================================"
echo ""
echo "1. Test locally:"
echo "   cd ${DIST_DIR}"
echo "   python3 -m http.server 8080"
echo "   Open http://localhost:8080"
echo ""
echo "2. Deploy to production:"
echo "   - Upload contents of ${DIST_DIR}/ to your web server"
echo "   - Configure MIME types:"
echo "     .wasm -> application/wasm"
echo "   - Enable gzip/brotli compression"
echo "   - Set cache headers:"
echo "     .wasm, .js -> max-age=31536000, immutable"
echo "     index.html -> max-age=0, must-revalidate"
echo ""
echo "3. Verify PWA installation:"
echo "   - Open DevTools > Application > Manifest"
echo "   - Check Service Worker registration"
echo "   - Test 'Add to Home Screen'"
echo ""

print_success "Build complete! Output in: ${DIST_DIR}"
