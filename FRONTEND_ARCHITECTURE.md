# Frontend Architecture - Multithreaded WASM

## Overview

The Rusty Audio frontend provides a production-ready multithreaded WASM experience with progressive loading, comprehensive error handling, and real-time performance monitoring.

## Architecture Components

### 1. Progressive Loading UI (`index.html`)

**Features:**
- Animated gradient loading screen
- Real-time download progress bar (tracks 5.9MB WASM binary)
- Feature detection display
- Worker thread pool visualization
- Graceful error handling with detailed diagnostics

**Visual Elements:**
- **Loading Header**: Gradient-animated "Rusty Audio" branding
- **Progress Bar**: Real-time download progress with MB/total display
- **Feature Support Grid**: Visual capability detection matrix
- **Thread Status**: Live worker pool visualization with per-thread indicators
- **Performance Monitor**: HUD overlay with FPS, frame time, memory, thread utilization

**Responsive Design:**
- Mobile-optimized layouts (600px breakpoint)
- Touch-friendly controls
- Adaptive feature display (grid → single column on mobile)

### 2. Initialization System (`rusty-audio-init.js`)

**Initialization Sequence:**

```
1. Feature Detection (5% progress)
   └─ WebAssembly, Threads, SharedArrayBuffer, Atomics
   └─ Cross-Origin Isolation, Service Worker
   └─ Web Audio API, WebGPU, OffscreenCanvas
   └─ Hardware Concurrency detection

2. Service Worker Registration (15% progress)
   └─ Install with COOP/COEP headers
   └─ Update detection and notification

3. Progress Tracking Setup (25% progress)
   └─ Hook fetch() to monitor WASM download
   └─ Real-time progress bar updates

4. Worker Pool Initialization (20% progress)
   └─ Create WasmWorkerPool instance
   └─ Determine max workers (navigator.hardwareConcurrency)
   └─ Initialize min workers (2 or max, whichever is less)
   └─ Display thread visualization grid

5. WASM Module Loading (75% progress)
   └─ Wait for wasm-bindgen availability
   └─ Compile WASM module
   └─ Initialize shared memory

6. Worker Thread Setup (95% progress)
   └─ Initialize pool with WASM module and memory
   └─ Setup SharedArrayBuffer communication
   └─ Start worker health monitoring

7. Finalization (100% progress)
   └─ Setup error handlers (WGPU context loss, etc.)
   └─ Enable keyboard shortcuts
   └─ Start performance monitoring
   └─ Hide loading overlay
```

**State Management:**
```javascript
window.rustyAudio = {
  state: {
    features: {},           // Browser capability detection
    workerPool: null,       // WasmWorkerPool instance
    wasmInitialized: false, // Initialization status
    perfStats: {}           // Performance metrics
  },
  elements: {},             // DOM element references
  log(),                    // Timestamped logging
  error(),                  // Error logging
  updateThreadIndicator(),  // Thread visualization update
  setAudioLatency()         // Audio latency reporting
}
```

### 3. Worker Pool Management (`wasm-worker-init.js`)

**WasmWorkerPool Class:**
- **Dynamic Scaling**: Grows from min to max workers based on load
- **Task Queue**: Pending tasks automatically assigned to available workers
- **Error Recovery**: Automatic worker restart on failure
- **Health Monitoring**: Periodic stats reporting via custom events

**Worker Lifecycle:**
```
Create Worker → Initialize with WASM module → Await init-complete
    ↓
  Ready → Receive task → Execute → Post result → Mark available
    ↓
  Error → Terminate → Recreate (if below min workers)
```

**Statistics Tracking:**
- Total workers active
- Available vs. busy workers
- Pending task queue length
- Total tasks completed per worker
- Uptime per worker

**WorkerHealthMonitor Class:**
- Periodic health checks (5-second intervals)
- Dispatches `wasm-worker-health` custom events
- Warns when all workers are busy with pending tasks
- Logs comprehensive pool statistics

### 4. Service Worker (`service-worker.js`)

**Caching Strategy:**
- **HTML**: Network-first with cache fallback
- **WASM/JS**: Cache-first with network fallback
- **Static Assets**: Cache-first, long-lived (immutable)

**Cross-Origin Isolation:**
All responses automatically injected with:
```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
Cross-Origin-Resource-Policy: cross-origin
```

**Cache Management:**
- Version-based cache naming (`rusty-audio-v3`)
- Automatic old cache cleanup on activation
- Manual cache clearing via message API

**Performance Monitoring:**
- Request count tracking
- Cache hit rate calculation (logged every 60 seconds)
- Per-request timing available in DevTools

