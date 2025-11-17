use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use parking_lot::Mutex;
use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

const SAMPLE_RATE: f32 = 48000.0;
const RENDER_QUANTUM_SIZE: usize = 128;

/// Memory tracking allocator for benchmarking
#[derive(Default)]
struct TrackingAllocator {
    allocated: AtomicUsize,
    deallocated: AtomicUsize,
    peak_memory: AtomicUsize,
}

impl TrackingAllocator {
    fn reset(&self) {
        self.allocated.store(0, Ordering::SeqCst);
        self.deallocated.store(0, Ordering::SeqCst);
        self.peak_memory.store(0, Ordering::SeqCst);
    }

    fn current_usage(&self) -> usize {
        self.allocated.load(Ordering::SeqCst) - self.deallocated.load(Ordering::SeqCst)
    }

    fn peak_usage(&self) -> usize {
        self.peak_memory.load(Ordering::SeqCst)
    }

    fn total_allocated(&self) -> usize {
        self.allocated.load(Ordering::SeqCst)
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let size = layout.size();
            let prev_allocated = self.allocated.fetch_add(size, Ordering::SeqCst);
            let current_usage = prev_allocated + size - self.deallocated.load(Ordering::SeqCst);

            // Update peak memory if necessary
            let mut peak = self.peak_memory.load(Ordering::SeqCst);
            while current_usage > peak {
                match self.peak_memory.compare_exchange_weak(
                    peak,
                    current_usage,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocated.fetch_add(layout.size(), Ordering::SeqCst);
        System.dealloc(ptr, layout)
    }
}

/// Audio buffer with circular buffer implementation
struct CircularAudioBuffer {
    buffer: VecDeque<f32>,
    capacity: usize,
}

impl CircularAudioBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn push(&mut self, sample: f32) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(sample);
    }

    fn extend_from_slice(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.push(sample);
        }
    }

    fn drain_samples(&mut self, count: usize) -> Vec<f32> {
        let mut result = Vec::with_capacity(count);
        for _ in 0..count.min(self.buffer.len()) {
            if let Some(sample) = self.buffer.pop_front() {
                result.push(sample);
            }
        }
        result
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }
}

/// Memory-efficient audio processing chain
struct AudioProcessor {
    input_buffer: CircularAudioBuffer,
    output_buffer: CircularAudioBuffer,
    temp_buffer: Vec<f32>,
    fft_buffer: Vec<f32>,
    state: ProcessorState,
}

#[derive(Clone)]
struct ProcessorState {
    gain: f32,
    filter_state: [f32; 4], // Biquad filter state
    sample_rate: f32,
}

impl AudioProcessor {
    fn new(buffer_size: usize, sample_rate: f32) -> Self {
        Self {
            input_buffer: CircularAudioBuffer::new(buffer_size),
            output_buffer: CircularAudioBuffer::new(buffer_size),
            temp_buffer: Vec::with_capacity(RENDER_QUANTUM_SIZE),
            fft_buffer: Vec::with_capacity(2048),
            state: ProcessorState {
                gain: 1.0,
                filter_state: [0.0; 4],
                sample_rate,
            },
        }
    }

    fn process_quantum(&mut self, input: &[f32]) -> Vec<f32> {
        // Clear temp buffer and ensure capacity
        self.temp_buffer.clear();
        self.temp_buffer.reserve(input.len());

        // Simple gain processing
        for &sample in input {
            let processed = sample * self.state.gain;
            self.temp_buffer.push(processed);
        }

        // Update output buffer
        self.output_buffer.extend_from_slice(&self.temp_buffer);

        // Return processed samples
        self.temp_buffer.clone()
    }

    fn set_gain(&mut self, gain: f32) {
        self.state.gain = gain;
    }

    fn get_output_samples(&mut self, count: usize) -> Vec<f32> {
        self.output_buffer.drain_samples(count)
    }
}

/// Shared audio processor for multithreaded scenarios
type SharedProcessor = Arc<Mutex<AudioProcessor>>;

