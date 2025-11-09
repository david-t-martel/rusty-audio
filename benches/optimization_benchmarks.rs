//! Comprehensive benchmarks for audio performance optimizations
//!
//! This benchmark suite validates the performance improvements from:
//! - Phase 1.3: Pre-allocated buffer pool
//! - Phase 3: Parallel EQ band processing
//! - Phase 4.1: Cache-line alignment
//! - Phase 4.2: Zero-copy audio pipeline

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rusty_audio::audio_performance::{
    OptimizedBufferPool, OptimizedEqProcessor, OptimizedSpectrumProcessor,
};
use rusty_audio::audio_performance_optimized::{
    AlignedBuffer, OptimizedBufferPoolV2, ParallelEqProcessor, PooledSpectrumProcessor,
    ZeroCopyAudioPipeline,
};
use std::sync::Arc;
use web_audio_api::{context::AudioContext, node::AnalyserNode};

/// Benchmark buffer pool performance (Phase 1.3)
fn benchmark_buffer_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pool");

    // Test different pool sizes and buffer sizes
    for pool_size in &[8, 16, 32] {
        for buffer_size in &[512, 1024, 2048, 4096] {
            let pool_v1 = OptimizedBufferPool::new(*pool_size, *buffer_size);
            let pool_v2 = Arc::new(OptimizedBufferPoolV2::new(*pool_size, *buffer_size));

            // Benchmark old buffer pool
            group.throughput(Throughput::Elements(*buffer_size as u64));
            group.bench_with_input(
                BenchmarkId::new(
                    "v1_acquire_release",
                    format!("{}x{}", pool_size, buffer_size),
                ),
                buffer_size,
                |b, &size| {
                    b.iter(|| {
                        // Simulate allocation pattern
                        for _ in 0..10 {
                            if let Some(buffer) = pool_v1.acquire() {
                                // Simulate some work
                                black_box(&buffer);
                                pool_v1.release(buffer);
                            }
                        }
                    });
                },
            );

            // Benchmark new optimized buffer pool
            group.bench_with_input(
                BenchmarkId::new(
                    "v2_acquire_release",
                    format!("{}x{}", pool_size, buffer_size),
                ),
                buffer_size,
                |b, &size| {
                    let pool = Arc::clone(&pool_v2);
                    b.iter(|| {
                        for _ in 0..10 {
                            if let Some(mut buffer) = pool.acquire() {
                                // Simulate work with cache-aligned buffer
                                let slice = buffer.as_mut_slice();
                                black_box(slice);
                                pool.release(buffer);
                            }
                        }
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark spectrum processing with and without buffer pooling
fn benchmark_spectrum_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("spectrum_processing");

    let context = AudioContext::default();
    let mut analyser = context.create_analyser();
    analyser.set_fft_size(2048);

    // Original spectrum processor
    let mut processor_v1 = OptimizedSpectrumProcessor::new(2048);

    // New pooled spectrum processor
    let buffer_pool = Arc::new(OptimizedBufferPoolV2::new(16, 2048));
    let mut processor_v2 = PooledSpectrumProcessor::new(Arc::clone(&buffer_pool), 2048);

    group.throughput(Throughput::Elements(1024)); // Processing 1024 frequency bins

    // Benchmark original processor (allocates on each call)
    group.bench_function("original_with_allocations", |b| {
        b.iter(|| {
            let spectrum = processor_v1.process_spectrum(&mut analyser);
            black_box(spectrum);
        });
    });

    // Benchmark pooled processor (no allocations)
    group.bench_function("pooled_zero_alloc", |b| {
        b.iter(|| {
            let spectrum = processor_v2.process_spectrum_pooled(&mut analyser);
            black_box(spectrum);
        });
    });

    group.finish();
}

/// Benchmark parallel EQ processing (Phase 3)
fn benchmark_parallel_eq(c: &mut Criterion) {
    let mut group = c.benchmark_group("eq_processing");

    const SAMPLE_RATE: f32 = 44100.0;
    const NUM_BANDS: usize = 8;

    // Test different block sizes
    for block_size in &[128, 256, 512, 1024, 2048] {
        let input = vec![0.5f32; *block_size];
        let mut output = vec![0.0f32; *block_size];

        // Original sequential EQ processor
        let mut seq_processor = OptimizedEqProcessor::new(NUM_BANDS, SAMPLE_RATE);
        seq_processor.prepare(*block_size);

        // Set up some EQ bands
        for i in 0..NUM_BANDS {
            let freq = 60.0 * 2.0_f32.powi(i as i32);
            seq_processor.update_band(i, freq, 1.0, 3.0); // +3dB gain
        }

        // New parallel EQ processor
        let mut par_processor = ParallelEqProcessor::new(NUM_BANDS, SAMPLE_RATE, *block_size);
        for i in 0..NUM_BANDS {
            let freq = 60.0 * 2.0_f32.powi(i as i32);
            par_processor.update_band(i, freq, 1.0, 3.0);
        }

        group.throughput(Throughput::Elements(*block_size as u64));

        // Benchmark sequential processing
        group.bench_with_input(
            BenchmarkId::new("sequential", block_size),
            block_size,
            |b, _| {
                b.iter(|| {
                    seq_processor.process(&input, &mut output);
                    black_box(&output);
                });
            },
        );

        // Benchmark parallel processing
        group.bench_with_input(
            BenchmarkId::new("parallel_rayon", block_size),
            block_size,
            |b, _| {
                b.iter(|| {
                    par_processor.process_parallel(&input, &mut output);
                    black_box(&output);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark cache-line alignment impact (Phase 4.1)
fn benchmark_cache_alignment(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_alignment");

    const SIZE: usize = 4096;

    // Unaligned buffer (standard Vec)
    let mut unaligned = vec![0.0f32; SIZE];

    // Cache-line aligned buffer
    let mut aligned = AlignedBuffer::new(SIZE);

    group.throughput(Throughput::Bytes((SIZE * 4) as u64));

    // Benchmark unaligned memory access
    group.bench_function("unaligned_access", |b| {
        b.iter(|| {
            // Simulate processing with potential false sharing
            for i in 0..SIZE {
                unaligned[i] = unaligned[i] * 0.5 + 0.25;
            }
            black_box(&unaligned);
        });
    });

    // Benchmark aligned memory access
    group.bench_function("aligned_access", |b| {
        b.iter(|| {
            let slice = aligned.as_mut_slice();
            for i in 0..SIZE {
                slice[i] = slice[i] * 0.5 + 0.25;
            }
            black_box(slice);
        });
    });

    // Multi-threaded access benchmark
    group.bench_function("aligned_parallel_access", |b| {
        use rayon::prelude::*;
        b.iter(|| {
            let slice = aligned.as_mut_slice();
            slice.par_chunks_mut(64).for_each(|chunk| {
                for sample in chunk {
                    *sample = *sample * 0.5 + 0.25;
                }
            });
            black_box(slice);
        });
    });

    group.finish();
}

/// Benchmark zero-copy pipeline (Phase 4.2)
fn benchmark_zero_copy_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("zero_copy_pipeline");

    const BLOCK_SIZE: usize = 1024;
    const NUM_BANDS: usize = 8;
    const SAMPLE_RATE: f32 = 44100.0;
    const FFT_SIZE: usize = 2048;

    let context = AudioContext::default();
    let mut analyser = context.create_analyser();
    analyser.set_fft_size(FFT_SIZE);

    let input = vec![0.5f32; BLOCK_SIZE];
    let mut output = vec![0.0f32; BLOCK_SIZE];

    // Traditional pipeline with multiple allocations
    let mut traditional_spectrum = OptimizedSpectrumProcessor::new(FFT_SIZE);
    let mut traditional_eq = OptimizedEqProcessor::new(NUM_BANDS, SAMPLE_RATE);
    traditional_eq.prepare(BLOCK_SIZE);

    // Zero-copy pipeline
    let mut zero_copy = ZeroCopyAudioPipeline::new(BLOCK_SIZE, NUM_BANDS, SAMPLE_RATE, FFT_SIZE);

    group.throughput(Throughput::Elements(BLOCK_SIZE as u64));

    // Benchmark traditional pipeline
    group.bench_function("traditional_with_copies", |b| {
        b.iter(|| {
            // Copy input to working buffer (allocation 1)
            let mut working = input.clone();

            // Process EQ
            traditional_eq.process(&working, &mut output);

            // Copy for spectrum analysis (allocation 2)
            let spectrum = traditional_spectrum.process_spectrum(&mut analyser);

            black_box(&output);
            black_box(spectrum);
        });
    });

    // Benchmark zero-copy pipeline
    group.bench_function("zero_copy_optimized", |b| {
        b.iter(|| {
            let spectrum = zero_copy.process_zero_copy(&input, &mut output, &mut analyser);
            black_box(&output);
            black_box(spectrum);
        });
    });

    group.finish();
}

/// Memory bandwidth benchmark
fn benchmark_memory_bandwidth(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_bandwidth");

    // Test different data sizes to see cache effects
    for size in &[1024, 4096, 16384, 65536, 262144] {
        let src = vec![0.5f32; *size];
        let mut dst = vec![0.0f32; *size];

        group.throughput(Throughput::Bytes((*size * 4) as u64));

        // Benchmark traditional copy
        group.bench_with_input(BenchmarkId::new("vec_copy", size), size, |b, &size| {
            b.iter(|| {
                dst.copy_from_slice(&src[..size]);
                black_box(&dst);
            });
        });

        // Benchmark unsafe memcpy
        group.bench_with_input(BenchmarkId::new("unsafe_memcpy", size), size, |b, &size| {
            b.iter(|| {
                unsafe {
                    std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), size);
                }
                black_box(&dst);
            });
        });

        // Benchmark in-place processing (zero-copy)
        group.bench_with_input(
            BenchmarkId::new("in_place_processing", size),
            size,
            |b, &size| {
                let mut data = src.clone();
                b.iter(|| {
                    // Process in-place - no copy needed
                    for i in 0..size {
                        data[i] *= 0.5;
                    }
                    black_box(&data);
                });
            },
        );
    }

    group.finish();
}

/// Comprehensive end-to-end benchmark
fn benchmark_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");

    const BLOCK_SIZE: usize = 1024;
    const NUM_BANDS: usize = 8;
    const SAMPLE_RATE: f32 = 44100.0;
    const FFT_SIZE: usize = 2048;
    const NUM_ITERATIONS: usize = 100;

    let context = AudioContext::default();
    let mut analyser = context.create_analyser();
    analyser.set_fft_size(FFT_SIZE);

    let input = vec![0.5f32; BLOCK_SIZE];
    let mut output = vec![0.0f32; BLOCK_SIZE];

    // Setup both pipelines
    let mut traditional_spectrum = OptimizedSpectrumProcessor::new(FFT_SIZE);
    let mut traditional_eq = OptimizedEqProcessor::new(NUM_BANDS, SAMPLE_RATE);
    traditional_eq.prepare(BLOCK_SIZE);

    let mut optimized_pipeline =
        ZeroCopyAudioPipeline::new(BLOCK_SIZE, NUM_BANDS, SAMPLE_RATE, FFT_SIZE);

    group.throughput(Throughput::Elements((BLOCK_SIZE * NUM_ITERATIONS) as u64));

    // Benchmark complete audio frame processing - Traditional
    group.bench_function("traditional_complete", |b| {
        b.iter(|| {
            for _ in 0..NUM_ITERATIONS {
                let mut working = input.clone();
                traditional_eq.process(&working, &mut output);
                let spectrum = traditional_spectrum.process_spectrum(&mut analyser);
                black_box(&output);
                black_box(spectrum);
            }
        });
    });

    // Benchmark complete audio frame processing - Optimized
    group.bench_function("optimized_complete", |b| {
        b.iter(|| {
            for _ in 0..NUM_ITERATIONS {
                let spectrum =
                    optimized_pipeline.process_zero_copy(&input, &mut output, &mut analyser);
                black_box(&output);
                black_box(spectrum);
            }
        });
    });

    // Print statistics after benchmark
    println!("\n=== Pipeline Statistics ===");
    println!("{}", optimized_pipeline.stats());

    group.finish();
}

criterion_group!(
    benches,
    benchmark_buffer_pool,
    benchmark_spectrum_processing,
    benchmark_parallel_eq,
    benchmark_cache_alignment,
    benchmark_zero_copy_pipeline,
    benchmark_memory_bandwidth,
    benchmark_end_to_end,
);

criterion_main!(benches);