**Message API:**
```javascript
// Skip waiting and activate immediately
navigator.serviceWorker.controller.postMessage({ type: 'SKIP_WAITING' });

// Clear all caches
const channel = new MessageChannel();
navigator.serviceWorker.controller.postMessage(
  { type: 'CLEAR_CACHE' },
  [channel.port2]
);

// Get cache statistics
navigator.serviceWorker.controller.postMessage(
  { type: 'GET_CACHE_SIZE' },
  [channel.port2]
);
```

### 5. Performance Monitoring System

**Real-Time Metrics:**
- **FPS**: Calculated every second, color-coded (green >55, yellow 30-55, red <30)
- **Frame Time**: Per-frame delta time in ms (green <16.7, yellow <33, red >33)
- **Memory**: JS heap size in MB (yellow >500MB)
- **Threads**: Active/total worker count with live visualization
- **Audio Latency**: Reported by Rust backend via `setAudioLatency()`

**Thread Visualization:**
- Grid of colored squares (one per worker)
- **Active**: Pulsing gradient animation
- **Idle**: Low-opacity gray
- Updates in sync with worker pool state

**Keyboard Shortcuts:**
- `Ctrl+Shift+P`: Toggle performance monitor overlay
- `Ctrl+Shift+R`: Force reload (bypass cache)

### 6. Error Handling & Fallbacks

**Critical Feature Checks:**
```javascript
if (!WebAssembly) {
  → Show error overlay with browser upgrade suggestion
  → Halt initialization
}

if (!SharedArrayBuffer) {
  → Log warning about single-threaded mode
  → Continue initialization without worker pool
  → Display warning in feature support grid
}

if (!crossOriginIsolated) {
  → Log warning about missing COOP/COEP headers
  → Explain HTTPS + header requirements
  → Continue in degraded mode
}
```

**WGPU Error Handling:**
- `webglcontextlost` event listener → prevent default, log error
- `webglcontextrestored` event listener → log restoration
- Global WASM error handlers for runtime exceptions

**Graceful Degradation:**
- Service Worker failure → Continue without offline support
- Worker pool failure → Single-threaded WASM execution
- WebGPU unavailable → Fall back to WebGL (if supported by backend)

**Error Display:**
- Full-screen overlay with gradient background
- Detailed error message
- Browser capability checklist (✓ supported, ✗ missing, ⚠ degraded)
- Suggested remediation steps

## File Structure

```
├── index.html                      # Main HTML with embedded styles
│   ├── Loading overlay UI
│   ├── Progress bar
│   ├── Feature detection display
│   ├── Thread pool visualization
│   ├── Performance monitor HUD
│   └── Error overlay
│
├── static/
│   ├── rusty-audio-init.js         # Main initialization orchestrator
│   │   ├── Feature detection
│   │   ├── Service worker registration
│   │   ├── WASM loading with progress
│   │   ├── Worker pool setup
│   │   ├── Performance monitoring
│   │   └── Error handling
│   │
│   ├── wasm-worker-init.js         # Worker pool management
│   │   ├── WasmWorkerPool class
│   │   ├── Worker lifecycle management
│   │   ├── Task queue and distribution
│   │   ├── Error recovery
│   │   └── WorkerHealthMonitor
│   │
│   ├── service-worker.js           # Offline-first PWA caching
│   │   ├── Cache management
│   │   ├── COOP/COEP header injection
│   │   ├── Network strategies
│   │   └── Performance tracking
│   │
│   └── _headers                    # Cloudflare/CDN header config
│       ├── Cross-origin isolation
│       ├── Content security policy
│       └── Cache control directives
```

## Browser Compatibility

**Minimum Requirements:**
- Chrome 87+ / Edge 87+ (WebAssembly, SharedArrayBuffer, Service Worker)
- Firefox 85+ (SharedArrayBuffer re-enabled with COOP/COEP)
- Safari 14.1+ (WebAssembly threads, SharedArrayBuffer)

**Optional Features:**
- **WebGPU**: Chrome 113+, Edge 113+ (improves rendering performance)
- **OffscreenCanvas**: Chrome 69+, Firefox 105+ (enables canvas workers)

**Cross-Origin Isolation Requirements:**
- **HTTPS** mandatory for SharedArrayBuffer
- **COOP**: `same-origin` header required
- **COEP**: `require-corp` header required
- **CORP**: `cross-origin` on WASM/JS resources

## Performance Targets

**Loading Performance:**
- Initial paint: <1s (service worker cache hit)
- WASM download: ~2-3s on 3G (5.9MB optimized)
- Time to interactive: <5s on fast 3G

