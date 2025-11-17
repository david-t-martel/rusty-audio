use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;
use web_audio_api::context::{AudioContext, BaseAudioContext, OfflineAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};
use web_audio_api::AudioBuffer;

const SAMPLE_RATE: f32 = 48000.0;
const RENDER_SIZE: usize = 128; // RENDER_QUANTUM_SIZE

/// Generate a test sine wave buffer
fn generate_test_buffer(ctx: &OfflineAudioContext, duration: f64, frequency: f64) -> AudioBuffer {
    let length = (SAMPLE_RATE as f64 * duration) as usize;
    let mut buffer = ctx.create_buffer(2, length, SAMPLE_RATE);

    for channel in 0..2 {
        let mut data = vec![0.0; length];
        for (i, sample) in data.iter_mut().enumerate() {
            *sample = ((i as f64 * frequency * 2.0 * std::f64::consts::PI / SAMPLE_RATE as f64)
                .sin()
                * 0.5) as f32;
        }
        buffer.copy_to_channel(&data, channel).unwrap();
    }

    buffer
}

/// Benchmark basic audio context creation
fn bench_audio_context_creation(c: &mut Criterion) {
    c.bench_function("audio_context_creation", |b| {
        b.iter(|| {
            let _context = AudioContext::default();
        });
    });
}

/// Benchmark offline rendering performance
fn bench_offline_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("offline_rendering");

    for duration in [1, 5, 10].iter() {
        let samples = SAMPLE_RATE as usize * duration;
        group.bench_with_input(
            BenchmarkId::new("duration", duration),
            &samples,
            |b, &samples| {
                b.iter(|| {
                    let mut ctx = OfflineAudioContext::new(2, samples, SAMPLE_RATE);
                    let mut osc = ctx.create_oscillator();
                    osc.connect(&ctx.destination());
                    osc.start();
                    ctx.start_rendering_sync();
                });
            },
        );
    }
    group.finish();
}

/// Benchmark buffer source playback
fn bench_buffer_playback(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_playback");

    for &buffer_size in &[SAMPLE_RATE as usize, SAMPLE_RATE as usize * 5] {
        group.bench_with_input(
            BenchmarkId::new("buffer_size", buffer_size),
            &buffer_size,
            |b, &buffer_size| {
                let ctx = OfflineAudioContext::new(2, buffer_size * 2, SAMPLE_RATE);
                let buffer =
                    generate_test_buffer(&ctx, buffer_size as f64 / SAMPLE_RATE as f64, 440.0);

                b.iter(|| {
                    let mut source = ctx.create_buffer_source();
                    source.set_buffer(black_box(buffer.clone()));
                    source.connect(&ctx.destination());
                    source.start();
                });
            },
        );
    }
    group.finish();
}

/// Benchmark EQ band processing
fn bench_eq_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("eq_processing");

    for &num_bands in &[4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::new("num_bands", num_bands),
            &num_bands,
            |b, &num_bands| {
                let samples = SAMPLE_RATE as usize * 2;
                let mut ctx = OfflineAudioContext::new(2, samples, SAMPLE_RATE);
                let buffer = generate_test_buffer(&ctx, 2.0, 440.0);

                b.iter(|| {
                    let mut source = ctx.create_buffer_source();
                    source.set_buffer(buffer.clone());

                    let mut previous_node: &dyn AudioNode = &source;
                    for i in 0..num_bands {
                        let mut band = ctx.create_biquad_filter();
                        band.frequency().set_value(60.0 * 2.0_f32.powi(i));
                        band.q().set_value(1.0);
                        band.gain().set_value(0.0);
                        previous_node.connect(&band);
                        previous_node = &band;
                    }
                    previous_node.connect(&ctx.destination());

                    source.start();
                    ctx.start_rendering_sync();
                });
            },
        );
    }
    group.finish();
}

