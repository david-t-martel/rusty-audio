//! Benchmarks specifically targeting identified performance bottlenecks
//!
//! This benchmark suite measures the exact performance issues identified in the
//! performance analysis, providing quantitative metrics for optimization work.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use parking_lot::{Mutex, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const AUDIO_BUFFER_SIZE: usize = 256;
const RING_BUFFER_SIZE: usize = 4096;

/// Benchmark the current mutex-based audio callback implementation
fn bench_audio_callback_with_mutex(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_callback_locks");

    for num_threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("mutex_callback", num_threads),
            num_threads,
            |b, &num_threads| {
                let callback = Arc::new(Mutex::new(|data: &mut [f32]| {
                    // Simulate audio processing
                    for sample in data.iter_mut() {
                        *sample *= 0.5;
                    }
                }));

                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let cb = callback.clone();
                            thread::spawn(move || {
                                let mut buffer = vec![1.0f32; AUDIO_BUFFER_SIZE];
                                let mut locked = cb.lock();
                                locked(&mut buffer);
                                black_box(buffer);
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark lock-free alternative using atomics
fn bench_audio_callback_lock_free(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_callback_lockfree");

    for num_threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("atomic_callback", num_threads),
            num_threads,
            |b, &num_threads| {
                // Simulate lock-free state management
                let gain = Arc::new(AtomicUsize::new(0x3f000000)); // 0.5 as f32 bits

                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let gain = gain.clone();
                            thread::spawn(move || {
                                let mut buffer = vec![1.0f32; AUDIO_BUFFER_SIZE];
                                let gain_bits = gain.load(Ordering::Relaxed);
                                let gain_value = f32::from_bits(gain_bits as u32);

                                for sample in buffer.iter_mut() {
                                    *sample *= gain_value;
                                }
                                black_box(buffer);
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Ring buffer with multiple RwLocks (current implementation)
struct RwLockRingBuffer {
    buffer: Arc<RwLock<Vec<f32>>>,
    write_pos: Arc<RwLock<usize>>,
    read_pos: Arc<RwLock<usize>>,
    capacity: usize,
}

impl RwLockRingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(vec![0.0; capacity])),
            write_pos: Arc::new(RwLock::new(0)),
            read_pos: Arc::new(RwLock::new(0)),
            capacity,
        }
    }

    fn write(&self, data: &[f32]) -> usize {
        let mut buffer = self.buffer.write();
        let mut write_pos = self.write_pos.write();
        let read_pos = *self.read_pos.read();

        let mut written = 0;
        for &sample in data {
            let next_pos = (*write_pos + 1) % self.capacity;
            if next_pos == read_pos {
                break; // Buffer full
            }
            buffer[*write_pos] = sample;
            *write_pos = next_pos;
            written += 1;
        }
        written
    }

    fn read(&self, output: &mut [f32]) -> usize {
        let buffer = self.buffer.read();
        let write_pos = *self.write_pos.read();
        let mut read_pos = self.read_pos.write();

        let mut read = 0;
        for sample in output.iter_mut() {
            if *read_pos == write_pos {
                *sample = 0.0;
            } else {
                *sample = buffer[*read_pos];
                *read_pos = (*read_pos + 1) % self.capacity;
                read += 1;
            }
        }
        read
    }
}

/// Lock-free ring buffer implementation
struct LockFreeRingBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
}

impl LockFreeRingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            capacity,
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
        }
    }

    fn write(&self, data: &[f32]) -> usize {
        let mut write_pos = self.write_pos.load(Ordering::Acquire);
        let read_pos = self.read_pos.load(Ordering::Acquire);

        let mut written = 0;
        for &sample in data {
            let next_pos = (write_pos + 1) % self.capacity;
            if next_pos == read_pos {
                break; // Buffer full
            }
            unsafe {
                // Safe because we control access patterns
                *self.buffer.as_ptr().add(write_pos) as *mut f32 = sample;
            }
            write_pos = next_pos;
            written += 1;
        }

        self.write_pos.store(write_pos, Ordering::Release);
        written
    }

    fn read(&self, output: &mut [f32]) -> usize {
        let mut read_pos = self.read_pos.load(Ordering::Acquire);
        let write_pos = self.write_pos.load(Ordering::Acquire);

        let mut read = 0;
        for sample in output.iter_mut() {
            if read_pos == write_pos {
                *sample = 0.0;
            } else {
                *sample = unsafe {
                    // Safe because we control access patterns
                    *self.buffer.as_ptr().add(read_pos)
                };
                read_pos = (read_pos + 1) % self.capacity;
                read += 1;
            }
        }

        self.read_pos.store(read_pos, Ordering::Release);
        read
    }
}

