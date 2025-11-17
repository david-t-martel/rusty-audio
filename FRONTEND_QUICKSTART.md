# Frontend Quick Start Guide

## Overview

This guide provides quick instructions for working with the enhanced multithreaded WASM frontend.

## Local Development

### Prerequisites
- Modern browser (Chrome 87+, Firefox 85+, Safari 14.1+)
- Trunk installed: `cargo install trunk`
- HTTPS server for threading support (or use Trunk's built-in server)

### Build and Run

```bash
# Development build with hot reload
trunk serve

# Release build
trunk build --release

# Output in dist/ directory
# Open http://localhost:8080 (Trunk automatically serves with HTTPS)
```

### Enable Threading Support

Trunk automatically adds COOP/COEP headers when serving. For production deployment:

1. Verify HTTPS is enabled
2. Ensure `static/_headers` is deployed
3. Check `crossOriginIsolated` in browser console

```javascript
// In browser console:
console.log(crossOriginIsolated); // Should be true
console.log(typeof SharedArrayBuffer); // Should be "function"
```

## File Structure

```
rusty-audio/
├── index.html                      # Main HTML with loading UI
├── static/
│   ├── rusty-audio-init.js         # Main initialization (632 lines)
│   ├── wasm-worker-init.js         # Worker pool manager (351 lines)
│   ├── service-worker.js           # PWA caching (236 lines)
│   ├── test-frontend.html          # Test suite (504 lines)
│   ├── _headers                    # CDN headers config
│   └── manifest.webmanifest        # PWA manifest
├── FRONTEND_ARCHITECTURE.md        # Complete architecture docs
├── FRONTEND_ENHANCEMENT_SUMMARY.md # Implementation summary
└── FRONTEND_QUICKSTART.md          # This file
```

## Key Features

### 1. Progressive Loading

**What you get:**
- Real-time download progress (0-100%)
- Status messages for each initialization phase
- Feature detection display
- Worker thread visualization
- Smooth animations and transitions

**Customize:**
Edit `index.html` styles to change colors, fonts, or layout.

### 2. Performance Monitor

**Toggle:** Press `Ctrl+Shift+P`

**Metrics:**
- FPS (frames per second)
- Frame Time (ms per frame)
- Memory (MB used)
- Threads (active/total workers)
- Audio Latency (reported by Rust backend)

**Customize:**
```javascript
// Change update interval:
// Edit startPerformanceMonitoring() in rusty-audio-init.js
```

### 3. Worker Pool

**Automatic Management:**
- Scales from 2 to hardware_concurrency workers
- Queues tasks when all workers busy
- Recovers from worker failures
- Reports health via custom events

**Monitor:**
```javascript
window.addEventListener('wasm-worker-health', (event) => {
  console.log(event.detail);
  // {totalWorkers, availableWorkers, busyWorkers, pendingTasks, totalTasks}
});
```

### 4. Service Worker

**Cache Strategy:**
- HTML: Network-first (ensures updates)
- WASM/JS: Cache-first (fast loading)
- Static assets: Cache-first (long-lived)

**Management:**
```javascript
// Clear cache:
const channel = new MessageChannel();
navigator.serviceWorker.controller.postMessage(
  { type: 'CLEAR_CACHE' },
  [channel.port2]
);

// Get cache size:
navigator.serviceWorker.controller.postMessage(
  { type: 'GET_CACHE_SIZE' },
  [channel.port2]
);
```

## Testing

### Run Test Suite

```bash
# Open in browser:
http://localhost:8080/static/test-frontend.html
```

**Tests include:**
- Feature detection (10 tests)
- Worker pool management
- Service Worker functionality
- Performance benchmarks
- WASM loading capabilities

### Manual Testing Checklist

- [ ] Load page and verify progress bar shows 0-100%
- [ ] Check feature detection grid displays correctly
- [ ] Verify thread indicators show correct count
- [ ] Toggle performance monitor with Ctrl+Shift+P
- [ ] Test offline mode (DevTools → Network → Offline)
- [ ] Verify worker recovery (check console for recovery messages)
- [ ] Test on mobile (responsive design)
- [ ] Check different browsers (Chrome, Firefox, Safari)

## Integration with Rust

### Report Audio Latency

Add to your Rust audio backend:

```rust
#[wasm_bindgen]
pub fn report_audio_latency(latency_ms: f64) {
    use wasm_bindgen::prelude::*;
    use web_sys::window;

    if let Some(win) = window() {
        if let Ok(rusty_audio) = js_sys::Reflect::get(&win, &"rustyAudio".into()) {
            if let Ok(set_latency) = js_sys::Reflect::get(&rusty_audio, &"setAudioLatency".into()) {
                if let Ok(func) = set_latency.dyn_into::<js_sys::Function>() {
                    let _ = func.call1(&JsValue::NULL, &JsValue::from_f64(latency_ms));
                }
            }
        }
    }
}
```

Call from your audio callback:
```rust
// In your audio processing loop:
let latency = calculate_latency(); // Your latency calculation
report_audio_latency(latency);
```

### Update Thread Indicators

If you manage threads manually:

```rust
#[wasm_bindgen]
pub fn set_worker_active(worker_id: usize, active: bool) {
    if let Some(win) = web_sys::window() {
        if let Ok(rusty_audio) = js_sys::Reflect::get(&win, &"rustyAudio".into()) {
            if let Ok(update_fn) = js_sys::Reflect::get(&rusty_audio, &"updateThreadIndicator".into()) {
                if let Ok(func) = update_fn.dyn_into::<js_sys::Function>() {
                    let args = js_sys::Array::new();
                    args.push(&JsValue::from_f64(worker_id as f64));
                    args.push(&JsValue::from_bool(active));
                    let _ = func.apply(&JsValue::NULL, &args);
                }
            }
        }
    }
}
```

### Access Worker Pool from Rust

The worker pool is managed by JavaScript. For Rust-side worker management, use `wasm-bindgen-rayon` or similar.

## Customization

### Change Color Scheme

Edit `index.html` CSS:
```css
/* Loading screen gradient */
background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);

/* Progress bar gradient */
background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);

/* Change to your brand colors */
```

### Modify Loading Messages

Edit `rusty-audio-init.js`:
```javascript
// Search for updateMessage() calls:
updateMessage('Your custom message here...');
```

### Adjust Worker Pool Size

Edit `rusty-audio-init.js`:
```javascript
// In initializeWorkerPool():
state.workerPool = new WasmWorkerPool({
  maxWorkers: 8,        // Increase max workers
  minWorkers: 4         // Increase min workers
});
```

### Change Performance Monitor Position

Edit `index.html` CSS:
```css
#perf-monitor {
  /* Change position: */
  top: auto;
  bottom: 10px;
  left: 10px;
  right: auto;
}
```

## Troubleshooting

### Progress Bar Stuck at 0%

**Cause:** WASM file not found or fetch hook failed

**Fix:**
```bash
# Verify WASM file exists:
ls dist/rusty-audio_bg.wasm

# Check browser console for fetch errors
# Rebuild with Trunk:
trunk build --release
```

### "SharedArrayBuffer is not defined"

**Cause:** Missing COOP/COEP headers

**Fix:**
```javascript
// Check in browser console:
console.log(crossOriginIsolated); // Should be true

// If false, verify headers in Network tab (DevTools)
// Look for:
// Cross-Origin-Opener-Policy: same-origin
// Cross-Origin-Embedder-Policy: require-corp
```

### Workers Not Initializing

**Cause:** Missing worker script or WASM module

**Fix:**
```bash
# Verify worker script exists:
ls dist/*.worker.js

# Check WASM was built with threads:
wasm-objdump -h dist/rusty-audio_bg.wasm | grep -i thread

# Rebuild with threading enabled
```

### Performance Monitor Not Updating

**Cause:** requestAnimationFrame not called or page in background

**Fix:**
```javascript
// Check if visible:
console.log(document.visibilityState); // Should be "visible"

// Manually trigger update:
document.getElementById('perf-monitor').classList.add('visible');
```

### Service Worker Not Caching

**Cause:** HTTPS required, or cache quota exceeded

**Fix:**
```javascript
// Check Service Worker status:
navigator.serviceWorker.ready.then(reg => {
  console.log('Service Worker ready:', reg);
});

// Clear all caches:
caches.keys().then(keys => {
  return Promise.all(keys.map(key => caches.delete(key)));
});
```

## Performance Tips

### Optimize WASM Size

```toml
# In Trunk.toml:
[build]
target = "index.html"
release = true

[[build.rust]]
wasm_opt = "z"  # Maximum size optimization
```

### Reduce Initial Load Time

```html
<!-- Add resource hints in index.html: -->
<link rel="preconnect" href="https://your-cdn.com">
<link rel="dns-prefetch" href="https://your-cdn.com">
```

### Improve Cache Hit Rate

```javascript
// In service-worker.js, add more assets:
const CORE_ASSETS = [
  "/",
  "/index.html",
  "/rusty-audio.js",
  "/rusty-audio_bg.wasm",
  // Add frequently used assets:
  "/your-audio-file.mp3",
  "/your-image.png"
];
```

### Reduce Worker Overhead

```javascript
// In rusty-audio-init.js:
const maxWorkers = Math.min(navigator.hardwareConcurrency || 4, 4);
// Limit to 4 workers max, even on high-core-count systems
```

## Keyboard Shortcuts

- **Ctrl+Shift+P**: Toggle performance monitor
- **Ctrl+Shift+R**: Force reload (bypass cache)
- **F12**: Open DevTools (browser default)

## Browser DevTools Tips

### Monitor Performance

```
Chrome DevTools:
1. Press F12
2. Performance tab → Record
3. Interact with app
4. Stop recording
5. Analyze frame times, memory, etc.
```

### Inspect Service Worker

```
Chrome DevTools:
1. Application tab
2. Service Workers section
3. View registration, cache, update status
4. Simulate offline mode
```

### Check WASM Threads

```
Chrome DevTools:
1. Sources tab
2. Look for worker files (*.worker.js)
3. Set breakpoints in worker code
4. Inspect SharedArrayBuffer
```

### Monitor Network

```
Chrome DevTools:
1. Network tab
2. Filter by "WS, Wasm"
3. Check WASM download size and time
4. Verify COOP/COEP headers in Response Headers
```

## Further Reading

- **Complete Architecture**: `FRONTEND_ARCHITECTURE.md`
- **Implementation Details**: `FRONTEND_ENHANCEMENT_SUMMARY.md`
- **WASM Threading**: [WebAssembly Threads Proposal](https://github.com/WebAssembly/threads)
- **Service Workers**: [MDN Service Worker API](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API)
- **wasm-bindgen**: [The wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)

## Support

For issues or questions:
1. Check browser console for errors
2. Run test suite: `/static/test-frontend.html`
3. Review architecture docs
4. Check compatibility matrix in `FRONTEND_ARCHITECTURE.md`

## Quick Commands Reference

```bash
# Development
trunk serve                          # Start dev server with hot reload
trunk serve --open                   # Start and open browser
trunk serve --port 3000              # Custom port

# Production
trunk build --release                # Optimized build
trunk build --release --public-url / # For root deployment

# Testing
open http://localhost:8080/static/test-frontend.html

# Deployment
# See scripts/deploy-pwa-cdn.sh for CDN deployment

# Debugging
RUST_LOG=debug trunk serve           # Enable debug logging
```

---

**Quick Start Complete!** Your frontend is now ready for production with multithreading, performance monitoring, and comprehensive error handling.