/// Benchmark basic buffer allocation
fn bench_buffer_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_allocation");

    for &size in &[512, 1024, 4096, 16384, 65536] {
        group.bench_with_input(BenchmarkId::new("buffer_size", size), &size, |b, &size| {
            b.iter(|| {
                let _buffer: Vec<f32> = vec![0.0; black_box(size)];
            });
        });
    }

    group.bench_function("audio_buffer_with_capacity", |b| {
        b.iter(|| {
            let _buffer: Vec<f32> = Vec::with_capacity(black_box(4096));
        });
    });

    group.finish();
}

/// Benchmark circular buffer operations
fn bench_circular_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("circular_buffer");

    let mut buffer = CircularAudioBuffer::new(4096);
    let test_data: Vec<f32> = (0..RENDER_QUANTUM_SIZE)
        .map(|i| i as f32 / RENDER_QUANTUM_SIZE as f32)
        .collect();

    group.bench_function("push_samples", |b| {
        b.iter(|| {
            for &sample in black_box(&test_data) {
                buffer.push(sample);
            }
        });
    });

    group.bench_function("extend_from_slice", |b| {
        b.iter(|| {
            buffer.extend_from_slice(black_box(&test_data));
        });
    });

    group.bench_function("drain_samples", |b| {
        // Fill buffer first
        for _ in 0..100 {
            buffer.extend_from_slice(&test_data);
        }

        b.iter(|| {
            let _samples = buffer.drain_samples(black_box(RENDER_QUANTUM_SIZE));
        });
    });

    group.finish();
}

/// Benchmark audio processor memory usage
fn bench_audio_processor(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_processor");

    let test_input: Vec<f32> = (0..RENDER_QUANTUM_SIZE)
        .map(|i| (i as f32 * 0.1).sin())
        .collect();

    group.bench_function("processor_creation", |b| {
        b.iter(|| {
            let _processor = AudioProcessor::new(black_box(4096), black_box(SAMPLE_RATE));
        });
    });

    group.bench_function("quantum_processing", |b| {
        let mut processor = AudioProcessor::new(4096, SAMPLE_RATE);
        processor.set_gain(0.5);

        b.iter(|| {
            let _output = processor.process_quantum(black_box(&test_input));
        });
    });

    group.bench_function("continuous_processing", |b| {
        let mut processor = AudioProcessor::new(8192, SAMPLE_RATE);

        b.iter(|| {
            // Process 10 quantum blocks
            for _ in 0..10 {
                let _output = processor.process_quantum(black_box(&test_input));
            }
        });
    });

    group.finish();
}

/// Benchmark memory allocation patterns during real-time processing
fn bench_realtime_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("realtime_allocation");

    let test_input: Vec<f32> = (0..RENDER_QUANTUM_SIZE)
        .map(|i| (i as f32 * 0.1).sin())
        .collect();

    group.bench_function("no_allocation_processing", |b| {
        let mut processor = AudioProcessor::new(8192, SAMPLE_RATE);
        let mut output_buffer = Vec::with_capacity(RENDER_QUANTUM_SIZE);

        b.iter(|| {
            // Process without allocating new memory
            output_buffer.clear();

            for &sample in black_box(&test_input) {
                let processed = sample * 0.5;
                output_buffer.push(processed);
            }

            black_box(&output_buffer);
        });
    });

    group.bench_function("allocation_heavy_processing", |b| {
        let mut processor = AudioProcessor::new(8192, SAMPLE_RATE);

        b.iter(|| {
            // Process with allocations (bad practice)
            let mut output_buffer = Vec::new(); // New allocation each time

            for &sample in black_box(&test_input) {
                let processed = sample * 0.5;
                output_buffer.push(processed);
            }

            black_box(output_buffer);
        });
    });

    group.bench_function("pre_allocated_processing", |b| {
        let mut processor = AudioProcessor::new(8192, SAMPLE_RATE);
        let mut output_buffer = vec![0.0; RENDER_QUANTUM_SIZE];

        b.iter(|| {
            // Process using pre-allocated buffer
            for (i, &sample) in black_box(&test_input).iter().enumerate() {
                output_buffer[i] = sample * 0.5;
            }

            black_box(&output_buffer);
        });
    });

    group.finish();
}

