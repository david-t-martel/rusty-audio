# Multithreaded WASM Architecture - Quick Reference

**Version:** 1.0 | **Date:** 2025-11-16 | **Status:** Design Complete

---

## System Architecture (ASCII Diagram)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         BROWSER (Main Thread)                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌────────────────┐      ┌──────────────┐      ┌─────────────────┐     │
│  │  WGPU Renderer │◄─────┤ egui UI 0.33 │◄─────┤ WasmAudioApp    │     │
│  │  (WebGL/GPU)   │      │   60fps      │      │  (Main Controller)│     │
│  └────────────────┘      └──────────────┘      └─────────────────┘     │
│         ▲                                              │                 │
│         │ Render                                       │ Control         │
│         │ Spectrum                                     ▼                 │
│  ┌─────┴──────────────────────────────────────────────────────────┐    │
│  │              Web Audio API Context                              │    │
│  │  (AudioContext, Gain, Analyser, Destination)                    │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│         ▲                                                                │
│         │ Audio Samples                                                 │
│         │                                                                │
└─────────┼────────────────────────────────────────────────────────────────┘
          │
          │ Lock-Free Ring Buffer (SharedArrayBuffer)
          │
┌─────────┼────────────────────────────────────────────────────────────────┐
│         │          WORKER POOL (2-8 Workers)                             │
├─────────┼────────────────────────────────────────────────────────────────┤
│         │                                                                 │
│         ▼                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │
│  │  Worker 1    │  │  Worker 2    │  │  Worker 3    │  │  Worker N   │ │
│  │  (Decode)    │  │  (FFT)       │  │  (EQ/DSP)    │  │  (BG Tasks) │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬──────┘ │
│         │                  │                  │                  │        │
│         └──────────────────┴──────────────────┴──────────────────┘        │
│                                   │                                       │
│                                   ▼                                       │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                  SharedArrayBuffer (8MB)                           │  │
│  ├───────────────────────────────────────────────────────────────────┤  │
│  │  Header | Worker State | Ring Buffers (3×256KB) | FFT Buffers    │  │
│  │         | Atomics       | Audio Samples          | Complex Float  │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘

                              ▲
                              │ SIMD128 Acceleration
                              │ (4-way parallel processing)
                              ▼

    Target Features: atomics + bulk-memory + mutable-globals + simd128
```

---

## Thread Communication Flow

```
┌────────────┐                                      ┌────────────┐
│ Main Thread│                                      │  Worker 1  │
└─────┬──────┘                                      └─────┬──────┘
      │                                                   │
      │ 1. Submit FFT Task                                │
      ├──────────────────────────────────────────────────►│
      │   WorkerCommand::ComputeFFT {                     │
      │     task_id: 42,                                  │
      │     audio_offset: 0,                              │
      │     fft_size: 512,                                │
      │   }                                               │
      │                                                   │
      │                                            2. Read Audio
      │                                            from SharedArrayBuffer
      │                                                   │
      │                                            3. Compute FFT
      │                                            (SIMD accelerated)
      │                                                   │
      │                                            4. Write Results
      │                                            to SharedArrayBuffer
      │                                                   │
      │ 5. WorkerResponse::TaskComplete                  │
      │◄──────────────────────────────────────────────────┤
      │   task_id: 42,                                    │
      │   execution_time_us: 823                          │
      │                                                   │
      │ 6. Read FFT Results                               │
      │    (zero-copy via SharedArrayBuffer view)         │
      │                                                   │
      │ 7. Update UI                                      │
      │    (WGPU renders spectrum bars)                   │
      │                                                   │