/// Benchmark FFT analysis (analyser node)
fn bench_fft_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_analysis");

    for &fft_size in &[256, 512, 1024, 2048] {
        group.bench_with_input(
            BenchmarkId::new("fft_size", fft_size),
            &fft_size,
            |b, &fft_size| {
                let samples = SAMPLE_RATE as usize;
                let mut ctx = OfflineAudioContext::new(2, samples, SAMPLE_RATE);
                let buffer = generate_test_buffer(&ctx, 1.0, 440.0);

                b.iter(|| {
                    let mut analyser = ctx.create_analyser();
                    analyser.set_fft_size(fft_size);

                    let mut source = ctx.create_buffer_source();
                    source.set_buffer(buffer.clone());
                    source.connect(&analyser);
                    analyser.connect(&ctx.destination());
                    source.start();

                    // Simulate getting frequency data
                    let mut frequency_data = vec![0.0; analyser.frequency_bin_count()];
                    analyser.get_float_frequency_data(&mut frequency_data);

                    ctx.start_rendering_sync();
                });
            },
        );
    }
    group.finish();
}

/// Benchmark gain node processing
fn bench_gain_processing(c: &mut Criterion) {
    let samples = SAMPLE_RATE as usize * 2;

    c.bench_function("gain_processing", |b| {
        let mut ctx = OfflineAudioContext::new(2, samples, SAMPLE_RATE);
        let buffer = generate_test_buffer(&ctx, 2.0, 440.0);

        b.iter(|| {
            let mut source = ctx.create_buffer_source();
            source.set_buffer(buffer.clone());

            let gain = ctx.create_gain();
            gain.gain().set_value(0.5);

            source.connect(&gain);
            gain.connect(&ctx.destination());
            source.start();

            ctx.start_rendering_sync();
        });
    });
}

/// Benchmark audio graph complexity
fn bench_complex_audio_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_audio_graph");

    group.bench_function("full_pipeline", |b| {
        let samples = SAMPLE_RATE as usize;
        let mut ctx = OfflineAudioContext::new(2, samples, SAMPLE_RATE);
        let buffer = generate_test_buffer(&ctx, 1.0, 440.0);

        b.iter(|| {
            // Source
            let mut source = ctx.create_buffer_source();
            source.set_buffer(buffer.clone());

            // Gain
            let gain = ctx.create_gain();
            gain.gain().set_value(0.7);

            // EQ bands (8 bands)
            let mut eq_bands = Vec::new();
            for i in 0..8 {
                let mut band = ctx.create_biquad_filter();
                band.frequency().set_value(60.0 * 2.0_f32.powi(i));
                band.q().set_value(1.0);
                band.gain().set_value(0.0);
                eq_bands.push(band);
            }

            // Analyser
            let mut analyser = ctx.create_analyser();
            analyser.set_fft_size(1024);

            // Connect the graph
            source.connect(&gain);
            let mut previous_node: &dyn AudioNode = &gain;
            for band in &eq_bands {
                previous_node.connect(band);
                previous_node = band;
            }
            previous_node.connect(&analyser);
            analyser.connect(&ctx.destination());

            source.start();
            ctx.start_rendering_sync();
        });
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    group.bench_function("buffer_allocation", |b| {
        let ctx = OfflineAudioContext::new(2, SAMPLE_RATE as usize, SAMPLE_RATE);

        b.iter(|| {
            let _buffer = ctx.create_buffer(2, black_box(SAMPLE_RATE as usize), SAMPLE_RATE);
        });
    });

    group.bench_function("spectrum_vector_allocation", |b| {
        let sizes = vec![256, 512, 1024, 2048];
        let mut index = 0;

        b.iter(|| {
            let size = sizes[index % sizes.len()];
            let _spectrum: Vec<f32> = vec![0.0; black_box(size)];
            index += 1;
        });
    });

    group.finish();
}

/// Benchmark render quantum processing
fn bench_render_quantum(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_quantum");

    group.bench_function("quantum_128_samples", |b| {
        let samples = RENDER_SIZE * 100; // Process 100 quantum blocks
        let mut ctx = OfflineAudioContext::new(2, samples, SAMPLE_RATE);
        let mut osc = ctx.create_oscillator();
        osc.connect(&ctx.destination());
        osc.start();

        b.iter(|| {
            ctx.start_rendering_sync();
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));
    targets =
        bench_audio_context_creation,
        bench_offline_rendering,
        bench_buffer_playback,
        bench_eq_processing,
        bench_fft_analysis,
        bench_gain_processing,
        bench_complex_audio_graph,
        bench_memory_allocation,
        bench_render_quantum
}

criterion_main!(benches);
