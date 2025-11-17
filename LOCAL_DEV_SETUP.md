# Local Development Setup for Rusty Audio

Complete guide for setting up a local development environment with proper multithreading support.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Development Workflows](#development-workflows)
4. [Configuration Files](#configuration-files)
5. [Testing and Validation](#testing-and-validation)
6. [Troubleshooting](#troubleshooting)
7. [Advanced Usage](#advanced-usage)

---

## Prerequisites

### Required Tools

1. **Rust and Cargo** (latest stable)
   ```bash
   # Install via rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Add WASM target
   rustup target add wasm32-unknown-unknown
   ```

2. **Trunk** (WASM bundler)
   ```bash
   cargo install trunk
   ```

3. **Node.js** (v18 or later)
   ```bash
   # Verify installation
   node --version  # Should be >= 18.0.0
   npm --version   # Should be >= 9.0.0
   ```

4. **wasm-opt** (optional but recommended)
   ```bash
   # macOS
   brew install binaryen

   # Ubuntu/Debian
   sudo apt install binaryen

   # Windows
   # Download from https://github.com/WebAssembly/binaryen/releases
   ```

### Optional Tools

- **Playwright** (for browser testing)
- **Brotli** (for compression)
- **Wrangler** (for Cloudflare Workers deployment)

---

## Quick Start

### 1. Clone and Install Dependencies

```bash
cd rusty-audio

# Install Node.js dependencies
npm install
```

### 2. Build the WASM Application

```bash
# Option A: Using npm scripts (recommended)
npm run build:wasm

# Option B: Using trunk directly
trunk build --release

# Option C: Fast debug build
npm run build:debug
```

### 3. Start the Development Server

```bash
# Option A: Node.js development server (recommended for local dev)
npm run dev

# Option B: Miniflare (Cloudflare Workers emulator)
npm run dev:miniflare

# Option C: Build and serve in one command
./scripts/build-and-serve.sh
```

### 4. Access the Application

Open your browser to:
- **Main app**: http://localhost:8080
- **Health check**: http://localhost:8080/health
- **Features API**: http://localhost:8080/api/features
- **WASM info**: http://localhost:8080/api/wasm/info

---

## Development Workflows

### Workflow 1: Standard Development (Fastest)

For rapid iteration during development:

```bash
# Terminal 1: Watch and rebuild on changes
npm run watch:wasm

# Terminal 2: Run development server
npm run dev

# Now edit src/ files - they'll rebuild automatically
```

### Workflow 2: Build and Serve (Most Reliable)

For comprehensive building with validation:

```bash
# Build, optimize, compress, and serve
./scripts/build-and-serve.sh --prod

# Or skip optimization for faster builds
./scripts/build-and-serve.sh --skip-optimize
```

### Workflow 3: Miniflare (Cloudflare Testing)

To test Cloudflare Workers deployment locally:

```bash
# Start Miniflare with persistent storage
npm run dev:miniflare

# Or use wrangler directly
wrangler dev --local --persist
```

### Workflow 4: Production Testing

To test production build locally:

```bash
# Build production assets
npm run build

# Serve with production settings
npm run serve:prod
```

---

## Configuration Files

### `wrangler.toml` - Cloudflare Workers Configuration

Configures Miniflare and Cloudflare Workers deployment:

```toml
[dev]
ip = "127.0.0.1"
port = 8787
local_protocol = "http"

# Critical headers for multithreading
[[dev.headers]]
name = "Cross-Origin-Opener-Policy"
value = "same-origin"

[[dev.headers]]
name = "Cross-Origin-Embedder-Policy"
value = "require-corp"

[[dev.headers]]
name = "Cross-Origin-Resource-Policy"
value = "cross-origin"
```

### `Trunk.toml` - WASM Build Configuration

Controls how Trunk builds the WASM application:

```toml
[build]
dist = "dist"
release = true

[build.rust]
rustflags = ["-C", "lto=off"]
cargo-args = ["--lib", "--no-default-features"]

[serve]
open = true
```

### `package.json` - Build Scripts

Key npm scripts for development:

- `npm run dev` - Start development server
- `npm run build` - Production build
- `npm run build:wasm` - Build WASM only
- `npm run validate` - Validate build
- `npm run health` - Health check
- `npm run test:threading` - Test multithreading

---

## Testing and Validation

### Validate Build

After building, validate the output:

```bash
npm run validate
```

**Checks:**
- ✓ Required files present
- ✓ WASM binary integrity
- ✓ JavaScript bindings
- ✓ Proper headers in HTML
- ✓ Threading markers

### Health Check

Check if the server is running correctly:

```bash
# Start server first
npm run dev

# In another terminal
npm run health
```

**Tests:**
- ✓ Server connectivity
- ✓ Endpoint availability
- ✓ COOP/COEP/CORP headers
- ✓ WASM binary accessibility
- ✓ Feature detection

### Threading Test

Test multithreading capabilities:

```bash
# Requires Playwright
npm run test:threading
```

**Validates:**
- ✓ Cross-origin isolation
- ✓ SharedArrayBuffer availability
- ✓ Web Worker support
- ✓ WASM threading features
- ✓ Response headers

### Browser Testing

Manual testing in browsers:

1. **Open Developer Tools** (F12)
2. **Check Console** for:
   ```javascript
   console.log('crossOriginIsolated:', crossOriginIsolated);  // Should be true
   console.log('SharedArrayBuffer:', typeof SharedArrayBuffer);  // Should be 'function'
   ```
3. **Check Network Tab**:
   - `rusty_audio_bg.wasm` should have `Content-Type: application/wasm`
   - Response headers should include COOP/COEP/CORP
4. **Check Application Tab**:
   - Service Worker should be registered (if enabled)
   - Cache should be populated

---

## Troubleshooting

### Issue: SharedArrayBuffer is undefined

**Symptoms:**
- `crossOriginIsolated === false`
- `SharedArrayBuffer is not defined`
- WASM threading fails

**Solutions:**

1. **Check Response Headers**:
   ```bash
   curl -I http://localhost:8080
   ```

   Should include:
   ```
   Cross-Origin-Opener-Policy: same-origin
   Cross-Origin-Embedder-Policy: require-corp
   Cross-Origin-Resource-Policy: cross-origin
   ```

2. **Restart Server**:
   ```bash
   # Kill any running servers
   pkill -f "node.*dev-server"

   # Start fresh
   npm run dev
   ```

3. **Clear Browser Cache**:
   - Open DevTools → Application → Clear Storage
   - Hard reload: Ctrl+Shift+R (Windows/Linux) or Cmd+Shift+R (Mac)

### Issue: WASM Binary Not Found

**Symptoms:**
- 404 error for `rusty_audio_bg.wasm`
- Build validation fails

**Solutions:**

1. **Rebuild WASM**:
   ```bash
   npm run clean
   npm run build:wasm
   ```

2. **Check dist directory**:
   ```bash
   ls -la dist/
   ```

   Should contain:
   - `index.html`
   - `rusty_audio_bg.wasm`
   - `rusty_audio.js`

3. **Verify build output**:
   ```bash
   npm run validate
   ```

### Issue: Server Port Already in Use

**Symptoms:**
- `EADDRINUSE` error
- Server fails to start

**Solutions:**

1. **Find and kill process**:
   ```bash
   # macOS/Linux
   lsof -ti:8080 | xargs kill -9

   # Windows
   netstat -ano | findstr :8080
   taskkill /PID <PID> /F
   ```

2. **Use different port**:
   ```bash
   npm run dev -- --port 8081
   ```

### Issue: Build is Slow

**Symptoms:**
- `trunk build` takes >5 minutes
- Frequent rebuilds during development

**Solutions:**

1. **Use debug build**:
   ```bash
   npm run build:debug  # Much faster than release
   ```

2. **Skip optimization**:
   ```bash
   ./scripts/build-and-serve.sh --skip-optimize
   ```

3. **Use incremental builds**:
   ```bash
   # Watch mode - only rebuilds changed files
   npm run watch:wasm
   ```

### Issue: MIME Type Warnings

**Symptoms:**
- Browser console shows MIME type warnings
- JavaScript/WASM fails to load

**Solutions:**

1. **Check server headers**:
   ```bash
   curl -I http://localhost:8080/rusty_audio_bg.wasm
   ```

   Should show: `Content-Type: application/wasm`

2. **Restart development server**:
   ```bash
   npm run dev
   ```

---

## Advanced Usage

### Custom Build Flags

Set environment variables for custom builds:

```bash
# Maximum optimization
RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals,+simd" \
  npm run build:wasm

# Debug symbols
RUSTFLAGS="-C debuginfo=2" npm run build:debug

# Custom stack size
RUSTFLAGS="-C link-arg=-zstack-size=8388608" npm run build:wasm
```

### Profiling Build Performance

```bash
# Time the build
time npm run build:wasm

# Verbose cargo output
CARGO_LOG=cargo::core::compiler::fingerprint=trace \
  trunk build --release
```

### Custom Server Options

```bash
# Start server with options
node scripts/dev-server.js --port 3000 --verbose

# Production mode
NODE_ENV=production npm run serve
```

### Deploy to Cloudflare Pages

```bash
# Login to Cloudflare
wrangler login

# Build for production
npm run build

# Deploy
npm run deploy:cloudflare
```

### Deploy to Netlify

```bash
# Install Netlify CLI
npm install -g netlify-cli

# Login
netlify login

# Deploy
npm run deploy:netlify
```

### Deploy to Vercel

```bash
# Install Vercel CLI
npm install -g vercel

# Deploy
npm run deploy:vercel
```

---

## Performance Optimization

### Build Size Optimization

```bash
# 1. Build with release optimizations
npm run build

# 2. Run wasm-opt
npm run optimize:wasm

# 3. Compress assets
npm run compress

# 4. Check final sizes
du -h dist/rusty_audio_bg.wasm*
```

### Runtime Performance Monitoring

The development server includes a performance monitor. Enable it by pressing `P` in the browser or adding to URL:

```
http://localhost:8080/?perf=1
```

### Bundle Analysis

Analyze the WASM bundle:

```bash
# Install wasm-objdump
cargo install wasm-tools

# Dump WASM sections
wasm-objdump -h dist/rusty_audio_bg.wasm

# Analyze size
wasm-objdump -s dist/rusty_audio_bg.wasm | less
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | Server port | `8080` |
| `NODE_ENV` | Environment mode | `development` |
| `RUSTFLAGS` | Rust compiler flags | See Trunk.toml |
| `RUST_LOG` | Rust logging level | `warn` |
| `MAX_WORKER_THREADS` | Max worker threads | `8` |

---

## Development Server API

### Endpoints

- `GET /` - Main application
- `GET /health` - Health check
- `GET /api/features` - Feature detection
- `GET /api/wasm/info` - WASM binary information
- `GET /api/metrics` - Performance metrics

### Example API Usage

```bash
# Check health
curl http://localhost:8080/health | jq

# Get features
curl http://localhost:8080/api/features | jq

# Get WASM info
curl http://localhost:8080/api/wasm/info | jq
```

---

## Next Steps

1. **Review the architecture**: See `WASM_MULTITHREADED_WGPU_ARCHITECTURE.md`
2. **Understand threading**: Read `WASM_MULTITHREADING_GUIDE.md`
3. **Deploy to production**: Follow `MULTITHREADED_WASM_DEPLOYMENT.md`
4. **Optimize performance**: See `PERFORMANCE_OPTIMIZATION_GUIDE.md`

---

## Getting Help

- **Documentation**: Check the `docs/` directory
- **Issues**: Open an issue on GitHub
- **Logs**: Check browser DevTools console
- **Verbose mode**: Run `npm run dev -- --verbose`

---

**Last Updated**: 2024-11-16

**Version**: 0.2.0
