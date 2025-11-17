# WASM Threading Setup Guide

This document describes the multithreaded WASM setup for Rusty Audio and provides testing instructions.

## Overview

Rusty Audio is configured for multithreaded WASM execution using SharedArrayBuffer and Web Workers. This enables parallel audio processing for improved performance.

## Architecture

### Cross-Origin Isolation

To enable SharedArrayBuffer (required for WASM threads), the application implements cross-origin isolation through:

1. **HTTP Headers** (via `_headers` file):
   - `Cross-Origin-Opener-Policy: same-origin`
   - `Cross-Origin-Embedder-Policy: require-corp`
   - `Cross-Origin-Resource-Policy: cross-origin`

2. **Service Worker** (dynamic header injection):
   - Adds COOP/COEP headers to all responses
   - Ensures cached resources have proper headers
   - Handles offline mode with threading support

3. **HTML Meta Tags** (fallback):
   - `<meta http-equiv="Cross-Origin-Opener-Policy" content="same-origin"/>`
   - `<meta http-equiv="Cross-Origin-Embedder-Policy" content="require-corp"/>`

### Files Modified/Created

#### Modified Files:
- `index.html` - Added feature detection, loading states, error handling
- `static/service-worker.js` - Added COOP/COEP header injection
- `static/_headers` - Updated with comprehensive cross-origin headers
- `static/manifest.webmanifest` - Enhanced PWA metadata

#### New Files:
- `static/wasm-worker-init.js` - Worker pool management system
- `WASM_THREADING_SETUP.md` - This documentation
- `BROWSER_COMPATIBILITY.md` - Browser support matrix

## Browser Compatibility

### Fully Supported (WASM Threads Enabled)

| Browser | Version | SharedArrayBuffer | Notes |
|---------|---------|-------------------|-------|
| Chrome/Edge | 92+ | ✅ Yes | Full support with HTTPS + headers |
| Firefox | 79+ | ✅ Yes | Full support with HTTPS + headers |
| Safari | 15.2+ | ✅ Yes | iOS 15.2+, macOS 12.1+ |
| Opera | 78+ | ✅ Yes | Chromium-based, same as Chrome |

### Partial Support (No Threads, Single-threaded WASM)

| Browser | Version | SharedArrayBuffer | Notes |
|---------|---------|-------------------|-------|
| Chrome | 60-91 | ❌ No | Falls back to single-threaded |
| Firefox | 52-78 | ❌ No | Falls back to single-threaded |
| Safari | 10.1-15.1 | ❌ No | Falls back to single-threaded |
| Mobile Chrome | Android 92+ | ✅ Yes | Requires HTTPS |
| Mobile Safari | iOS 15.2+ | ✅ Yes | Full support |

### Not Supported

- Internet Explorer (no WASM support)
- Chrome < 60 (no WASM support)
- Firefox < 52 (no WASM support)
- Safari < 10.1 (no WASM support)

## Testing Instructions

### 1. Local Development Testing

#### Test with Trunk (Development Server)

```bash
# Build and serve locally
trunk serve --open

# Access at: http://127.0.0.1:8080
```

**Note**: `window.crossOriginIsolated` will be **false** on HTTP/localhost. This is expected - SharedArrayBuffer may still work in development mode on some browsers.

#### Test with HTTPS (Required for Production)

```bash
# Option 1: Use Trunk with HTTPS
trunk serve --open --address 0.0.0.0 --port 8080

# Option 2: Build and serve with Python HTTPS server
trunk build --release
cd dist
python -m http.server 8443 --bind 0.0.0.0

# Option 3: Use Caddy (automatically provides HTTPS)
caddy file-server --root dist --listen :8443
```

### 2. Verify Cross-Origin Isolation

Open browser DevTools console and run:

```javascript
// Check if cross-origin isolated
console.log('Cross-Origin Isolated:', window.crossOriginIsolated);

// Check SharedArrayBuffer availability
console.log('SharedArrayBuffer:', typeof SharedArrayBuffer !== 'undefined');

// Check feature detection results
// (automatically logged by index.html on page load)
```

