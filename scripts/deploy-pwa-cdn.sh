#!/usr/bin/env bash
# PWA/WASM Deployment Script with CDN Optimization
# Supports GitHub Pages, Cloudflare Pages, Netlify, and custom CDN

set -euo pipefail

# Configuration
VERSION="${VERSION:-0.1.0}"
ENVIRONMENT="${ENVIRONMENT:-production}"
CDN_PROVIDER="${CDN_PROVIDER:-github-pages}"  # github-pages, cloudflare, netlify, custom
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"
BUILD_DIR="${DIST_DIR}/pwa"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Header
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Rusty Audio - PWA Deployment${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""
echo -e "${YELLOW}Version: $VERSION${NC}"
echo -e "${YELLOW}Environment: $ENVIRONMENT${NC}"
echo -e "${YELLOW}CDN Provider: $CDN_PROVIDER${NC}"
echo ""

# Prerequisites check
print_info "[1/10] Checking prerequisites..."

if ! command -v rustc >/dev/null 2>&1; then
    print_error "Rust not installed. Install from https://rustup.rs/"
    exit 1
fi

if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    print_warning "WASM target not installed. Installing..."
    rustup target add wasm32-unknown-unknown
fi

if ! command -v wasm-pack >/dev/null 2>&1; then
    print_warning "wasm-pack not found. Installing..."
    cargo install wasm-pack --locked
fi

if ! command -v wasm-opt >/dev/null 2>&1; then
    print_warning "wasm-opt not found. Optimization will be limited."
    print_info "Install binaryen: https://github.com/WebAssembly/binaryen"
fi

print_success "Prerequisites OK"

# Clean previous builds
print_info "[2/10] Cleaning previous builds..."
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"
print_success "Clean complete"

# Build WASM with wasm-pack
print_info "[3/10] Building WASM with wasm-pack..."

if [ "$ENVIRONMENT" = "production" ]; then
    WASM_PACK_FLAGS="--release"
    OPTIMIZE_LEVEL="-Oz"
else
    WASM_PACK_FLAGS="--dev"
    OPTIMIZE_LEVEL="-O1"
fi

wasm-pack build \
    --target web \
    --out-dir "${BUILD_DIR}/pkg" \
    $WASM_PACK_FLAGS \
    -- --no-default-features

if [ $? -ne 0 ]; then
    print_error "WASM build failed!"
    exit 1
fi

print_success "WASM build complete"

# Optimize WASM binary
print_info "[4/10] Optimizing WASM binary..."

WASM_FILE="${BUILD_DIR}/pkg/rusty_audio_bg.wasm"

if [ -f "$WASM_FILE" ]; then
    WASM_SIZE_BEFORE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null)
    print_info "WASM size before optimization: $(numfmt --to=iec-i --suffix=B "$WASM_SIZE_BEFORE" 2>/dev/null || echo "${WASM_SIZE_BEFORE} bytes")"

    if command -v wasm-opt >/dev/null 2>&1; then
        # Create backup
        cp "$WASM_FILE" "${WASM_FILE}.bak"

        # Optimize
        wasm-opt "$OPTIMIZE_LEVEL" \
            --enable-simd \
            --enable-bulk-memory \
            --enable-sign-ext \
            --enable-mutable-globals \
            --enable-nontrapping-float-to-int \
            --strip-debug \
            --strip-dwarf \
            --strip-producers \
            --vacuum \
            --dce \
            --duplicate-function-elimination \
            --output "$WASM_FILE" \
            "${WASM_FILE}.bak"

        rm "${WASM_FILE}.bak"

        WASM_SIZE_AFTER=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null)
        REDUCTION=$((WASM_SIZE_BEFORE - WASM_SIZE_AFTER))
        PERCENTAGE=$(awk "BEGIN {printf \"%.2f\", ($REDUCTION / $WASM_SIZE_BEFORE) * 100}")

        print_success "WASM optimized: $(numfmt --to=iec-i --suffix=B "$WASM_SIZE_AFTER" 2>/dev/null || echo "${WASM_SIZE_AFTER} bytes")"
        print_success "Size reduction: $(numfmt --to=iec-i --suffix=B "$REDUCTION" 2>/dev/null || echo "${REDUCTION} bytes") (${PERCENTAGE}%)"
    else
        print_warning "wasm-opt not available, skipping optimization"
    fi
else
    print_error "WASM file not found: $WASM_FILE"
    exit 1
fi

# Copy static assets
print_info "[5/10] Copying static assets..."

