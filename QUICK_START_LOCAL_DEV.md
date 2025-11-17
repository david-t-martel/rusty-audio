# Quick Start: Local Development

**TL;DR: Get Rusty Audio running locally in 3 minutes**

## One-Command Setup

```bash
# 1. Install dependencies
npm install

# 2. Build WASM
npm run build:wasm

# 3. Start server
npm run dev
```

**Then open**: http://localhost:8080

---

## Common Commands

| Task | Command |
|------|---------|
| **Development** | `npm run dev` |
| **Build** | `npm run build:wasm` |
| **Production** | `npm run build` |
| **Validate** | `npm run validate` |
| **Health Check** | `npm run health` |
| **Test Threading** | `npm run test:threading` |
| **Clean** | `npm run clean` |

---

## Troubleshooting Quick Fixes

### 🔴 SharedArrayBuffer undefined

```bash
# Check headers
curl -I http://localhost:8080 | grep -i "cross-origin"

# Restart server
npm run dev
```

Should show:
```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

### 🔴 WASM not found

```bash
npm run clean
npm run build:wasm
npm run dev
```

### 🔴 Port in use

```bash
# Use different port
npm run dev -- --port 8081
```

### 🔴 Build too slow

```bash
# Debug build (faster)
npm run build:debug
npm run dev
```

---

## File Structure

```
rusty-audio/
├── dist/                    # Built output (don't commit)
│   ├── index.html
│   ├── rusty_audio_bg.wasm  # Main WASM binary
│   └── rusty_audio.js       # JS bindings
│
├── scripts/                 # Development scripts
│   ├── dev-server.js        # Local dev server
│   ├── build-and-serve.sh   # Unified build script
│   ├── validate-build.js    # Build validator
│   ├── health-check.js      # Health checker
│   └── test-threading.js    # Threading tests
│
├── static/                  # Static assets
│   ├── icons/
│   ├── manifest.webmanifest
│   └── service-worker.js
│
├── wrangler.toml           # Miniflare config
├── Trunk.toml              # WASM build config
└── package.json            # npm scripts
```

---

## API Endpoints

| Endpoint | Description |
|----------|-------------|
| `/` | Main application |
| `/health` | Health check |
| `/api/features` | Feature detection |
| `/api/wasm/info` | WASM binary info |
| `/api/metrics` | Performance metrics |

---

## Verification Checklist

After starting the server, verify:

- [ ] `crossOriginIsolated === true` (check browser console)
- [ ] `typeof SharedArrayBuffer === 'function'`
- [ ] WASM binary loads without errors
- [ ] Service worker registers (if enabled)
- [ ] No MIME type warnings in console

---

## Development Modes

### Fast Development (Recommended)

```bash
# Terminal 1: Auto-rebuild
npm run watch:wasm

# Terminal 2: Server
npm run dev
```

### Production Testing

```bash
npm run build
npm run serve:prod
```

### Miniflare (Cloudflare Workers)

```bash
npm run dev:miniflare
```

---

## Next Steps

1. ✅ **Verify setup**: `npm run health`
2. 📚 **Read full docs**: `LOCAL_DEV_SETUP.md`
3. 🏗️ **Understand architecture**: `WASM_MULTITHREADED_WGPU_ARCHITECTURE.md`
4. 🚀 **Deploy**: `MULTITHREADED_WASM_DEPLOYMENT.md`

---

## Getting Help

- **Verbose logging**: `npm run dev -- --verbose`
- **Validate build**: `npm run validate`
- **Check health**: `npm run health`
- **Test threading**: `npm run test:threading`

---

**Need more details?** See `LOCAL_DEV_SETUP.md` for comprehensive documentation.

**Last Updated**: 2024-11-16
