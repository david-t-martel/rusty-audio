#!/usr/bin/env bash
# Icon generation helper script for Rusty Audio PWA
# Requires ImageMagick or Inkscape

set -euo pipefail

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WWW_DIR="${PROJECT_ROOT}/www"
MASTER_ICON="${1:-}"

echo "======================================"
echo "  Rusty Audio - Icon Generator"
echo "======================================"
echo ""

# Check for master icon
if [[ -z "$MASTER_ICON" ]] || [[ ! -f "$MASTER_ICON" ]]; then
    print_error "Usage: $0 <master-icon.png|svg>"
    echo ""
    echo "Master icon requirements:"
    echo "  - PNG: Minimum 512x512px, square aspect ratio"
    echo "  - SVG: Vector format (any size)"
    echo ""
    echo "Example:"
    echo "  $0 logo-512.png"
    echo "  $0 logo.svg"
    exit 1
fi

print_info "Master icon: $MASTER_ICON"
MASTER_EXT="${MASTER_ICON##*.}"

# Detect available tools
HAS_IMAGEMAGICK=false
HAS_INKSCAPE=false

if command -v convert >/dev/null 2>&1; then
    HAS_IMAGEMAGICK=true
    print_success "ImageMagick found"
fi

if command -v inkscape >/dev/null 2>&1; then
    HAS_INKSCAPE=true
    print_success "Inkscape found"
fi

if [[ "$HAS_IMAGEMAGICK" == false ]] && [[ "$HAS_INKSCAPE" == false ]]; then
    print_error "No image processing tool found"
    echo ""
    echo "Install one of the following:"
    echo ""
    echo "ImageMagick:"
    echo "  Linux: sudo apt-get install imagemagick"
    echo "  macOS: brew install imagemagick"
    echo ""
    echo "Inkscape (for SVG):"
    echo "  Linux: sudo apt-get install inkscape"
    echo "  macOS: brew install inkscape"
    exit 1
fi

# Icon sizes
PWA_SIZES=(72 96 128 144 152 192 384 512)
FAVICON_SIZES=(16 32)

# Create output directory
mkdir -p "$WWW_DIR"

print_info "Generating PWA icons..."
echo ""

# Generate PWA icons
for size in "${PWA_SIZES[@]}"; do
    output="${WWW_DIR}/icon-${size}.png"

    if [[ "$MASTER_EXT" == "svg" ]] && [[ "$HAS_INKSCAPE" == true ]]; then
        # Use Inkscape for SVG
        inkscape -w "$size" -h "$size" "$MASTER_ICON" -o "$output" 2>/dev/null
    elif [[ "$HAS_IMAGEMAGICK" == true ]]; then
        # Use ImageMagick for raster images
        convert "$MASTER_ICON" -resize "${size}x${size}" "$output"
    else
        print_error "Cannot process $MASTER_EXT files"
        exit 1
    fi

    print_success "Generated: icon-${size}.png"
done

echo ""
print_info "Generating favicons..."
echo ""

# Generate favicons
for size in "${FAVICON_SIZES[@]}"; do
    output="${WWW_DIR}/favicon-${size}x${size}.png"

    if [[ "$MASTER_EXT" == "svg" ]] && [[ "$HAS_INKSCAPE" == true ]]; then
        inkscape -w "$size" -h "$size" "$MASTER_ICON" -o "$output" 2>/dev/null
    elif [[ "$HAS_IMAGEMAGICK" == true ]]; then
        convert "$MASTER_ICON" -resize "${size}x${size}" "$output"
    fi

    print_success "Generated: favicon-${size}x${size}.png"
done

# Optimize PNGs if available
echo ""
if command -v optipng >/dev/null 2>&1; then
    print_info "Optimizing PNGs with optipng..."
    for file in "${WWW_DIR}"/*.png; do
        optipng -quiet -o2 "$file"
    done
    print_success "PNG optimization complete"
elif command -v pngquant >/dev/null 2>&1; then
    print_info "Optimizing PNGs with pngquant..."
    pngquant --force --ext .png "${WWW_DIR}"/*.png
    print_success "PNG optimization complete"
else
    print_warning "PNG optimizer not found (optipng or pngquant recommended)"
fi

# Calculate total size
echo ""
print_info "Icon summary:"
echo ""

total_size=0
for file in "${WWW_DIR}"/icon-*.png "${WWW_DIR}"/favicon-*.png; do
    if [[ -f "$file" ]]; then
        size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null)
        size_kb=$((size / 1024))
        total_size=$((total_size + size))
        filename=$(basename "$file")
        printf "  %-25s %8s KB\n" "$filename" "$size_kb"
    fi
done

total_kb=$((total_size / 1024))
echo ""
print_success "Total icon size: ${total_kb} KB"

# Recommendations
echo ""
echo "======================================"
echo "  Recommendations"
echo "======================================"
echo ""

if [[ $total_size -gt 500000 ]]; then
    print_warning "Icon set is large (>${total_kb}KB). Consider:"
    echo "  - Using simpler artwork"
    echo "  - Reducing color palette"
    echo "  - Running pngquant for lossy compression"
else
    print_success "Icon set size is optimal"
fi

echo ""
echo "Next steps:"
echo "  1. Review generated icons in: ${WWW_DIR}/"
echo "  2. Test maskable icon: https://maskable.app/"
echo "  3. Build WASM: ./scripts/build-wasm.sh"
echo "  4. Deploy: ./scripts/deploy-wasm.sh local"
echo ""

print_success "Icon generation complete!"