cp -r "${PROJECT_ROOT}/static/"* "$BUILD_DIR/"
cp "${PROJECT_ROOT}/index.html" "$BUILD_DIR/"

# Verify required files
REQUIRED_FILES=(
    "index.html"
    "manifest.webmanifest"
    "service-worker.js"
    "_headers"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ ! -f "${BUILD_DIR}/${file}" ]; then
        print_error "Required file missing: ${file}"
        exit 1
    fi
done

print_success "Static assets copied"

# Generate build manifest
print_info "[6/10] Generating build manifest..."

BUILD_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
GIT_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
WASM_SIZE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null)

cat > "${BUILD_DIR}/build-manifest.json" <<EOF
{
  "name": "rusty-audio",
  "version": "${VERSION}",
  "environment": "${ENVIRONMENT}",
  "build_date": "${BUILD_DATE}",
  "git_hash": "${GIT_HASH}",
  "wasm_size": ${WASM_SIZE},
  "cdn_provider": "${CDN_PROVIDER}"
}
EOF

print_success "Build manifest created"

# Compress assets for CDN
print_info "[7/10] Compressing assets for CDN..."

# Gzip compression
if command -v gzip >/dev/null 2>&1; then
    find "$BUILD_DIR" -type f \( -name "*.wasm" -o -name "*.js" -o -name "*.css" -o -name "*.html" -o -name "*.json" \) | while read -r file; do
        gzip -9 -k "$file"
    done
    print_success "Gzip compression complete"
else
    print_warning "gzip not found, skipping gzip compression"
fi

# Brotli compression
if command -v brotli >/dev/null 2>&1; then
    find "$BUILD_DIR" -type f \( -name "*.wasm" -o -name "*.js" -o -name "*.css" -o -name "*.html" -o -name "*.json" \) | while read -r file; do
        brotli -9 -k "$file"
    done
    print_success "Brotli compression complete"
else
    print_warning "brotli not found, skipping brotli compression"
fi

# CDN-specific headers
print_info "[8/10] Configuring CDN headers..."

case "$CDN_PROVIDER" in
    "github-pages")
        # GitHub Pages uses _headers file from static/
        print_info "Using _headers file for GitHub Pages"
        ;;

    "cloudflare")
        # Create Cloudflare-specific _headers
        cat > "${BUILD_DIR}/_headers" <<'EOF'
