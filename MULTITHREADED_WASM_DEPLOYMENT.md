# Multithreaded WASM Deployment Guide

This document provides comprehensive instructions for building and deploying Rusty Audio with multithreaded WASM support using SharedArrayBuffer.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Build Configuration](#build-configuration)
4. [Building for Production](#building-for-production)
5. [Testing Locally](#testing-locally)
6. [Deployment Options](#deployment-options)
7. [Verification](#verification)
8. [Troubleshooting](#troubleshooting)

## Overview

Multithreaded WASM enables SharedArrayBuffer, which allows true parallelism in WebAssembly applications. This requires:

1. **Build-time configuration**: WASM atomics and bulk-memory features
2. **Server-side headers**: Cross-Origin-Opener-Policy (COOP) and Cross-Origin-Embedder-Policy (COEP)
3. **Service worker support**: Injecting headers when CDN doesn't support them

## Prerequisites

### Required Tools

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install trunk (WASM build tool)
cargo install trunk

# Install wasm-bindgen-cli (must match version in Cargo.lock)
cargo install wasm-bindgen-cli --version 0.2.x

# Install wasm-opt (for optimization)
# Windows: Download from https://github.com/WebAssembly/binaryen/releases
# Linux: apt install binaryen
# macOS: brew install binaryen
```

### Optional Tools

```bash
# For local HTTPS testing
cargo install basic-http-server

# For analyzing WASM bundle size
cargo install twiggy
```

## Build Configuration

### 1. Cargo Configuration (.cargo/config.toml)

The `.cargo/config.toml` file has been configured with:

```toml
[target.wasm32-unknown-unknown]
rustflags = [
    "-C", "target-feature=+atomics,+bulk-memory,+mutable-globals",
    "-C", "link-arg=--shared-memory",
    "-C", "link-arg=--max-memory=4294967296",  # 4GB max
]
```

**Key Features:**
- `+atomics`: Enable atomic operations for threading
- `+bulk-memory`: Bulk memory operations for efficiency
- `+mutable-globals`: Required for thread-local storage
- `--shared-memory`: Enable SharedArrayBuffer
- `--max-memory`: Set maximum WASM memory (4GB)

### 2. Trunk Configuration (Trunk.toml)

The `Trunk.toml` file configures wasm-opt:

```toml
[build.wasm-opt]
version = "116"
args = [
    "--enable-threads",
    "--enable-bulk-memory",
    "--enable-mutable-globals",
    "-Oz"  # Optimize for size
]
```

### 3. Service Worker (static/service-worker.js)

The service worker injects COOP/COEP headers:

```javascript
const COOP_COEP_HEADERS = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
  'Cross-Origin-Resource-Policy': 'cross-origin'
};
```

## Building for Production

### Standard Build Command

```bash
# Build with trunk (recommended)
trunk build --release

# Output directory: dist/
```

### Advanced Build Options

```bash
# Build with custom optimization
RUSTFLAGS="-C opt-level=z" trunk build --release

# Build with verbose output for debugging
trunk build --release -v

# Clean build (remove all cached artifacts)
trunk clean && trunk build --release
```

### Build Output Verification

After building, verify the output:

```bash
# Check dist/ directory structure
ls -lh dist/

# Expected files:
# - index.html
# - rusty-audio.js
# - rusty-audio_bg.wasm
# - service-worker.js
# - manifest.webmanifest
# - icons/
# - _headers (for Netlify/Cloudflare)

# Verify WASM file has threading enabled
wasm-objdump -x dist/rusty-audio_bg.wasm | grep -i "shared"
# Should see: "shared memory" or "atomics"
```

## Testing Locally

### Option 1: Using Trunk Serve (Development)

```bash
# Start development server with auto-reload
trunk serve --release

# Server runs on http://127.0.0.1:8080
```

**Note**: Trunk serve automatically injects COOP/COEP headers in development mode.

### Option 2: Using Local HTTPS Server (Production-like)

For testing service worker and COOP/COEP headers:

```bash
# Install basic-http-server with HTTPS support
cargo install basic-http-server

# Generate self-signed certificate (one-time setup)
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Serve dist/ directory with HTTPS
basic-http-server dist/ \
  --addr 127.0.0.1:8443 \
  --https \
  --cert cert.pem \
  --key key.pem

# Open https://127.0.0.1:8443 in browser
# Accept self-signed certificate warning
```

### Option 3: Using Python HTTP Server with Custom Headers

```bash
cd dist

# Create simple server script
cat > server.py << 'EOF'
from http.server import HTTPServer, SimpleHTTPRequestHandler

class CORSRequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Resource-Policy', 'cross-origin')
        SimpleHTTPRequestHandler.end_headers(self)

httpd = HTTPServer(('127.0.0.1', 8080), CORSRequestHandler)
print('Server running on http://127.0.0.1:8080')
httpd.serve_forever()
EOF

python3 server.py
```

### Verify SharedArrayBuffer Support

Open browser console and check:

```javascript
// Should return true
console.log(typeof SharedArrayBuffer !== 'undefined');

// Check cross-origin isolation
console.log(crossOriginIsolated);  // Should be true
```

## Deployment Options

### Option 1: Netlify (Recommended)

**Configuration**: `netlify.toml` (already created)

**Deployment Steps**:

```bash
# Install Netlify CLI
npm install -g netlify-cli

# Login to Netlify
netlify login

# Initialize project (first time only)
netlify init

# Deploy to preview
netlify deploy

# Deploy to production
netlify deploy --prod
```

**Netlify UI Deployment**:

1. Go to https://app.netlify.com
2. Click "Add new site" → "Import an existing project"
3. Connect to your Git repository
4. Build settings:
   - Build command: `trunk build --release`
   - Publish directory: `dist`
5. Deploy!

**Header Verification**:

After deployment, verify headers:

```bash
curl -I https://your-app.netlify.app | grep -i "cross-origin"

# Expected output:
# Cross-Origin-Opener-Policy: same-origin
# Cross-Origin-Embedder-Policy: require-corp
```

### Option 2: Vercel

**Configuration**: `vercel.json` (already created)

**Deployment Steps**:

```bash
# Install Vercel CLI
npm install -g vercel

# Login to Vercel
vercel login

# Deploy to preview
vercel

# Deploy to production
vercel --prod
```

**Note**: Vercel requires additional setup for Rust builds. Consider using GitHub Actions for building.

### Option 3: Cloudflare Pages

**Configuration**: Use `static/_headers` file (already exists)

**Deployment via Wrangler**:

```bash
# Install Wrangler
npm install -g wrangler

# Login to Cloudflare
wrangler login

# Deploy
wrangler pages publish dist --project-name rusty-audio
```

**Deployment via Cloudflare UI**:

1. Go to https://dash.cloudflare.com
2. Navigate to Pages
3. Create new project from Git
4. Build settings:
   - Build command: `trunk build --release`
   - Build output directory: `dist`
5. Deploy!

### Option 4: GitHub Pages

**Limitation**: GitHub Pages doesn't support custom headers by default. Must rely on service worker.

**Deployment Steps**:

```bash
# Build for production
trunk build --release --public-url /rusty-audio/

# Deploy using gh-pages or GitHub Actions
# See .github/workflows/ for CI/CD examples
```

**Important**: GitHub Pages requires service worker to inject COOP/COEP headers.

### Option 5: Self-Hosted (Nginx/Apache)

**Nginx Configuration**:

```nginx
server {
    listen 443 ssl http2;
    server_name rusty-audio.example.com;

    root /var/www/rusty-audio/dist;
    index index.html;

    # CRITICAL: Headers for SharedArrayBuffer support
    add_header Cross-Origin-Opener-Policy "same-origin" always;
    add_header Cross-Origin-Embedder-Policy "require-corp" always;
    add_header Cross-Origin-Resource-Policy "cross-origin" always;

    # Security headers
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "DENY" always;
    add_header Referrer-Policy "no-referrer" always;

    # WASM-specific content type
    location ~* \.wasm$ {
        add_header Content-Type "application/wasm";
        add_header Cache-Control "public, max-age=31536000, immutable";
        add_header Cross-Origin-Opener-Policy "same-origin" always;
        add_header Cross-Origin-Embedder-Policy "require-corp" always;
    }

    # Service worker - no caching
    location = /service-worker.js {
        add_header Cache-Control "no-store, no-cache, must-revalidate";
        add_header Cross-Origin-Opener-Policy "same-origin" always;
        add_header Cross-Origin-Embedder-Policy "require-corp" always;
    }

    # SPA routing
    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

**Apache Configuration** (.htaccess):

```apache
# CRITICAL: Headers for SharedArrayBuffer support
Header always set Cross-Origin-Opener-Policy "same-origin"
Header always set Cross-Origin-Embedder-Policy "require-corp"
Header always set Cross-Origin-Resource-Policy "cross-origin"

# Security headers
Header always set X-Content-Type-Options "nosniff"
Header always set X-Frame-Options "DENY"
Header always set Referrer-Policy "no-referrer"

# WASM content type
<FilesMatch "\.wasm$">
    Header set Content-Type "application/wasm"
    Header set Cache-Control "public, max-age=31536000, immutable"
</FilesMatch>

# Service worker - no caching
<FilesMatch "service-worker\.js$">
    Header set Cache-Control "no-store, no-cache, must-revalidate"
</FilesMatch>

# SPA routing
RewriteEngine On
RewriteCond %{REQUEST_FILENAME} !-f
RewriteCond %{REQUEST_FILENAME} !-d
RewriteRule ^ index.html [L]
```

## Verification

### 1. Check Build Artifacts

```bash
# Verify WASM has threading features
wasm-objdump -x dist/rusty-audio_bg.wasm | grep -A 10 "memory"

# Expected output should include:
# - shared memory: true
# - atomics: enabled

# Check file sizes
ls -lh dist/
# rusty-audio_bg.wasm should be ~2-5 MB (depending on features)
```

### 2. Browser Console Checks

After deployment, open browser DevTools and run:

```javascript
// 1. Check SharedArrayBuffer availability
console.log('SharedArrayBuffer available:', typeof SharedArrayBuffer !== 'undefined');
// Expected: true

// 2. Check cross-origin isolation
console.log('Cross-origin isolated:', crossOriginIsolated);
// Expected: true

// 3. Check service worker registration
navigator.serviceWorker.getRegistration().then(reg => {
    console.log('Service Worker registered:', !!reg);
});
// Expected: Service Worker registered: true

// 4. Check headers via fetch
fetch(window.location.href).then(response => {
    console.log('COOP:', response.headers.get('cross-origin-opener-policy'));
    console.log('COEP:', response.headers.get('cross-origin-embedder-policy'));
});
// Expected:
// COOP: same-origin
// COEP: require-corp
```

### 3. Network Tab Verification

1. Open DevTools → Network tab
2. Reload page
3. Click on main document (index.html)
4. Check Response Headers:
   - `Cross-Origin-Opener-Policy: same-origin`
   - `Cross-Origin-Embedder-Policy: require-corp`
5. Check WASM file headers:
   - `Content-Type: application/wasm`

### 4. Lighthouse Audit

```bash
# Install Lighthouse
npm install -g @lhci/cli lighthouse

# Run audit
lighthouse https://your-app.netlify.app \
  --output html \
  --output-path ./lighthouse-report.html

# Check PWA score and COOP/COEP warnings
```

## Troubleshooting

### Issue 1: SharedArrayBuffer is undefined

**Symptoms**:
```javascript
console.log(typeof SharedArrayBuffer);  // undefined
```

**Solutions**:

1. **Check headers are set**:
   ```bash
   curl -I https://your-app.com | grep -i cross-origin
   ```
   Should see COOP and COEP headers.

2. **Verify service worker is active**:
   ```javascript
   navigator.serviceWorker.getRegistration().then(reg => console.log(reg));
   ```

3. **Check crossOriginIsolated**:
   ```javascript
   console.log(crossOriginIsolated);  // Should be true
   ```

4. **Force service worker update**:
   - DevTools → Application → Service Workers → Unregister
   - Hard reload (Ctrl+Shift+R)

### Issue 2: WASM fails to load with threading error

**Symptoms**:
```
LinkError: WebAssembly.instantiate(): Imported memory must be shared
```

**Solutions**:

1. **Verify build flags**:
   ```bash
   # Check .cargo/config.toml has:
   # -C target-feature=+atomics,+bulk-memory,+mutable-globals
   # -C link-arg=--shared-memory
   ```

2. **Rebuild from scratch**:
   ```bash
   trunk clean
   cargo clean
   trunk build --release
   ```

3. **Check wasm-opt version**:
   ```bash
   wasm-opt --version
   # Should be version 116+
   ```

### Issue 3: Service worker not injecting headers

**Symptoms**:
- Headers missing in Network tab
- crossOriginIsolated = false

**Solutions**:

1. **Check service worker script**:
   - View source of `/service-worker.js`
   - Verify `addCrossOriginHeaders()` function exists

2. **Force service worker reload**:
   ```javascript
   // In console
   navigator.serviceWorker.getRegistrations().then(regs => {
       regs.forEach(reg => reg.unregister());
   });
   location.reload();
   ```

3. **Check service worker scope**:
   ```javascript
   navigator.serviceWorker.getRegistration().then(reg => {
       console.log('Scope:', reg.scope);
   });
   // Should be: https://your-app.com/
   ```

### Issue 4: CDN/Server headers conflict

**Symptoms**:
- Multiple COOP/COEP headers in response
- Browser warning about duplicate headers

**Solutions**:

1. **Remove duplicate header sources**:
   - Check CDN configuration (Netlify/Vercel/Cloudflare)
   - Check service worker (may need to skip header injection if CDN sets them)

2. **Conditional header injection in service worker**:
   ```javascript
   function addCrossOriginHeaders(response) {
       const headers = new Headers(response.headers);

       // Only add if not already present
       if (!headers.has('Cross-Origin-Opener-Policy')) {
           headers.set('Cross-Origin-Opener-Policy', 'same-origin');
       }
       // ... same for other headers
   }
   ```

### Issue 5: Build fails with "undefined reference to __wasm_init_memory"

**Symptoms**:
```
error: linking with `rust-lld` failed
undefined reference to __wasm_init_memory
```

**Solutions**:

1. **Update Rust toolchain**:
   ```bash
   rustup update stable
   rustup target add wasm32-unknown-unknown --toolchain stable
   ```

2. **Update wasm-bindgen**:
   ```bash
   cargo install wasm-bindgen-cli --force
   ```

3. **Clean and rebuild**:
   ```bash
   cargo clean
   rm -rf target/
   trunk build --release
   ```

### Issue 6: Performance degradation with threading

**Symptoms**:
- App slower than single-threaded version
- High memory usage

**Solutions**:

1. **Profile with DevTools**:
   - Performance tab → Record → Analyze bottlenecks

2. **Adjust thread pool size** (in Rust code):
   ```rust
   // Use fewer threads if overhead is high
   rayon::ThreadPoolBuilder::new()
       .num_threads(2)  // Start with 2 threads
       .build_global()
       .unwrap();
   ```

3. **Use threading selectively**:
   - Only parallelize CPU-intensive operations
   - Avoid threading for small tasks (overhead > benefit)

### Issue 7: "Failed to fetch" errors in console

**Symptoms**:
```
Failed to fetch dynamically imported module
```

**Solutions**:

1. **Check CORS headers for subresources**:
   ```bash
   curl -I https://your-app.com/rusty-audio_bg.wasm | grep -i cross-origin
   ```

2. **Add Cross-Origin-Resource-Policy**:
   ```
   Cross-Origin-Resource-Policy: cross-origin
   ```

3. **Verify all assets are on same origin** or have proper CORS headers.

## Advanced Configuration

### Custom Memory Limits

Adjust maximum WASM memory in `.cargo/config.toml`:

```toml
[target.wasm32-unknown-unknown]
rustflags = [
    # ... other flags ...
    "-C", "link-arg=--max-memory=2147483648",  # 2GB instead of 4GB
]
```

### Thread Pool Configuration

In your Rust code:

```rust
use rayon::ThreadPoolBuilder;

fn init_thread_pool() {
    let num_threads = num_cpus::get().min(4);  // Cap at 4 threads
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .thread_name(|i| format!("wasm-worker-{}", i))
        .build_global()
        .expect("Failed to build thread pool");
}
```

### Memory Profiling

```bash
# Use Chrome DevTools Memory Profiler
# 1. Open DevTools → Memory
# 2. Take heap snapshot
# 3. Look for WASM memory usage
# 4. Check for memory leaks in SharedArrayBuffer

# Use twiggy for WASM analysis
twiggy top dist/rusty-audio_bg.wasm
twiggy dominators dist/rusty-audio_bg.wasm
```

## Performance Optimization Checklist

- [ ] Enable wasm-opt with `-Oz` flag
- [ ] Use `opt-level = "z"` in Cargo.toml for release profile
- [ ] Strip debug symbols: `strip = true`
- [ ] Enable aggressive caching for WASM files (1 year max-age)
- [ ] Use CDN for global distribution (Cloudflare, Netlify, etc.)
- [ ] Enable HTTP/2 or HTTP/3 on server
- [ ] Compress WASM with Brotli (handled by CDN usually)
- [ ] Lazy-load non-critical WASM modules
- [ ] Profile and optimize hot paths in Rust code

## Security Checklist

- [ ] COOP: same-origin header set
- [ ] COEP: require-corp header set
- [ ] CSP with 'wasm-unsafe-eval' for WASM
- [ ] X-Frame-Options: DENY
- [ ] X-Content-Type-Options: nosniff
- [ ] Referrer-Policy: no-referrer
- [ ] HTTPS only (HSTS enabled)
- [ ] Service worker served over HTTPS
- [ ] No inline scripts (use CSP to enforce)
- [ ] Subresource Integrity (SRI) for third-party resources

## Resources

- [MDN: SharedArrayBuffer](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer)
- [MDN: Cross-Origin Isolation](https://developer.mozilla.org/en-US/docs/Web/API/crossOriginIsolated)
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [Trunk Documentation](https://trunkrs.dev/)

## Support

For issues related to:
- **Build failures**: Check GitHub Issues or Rust WASM Discord
- **Deployment**: Consult CDN-specific documentation
- **Performance**: Use browser DevTools profiler and Rust profiling tools
- **Security**: Review OWASP WASM security guidelines

---

**Last Updated**: 2025-01-16
**Version**: 1.0.0