**Expected Output (Production/HTTPS):**
```
Cross-Origin Isolated: true
SharedArrayBuffer: true
[Rusty Audio] Feature detection: {
  WebAssembly: true,
  WebAssembly Threads: true,
  SharedArrayBuffer: true,
  Cross-Origin Isolation: true,
  Service Worker: true,
  Web Audio API: true
}
```

### 3. Verify Service Worker Headers

```javascript
// Check service worker status
navigator.serviceWorker.getRegistration().then(reg => {
  console.log('Service Worker:', reg?.active?.state);
});

// Check response headers
fetch('/').then(response => {
  console.log('COOP:', response.headers.get('Cross-Origin-Opener-Policy'));
  console.log('COEP:', response.headers.get('Cross-Origin-Embedder-Policy'));
  console.log('CORP:', response.headers.get('Cross-Origin-Resource-Policy'));
});
```

**Expected Output:**
```
Service Worker: activated
COOP: same-origin
COEP: require-corp
CORP: cross-origin
```

### 4. Test Worker Pool Initialization

```javascript
// Check worker pool (if initialized)
if (window.wasmThreadPool) {
  console.log('Worker Pool:', window.wasmThreadPool);
  console.log('Max Workers:', window.wasmThreadPool.maxWorkers);
}

// Check WasmWorkerPool class
if (window.WasmWorkerPool) {
  console.log('WasmWorkerPool available');
  const pool = new WasmWorkerPool();
  console.log('Pool stats:', pool.getStats());
}
```

### 5. Performance Testing

```javascript
// Measure WASM load time
performance.mark('wasm-start');
// ... after WASM loads ...
performance.mark('wasm-end');
performance.measure('wasm-load', 'wasm-start', 'wasm-end');
console.log(performance.getEntriesByName('wasm-load'));

// Check thread count
console.log('Hardware Concurrency:', navigator.hardwareConcurrency);
```

## Deployment Configuration

### CDN Deployment (Cloudflare Pages, Netlify, Vercel)

All major CDNs support custom headers via configuration files:

#### Cloudflare Pages
The `static/_headers` file is automatically recognized by Cloudflare Pages.

**Deployment command:**
```bash
trunk build --release
# Upload dist/ to Cloudflare Pages
```

**Verify headers:**
```bash
curl -I https://your-domain.pages.dev/ | grep -i "cross-origin"
```

#### Netlify
Rename `_headers` to `_headers` (Netlify format is the same).

**netlify.toml** (optional):
```toml
[[headers]]
  for = "/*"
  [headers.values]
    Cross-Origin-Opener-Policy = "same-origin"
    Cross-Origin-Embedder-Policy = "require-corp"
    Cross-Origin-Resource-Policy = "cross-origin"
```

#### Vercel
Create `vercel.json`:

```json
{
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        { "key": "Cross-Origin-Opener-Policy", "value": "same-origin" },
        { "key": "Cross-Origin-Embedder-Policy", "value": "require-corp" },
        { "key": "Cross-Origin-Resource-Policy", "value": "cross-origin" }
      ]
    }
  ]
}
```

### Apache Server (.htaccess)

```apache
<IfModule mod_headers.c>
  Header set Cross-Origin-Opener-Policy "same-origin"
  Header set Cross-Origin-Embedder-Policy "require-corp"
  Header set Cross-Origin-Resource-Policy "cross-origin"

  <FilesMatch "\.(wasm|js)$">
    Header set Cache-Control "public, max-age=31536000, immutable"
  </FilesMatch>
</IfModule>
```

### Nginx (nginx.conf)

```nginx
location / {
  add_header Cross-Origin-Opener-Policy "same-origin";
  add_header Cross-Origin-Embedder-Policy "require-corp";
  add_header Cross-Origin-Resource-Policy "cross-origin";

  # Cache WASM and JS files
  location ~* \.(wasm|js)$ {
    add_header Cache-Control "public, max-age=31536000, immutable";
  }
}
```