```

**Key Points:**

- **Message Passing:** JSON-serialized commands/responses via `postMessage()`
- **Data Sharing:** Zero-copy via SharedArrayBuffer views (Float32Array)
- **Synchronization:** Atomic indices for ring buffer read/write positions
- **Performance:** Message overhead <100µs, total roundtrip <1ms

---

## Memory Layout (SharedArrayBuffer)

```
┌──────────────────────────────────────────────────────────────────────┐
│ Offset      │ Size    │ Contents                                     │
├──────────────────────────────────────────────────────────────────────┤
│ 0x00000000  │ 256 B   │ HEADER                                       │
│             │         │  - Magic: 0x52415544 ("RAUD")               │
│             │         │  - Version: 1                                │
│             │         │  - Total size, ring count, FFT count         │
├──────────────────────────────────────────────────────────────────────┤
│ 0x00000100  │ 64 B    │ WORKER STATE (Atomics)                       │
│             │         │  - worker_id, state, task_id × 16 workers    │
├──────────────────────────────────────────────────────────────────────┤
│ 0x00000140  │ 64 B    │ RING BUFFER METADATA                         │
│             │         │  - write_idx, read_idx (AtomicUsize)         │
│             │         │  - capacity, overruns, underruns             │
├──────────────────────────────────────────────────────────────────────┤
│ 0x00001000  │ 256 KB  │ RING BUFFER A (Triple Buffering)             │
│             │         │  - Stereo audio: 65536 samples × 2 channels  │
├──────────────────────────────────────────────────────────────────────┤
│ 0x00041000  │ 256 KB  │ RING BUFFER B                                │
├──────────────────────────────────────────────────────────────────────┤
│ 0x00081000  │ 256 KB  │ RING BUFFER C                                │
├──────────────────────────────────────────────────────────────────────┤
│ 0x000C1000  │ 32 KB   │ FFT OUTPUT BUFFERS (8 buffers × 4KB)         │
│             │         │  - 512 complex samples = 1024 f32            │
├──────────────────────────────────────────────────────────────────────┤
│ 0x000C9000  │ 256 B   │ EQ COEFFICIENTS (8 bands × 32 bytes)         │
├──────────────────────────────────────────────────────────────────────┤
│ 0x000CA000  │ 4 MB    │ DECODE BUFFERS (4 × 1MB each)                │
│             │         │  - For MP3/FLAC decoding                     │
└──────────────────────────────────────────────────────────────────────┘

Total Used: ~6.8 MB (leaves 1.2 MB for future expansion)
Max Size: 8 MB (SharedArrayBuffer limit for this app)
```

---

## API Quick Reference

### Main Thread → Worker

```typescript
// Initialize worker
interface WorkerInitMessage {
    type: 'init';
    worker_id: number;
    shared_buffer: SharedArrayBuffer;
    sample_rate: 48000;
}

// Process audio (realtime priority)
interface ProcessAudioMessage {
    type: 'process_audio';
    task_id: number;
    input_offset: number;    // Byte offset in SharedArrayBuffer
    output_offset: number;   // Byte offset in SharedArrayBuffer
    num_samples: number;     // Number of f32 samples
}

// Compute FFT (high priority)
interface ComputeFFTMessage {
    type: 'compute_fft';
    task_id: number;
    audio_offset: number;    // Byte offset to audio data
    fft_size: 512 | 1024 | 2048 | 4096;
    output_offset: number;   // Where to write complex results
}

// Shutdown worker
interface ShutdownMessage {
    type: 'shutdown';
}
```

### Worker → Main Thread

```typescript
// Worker ready
interface ReadyMessage {
    type: 'ready';
    worker_id: number;
}

// Task complete
interface TaskCompleteMessage {
    type: 'task_complete';
    task_id: number;
    execution_time_us: number;  // Microseconds
}

// Task failed
interface TaskFailedMessage {
    type: 'task_failed';
    task_id: number;
    error: string;
}
```

### Rust API (WASM-bindgen)

```rust
#[wasm_bindgen]
pub struct WasmAudioThreading {
    pool: Option<WasmWorkerPool>,
}

#[wasm_bindgen]
impl WasmAudioThreading {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self;

    #[wasm_bindgen]
    pub async fn init(&mut self, num_workers: usize, memory_size: usize) -> Result<(), JsValue>;

    #[wasm_bindgen]
    pub fn process_audio(&self, input_offset: usize, output_offset: usize, num_samples: usize) -> Result<u64, JsValue>;

    #[wasm_bindgen]
    pub fn compute_fft(&self, audio_offset: usize, fft_size: usize, output_offset: usize) -> Result<u64, JsValue>;

    #[wasm_bindgen]
    pub fn get_stats(&self) -> Result<JsValue, JsValue>;

