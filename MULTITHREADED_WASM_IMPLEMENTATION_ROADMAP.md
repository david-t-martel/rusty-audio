# Multithreaded WASM Implementation Roadmap

**Project:** Rusty Audio - Multithreaded WASM Deployment
**Target:** Production-ready multithreaded WASM with WGPU rendering
**Timeline:** 8-10 weeks (phased approach)

---

## Overview

This roadmap breaks down the implementation of multithreaded WASM support into 6 phases, each with clear deliverables and testing criteria. The architecture is designed to be incrementally testable, with each phase building on the previous one.

---

## Phase 0: Prerequisites & Environment Setup (Week 1)

### Goals
- Verify build toolchain supports threading
- Enable required Rust features
- Set up testing infrastructure

### Tasks

#### 0.1 Verify Rust Toolchain
```bash
# Install/update required tools
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.92
cargo install wasm-pack
cargo install wasm-opt  # Part of binaryen toolkit

# Verify versions
rustc --version  # Should be 1.75+
wasm-bindgen --version  # Should be 0.2.92+
```

#### 0.2 Update Cargo Dependencies

**File:** `Cargo.toml`

```toml
[dependencies]
# ... existing dependencies ...

# Threading support for WASM
wasm-bindgen = "0.2.92"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "AudioContext",
    "Worker",
    "MessageEvent",
    "MessagePort",
    "SharedArrayBuffer",
    "Atomics",
    # ... existing features ...
]}

# Optional: Enable threading in rayon for WASM
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen-rayon = { version = "1.2", optional = true }
```

#### 0.3 Configure Threading Features

**File:** `.cargo/config.toml`

Already configured! Verify these flags are present:

```toml
[target.wasm32-unknown-unknown]
rustflags = [
    "-C", "target-feature=+atomics,+bulk-memory,+mutable-globals,+simd128",
    "-C", "link-arg=--shared-memory",
    "-C", "link-arg=--max-memory=4294967296",
]
```

#### 0.4 Set Up WASM Testing

**File:** `tests/wasm_threading_tests.rs`

```rust
#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_shared_array_buffer_available() {
    use wasm_bindgen::JsCast;
    use js_sys::SharedArrayBuffer;

    let window = web_sys::window().expect("No window");
    let shared_array_buffer = SharedArrayBuffer::new(1024);
    assert!(shared_array_buffer.is_ok(), "SharedArrayBuffer not available");
}

#[wasm_bindgen_test]
async fn test_worker_creation() {
    use web_sys::Worker;

    let worker = Worker::new("./test-worker.js");
    assert!(worker.is_ok(), "Worker creation failed");
}
```

**Run tests:**
```bash
wasm-pack test --headless --firefox
```

### Deliverables
- ✅ Toolchain verified and up-to-date
- ✅ Threading flags enabled in `.cargo/config.toml`
- ✅ WASM test infrastructure working
- ✅ Browser test environment configured

### Success Criteria
- `cargo build --target wasm32-unknown-unknown` succeeds with threading flags
- `wasm-pack test --headless --firefox` passes basic tests
- SharedArrayBuffer test passes in Firefox/Chrome with COOP/COEP headers

---

## Phase 1: Lock-Free Ring Buffer Implementation (Week 2)

### Goals
- Implement thread-safe audio ring buffer using atomics
- Verify zero-copy data transfer
- Benchmark single-producer/single-consumer (SPSC) throughput

### Tasks

#### 1.1 Implement Core Ring Buffer

**File:** `src/audio/threading/lock_free_ring.rs`

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::marker::PhantomData;

/// Lock-free SPSC ring buffer for audio samples
///
/// Uses atomic indices for synchronization between producer (worker)
/// and consumer (main thread).
pub struct LockFreeAudioRing {
    buffer: *mut f32,
    capacity: usize,

    write_index: AtomicUsize,
    read_index: AtomicUsize,

    // Metrics
    overruns: AtomicUsize,
    underruns: AtomicUsize,

    _marker: PhantomData<*mut f32>,
}

unsafe impl Send for LockFreeAudioRing {}
unsafe impl Sync for LockFreeAudioRing {}

