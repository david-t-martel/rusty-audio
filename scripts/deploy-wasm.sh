#!/usr/bin/env bash
# Deployment script for Rusty Audio PWA
# Supports multiple deployment targets: local testing, GitHub Pages, Cloudflare Pages, etc.

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${PROJECT_ROOT}/dist"

print_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Deployment target selection
DEPLOY_TARGET="${1:-local}"

echo "======================================"
echo "  Rusty Audio - PWA Deployment"
echo "======================================"
echo ""

# Check if build exists
if [[ ! -d "$DIST_DIR" ]] || [[ ! -f "${DIST_DIR}/rusty_audio_bg.wasm" ]]; then
    print_error "Build not found. Run ./scripts/build-wasm.sh first"
    exit 1
fi

case "$DEPLOY_TARGET" in
    local)
        deploy_local
        ;;
    github)
        deploy_github_pages
        ;;
    cloudflare)
        deploy_cloudflare_pages
        ;;
    netlify)
        deploy_netlify
        ;;
    vercel)
        deploy_vercel
        ;;
    docker)
        deploy_docker
        ;;
    *)
        print_error "Unknown deployment target: $DEPLOY_TARGET"
        echo "Usage: $0 [local|github|cloudflare|netlify|vercel|docker]"
        exit 1
        ;;
esac

# Local development server deployment
deploy_local() {
    print_info "Starting local development server..."
    echo ""

    cd "$DIST_DIR"

    # Check for available server options
    if command -v python3 >/dev/null 2>&1; then
        print_info "Using Python HTTP server on port 8080"
        echo ""
        print_success "Server running at: http://localhost:8080"
        echo "Press Ctrl+C to stop"
        echo ""
        python3 -m http.server 8080
    elif command -v python >/dev/null 2>&1; then
        print_info "Using Python HTTP server on port 8080"
        echo ""
        print_success "Server running at: http://localhost:8080"
        echo "Press Ctrl+C to stop"
        echo ""
        python -m http.server 8080
    elif command -v npx >/dev/null 2>&1; then
        print_info "Using http-server (npm) on port 8080"
        echo ""
        print_success "Server running at: http://localhost:8080"
        echo "Press Ctrl+C to stop"
        echo ""
        npx http-server -p 8080 -c-1
    else
        print_error "No HTTP server found. Install Python or Node.js"
        exit 1
    fi
}