## Troubleshooting

### Issue: `SharedArrayBuffer is not defined`

**Cause**: Cross-origin isolation not enabled.

**Solutions**:
1. Verify HTTPS is being used (not HTTP)
2. Check `window.crossOriginIsolated === true`
3. Inspect response headers for COOP/COEP
4. Clear browser cache and service worker
5. Verify `_headers` file is being deployed

**Debug command:**
```javascript
fetch('/').then(r => {
  console.log('COOP:', r.headers.get('Cross-Origin-Opener-Policy'));
  console.log('COEP:', r.headers.get('Cross-Origin-Embedder-Policy'));
});
```

### Issue: Service Worker Not Activating

**Cause**: Service worker registration failed or cached old version.

**Solutions**:
1. Unregister old service worker:
   ```javascript
   navigator.serviceWorker.getRegistrations().then(registrations => {
     registrations.forEach(reg => reg.unregister());
   });
   ```
2. Hard refresh: Ctrl+Shift+R (Windows/Linux) or Cmd+Shift+R (Mac)
3. Clear site data in DevTools (Application tab → Clear storage)

### Issue: WASM Load Timeout

**Cause**: WASM file too large or network issues.

**Solutions**:
1. Check WASM file size: `ls -lh dist/*.wasm`
2. Enable compression at CDN level (gzip/brotli)
3. Increase timeout in `index.html` (line 259)
4. Monitor Network tab in DevTools

### Issue: Worker Initialization Failure

**Cause**: Worker script not found or CORS issues.

**Solutions**:
1. Verify worker files exist in dist:
   ```bash
   ls dist/*.worker.js dist/*.worker.wasm
   ```
2. Check browser console for CORS errors
3. Ensure worker files have proper CORS headers
4. Verify CSP allows `worker-src 'self' blob:`

## Performance Metrics

### Expected Performance (with Threading)

- **WASM Load Time**: < 500ms (on fast connection)
- **Worker Pool Init**: < 200ms
- **Audio Processing Latency**: < 10ms
- **Thread Count**: Up to `navigator.hardwareConcurrency` (typically 4-16)

### Measuring Performance

```javascript
// WASM load time
performance.getEntriesByType('navigation')[0].domContentLoadedEventEnd

// Worker pool stats
if (window.WasmWorkerPool) {
  const pool = new WasmWorkerPool();
  console.log(pool.getStats());
}

// Audio latency (measure in app)
```

## Security Considerations

### Content Security Policy (CSP)

The application uses a strict CSP while allowing necessary WASM features:

```
script-src 'self' 'wasm-unsafe-eval' 'unsafe-eval'
worker-src 'self' blob:
child-src 'self' blob:
```

- `wasm-unsafe-eval` - Required for WASM instantiation
- `unsafe-eval` - Required for WASM worker initialization
- `blob:` - Required for inline worker scripts

### Cross-Origin Resource Sharing (CORS)

All resources must be served from the same origin OR have proper CORS headers:

```
Cross-Origin-Resource-Policy: cross-origin
```

This allows the service worker to cache and serve resources while maintaining cross-origin isolation.

## Future Enhancements

- [ ] Add worker thread pool size auto-tuning based on device capabilities
- [ ] Implement worker task prioritization for audio processing
- [ ] Add telemetry for thread utilization metrics
- [ ] Optimize worker warm-up time
- [ ] Add graceful degradation for low-end devices (reduce thread count)

## Resources

- [MDN: SharedArrayBuffer](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer)
- [MDN: Cross-Origin Isolation](https://developer.mozilla.org/en-US/docs/Web/API/crossOriginIsolated)
- [wasm-bindgen Threading](https://rustwasm.github.io/wasm-bindgen/reference/threading.html)
- [Chrome: Enabling SharedArrayBuffer](https://developer.chrome.com/blog/enabling-shared-array-buffer/)