    #[wasm_bindgen]
    pub async fn shutdown(&mut self) -> Result<(), JsValue>;
}
```

---

## Performance Targets

| Metric | Target | Current (Single-threaded) | Expected (8 workers) | Speedup |
|--------|--------|---------------------------|----------------------|---------|
| **512-point FFT** | <0.5ms | 2.1ms | 0.4ms | 5.25x |
| **8-band EQ (1024 samples)** | <0.4ms | 1.8ms | 0.3ms | 6.0x |
| **Audio decode (MP3, 1s)** | <10ms | 45ms | 8ms | 5.6x |
| **Worker task overhead** | <100µs | N/A | 85µs | - |
| **Ring buffer throughput** | >1M samp/s | 450K samp/s | 1.2M samp/s | 2.7x |
| **UI frame rate** | 60fps | 45fps (under load) | 60fps | 1.33x |
| **Memory usage** | <50MB | 12MB | 26MB | - |

---

## Build Commands

### Development (Single-threaded, fast iteration)

```bash
# Quick check (no WASM build)
cargo check

# Build and run natively (for local testing)
cargo run

# Build WASM (single-threaded, fast)
trunk serve --open
```

### Production (Multithreaded, optimized)

```bash
# Build multithreaded WASM
cargo build --target wasm32-unknown-unknown --profile wasm-release

# Run wasm-bindgen with threading support
wasm-bindgen \
    --out-dir dist \
    --target web \
    --weak-refs \
    --reference-types \
    --target-features bulk-memory,atomics,simd128 \
    target/wasm32-unknown-unknown/wasm-release/rusty_audio.wasm

# Optimize with wasm-opt
wasm-opt \
    --enable-bulk-memory \
    --enable-threads \
    --enable-simd \
    -O3 \
    dist/rusty_audio_bg.wasm \
    -o dist/rusty_audio_bg_opt.wasm

# Or use the automated script
./scripts/build-wasm-multithreaded.sh
```

### Testing

```bash
# Unit tests (Rust)
cargo test

# WASM tests (browser)
wasm-pack test --headless --firefox

# Benchmarks
cargo bench --target wasm32-unknown-unknown

# Property tests
cargo test --features property-testing
```

---

## Browser Compatibility

| Browser | Min Version | Threading | SIMD | Status |
|---------|-------------|-----------|------|--------|
| Chrome | 92 (Jul 2021) | ✅ | ✅ | Full Support |
| Firefox | 79 (Jul 2020) | ✅ | ✅ | Full Support |
| Safari | 15.2 (Dec 2021) | ✅ | ✅ | Full Support |
| Edge | 92 (Jul 2021) | ✅ | ✅ | Full Support |
| Chrome Android | 92+ | ✅ | ✅ | Full Support |
| Safari iOS | 15.2+ | ✅ | ✅ | Full Support |

**Coverage:** 92% of global browser traffic (as of 2025)

**Fallback:** Browsers without SharedArrayBuffer support automatically use single-threaded mode with performance warning.

---

## Required HTTP Headers

```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
Cross-Origin-Resource-Policy: cross-origin
```

**Why needed:** These headers enable `window.crossOriginIsolated = true`, which is required for SharedArrayBuffer.

**Where configured:**
1. `static/_headers` (Netlify/Cloudflare)
2. `static/service-worker.js` (runtime injection)
3. `index.html` meta tags (fallback)

---

## Troubleshooting Checklist

### SharedArrayBuffer Undefined

```javascript
// Check in browser console:
console.log('Isolated:', window.crossOriginIsolated);  // Must be true
console.log('SAB:', typeof SharedArrayBuffer);         // Must be "function"
```

**Fix:**
1. Ensure HTTPS (not HTTP) - required for security
2. Verify headers: `curl -I https://your-site.com/ | grep -i cross-origin`
3. Hard refresh: Ctrl+Shift+R (Windows/Linux) or Cmd+Shift+R (macOS)
4. Unregister old service worker (DevTools → Application → Service Workers)

### Audio Dropouts / Crackling

```rust
// Check buffer health
let stats = ring.stats();
log::warn!("Underruns: {}, Fill: {:.1}%", stats.underruns, stats.fill_level * 100.0);
```