/// Benchmark ring buffer implementations
fn bench_ring_buffer_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer");
    group.throughput(Throughput::Bytes((AUDIO_BUFFER_SIZE * 4) as u64));

    // Benchmark RwLock-based ring buffer
    group.bench_function("rwlock_ring_buffer", |b| {
        let buffer = RwLockRingBuffer::new(RING_BUFFER_SIZE);
        let input = vec![1.0f32; AUDIO_BUFFER_SIZE];
        let mut output = vec![0.0f32; AUDIO_BUFFER_SIZE];

        b.iter(|| {
            buffer.write(&input);
            buffer.read(&mut output);
            black_box(&output);
        });
    });

    // Benchmark lock-free ring buffer
    group.bench_function("lockfree_ring_buffer", |b| {
        let buffer = LockFreeRingBuffer::new(RING_BUFFER_SIZE);
        let input = vec![1.0f32; AUDIO_BUFFER_SIZE];
        let mut output = vec![0.0f32; AUDIO_BUFFER_SIZE];

        b.iter(|| {
            buffer.write(&input);
            buffer.read(&mut output);
            black_box(&output);
        });
    });

    group.finish();
}

/// Benchmark concurrent ring buffer access (producer/consumer pattern)
fn bench_ring_buffer_concurrent(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer_concurrent");

    // RwLock version with contention
    group.bench_function("rwlock_contention", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;

            for _ in 0..iters {
                let buffer = Arc::new(RwLockRingBuffer::new(RING_BUFFER_SIZE));
                let buffer_write = buffer.clone();
                let buffer_read = buffer.clone();

                let start = Instant::now();

                // Producer thread
                let producer = thread::spawn(move || {
                    let data = vec![1.0f32; AUDIO_BUFFER_SIZE];
                    for _ in 0..100 {
                        buffer_write.write(&data);
                    }
                });

                // Consumer thread
                let consumer = thread::spawn(move || {
                    let mut output = vec![0.0f32; AUDIO_BUFFER_SIZE];
                    for _ in 0..100 {
                        buffer_read.read(&mut output);
                    }
                    black_box(output);
                });

                producer.join().unwrap();
                consumer.join().unwrap();

                total_duration += start.elapsed();
            }

            total_duration
        });
    });

    // Lock-free version without contention
    group.bench_function("lockfree_no_contention", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::ZERO;

            for _ in 0..iters {
                let buffer = Arc::new(LockFreeRingBuffer::new(RING_BUFFER_SIZE));
                let buffer_write = buffer.clone();
                let buffer_read = buffer.clone();

                let start = Instant::now();

                // Producer thread
                let producer = thread::spawn(move || {
                    let data = vec![1.0f32; AUDIO_BUFFER_SIZE];
                    for _ in 0..100 {
                        buffer_write.write(&data);
                    }
                });

                // Consumer thread
                let consumer = thread::spawn(move || {
                    let mut output = vec![0.0f32; AUDIO_BUFFER_SIZE];
                    for _ in 0..100 {
                        buffer_read.read(&mut output);
                    }
                    black_box(output);
                });

                producer.join().unwrap();
                consumer.join().unwrap();

                total_duration += start.elapsed();
            }

            total_duration
        });
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocations");

    // Benchmark repeated allocations (current pattern)
    group.bench_function("repeated_alloc", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let buffer = vec![0.0f32; AUDIO_BUFFER_SIZE];
                black_box(buffer);
            }
        });
    });

    // Benchmark with pre-allocated buffer (recommended pattern)
    group.bench_function("preallocated", |b| {
        let mut buffer = vec![0.0f32; AUDIO_BUFFER_SIZE];
        b.iter(|| {
            for _ in 0..100 {
                buffer.fill(0.0);
                black_box(&buffer);
            }
        });
    });

    // Benchmark Arc allocation overhead
    group.bench_function("arc_allocation", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let data = Arc::new(vec![0.0f32; AUDIO_BUFFER_SIZE]);
                black_box(data);
            }
        });
    });

    group.finish();
}

