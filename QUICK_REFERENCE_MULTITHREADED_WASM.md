# Quick Reference: Multithreaded WASM Build & Deploy

## Essential Build Commands

```bash
# Production build
trunk build --release

# Clean build (recommended before deployment)
trunk clean && cargo clean && trunk build --release

# Development server with auto-reload
trunk serve --release

# Check build configuration
cargo check --target wasm32-unknown-unknown
```

## Verification Commands

```bash
# Verify WASM has threading features
wasm-objdump -x dist/rusty-audio_bg.wasm | grep -i "shared"
# Expected: "shared memory" or atomics features

# Check file sizes
ls -lh dist/
# rusty-audio_bg.wasm should be ~2-5 MB

# Verify headers after deployment
curl -I https://your-site.com | grep -i "cross-origin"
# Expected: COOP and COEP headers present
```

## Browser Console Verification

```javascript
// Check SharedArrayBuffer availability
console.log('SharedArrayBuffer:', typeof SharedArrayBuffer !== 'undefined');
// Expected: true

// Check cross-origin isolation
console.log('crossOriginIsolated:', crossOriginIsolated);
// Expected: true

// Verify service worker
navigator.serviceWorker.getRegistration().then(reg =>
    console.log('Service Worker:', !!reg)
);
// Expected: true
```

## Deployment Commands

### Netlify
```bash
netlify deploy --prod
```

### Vercel
```bash
vercel --prod
```

### Cloudflare Pages
```bash
wrangler pages publish dist --project-name rusty-audio
```

## Required Headers

All deployments must include:
```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
Cross-Origin-Resource-Policy: cross-origin
```

## Configuration Files

- `.cargo/config.toml` - WASM build flags
- `Trunk.toml` - wasm-opt configuration
- `netlify.toml` - Netlify deployment config
- `vercel.json` - Vercel deployment config
- `static/service-worker.js` - Service worker with headers
- `static/_headers` - Static headers file

## Troubleshooting One-Liners

```bash
# Fix: SharedArrayBuffer undefined
# 1. Hard reload: Ctrl+Shift+R
# 2. Unregister service worker:
navigator.serviceWorker.getRegistrations().then(r => r.forEach(reg => reg.unregister()))

# Fix: WASM fails to load
# Rebuild from scratch:
trunk clean && cargo clean && trunk build --release

# Fix: Headers not applied
# Check headers in Network tab, then verify CDN config

# Fix: Performance issues
# Profile with DevTools Performance tab
# Check WASM size: ls -lh dist/*.wasm
```

## Key WASM Build Flags

```toml
# In .cargo/config.toml
rustflags = [
    "-C", "target-feature=+atomics,+bulk-memory,+mutable-globals",
    "-C", "link-arg=--shared-memory",
    "-C", "link-arg=--max-memory=4294967296"
]
```

## wasm-opt Configuration

```toml
# In Trunk.toml
[build.wasm-opt]
args = [
    "--enable-threads",
    "--enable-bulk-memory",
    "--enable-mutable-globals",
    "-Oz"
]
```

## Browser Support

Minimum versions:
- Chrome 92+
- Firefox 89+
- Safari 15.2+
- Edge 92+

## Common Issues & Quick Fixes

| Issue | Quick Fix |
|-------|-----------|
| SharedArrayBuffer undefined | Hard reload + check headers |
| WASM import error | Rebuild from scratch |
| Service worker not registering | HTTPS required |
| Headers not applied | Verify CDN config |
| Performance degradation | Profile and optimize thread pool |

## Performance Targets

- First Contentful Paint: < 2s
- Time to Interactive: < 3s
- WASM bundle: 2-5 MB (compressed: 0.5-1.5 MB)
- Thread pool: 2-4 threads recommended

## Documentation

Full guides available:
- `MULTITHREADED_WASM_DEPLOYMENT.md` - Complete deployment guide
- `DEPLOYMENT_CHECKLIST_MULTITHREADED.md` - Step-by-step checklist
- `MULTITHREADED_WASM_CONFIG_SUMMARY.md` - Configuration summary

---

**Build**: `trunk build --release`
**Deploy**: `netlify deploy --prod` (or Vercel/Cloudflare)
**Verify**: Check `crossOriginIsolated` in browser console