impl LockFreeAudioRing {
    /// Create from raw pointer to SharedArrayBuffer
    ///
    /// # Safety
    /// - `buffer` must point to valid SharedArrayBuffer memory
    /// - `capacity` must match allocated buffer size
    /// - Buffer must remain valid for lifetime of this struct
    pub unsafe fn from_raw_parts(buffer: *mut f32, capacity: usize) -> Self {
        Self {
            buffer,
            capacity,
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(0),
            overruns: AtomicUsize::new(0),
            underruns: AtomicUsize::new(0),
            _marker: PhantomData,
        }
    }

    /// Write samples to buffer (producer side)
    ///
    /// Returns number of samples written (may be less than input if buffer full)
    pub fn write(&self, samples: &[f32]) -> usize {
        let write_idx = self.write_index.load(Ordering::Acquire);
        let read_idx = self.read_index.load(Ordering::Acquire);

        // Calculate free space
        let used = if write_idx >= read_idx {
            write_idx - read_idx
        } else {
            self.capacity - read_idx + write_idx
        };
        let free = self.capacity - used - 1; // Reserve 1 slot to distinguish full/empty

        let to_write = samples.len().min(free);

        if to_write < samples.len() {
            self.overruns.fetch_add(1, Ordering::Relaxed);
        }

        // Write samples (may wrap around)
        unsafe {
            let end_idx = write_idx + to_write;
            if end_idx <= self.capacity {
                // No wraparound - contiguous copy
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr(),
                    self.buffer.add(write_idx),
                    to_write,
                );
            } else {
                // Wraparound - split copy
                let first_chunk = self.capacity - write_idx;
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr(),
                    self.buffer.add(write_idx),
                    first_chunk,
                );
                std::ptr::copy_nonoverlapping(
                    samples.as_ptr().add(first_chunk),
                    self.buffer,
                    to_write - first_chunk,
                );
            }
        }

        // Publish write (Release ensures all writes above are visible)
        self.write_index.store(
            (write_idx + to_write) % self.capacity,
            Ordering::Release,
        );

        to_write
    }

    /// Read samples from buffer (consumer side)
    ///
    /// Fills output with available samples, pads with silence if underrun
    pub fn read(&self, output: &mut [f32]) -> usize {
        let write_idx = self.write_index.load(Ordering::Acquire);
        let read_idx = self.read_index.load(Ordering::Acquire);

        // Calculate available samples
        let available = if write_idx >= read_idx {
            write_idx - read_idx
        } else {
            self.capacity - read_idx + write_idx
        };

        let to_read = output.len().min(available);

        if to_read < output.len() {
            self.underruns.fetch_add(1, Ordering::Relaxed);
            // Fill remainder with silence
            for i in to_read..output.len() {
                output[i] = 0.0;
            }
        }

        // Read samples
        unsafe {
            let end_idx = read_idx + to_read;
            if end_idx <= self.capacity {
                std::ptr::copy_nonoverlapping(
                    self.buffer.add(read_idx),
                    output.as_mut_ptr(),
                    to_read,
                );
            } else {
                let first_chunk = self.capacity - read_idx;
                std::ptr::copy_nonoverlapping(
                    self.buffer.add(read_idx),
                    output.as_mut_ptr(),
                    first_chunk,
                );
                std::ptr::copy_nonoverlapping(
                    self.buffer,
                    output.as_mut_ptr().add(first_chunk),
                    to_read - first_chunk,
                );
            }
        }

        // Publish read
        self.read_index.store(
            (read_idx + to_read) % self.capacity,
            Ordering::Release,
        );

        to_read
    }

    /// Get current buffer fill level (0.0 = empty, 1.0 = full)
    pub fn fill_level(&self) -> f32 {
        let write_idx = self.write_index.load(Ordering::Relaxed);
        let read_idx = self.read_index.load(Ordering::Relaxed);

        let used = if write_idx >= read_idx {
            write_idx - read_idx
        } else {
            self.capacity - read_idx + write_idx
        };

        used as f32 / self.capacity as f32
    }

    /// Get number of samples available to read
    pub fn available(&self) -> usize {
        let write_idx = self.write_index.load(Ordering::Relaxed);
        let read_idx = self.read_index.load(Ordering::Relaxed);

        if write_idx >= read_idx {
            write_idx - read_idx
        } else {
            self.capacity - read_idx + write_idx
        }
    }

    /// Get overrun/underrun statistics
    pub fn stats(&self) -> BufferStats {
        BufferStats {
            overruns: self.overruns.load(Ordering::Relaxed),
            underruns: self.underruns.load(Ordering::Relaxed),
            fill_level: self.fill_level(),
        }
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.overruns.store(0, Ordering::Relaxed);
        self.underruns.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BufferStats {
    pub overruns: usize,
    pub underruns: usize,
    pub fill_level: f32,
}
```

#### 1.2 WASM Bindings for SharedArrayBuffer

**File:** `src/audio/threading/shared_memory.rs`

```rust
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use js_sys::SharedArrayBuffer;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct SharedAudioBuffer {
    buffer: SharedArrayBuffer,
    capacity: usize,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl SharedAudioBuffer {
    /// Allocate a new SharedArrayBuffer for audio
    #[wasm_bindgen(constructor)]
    pub fn new(capacity_samples: usize) -> Result<SharedAudioBuffer, JsValue> {
        // Allocate 4 bytes per sample (f32)
        let byte_size = capacity_samples * 4;

        // Align to 64KB page boundary for WASM memory
        let aligned_size = (byte_size + 65535) & !65535;

        let buffer = SharedArrayBuffer::new(aligned_size as u32)?;

        Ok(Self {
            buffer,
            capacity: capacity_samples,
        })
    }

    /// Get the raw SharedArrayBuffer (for passing to workers)
    #[wasm_bindgen(getter)]
    pub fn buffer(&self) -> SharedArrayBuffer {
        self.buffer.clone()
    }

    /// Get capacity in samples
    #[wasm_bindgen(getter)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Create a Float32Array view of the buffer
    pub fn create_view(&self) -> js_sys::Float32Array {
        js_sys::Float32Array::new(&self.buffer)
    }
}
```

#### 1.3 Unit Tests

**File:** `tests/lock_free_ring_tests.rs`

```rust
#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use rusty_audio::audio::threading::{LockFreeAudioRing, SharedAudioBuffer};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_ring_buffer_basic_write_read() {
    let shared_buf = SharedAudioBuffer::new(1024).unwrap();
    let view = shared_buf.create_view();

    let ring = unsafe {
        LockFreeAudioRing::from_raw_parts(
            view.as_ptr() as *mut f32,
            shared_buf.capacity(),
        )
    };

    // Write samples
    let input: Vec<f32> = (0..512).map(|i| i as f32).collect();
    let written = ring.write(&input);
    assert_eq!(written, 512);

    // Read back
    let mut output = vec![0.0; 512];
    let read = ring.read(&mut output);
    assert_eq!(read, 512);

    // Verify data
    for i in 0..512 {
        assert_eq!(output[i], input[i]);
    }
}

#[wasm_bindgen_test]
fn test_ring_buffer_wraparound() {
    let shared_buf = SharedAudioBuffer::new(128).unwrap();
    let view = shared_buf.create_view();

    let ring = unsafe {
        LockFreeAudioRing::from_raw_parts(
            view.as_ptr() as *mut f32,
            shared_buf.capacity(),
        )
    };

    // Write 100 samples
    let input1: Vec<f32> = (0..100).map(|i| i as f32).collect();
    ring.write(&input1);

    // Read 80 samples
    let mut output1 = vec![0.0; 80];
    ring.read(&mut output1);

    // Write another 100 samples (will wrap)
    let input2: Vec<f32> = (100..200).map(|i| i as f32).collect();
    ring.write(&input2);

    // Read remaining
    let mut output2 = vec![0.0; 120];
    let read = ring.read(&mut output2);
    assert_eq!(read, 120);

    // Verify wraparound worked
    for i in 0..20 {
        assert_eq!(output2[i], input1[80 + i]);
    }
    for i in 0..100 {
        assert_eq!(output2[20 + i], input2[i]);
    }
}

#[wasm_bindgen_test]
fn test_ring_buffer_overrun() {
    let shared_buf = SharedAudioBuffer::new(128).unwrap();
    let view = shared_buf.create_view();

    let ring = unsafe {
        LockFreeAudioRing::from_raw_parts(
            view.as_ptr() as *mut f32,
            shared_buf.capacity(),
        )
    };

    // Write more than capacity
    let input: Vec<f32> = (0..256).map(|i| i as f32).collect();
    let written = ring.write(&input);

    // Should only write capacity - 1 (reserve 1 slot)
    assert_eq!(written, 127);

    let stats = ring.stats();
    assert_eq!(stats.overruns, 1);
}
```

### Deliverables
- ✅ Lock-free ring buffer implementation
- ✅ SharedArrayBuffer wrapper for WASM
- ✅ Comprehensive unit tests passing
- ✅ Benchmark showing >1M samples/sec throughput

### Success Criteria
- All ring buffer tests pass in browser
- Zero data corruption under concurrent access
- Fill level tracking accurate to ±1%
- Overrun/underrun detection working

---

## Phase 2: Worker Pool Infrastructure (Week 3-4)

### Goals
- Implement worker pool manager
- Create message passing protocol
- Establish worker lifecycle management

### Tasks

#### 2.1 Message Protocol

**File:** `src/audio/threading/messages.rs`

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerCommand {
    Initialize {
        worker_id: u32,
        shared_buffer_offset: usize,
        buffer_capacity: usize,
        sample_rate: u32,
    },
    ProcessAudio {
        task_id: u64,
        input_offset: usize,
        output_offset: usize,
        num_samples: usize,
    },
    ComputeFFT {
        task_id: u64,
        audio_offset: usize,
        fft_size: usize,
        output_offset: usize,
    },
    Shutdown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkerResponse {
    Ready {
        worker_id: u32,
    },
    TaskComplete {
        task_id: u64,
        execution_time_us: u64,
    },
    TaskFailed {
        task_id: u64,
        error: String,
    },
    ShuttingDown {
        worker_id: u32,
    },
}
```

#### 2.2 Worker Implementation

**File:** `static/wasm-audio-worker.js`

```javascript
// Import WASM module in worker context
importScripts('./rusty_audio.js');

let wasmModule = null;
let sharedBuffer = null;
let workerId = null;

// Initialize worker
self.onmessage = async function(event) {
    const msg = event.data;

    switch (msg.type) {
        case 'init':
            await initializeWorker(msg);
            break;
        case 'process_audio':
            processAudioTask(msg);
            break;
        case 'compute_fft':
            computeFFTTask(msg);
            break;
        case 'shutdown':
            shutdownWorker();
            break;
        default:
            console.error('Unknown message type:', msg.type);
    }
};

async function initializeWorker(msg) {
    workerId = msg.worker_id;
    sharedBuffer = msg.shared_buffer;

    // Load WASM module
    try {
        const wasm = await wasm_bindgen('./rusty_audio_bg.wasm');
        wasmModule = wasm;

        // Initialize audio processing
        wasmModule.init_worker(workerId, sharedBuffer, msg.buffer_capacity, msg.sample_rate);

        // Signal ready
        self.postMessage({
            type: 'ready',
            worker_id: workerId
        });
    } catch (error) {
        console.error('Worker init failed:', error);
        self.postMessage({
            type: 'task_failed',
            task_id: 0,
            error: error.toString()
        });
    }
}

function processAudioTask(msg) {
    const startTime = performance.now();

    try {
        wasmModule.process_audio_worker(
            msg.input_offset,
            msg.output_offset,
            msg.num_samples
        );

        const elapsed = (performance.now() - startTime) * 1000; // Convert to microseconds

        self.postMessage({
            type: 'task_complete',
            task_id: msg.task_id,
            execution_time_us: elapsed
        });
    } catch (error) {
        self.postMessage({
            type: 'task_failed',
            task_id: msg.task_id,
            error: error.toString()
        });
    }
}

function computeFFTTask(msg) {
    const startTime = performance.now();

    try {
        wasmModule.compute_fft_worker(
            msg.audio_offset,
            msg.fft_size,
            msg.output_offset
        );

        const elapsed = (performance.now() - startTime) * 1000;

        self.postMessage({
            type: 'task_complete',
            task_id: msg.task_id,
            execution_time_us: elapsed
        });
    } catch (error) {
        self.postMessage({
            type: 'task_failed',
            task_id: msg.task_id,
            error: error.toString()
        });
    }
}

function shutdownWorker() {
    self.postMessage({
        type: 'shutting_down',
        worker_id: workerId
    });
    self.close();
}
```

#### 2.3 Worker Pool Manager

**File:** `src/audio/threading/worker_pool.rs`

See full implementation in architecture document (Section 2.3).

Key methods to implement:
- `new()` - Create pool with N workers
- `submit_task()` - Queue task with priority
- `wait_for_task()` - Async wait for completion
- `shutdown()` - Graceful termination

#### 2.4 Integration Tests

```bash
# Run worker pool tests
wasm-pack test --headless --firefox -- --test worker_pool_tests
```

### Deliverables
- ✅ Worker pool manager implementation
- ✅ JavaScript worker script
- ✅ Message passing protocol
- ✅ Integration tests passing

### Success Criteria
- Pool can create and manage 8 workers
- Tasks are distributed round-robin
- Worker crash recovery works
- Message roundtrip < 1ms

---

## Phase 3: Audio Processing Integration (Week 5-6)

### Goals
- Integrate worker pool with existing audio backend
- Implement FFT computation on workers
- Integrate with WGPU rendering

### Tasks

#### 3.1 Update Web Audio Backend

**File:** `src/audio/web_audio_backend.rs`

Add worker pool integration:

```rust
#[cfg(target_arch = "wasm32")]
pub struct WebAudioBackend {
    context: WasmAudioContext,
    worker_pool: Option<Arc<WasmWorkerPool>>,
    shared_memory: Option<SharedMemoryHandle>,
    initialized: bool,
}

impl WebAudioBackend {
    pub async fn initialize_with_threading(&mut self, num_workers: usize) -> Result<()> {
        // Allocate SharedArrayBuffer (8MB)
        let shared_memory = SharedMemoryHandle::allocate(8 * 1024 * 1024)?;

        // Create worker pool
        let pool = WasmWorkerPool::new(num_workers, shared_memory.raw_buffer()).await?;

        self.worker_pool = Some(Arc::new(pool));
        self.shared_memory = Some(shared_memory);

        Ok(())
    }

    pub fn compute_fft_async(&self, audio_data: &[f32]) -> impl Future<Output = Vec<f32>> {
        let pool = self.worker_pool.clone().unwrap();
        let shared_mem = self.shared_memory.clone().unwrap();

        async move {
            // Write audio data to shared memory
            let audio_view = shared_mem.audio_ring_view(0);
            for (i, &sample) in audio_data.iter().enumerate() {
                audio_view.set_index(i as u32, sample);
            }

            // Submit FFT task
            let task_id = pool.submit_task(
                WorkerCommand::ComputeFFT {
                    task_id: 0,
                    audio_offset: 0,
                    fft_size: 512,
                    output_offset: 0,
                },
                TaskPriority::High,
            );

            // Wait for completion
            pool.wait_for_task(task_id).await;

            // Read FFT results
            let fft_view = shared_mem.fft_buffer_view(0);
            let mut results = Vec::with_capacity(512);
            for i in 0..512 {
                results.push(fft_view.get_index(i as u32));
            }

            results
        }
    }
}
```

#### 3.2 WGPU Spectrum Renderer Update

**File:** `src/ui/spectrum.rs`

Integrate async FFT computation:

```rust
pub struct SpectrumVisualizer {
    // ... existing fields ...
    fft_data: Arc<RwLock<Vec<f32>>>,
    last_fft_update: Instant,
}

impl SpectrumVisualizer {
    pub fn update_from_audio_backend(&mut self, backend: &WebAudioBackend, audio_samples: &[f32]) {
        // Throttle FFT updates to 60fps
        let now = Instant::now();
        if now.duration_since(self.last_fft_update).as_millis() < 16 {
            return;
        }
        self.last_fft_update = now;

        // Trigger async FFT computation
        let fft_data = self.fft_data.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let fft_result = backend.compute_fft_async(audio_samples).await;
            *fft_data.write() = fft_result;
        });
    }

    pub fn render(&self, ui: &mut egui::Ui) {
        let fft_data = self.fft_data.read();

        // Render spectrum bars using egui shapes
        ui.painter().rect_filled(
            ui.available_rect_before_wrap(),
            0.0,
            egui::Color32::BLACK,
        );

        let width = ui.available_width();
        let height = ui.available_height();
        let bar_width = width / fft_data.len() as f32;

        for (i, &magnitude) in fft_data.iter().enumerate() {
            let bar_height = magnitude * height;
            let rect = egui::Rect::from_min_size(
                egui::pos2(i as f32 * bar_width, height - bar_height),
                egui::vec2(bar_width * 0.9, bar_height),
            );

            ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(0, 255, 128));
        }
    }
}
```

### Deliverables
- ✅ Worker pool integrated with audio backend
- ✅ FFT computation offloaded to workers
- ✅ Spectrum visualization rendering at 60fps
- ✅ No audio dropouts during UI interaction

### Success Criteria
- FFT computation time < 1ms on worker
- Main thread FFT overhead < 0.5ms (just scheduling)
- Spectrum updates smoothly at 60fps
- Audio playback has <1% dropout rate

---

## Phase 4: Performance Optimization (Week 7)

### Goals
- Enable SIMD acceleration
- Implement memory prefetching
- Optimize worker scheduling

### Tasks

#### 4.1 SIMD FFT Implementation

**File:** `src/audio/threading/simd_fft.rs`

```rust
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
use std::arch::wasm32::*;

#[cfg(target_arch = "wasm32")]
pub fn fft_512_simd(input: &[f32], output: &mut [f32]) {
    #[cfg(target_feature = "simd128")]
    {
        // SIMD-accelerated FFT using WASM SIMD intrinsics
        // Process 4 samples at once

        unsafe {
            for i in (0..512).step_by(4) {
                let v = v128_load(input.as_ptr().add(i) as *const v128);

                // Apply FFT butterfly operation
                // (Simplified - full FFT implementation would go here)
                let result = f32x4_mul(v, v);

                v128_store(output.as_mut_ptr().add(i) as *mut v128, result);
            }
        }
    }

    #[cfg(not(target_feature = "simd128"))]
    {
        // Fallback to rustfft
        use rustfft::FftPlanner;
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(512);
        // ... (use rustfft)
    }
}
```

#### 4.2 Benchmark Suite

**File:** `benches/threading_benchmarks.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusty_audio::audio::threading::*;

fn benchmark_ring_buffer_throughput(c: &mut Criterion) {
    let shared_buf = SharedAudioBuffer::new(65536).unwrap();
    let view = shared_buf.create_view();
    let ring = unsafe {
        LockFreeAudioRing::from_raw_parts(view.as_ptr() as *mut f32, 65536)
    };

    let audio_chunk = vec![0.5f32; 512];

    c.bench_function("ring_buffer_write_512", |b| {
        b.iter(|| {
            ring.write(black_box(&audio_chunk));
        });
    });
}

fn benchmark_fft_512_simd(c: &mut Criterion) {
    let input = vec![0.5f32; 512];
    let mut output = vec![0.0f32; 512];

    c.bench_function("fft_512_simd", |b| {
        b.iter(|| {
            fft_512_simd(black_box(&input), black_box(&mut output));
        });
    });
}

criterion_group!(benches, benchmark_ring_buffer_throughput, benchmark_fft_512_simd);
criterion_main!(benches);
```

**Run benchmarks:**
```bash
cargo bench --target wasm32-unknown-unknown
```

### Deliverables
- ✅ SIMD FFT implementation
- ✅ Benchmark suite for critical paths
- ✅ Performance tuning based on profiling
- ✅ Memory prefetching optimizations

### Success Criteria
- FFT with SIMD is 3-4x faster than scalar
- Ring buffer throughput > 1M samples/sec
- Worker task scheduling overhead < 100µs
- Memory usage < 50MB for 8-worker pool

---

## Phase 5: Production Hardening (Week 8)

### Goals
- Error handling and recovery
- Telemetry and monitoring
- Browser compatibility testing

### Tasks

#### 5.1 Error Recovery

**File:** `src/audio/threading/error_recovery.rs`

```rust
pub struct WorkerRecoveryManager {
    pool: Arc<WasmWorkerPool>,
    failed_workers: Arc<RwLock<HashSet<u32>>>,
    max_retries: usize,
}

impl WorkerRecoveryManager {
    pub async fn handle_worker_failure(&self, worker_id: u32) -> Result<()> {
        log::error!("Worker {} failed, attempting recovery", worker_id);

        let mut failed = self.failed_workers.write();

        if failed.contains(&worker_id) {
            // Already failed once, try recreating
            log::warn!("Worker {} failed multiple times, recreating", worker_id);

            self.pool.recreate_worker(worker_id).await?;
            failed.remove(&worker_id);
        } else {
            // First failure, mark and retry
            failed.insert(worker_id);
        }

        Ok(())
    }
}
```

#### 5.2 Telemetry

**File:** `src/audio/threading/telemetry.rs`

```rust
use std::time::Instant;
use parking_lot::RwLock;

pub struct ThreadingTelemetry {
    task_completions: Arc<RwLock<Vec<TaskMetric>>>,
    worker_utilization: Arc<RwLock<[f32; 8]>>,
}

pub struct TaskMetric {
    pub task_id: u64,
    pub worker_id: u32,
    pub start_time: Instant,
    pub end_time: Instant,
    pub task_type: TaskType,
}

impl ThreadingTelemetry {
    pub fn record_task_completion(&self, metric: TaskMetric) {
        let mut completions = self.task_completions.write();
        completions.push(metric);

        // Keep only last 1000 tasks
        if completions.len() > 1000 {
            completions.drain(0..500);
        }
    }

    pub fn get_average_task_time(&self, task_type: TaskType) -> f64 {
        let completions = self.task_completions.read();

        let matching: Vec<_> = completions
            .iter()
            .filter(|m| m.task_type == task_type)
            .collect();

        if matching.is_empty() {
            return 0.0;
        }

        let total: f64 = matching
            .iter()
            .map(|m| m.end_time.duration_since(m.start_time).as_secs_f64())
            .sum();

        total / matching.len() as f64
    }

    pub fn get_worker_utilization(&self) -> [f32; 8] {
        *self.worker_utilization.read()
    }
}
```

#### 5.3 Browser Compatibility Matrix

Test on:
- ✅ Chrome 92+ (Windows, macOS, Linux)
- ✅ Firefox 79+ (Windows, macOS, Linux)
- ✅ Safari 15.2+ (macOS, iOS)
- ✅ Edge 92+ (Windows)
- ✅ Chrome Android 92+
- ✅ Samsung Internet 16.0+

### Deliverables
- ✅ Error recovery system
- ✅ Telemetry dashboard
- ✅ Cross-browser testing results
- ✅ Performance metrics collection

### Success Criteria
- Worker failures auto-recover within 500ms
- Telemetry overhead < 1% of CPU
- All target browsers pass compatibility tests
- Graceful degradation for unsupported browsers

---

## Phase 6: Deployment & Documentation (Week 9-10)

### Goals
- Production build pipeline
- CDN deployment
- Comprehensive documentation

### Tasks

#### 6.1 Production Build Script

**File:** `scripts/build-wasm-multithreaded.sh`

See architecture document (Section 9.1) for full script.

#### 6.2 CI/CD Pipeline

**File:** `.github/workflows/wasm-deploy.yml`

```yaml
name: WASM Multithreaded Deploy

on:
  push:
    branches: [main]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown

      - name: Install wasm tools
        run: |
          cargo install wasm-bindgen-cli wasm-pack
          sudo apt-get install -y binaryen

      - name: Build WASM
        run: ./scripts/build-wasm-multithreaded.sh

      - name: Run tests
        run: wasm-pack test --headless --firefox

      - name: Deploy to Netlify
        uses: nwtgck/actions-netlify@v1
        with:
          publish-dir: ./dist
          production-deploy: true
        env:
          NETLIFY_AUTH_TOKEN: ${{ secrets.NETLIFY_AUTH_TOKEN }}
          NETLIFY_SITE_ID: ${{ secrets.NETLIFY_SITE_ID }}
```

#### 6.3 User Documentation

**File:** `docs/MULTITHREADED_WASM_USER_GUIDE.md`

Create comprehensive user guide covering:
- Browser requirements
- How to enable/disable threading
- Performance tuning options
- Troubleshooting common issues

### Deliverables
- ✅ Automated build pipeline
- ✅ CDN deployment configured
- ✅ User documentation
- ✅ Developer documentation (this roadmap + architecture)

### Success Criteria
- CI/CD pipeline successfully deploys on every commit
- Documentation covers all user scenarios
- Production site loads in <3 seconds
- All production monitoring in place

---

## Risk Assessment

### High Risk

| Risk | Impact | Mitigation |
|------|--------|------------|
| **SharedArrayBuffer not available** | Application won't work | Implement single-threaded fallback, detect at runtime |
| **Browser COOP/COEP issues** | Workers won't start | Service worker header injection, multiple header sources |
| **Worker initialization timeout** | Poor user experience | Show loading screen, timeout after 30s with error message |

### Medium Risk

| Risk | Impact | Mitigation |
|------|--------|------------|
| **High latency on mobile** | Audio dropouts | Adaptive buffering, reduce worker count on mobile |
| **Memory usage too high** | Browser kills tab | Limit worker count, implement memory budget monitoring |
| **SIMD not available** | Slower performance | Fallback to scalar implementation, graceful degradation |

### Low Risk

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Worker crash** | Task fails | Auto-recovery, recreate worker, retry task |
| **Telemetry overhead** | Slight performance hit | Make telemetry optional, sample metrics instead of recording all |

---

## Success Metrics

### Performance Targets

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **FFT computation time** | <1ms | Criterion benchmarks |
| **Worker task overhead** | <100µs | Message roundtrip timing |
| **Audio dropout rate** | <0.1% | Ring buffer underrun count |
| **UI frame rate** | 60fps | egui frame time metrics |
| **Memory usage** | <50MB | Browser DevTools memory profiler |
| **Initial load time** | <3s | Lighthouse performance audit |

### Quality Targets

| Metric | Target |
|--------|--------|
| **Browser compatibility** | 90%+ of users |
| **Test coverage** | 85%+ |
| **Zero-day critical bugs** | 0 |
| **Documentation coverage** | 100% of public APIs |

---

## Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| **Phase 0** | Week 1 | Toolchain setup, testing infrastructure |
| **Phase 1** | Week 2 | Lock-free ring buffer |
| **Phase 2** | Week 3-4 | Worker pool infrastructure |
| **Phase 3** | Week 5-6 | Audio processing integration |
| **Phase 4** | Week 7 | Performance optimization |
| **Phase 5** | Week 8 | Production hardening |
| **Phase 6** | Week 9-10 | Deployment & documentation |

**Total: 8-10 weeks**

---

## Verification Checklist

### Before Production Release

- [ ] All unit tests passing (100%)
- [ ] All integration tests passing
- [ ] Benchmarks show expected performance gains
- [ ] Cross-browser testing complete
- [ ] Error recovery tested under stress
- [ ] Memory leaks checked (Chrome DevTools)
- [ ] Audio quality verified (no artifacts)
- [ ] Documentation reviewed
- [ ] CI/CD pipeline tested
- [ ] Production deployment tested on staging
- [ ] Monitoring/telemetry working
- [ ] Rollback plan documented

---

**Document Version:** 1.0
**Last Updated:** 2025-11-16
**Owner:** Rusty Audio Development Team