**Runtime Performance:**
- 60 FPS sustained (16.7ms frame budget)
- <500MB memory footprint
- <25ms audio processing latency
- <10ms worker task dispatch

**Network Performance:**
- 95%+ cache hit rate after first load
- Offline functionality via service worker
- Automatic updates without user interruption

## Testing Checklist

### Feature Detection
- [ ] WebAssembly availability
- [ ] WebAssembly threads validation
- [ ] SharedArrayBuffer presence
- [ ] Atomics support
- [ ] Cross-origin isolation status
- [ ] Service Worker registration
- [ ] Web Audio API
- [ ] WebGPU (optional)
- [ ] Hardware concurrency detection

### Loading Experience
- [ ] Progress bar tracks download (0-100%)
- [ ] Status messages update appropriately
- [ ] Feature grid displays all capabilities
- [ ] Thread visualization shows correct worker count
- [ ] Loading overlay fades out smoothly

### Worker Pool
- [ ] Workers initialize correctly
- [ ] Task queue functions properly
- [ ] Workers recover from errors
- [ ] Health monitor dispatches events
- [ ] Thread indicators update in real-time

### Performance Monitor
- [ ] FPS counter updates every second
- [ ] Frame time displays correctly
- [ ] Memory tracking works (Chrome only)
- [ ] Thread count reflects pool state
- [ ] Audio latency updates when set
- [ ] Color coding applies correctly
- [ ] Toggle shortcut (Ctrl+Shift+P) works

### Error Handling
- [ ] Missing WebAssembly shows error
- [ ] Missing SharedArrayBuffer shows warning
- [ ] WASM load failure displays error
- [ ] Worker failure triggers recovery
- [ ] WGPU context loss handled
- [ ] Service Worker failure non-blocking

### Offline/PWA
- [ ] Service worker installs
- [ ] Cache pre-populates with core assets
- [ ] Offline mode works after first load
- [ ] Updates detected and applied
- [ ] Cache clearing works via API

## Deployment Configuration

### CDN Headers (Cloudflare Workers)
See `static/_headers` for complete configuration. Key headers:
- COOP/COEP/CORP for threading
- CSP allowing WASM and workers
- Cache-Control for immutable assets
- Content-Type for WASM/JS

### Build Configuration
Trunk automatically:
- Optimizes WASM with `wasm-opt -Oz`
- Generates `rusty-audio.js` and `rusty-audio_bg.wasm`
- Copies `static/` directory to output
- Injects script tags into `index.html`

### Environment Variables
None required - all configuration is static.

## Debugging

### Console Logging
All logs timestamped relative to page load:
```
[0.05s] [Rusty Audio] Starting initialization
[0.12s] [Rusty Audio] Feature detection complete
[1.23s] [WASM Workers] Worker 0 initialized
[2.45s] [Rusty Audio] WASM module initialized successfully
```

### Performance Monitor
Press `Ctrl+Shift+P` to show/hide real-time metrics overlay.

### Service Worker DevTools
Chrome DevTools → Application → Service Workers:
- View cache contents
- Simulate offline
- Force update
- Unregister

### Thread Visualization
Visual indicators show worker status in real-time:
- **Pulsing gradient**: Worker executing task
- **Gray**: Worker idle
- **Count**: Active/Total in performance monitor

## Future Enhancements

### Planned Features
- [ ] Worker thread priority scheduling
- [ ] WASM streaming compilation
- [ ] IndexedDB for audio file caching
- [ ] WebRTC data channels for low-latency audio
- [ ] Worklet-based audio processing (if feasible)
- [ ] Background fetch for large audio files
- [ ] Web Share API for playlists
- [ ] Notifications for long-running tasks

### Performance Optimizations
- [ ] Lazy load non-critical WASM modules
- [ ] Code splitting for UI components
- [ ] Preconnect to CDN in HTML head
- [ ] Resource hints (preload, prefetch)
- [ ] WASM SIMD optimization
- [ ] WebGPU compute shaders for FFT

### Accessibility
- [ ] Screen reader announcements for loading states
- [ ] Keyboard navigation for all controls
- [ ] High contrast theme support
- [ ] Reduced motion preference support
- [ ] Focus management during initialization

## References

- [WebAssembly Threads Proposal](https://github.com/WebAssembly/threads)
- [SharedArrayBuffer Security Requirements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer#security_requirements)
- [Service Worker API](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API)
- [wasm-bindgen Documentation](https://rustwasm.github.io/wasm-bindgen/)
- [egui WASM Threading](https://github.com/emilk/egui/blob/master/crates/eframe/README.md#wasm-threads)