/*
  X-Content-Type-Options: nosniff
  X-Frame-Options: DENY
  Referrer-Policy: no-referrer
  Permissions-Policy: autoplay=(self), microphone=(self)
  Cross-Origin-Opener-Policy: same-origin
  Cross-Origin-Embedder-Policy: require-corp
  Content-Security-Policy: default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' https:; media-src 'self' blob:; worker-src 'self'; object-src 'none'; frame-ancestors 'none'

/service-worker.js
  Cache-Control: no-store

/manifest.webmanifest
  Content-Type: application/manifest+json; charset=utf-8
  Cache-Control: public, max-age=86400

/pkg/*
  Cache-Control: public, max-age=31536000, immutable

/icons/*
  Cache-Control: public, max-age=31536000, immutable
EOF
        print_success "Cloudflare headers configured"
        ;;

    "netlify")
        # Create Netlify-specific _headers
        cat > "${BUILD_DIR}/_headers" <<'EOF'
/*
  X-Content-Type-Options: nosniff
  X-Frame-Options: DENY
  Referrer-Policy: no-referrer
  Permissions-Policy: autoplay=(self), microphone=(self)
  Cross-Origin-Opener-Policy: same-origin
  Cross-Origin-Embedder-Policy: require-corp

/service-worker.js
  Cache-Control: no-store

/manifest.webmanifest
  Content-Type: application/manifest+json; charset=utf-8

/*.wasm
  Content-Type: application/wasm
  Cache-Control: public, max-age=31536000
EOF

        # Create _redirects for SPA routing
        cat > "${BUILD_DIR}/_redirects" <<'EOF'
# Redirect all routes to index.html for SPA
/*    /index.html   200
EOF
        print_success "Netlify headers configured"
        ;;

    *)
        print_warning "Unknown CDN provider: $CDN_PROVIDER"
        ;;
esac

# Generate deployment report
print_info "[9/10] Generating deployment report..."

cat > "${BUILD_DIR}/DEPLOYMENT-REPORT.md" <<EOF
# Rusty Audio PWA - Deployment Report

## Build Information

- **Version**: ${VERSION}
- **Environment**: ${ENVIRONMENT}
- **Build Date**: ${BUILD_DATE}
- **Git Hash**: ${GIT_HASH}
- **CDN Provider**: ${CDN_PROVIDER}

## Bundle Analysis

### WASM Bundle
- **Size**: $(numfmt --to=iec-i --suffix=B "$WASM_SIZE" 2>/dev/null || echo "${WASM_SIZE} bytes")
- **Optimization**: ${OPTIMIZE_LEVEL}

### Total Bundle Size
\`\`\`
$(du -sh "$BUILD_DIR" 2>/dev/null || echo "Size calculation unavailable")
\`\`\`

### Files Included

\`\`\`
$(find "$BUILD_DIR" -type f -not -path '*/\.*' | sort)
\`\`\`

## Deployment Checklist

- [ ] Test PWA locally: \`python3 -m http.server 8080 --directory ${BUILD_DIR}\`
- [ ] Verify service worker registration
- [ ] Test offline functionality
- [ ] Check manifest.json and icons
- [ ] Test on multiple browsers (Chrome, Firefox, Safari, Edge)
- [ ] Verify CSP headers
- [ ] Test audio playback
- [ ] Test EQ and effects
- [ ] Test "Add to Home Screen" functionality

## CDN Deployment Commands

### GitHub Pages
\`\`\`bash
gh-pages --dist ${BUILD_DIR} --branch gh-pages
\`\`\`

### Cloudflare Pages
\`\`\`bash
wrangler pages publish ${BUILD_DIR} --project-name rusty-audio
\`\`\`

### Netlify
\`\`\`bash
netlify deploy --prod --dir ${BUILD_DIR}
\`\`\`

## Performance Tips

1. **Enable compression**: Gzip and Brotli compressed versions are pre-generated
2. **CDN caching**: Static assets have long cache headers (1 year)
3. **Service worker**: Caches assets for offline use
4. **WASM optimization**: Optimized with wasm-opt

## Troubleshooting

### WASM not loading
- Check Content-Type header: \`application/wasm\`
- Verify CORS headers
- Check browser console for errors

### Service worker issues
- Clear cache: DevTools > Application > Clear storage
- Re-register service worker
- Check for console errors

### Audio issues
- Verify Web Audio API support
- Check browser permissions for autoplay
- Test with user gesture (click to play)

## Support

- GitHub: https://github.com/david-t-martel/rusty-audio
- Issues: https://github.com/david-t-martel/rusty-audio/issues
EOF

print_success "Deployment report created"

# Deployment
print_info "[10/10] Deployment instructions..."

echo ""
echo -e "${YELLOW}Deployment ready!${NC}"
echo ""
echo -e "${CYAN}Build directory: ${BUILD_DIR}${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo ""

case "$CDN_PROVIDER" in
    "github-pages")
        echo -e "${NC}1. Test locally:${NC}"
        echo -e "${CYAN}   cd ${BUILD_DIR} && python3 -m http.server 8080${NC}"
        echo ""
        echo -e "${NC}2. Deploy to GitHub Pages:${NC}"
        echo -e "${CYAN}   git add ${BUILD_DIR}${NC}"
        echo -e "${CYAN}   git commit -m 'Deploy PWA v${VERSION}'${NC}"
        echo -e "${CYAN}   git push origin gh-pages${NC}"
        echo ""
        echo -e "${NC}Or use GitHub Actions (recommended)${NC}"
        ;;

    "cloudflare")
        echo -e "${NC}1. Test locally:${NC}"
        echo -e "${CYAN}   cd ${BUILD_DIR} && python3 -m http.server 8080${NC}"
        echo ""
        echo -e "${NC}2. Deploy to Cloudflare Pages:${NC}"
        echo -e "${CYAN}   wrangler pages publish ${BUILD_DIR} --project-name rusty-audio${NC}"
        echo ""
        echo -e "${NC}3. Configure custom domain (optional):${NC}"
        echo -e "${CYAN}   wrangler pages project set rusty-audio --domain rusty-audio.example.com${NC}"
        ;;

    "netlify")
        echo -e "${NC}1. Test locally:${NC}"
        echo -e "${CYAN}   cd ${BUILD_DIR} && python3 -m http.server 8080${NC}"
        echo ""
        echo -e "${NC}2. Deploy to Netlify:${NC}"
        echo -e "${CYAN}   netlify deploy --prod --dir ${BUILD_DIR}${NC}"
        ;;
esac

echo ""
echo -e "${GREEN}Deployment complete!${NC}"
echo ""