# GitHub Pages deployment
deploy_github_pages() {
    print_info "Deploying to GitHub Pages..."

    # Check if gh CLI is available
    if ! command -v gh >/dev/null 2>&1; then
        print_error "GitHub CLI (gh) not found. Install from https://cli.github.com/"
        exit 1
    fi

    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_error "Not in a git repository"
        exit 1
    fi

    # Create gh-pages branch if it doesn't exist
    if ! git rev-parse --verify gh-pages >/dev/null 2>&1; then
        print_info "Creating gh-pages branch..."
        git checkout --orphan gh-pages
        git rm -rf .
        git commit --allow-empty -m "Initialize gh-pages branch"
        git checkout -
    fi

    # Copy files to gh-pages
    print_info "Copying build to gh-pages branch..."
    git checkout gh-pages
    git rm -rf .
    cp -r "${DIST_DIR}"/* .
    git add .
    git commit -m "Deploy: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
    git push origin gh-pages
    git checkout -

    print_success "Deployed to GitHub Pages"
    echo ""
    echo "Enable GitHub Pages in repository settings:"
    echo "  Settings > Pages > Source: gh-pages branch"
}

# Cloudflare Pages deployment
deploy_cloudflare_pages() {
    print_info "Deploying to Cloudflare Pages..."

    if ! command -v wrangler >/dev/null 2>&1; then
        print_error "Wrangler CLI not found. Install with: npm install -g wrangler"
        exit 1
    fi

    # Create _headers file for MIME types and caching
    cat > "${DIST_DIR}/_headers" <<EOF
/*
  X-Content-Type-Options: nosniff
  X-Frame-Options: DENY
  X-XSS-Protection: 1; mode=block
  Referrer-Policy: strict-origin-when-cross-origin

/*.wasm
  Content-Type: application/wasm
  Cache-Control: public, max-age=31536000, immutable

/*.js
  Content-Type: application/javascript
  Cache-Control: public, max-age=31536000, immutable

/index.html
  Cache-Control: public, max-age=0, must-revalidate

/manifest.json
  Cache-Control: public, max-age=86400
EOF

    print_info "Publishing to Cloudflare Pages..."
    wrangler pages deploy "$DIST_DIR" --project-name rusty-audio

    print_success "Deployed to Cloudflare Pages"
}

# Netlify deployment
deploy_netlify() {
    print_info "Deploying to Netlify..."

    if ! command -v netlify >/dev/null 2>&1; then
        print_error "Netlify CLI not found. Install with: npm install -g netlify-cli"
        exit 1
    fi

    # Create netlify.toml
    cat > "${PROJECT_ROOT}/netlify.toml" <<EOF
[build]
  publish = "dist"
  command = "./scripts/build-wasm.sh"

[[headers]]
  for = "/*.wasm"
  [headers.values]
    Content-Type = "application/wasm"
    Cache-Control = "public, max-age=31536000, immutable"

[[headers]]
  for = "/*.js"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"

[[headers]]
  for = "/index.html"
  [headers.values]
    Cache-Control = "public, max-age=0, must-revalidate"
EOF

    print_info "Publishing to Netlify..."
    netlify deploy --prod --dir "$DIST_DIR"

    print_success "Deployed to Netlify"
}

# Vercel deployment
deploy_vercel() {
    print_info "Deploying to Vercel..."

    if ! command -v vercel >/dev/null 2>&1; then
        print_error "Vercel CLI not found. Install with: npm install -g vercel"
        exit 1
    fi

    # Create vercel.json
    cat > "${PROJECT_ROOT}/vercel.json" <<EOF
{
  "buildCommand": "./scripts/build-wasm.sh",
  "outputDirectory": "dist",
  "headers": [
    {
      "source": "/(.*).wasm",
      "headers": [
        {
          "key": "Content-Type",
          "value": "application/wasm"
        },
        {
          "key": "Cache-Control",
          "value": "public, max-age=31536000, immutable"
        }
      ]
    },
    {
      "source": "/index.html",
      "headers": [
        {
          "key": "Cache-Control",
          "value": "public, max-age=0, must-revalidate"
        }
      ]
    }
  ]
}
EOF

    print_info "Publishing to Vercel..."
    vercel --prod

    print_success "Deployed to Vercel"
}

# Docker deployment
deploy_docker() {
    print_info "Building Docker container..."

    # Create Dockerfile
    cat > "${PROJECT_ROOT}/Dockerfile.wasm" <<EOF
FROM nginx:alpine

# Copy build artifacts
COPY dist/ /usr/share/nginx/html/

# Copy nginx configuration
COPY <<'NGINX_CONF' /etc/nginx/conf.d/default.conf
server {
    listen 80;
    server_name localhost;
    root /usr/share/nginx/html;
    index index.html;

    # MIME types
    types {
        application/wasm wasm;
    }

    # Gzip compression
    gzip on;
    gzip_types application/wasm application/javascript text/css;
    gzip_vary on;

    # Cache control
    location ~* \\.wasm$ {
        add_header Cache-Control "public, max-age=31536000, immutable";
        add_header Content-Type "application/wasm";
    }

    location ~* \\.js$ {
        add_header Cache-Control "public, max-age=31536000, immutable";
    }

    location = /index.html {
        add_header Cache-Control "public, max-age=0, must-revalidate";
    }

    # PWA manifest
    location = /manifest.json {
        add_header Cache-Control "public, max-age=86400";
    }

    # Fallback to index.html
    location / {
        try_files \$uri \$uri/ /index.html;
    }
}
NGINX_CONF

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
EOF

    # Build Docker image
    docker build -f Dockerfile.wasm -t rusty-audio-pwa:latest .

    print_success "Docker image built: rusty-audio-pwa:latest"
    echo ""
    echo "Run with: docker run -p 8080:80 rusty-audio-pwa:latest"
}

print_success "Deployment complete!"
