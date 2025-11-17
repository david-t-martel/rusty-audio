use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rusty_audio::audio_optimizations::*;
use rusty_audio::audio_performance::*;
use std::time::Duration;

/// Benchmark spectrum processing performance
fn bench_spectrum_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("spectrum_processing");

    for &fft_size in &[512, 1024, 2048, 4096] {
        group.throughput(Throughput::Elements(fft_size as u64));
        group.bench_with_input(
            BenchmarkId::new("fft_size", fft_size),
            &fft_size,
            |b, &size| {
                let mut processor = SpectrumProcessor::new(size);
                let input = vec![0.5f32; size];

                b.iter(|| {
                    // Simulate spectrum processing
                    for sample in &input {
                        black_box(sample);
                    }
                });
            },
        );
    }
    group.finish();
}

/// Benchmark ring buffer operations
fn bench_ring_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("ring_buffer");

    for &buffer_size in &[256, 512, 1024, 2048] {
        group.throughput(Throughput::Elements(buffer_size as u64));

        group.bench_with_input(
            BenchmarkId::new("write", buffer_size),
            &buffer_size,
            |b, &size| {
                let mut buffer = AudioRingBuffer::new(size);
                let data = vec![0.5f32; 128];

                b.iter(|| {
                    buffer.write(black_box(&data));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("read", buffer_size),
            &buffer_size,
            |b, &size| {
                let mut buffer = AudioRingBuffer::new(size);
                let data = vec![0.5f32; size / 2];
                buffer.write(&data);
                let mut output = vec![0.0f32; 128];

                b.iter(|| {
                    buffer.read(black_box(&mut output));
                });
            },
        );
    }
    group.finish();
}

/// Benchmark lock-free buffer operations
fn bench_lock_free_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lock_free_buffer");

    group.bench_function("single_write", |b| {
        let buffer = LockFreeAudioBuffer::new(1024);

        b.iter(|| {
            buffer.write_sample(black_box(0.5));
        });
    });

    group.bench_function("single_read", |b| {
        let buffer = LockFreeAudioBuffer::new(1024);
        for i in 0..512 {
            buffer.write_sample(i as f32);
        }

        b.iter(|| {
            black_box(buffer.read_sample());
        });
    });

    group.bench_function("concurrent_access", |b| {
        let buffer = Arc::new(LockFreeAudioBuffer::new(1024));
        let buffer_clone = Arc::clone(&buffer);

        // Spawn writer thread
        std::thread::spawn(move || loop {
            buffer_clone.write_sample(0.5);
            std::thread::yield_now();
        });

        b.iter(|| {
            black_box(buffer.read_sample());
        });
    });

    group.finish();
}

/// Benchmark EQ band processing
fn bench_eq_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("eq_processing");

    for &num_bands in &[4, 8, 16, 32] {
        group.throughput(Throughput::Elements(num_bands as u64 * 512));

        group.bench_with_input(
            BenchmarkId::new("bands", num_bands),
            &num_bands,
            |b, &bands| {
                let mut processor = EqBandOptimizer::new(bands);
                let input = vec![0.5f32; 512];
                let mut output = vec![0.0f32; 512];

                // Initialize bands
                for i in 0..bands {
                    let freq = 60.0 * 2.0_f32.powi(i as i32);
                    processor.update_band(i, freq, 1.0, 0.0);
                }

                b.iter(|| {
                    processor.process(black_box(&input), black_box(&mut output));
                });
            },
        );
    }
    group.finish();
}

/// Benchmark memory pool operations
fn bench_memory_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pool");

    group.bench_function("acquire_release", |b| {
        let mut pool = AudioBufferPool::new(10, 1024);

        b.iter(|| {
            if let Some(buffer) = pool.acquire() {
                black_box(&buffer);
                pool.release(buffer);
            }
        });
    });

    group.bench_function("pool_vs_alloc", |b| {
        let mut pool = AudioBufferPool::new(10, 1024);
        let mut use_pool = true;

        b.iter(|| {
            if use_pool {
                if let Some(buffer) = pool.acquire() {
                    black_box(&buffer);
                    pool.release(buffer);
                }
            } else {
                let buffer = Arc::new(vec![0.0f32; 1024]);
                black_box(&buffer);
            }
            use_pool = !use_pool;
        });
    });

    group.finish();
}