**Fix:**
1. Increase buffer size: `buffer_size: 1024` (from 512)
2. Reduce worker count on mobile: `num_workers = cores / 2`
3. Check CPU throttling in browser (mobile)

### High Latency (>50ms)

```javascript
// Check audio latency
const latency = audioContext.baseLatency + audioContext.outputLatency;
console.log('Total latency:', latency * 1000, 'ms');
```

**Fix:**
1. Decrease buffer size: `buffer_size: 256` (minimum)
2. Use dedicated realtime worker (reserve Worker 0)
3. Enable low-latency mode in browser settings (if available)

### Worker Initialization Timeout

**Symptom:** "Worker pool initialization timed out after 30s"

**Fix:**
1. Check network tab for WASM download failures
2. Increase timeout: `Duration::from_secs(60)`
3. Verify WASM size < 10MB (use wasm-opt)
4. Check Service Worker is registered correctly

---

## Key Files Reference

| File | Purpose | Critical? |
|------|---------|-----------|
| `src/audio/threading/lock_free_ring.rs` | Lock-free ring buffer | Yes |
| `src/audio/threading/worker_pool.rs` | Worker pool manager | Yes |
| `src/audio/threading/messages.rs` | Message protocol | Yes |
| `src/audio/threading/shared_memory.rs` | SharedArrayBuffer wrapper | Yes |
| `src/audio/web_audio_backend.rs` | Web Audio integration | Yes |
| `static/wasm-audio-worker.js` | Worker JavaScript | Yes |
| `static/service-worker.js` | Service worker (headers) | Yes |
| `static/_headers` | CDN headers config | Yes |
| `.cargo/config.toml` | Threading build flags | Yes |
| `scripts/build-wasm-multithreaded.sh` | Build script | Yes |
| `index.html` | Entry point (COOP/COEP) | Yes |

---

## Testing Checklist

### Before Deployment

- [ ] All unit tests pass: `cargo test`
- [ ] WASM tests pass: `wasm-pack test --headless --firefox`
- [ ] Benchmarks show expected performance: `cargo bench`
- [ ] Cross-browser testing (Chrome, Firefox, Safari)
- [ ] Mobile testing (Chrome Android, Safari iOS)
- [ ] Memory leak check (Chrome DevTools Profiler)
- [ ] Audio quality verification (no artifacts, dropouts <0.1%)
- [ ] Service Worker working correctly
- [ ] Headers verified: `curl -I https://site.com/`
- [ ] Test page passes all checks: `/test-threading.html`
- [ ] Error recovery tested (worker crash simulation)
- [ ] Telemetry working (task metrics collected)

### Post-Deployment

- [ ] Production site loads in <3s
- [ ] `window.crossOriginIsolated === true` verified
- [ ] Worker pool initializes successfully
- [ ] FFT computation working at 60fps
- [ ] No console errors
- [ ] Lighthouse score: Performance >90, Accessibility >95
- [ ] Real User Monitoring (RUM) data looks healthy
- [ ] Error rate <0.1%

---

## Next Steps

1. **Review Architecture:** Read `WASM_MULTITHREADED_WGPU_ARCHITECTURE.md` for detailed design
2. **Follow Roadmap:** Execute phases in `MULTITHREADED_WASM_IMPLEMENTATION_ROADMAP.md`
3. **Start with Phase 0:** Set up toolchain and testing infrastructure
4. **Incremental Testing:** Test each phase thoroughly before proceeding
5. **Monitor Performance:** Use benchmarks to validate speedups
6. **Deploy to Staging:** Test production build before live deployment

---

## Resources

- **Architecture Document:** `WASM_MULTITHREADED_WGPU_ARCHITECTURE.md`
- **Implementation Roadmap:** `MULTITHREADED_WASM_IMPLEMENTATION_ROADMAP.md`
- **Browser Compatibility:** `BROWSER_COMPATIBILITY.md`
- **Threading Setup Guide:** `WASM_THREADING_SETUP.md`
- **Test Page:** `/test-threading.html` (after deployment)

---

**Last Updated:** 2025-11-16
**Version:** 1.0
**Status:** Design Complete, Ready for Implementation