/// Benchmark shared processor access patterns
fn bench_shared_processor(c: &mut Criterion) {
    let mut group = c.benchmark_group("shared_processor");

    let processor = Arc::new(Mutex::new(AudioProcessor::new(8192, SAMPLE_RATE)));
    let test_input: Vec<f32> = (0..RENDER_QUANTUM_SIZE)
        .map(|i| (i as f32 * 0.1).sin())
        .collect();

    group.bench_function("mutex_access", |b| {
        let processor = processor.clone();

        b.iter(|| {
            let mut proc = processor.lock();
            let _output = proc.process_quantum(black_box(&test_input));
        });
    });

    group.bench_function("atomic_operations", |b| {
        use std::sync::atomic::{AtomicU32, Ordering};
        let gain = Arc::new(AtomicU32::new(0.5f32.to_bits()));

        b.iter(|| {
            let gain_value = f32::from_bits(gain.load(Ordering::Relaxed));
            let mut output = Vec::with_capacity(test_input.len());

            for &sample in black_box(&test_input) {
                output.push(sample * gain_value);
            }

            black_box(output);
        });
    });

    group.finish();
}

/// Benchmark memory usage with different data structures
fn bench_data_structures(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_structures");

    let size = 4096;

    group.bench_function("vec_allocation", |b| {
        b.iter(|| {
            let _data: Vec<f32> = vec![0.0; black_box(size)];
        });
    });

    group.bench_function("boxed_slice_allocation", |b| {
        b.iter(|| {
            let _data: Box<[f32]> = vec![0.0; black_box(size)].into_boxed_slice();
        });
    });

    group.bench_function("bytes_allocation", |b| {
        b.iter(|| {
            let data: Vec<u8> = vec![0; black_box(size * 4)]; // f32 = 4 bytes
            let _bytes = Bytes::from(data);
        });
    });

    group.bench_function("vecdeque_allocation", |b| {
        b.iter(|| {
            let _data: VecDeque<f32> = VecDeque::with_capacity(black_box(size));
        });
    });

    group.finish();
}

/// Benchmark garbage collection-like scenarios
fn bench_memory_pressure(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pressure");

    group.bench_function("fragmented_allocation", |b| {
        b.iter(|| {
            let mut buffers = Vec::new();

            // Allocate many small buffers (fragmentation)
            for size in [64, 128, 256, 512, 1024].iter().cycle().take(100) {
                let buffer: Vec<f32> = vec![0.0; *size];
                buffers.push(buffer);
            }

            // Drop every other buffer (more fragmentation)
            let mut i = 0;
            buffers.retain(|_| {
                i += 1;
                i % 2 == 0
            });

            black_box(buffers);
        });
    });

    group.bench_function("large_contiguous_allocation", |b| {
        b.iter(|| {
            let _buffer: Vec<f32> = vec![0.0; black_box(65536)];
        });
    });

    group.bench_function("temporary_allocation_storm", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let _temp: Vec<f32> = vec![0.0; black_box(128)];
            }
        });
    });

    group.finish();
}

/// Benchmark cache-friendly access patterns
fn bench_cache_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_patterns");

    let size = 16384;
    let data: Vec<f32> = (0..size).map(|i| i as f32).collect();

    group.bench_function("sequential_access", |b| {
        b.iter(|| {
            let mut sum = 0.0f32;
            for &value in black_box(&data) {
                sum += value;
            }
            black_box(sum);
        });
    });

    group.bench_function("strided_access", |b| {
        b.iter(|| {
            let mut sum = 0.0f32;
            let stride = 16;
            for i in (0..data.len()).step_by(stride) {
                sum += data[i];
            }
            black_box(sum);
        });
    });

    group.bench_function("random_access", |b| {
        use rand::{rngs::StdRng, Rng, SeedableRng};
        let mut rng = StdRng::seed_from_u64(42);
        let indices: Vec<usize> = (0..1000).map(|_| rng.gen_range(0..size)).collect();

        b.iter(|| {
            let mut sum = 0.0f32;
            for &index in black_box(&indices) {
                sum += data[index];
            }
            black_box(sum);
        });
    });

    group.finish();
}

criterion_group! {
    name = memory_benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));
    targets =
        bench_buffer_allocation,
        bench_circular_buffer,
        bench_audio_processor,
        bench_realtime_allocation_patterns,
        bench_shared_processor,
        bench_data_structures,
        bench_memory_pressure,
        bench_cache_patterns
}

criterion_main!(memory_benches);