/// Benchmark SIMD operations
#[cfg(target_arch = "x86_64")]
fn bench_simd_operations(c: &mut Criterion) {
    use rusty_audio::audio_optimizations::simd;

    let mut group = c.benchmark_group("simd_operations");

    for &buffer_size in &[128, 256, 512, 1024, 2048] {
        group.throughput(Throughput::Bytes(buffer_size as u64 * 4));

        // Benchmark SSE mixing
        group.bench_with_input(
            BenchmarkId::new("mix_sse", buffer_size),
            &buffer_size,
            |b, &size| {
                let mut output = vec![0.5f32; size];
                let input = vec![0.3f32; size];
                let gain = 0.7;

                b.iter(|| unsafe {
                    simd::mix_buffers_sse(
                        black_box(&mut output),
                        black_box(&input),
                        black_box(gain),
                    );
                });
            },
        );

        // Benchmark scalar mixing for comparison
        group.bench_with_input(
            BenchmarkId::new("mix_scalar", buffer_size),
            &buffer_size,
            |b, &size| {
                let mut output = vec![0.5f32; size];
                let input = vec![0.3f32; size];
                let gain = 0.7;

                b.iter(|| {
                    for (o, i) in output.iter_mut().zip(input.iter()) {
                        *o += i * gain;
                    }
                    black_box(&output);
                });
            },
        );

        // Benchmark RMS calculation
        group.bench_with_input(
            BenchmarkId::new("rms_sse", buffer_size),
            &buffer_size,
            |b, &size| {
                let buffer = vec![0.5f32; size];

                b.iter(|| unsafe {
                    black_box(simd::compute_rms_sse(&buffer));
                });
            },
        );

        // Benchmark peak detection
        group.bench_with_input(
            BenchmarkId::new("peak_sse", buffer_size),
            &buffer_size,
            |b, &size| {
                let buffer: Vec<f32> = (0..size).map(|i| (i as f32 / size as f32).sin()).collect();

                b.iter(|| unsafe {
                    black_box(simd::find_peak_sse(&buffer));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark FFT operations
fn bench_fft_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_operations");

    for &fft_size in &[256, 512, 1024, 2048, 4096] {
        group.throughput(Throughput::Elements(fft_size as u64));

        group.bench_with_input(
            BenchmarkId::new("optimized_fft", fft_size),
            &fft_size,
            |b, &size| {
                let mut fft = OptimizedFFT::new(size);
                let input: Vec<f32> = (0..size)
                    .map(|i| (2.0 * std::f32::consts::PI * i as f32 * 440.0 / 48000.0).sin())
                    .collect();
                let mut output = vec![0.0f32; size / 2];

                b.iter(|| {
                    fft.process(black_box(&input), black_box(&mut output));
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("stereo_fft", fft_size),
            &fft_size,
            |b, &size| {
                let mut fft = OptimizedFFT::new(size);
                let left: Vec<f32> = (0..size)
                    .map(|i| (2.0 * std::f32::consts::PI * i as f32 * 440.0 / 48000.0).sin())
                    .collect();
                let right: Vec<f32> = (0..size)
                    .map(|i| (2.0 * std::f32::consts::PI * i as f32 * 880.0 / 48000.0).sin())
                    .collect();
                let mut output_left = vec![0.0f32; size / 2];
                let mut output_right = vec![0.0f32; size / 2];

                b.iter(|| {
                    fft.process_stereo(
                        black_box(&left),
                        black_box(&right),
                        black_box(&mut output_left),
                        black_box(&mut output_right),
                    );
                });
            },
        );
    }

    group.finish();
}

/// Benchmark adaptive buffer management
fn bench_adaptive_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("adaptive_buffer");

    group.bench_function("underrun_handling", |b| {
        let manager = AdaptiveBufferManager::new(128, 2048);

        b.iter(|| {
            manager.record_underrun();
            black_box(manager.get_buffer_size());
        });
    });

    group.bench_function("successful_callback", |b| {
        let manager = AdaptiveBufferManager::new(128, 2048);

        b.iter(|| {
            manager.record_successful_callback();
            black_box(manager.get_buffer_size());
        });
    });

    group.finish();
}

/// Benchmark cache performance
fn bench_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");

    group.bench_function("cache_hit", |b| {
        let loader = CachedAudioLoader::new(10);
        let path = std::path::Path::new("test.wav");

        // Preload to ensure cache hit
        let _ = loader.load(path);

        b.iter(|| {
            black_box(loader.load(black_box(path)));
        });
    });

    group.bench_function("lru_eviction", |b| {
        let loader = CachedAudioLoader::new(2);
        let paths = vec![
            std::path::Path::new("test1.wav"),
            std::path::Path::new("test2.wav"),
            std::path::Path::new("test3.wav"),
        ];
        let mut idx = 0;

        b.iter(|| {
            let path = paths[idx % paths.len()];
            black_box(loader.load(black_box(path)));
            idx += 1;
        });
    });

    group.finish();
}

/// Benchmark end-to-end audio pipeline
fn bench_audio_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_pipeline");

    group.bench_function("complete_pipeline", |b| {
        let mut spectrum_processor = SpectrumProcessor::new(1024);
        let mut eq_processor = EqBandOptimizer::new(8);
        let mut ring_buffer = AudioRingBuffer::new(2048);
        let input = vec![0.5f32; 512];
        let mut output = vec![0.0f32; 512];

        // Initialize EQ bands
        for i in 0..8 {
            let freq = 60.0 * 2.0_f32.powi(i as i32);
            eq_processor.update_band(i, freq, 1.0, 0.0);
        }

        b.iter(|| {
            // Simulate complete audio pipeline
            ring_buffer.write(&input);
            let read = ring_buffer.read(&mut output);

            if read > 0 {
                eq_processor.process(&output[..read], &mut output[..read]);
                black_box(&output);
            }
        });
    });

    group.bench_function("parallel_processing", |b| {
        use rayon::prelude::*;

        let chunks: Vec<Vec<f32>> = (0..4).map(|_| vec![0.5f32; 256]).collect();

        b.iter(|| {
            chunks.par_iter().for_each(|chunk| {
                let sum: f32 = chunk.iter().sum();
                black_box(sum);
            });
        });
    });

    group.finish();
}

criterion_group! {
    name = performance;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));
    targets =
        bench_spectrum_processing,
        bench_ring_buffer,
        bench_lock_free_buffer,
        bench_eq_processing,
        bench_memory_pool,
        bench_fft_operations,
        bench_adaptive_buffer,
        bench_cache_performance,
        bench_audio_pipeline
}

#[cfg(target_arch = "x86_64")]
criterion_group! {
    name = simd;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(5));
    targets = bench_simd_operations
}

#[cfg(target_arch = "x86_64")]
criterion_main!(performance, simd);

#[cfg(not(target_arch = "x86_64"))]
criterion_main!(performance);