/// Benchmark GUI update patterns
fn bench_gui_update_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("gui_updates");

    // Simulate full redraw (current pattern)
    group.bench_function("full_redraw", |b| {
        b.iter(|| {
            // Simulate drawing 64 spectrum bars
            for i in 0..64 {
                let height = (i as f32 * 0.1).sin().abs();
                let color = [255u8, (i * 4) as u8, 0, 255];
                black_box((height, color));
            }

            // Simulate updating all UI elements
            for _ in 0..20 {
                let button_state = false;
                let slider_value = 0.5f32;
                black_box((button_state, slider_value));
            }
        });
    });

    // Simulate dirty region tracking (recommended)
    group.bench_function("dirty_regions", |b| {
        let mut dirty = vec![false; 64];
        let mut values = vec![0.0f32; 64];

        b.iter(|| {
            // Only update changed values
            for i in 0..64 {
                let new_value = (i as f32 * 0.1).sin().abs();
                if (new_value - values[i]).abs() > 0.01 {
                    dirty[i] = true;
                    values[i] = new_value;

                    // Only draw if dirty
                    let color = [255u8, (i * 4) as u8, 0, 255];
                    black_box((new_value, color));
                }
            }

            // Clear dirty flags
            dirty.fill(false);
        });
    });

    group.finish();
}

/// Benchmark spectrum processing algorithms
fn bench_spectrum_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("spectrum");

    let spectrum_data = vec![0.5f32; 512]; // Simulated FFT output

    // Current implementation: recalculating bins every frame
    group.bench_function("recalc_bins", |b| {
        let num_bars = 64;
        b.iter(|| {
            let mut processed = Vec::with_capacity(num_bars);
            for i in 0..num_bars {
                let bin_start = (i * spectrum_data.len()) / num_bars;
                let bin_end = ((i + 1) * spectrum_data.len()) / num_bars;

                let mut sum = 0.0f32;
                for j in bin_start..bin_end {
                    sum += spectrum_data[j];
                }
                processed.push(sum / (bin_end - bin_start) as f32);
            }
            black_box(processed);
        });
    });

    // Optimized: pre-calculated bins
    group.bench_function("precalc_bins", |b| {
        let num_bars = 64;
        // Pre-calculate bin boundaries
        let bins: Vec<(usize, usize)> = (0..num_bars)
            .map(|i| {
                let start = (i * spectrum_data.len()) / num_bars;
                let end = ((i + 1) * spectrum_data.len()) / num_bars;
                (start, end)
            })
            .collect();

        b.iter(|| {
            let mut processed = Vec::with_capacity(num_bars);
            for &(start, end) in &bins {
                let mut sum = 0.0f32;
                for j in start..end {
                    sum += spectrum_data[j];
                }
                processed.push(sum / (end - start) as f32);
            }
            black_box(processed);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_audio_callback_with_mutex,
    bench_audio_callback_lock_free,
    bench_ring_buffer_comparison,
    bench_ring_buffer_concurrent,
    bench_allocation_patterns,
    bench_gui_update_patterns,
    bench_spectrum_processing
);

criterion_main!(benches);