use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use rustfft::{FftPlanner, num_complex::Complex32};
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::f32::consts::PI;

const SAMPLE_RATE: f32 = 48000.0;
const TEST_DURATION: f32 = 1.0;

/// Pure sine wave generator for benchmarking
struct SineGenerator {
    frequency: f32,
    amplitude: f32,
    phase: f32,
}

impl SineGenerator {
    fn new(frequency: f32) -> Self {
        Self {
            frequency,
            amplitude: 1.0,
            phase: 0.0,
        }
    }

    fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }

    fn generate(&self, duration: f32, sample_rate: f32) -> Vec<f32> {
        let num_samples = (duration * sample_rate) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            let sample = self.amplitude * (2.0 * PI * self.frequency * t + self.phase).sin();
            samples.push(sample);
        }

        samples
    }
}

/// White noise generator for benchmarking
struct NoiseGenerator {
    amplitude: f32,
    seed: u64,
}

impl NoiseGenerator {
    fn new() -> Self {
        Self {
            amplitude: 1.0,
            seed: 42,
        }
    }

    fn generate(&self, duration: f32, sample_rate: f32) -> Vec<f32> {
        let num_samples = (duration * sample_rate) as usize;
        let mut samples = Vec::with_capacity(num_samples);
        let mut rng = StdRng::seed_from_u64(self.seed);

        for _ in 0..num_samples {
            let sample = self.amplitude * (rng.gen::<f32>() * 2.0 - 1.0);
            samples.push(sample);
        }

        samples
    }
}

/// FFT Analyzer for frequency domain analysis
struct FftAnalyzer {
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    fft_size: usize,
    window: Vec<f32>,
}

impl FftAnalyzer {
    fn new(fft_size: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);

        // Generate Hann window
        let window: Vec<f32> = (0..fft_size)
            .map(|i| {
                let factor = 2.0 * PI * i as f32 / (fft_size - 1) as f32;
                0.5 * (1.0 - factor.cos())
            })
            .collect();

        Self {
            fft,
            fft_size,
            window,
        }
    }

    fn analyze(&self, samples: &[f32]) -> Vec<f32> {
        if samples.len() < self.fft_size {
            return vec![0.0; self.fft_size / 2];
        }

        // Apply window and convert to complex
        let mut buffer: Vec<Complex32> = samples[..self.fft_size]
            .iter()
            .zip(&self.window)
            .map(|(&sample, &window)| Complex32::new(sample * window, 0.0))
            .collect();

        // Perform FFT
        self.fft.process(&mut buffer);

        // Calculate magnitude spectrum
        buffer[..self.fft_size / 2]
            .iter()
            .map(|c| c.norm())
            .collect()
    }
}

/// Calculate RMS (Root Mean Square) of a signal
fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

/// Calculate THD (Total Harmonic Distortion)
fn calculate_thd(spectrum: &[f32], fundamental_bin: usize, sample_rate: f32, fft_size: usize) -> f32 {
    if fundamental_bin >= spectrum.len() {
        return 0.0;
    }

    let fundamental_amplitude = spectrum[fundamental_bin];
    if fundamental_amplitude == 0.0 {
        return f32::INFINITY;
    }

    let mut harmonic_sum_squares = 0.0;

    // Sum harmonics (2nd through 10th)
    for harmonic in 2..=10 {
        let harmonic_bin = fundamental_bin * harmonic;
        if harmonic_bin < spectrum.len() {
            let harmonic_amplitude = spectrum[harmonic_bin];
            harmonic_sum_squares += harmonic_amplitude * harmonic_amplitude;
        }
    }

    (harmonic_sum_squares.sqrt() / fundamental_amplitude) * 100.0
}

/// Calculate Signal-to-Noise Ratio in dB
fn calculate_snr_db(signal_power: f32, noise_power: f32) -> f32 {
    if noise_power == 0.0 {
        f32::INFINITY
    } else {
        10.0 * (signal_power / noise_power).log10()
    }
}

/// Benchmark sine wave generation
fn bench_sine_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sine_generation");

    for &frequency in &[100.0, 440.0, 1000.0, 8000.0] {
        group.bench_with_input(
            BenchmarkId::new("frequency", frequency as u32),
            &frequency,
            |b, &freq| {
                let generator = SineGenerator::new(freq);
                b.iter(|| {
                    let _samples = generator.generate(black_box(TEST_DURATION), black_box(SAMPLE_RATE));
                });
            },
        );
    }
    group.finish();
}

/// Benchmark RMS calculation
fn bench_rms_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("rms_calculation");

    for &size in &[1024, 4096, 16384, 65536] {
        let generator = SineGenerator::new(1000.0);
        let samples = generator.generate(size as f32 / SAMPLE_RATE, SAMPLE_RATE);

        group.bench_with_input(
            BenchmarkId::new("sample_count", size),
            &samples,
            |b, samples| {
                b.iter(|| {
                    let _rms = calculate_rms(black_box(samples));
                });
            },
        );
    }
    group.finish();
}

/// Benchmark FFT analysis
fn bench_fft_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_analysis");

    for &fft_size in &[512, 1024, 2048, 4096] {
        let analyzer = FftAnalyzer::new(fft_size);
        let generator = SineGenerator::new(1000.0);
        let samples = generator.generate(TEST_DURATION, SAMPLE_RATE);

        group.bench_with_input(
            BenchmarkId::new("fft_size", fft_size),
            &(analyzer, samples),
            |b, (analyzer, samples)| {
                b.iter(|| {
                    let _spectrum = analyzer.analyze(black_box(samples));
                });
            },
        );
    }
    group.finish();
}

/// Benchmark THD calculation
fn bench_thd_calculation(c: &mut Criterion) {
    let fft_size = 2048;
    let analyzer = FftAnalyzer::new(fft_size);

    // Generate signal with harmonics
    let fundamental_freq = 1000.0;
    let fundamental = SineGenerator::new(fundamental_freq).with_amplitude(1.0);
    let second_harmonic = SineGenerator::new(fundamental_freq * 2.0).with_amplitude(0.1);
    let third_harmonic = SineGenerator::new(fundamental_freq * 3.0).with_amplitude(0.05);

    let mut samples = fundamental.generate(TEST_DURATION, SAMPLE_RATE);
    let samples2 = second_harmonic.generate(TEST_DURATION, SAMPLE_RATE);
    let samples3 = third_harmonic.generate(TEST_DURATION, SAMPLE_RATE);

    // Mix harmonics
    for i in 0..samples.len() {
        if i < samples2.len() {
            samples[i] += samples2[i];
        }
        if i < samples3.len() {
            samples[i] += samples3[i];
        }
    }

    let spectrum = analyzer.analyze(&samples);
    let fundamental_bin = ((fundamental_freq * fft_size as f32) / SAMPLE_RATE) as usize;

    c.bench_function("thd_calculation", |b| {
        b.iter(|| {
            let _thd = calculate_thd(
                black_box(&spectrum),
                black_box(fundamental_bin),
                black_box(SAMPLE_RATE),
                black_box(fft_size)
            );
        });
    });
}

/// Benchmark noise generation and analysis
fn bench_noise_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("noise_analysis");

    let noise_generator = NoiseGenerator::new();
    let noise_samples = noise_generator.generate(TEST_DURATION, SAMPLE_RATE);
    let signal_generator = SineGenerator::new(1000.0).with_amplitude(0.5);
    let signal_samples = signal_generator.generate(TEST_DURATION, SAMPLE_RATE);

    // Mix signal with noise
    let mut mixed_samples = signal_samples.clone();
    for i in 0..mixed_samples.len() {
        if i < noise_samples.len() {
            mixed_samples[i] += noise_samples[i] * 0.1; // 10% noise
        }
    }

    group.bench_function("signal_power", |b| {
        b.iter(|| {
            let signal_rms = calculate_rms(black_box(&signal_samples));
            let _signal_power = signal_rms * signal_rms;
        });
    });

    group.bench_function("noise_power", |b| {
        b.iter(|| {
            let noise_rms = calculate_rms(black_box(&noise_samples));
            let _noise_power = noise_rms * noise_rms;
        });
    });

    group.bench_function("snr_calculation", |b| {
        let signal_rms = calculate_rms(&signal_samples);
        let noise_rms = calculate_rms(&noise_samples) * 0.1; // Scaled noise
        let signal_power = signal_rms * signal_rms;
        let noise_power = noise_rms * noise_rms;

        b.iter(|| {
            let _snr = calculate_snr_db(black_box(signal_power), black_box(noise_power));
        });
    });

    group.finish();
}

/// Benchmark frequency response analysis
fn bench_frequency_response(c: &mut Criterion) {
    let mut group = c.benchmark_group("frequency_response");

    let fft_size = 2048;
    let analyzer = FftAnalyzer::new(fft_size);

    // Test multiple frequencies
    let test_frequencies = vec![100.0, 440.0, 1000.0, 2000.0, 4000.0, 8000.0];
    let test_signals: Vec<_> = test_frequencies.iter()
        .map(|&freq| {
            let generator = SineGenerator::new(freq).with_amplitude(1.0);
            generator.generate(TEST_DURATION, SAMPLE_RATE)
        })
        .collect();

    group.bench_function("multi_frequency_analysis", |b| {
        b.iter(|| {
            let mut response = Vec::new();
            for samples in black_box(&test_signals) {
                let spectrum = analyzer.analyze(samples);
                // Find peak frequency
                let max_bin = spectrum.iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                let peak_freq = (max_bin as f32 * SAMPLE_RATE) / fft_size as f32;
                let peak_magnitude = spectrum[max_bin];
                response.push((peak_freq, peak_magnitude));
            }
            let _response = response;
        });
    });

    group.finish();
}

/// Benchmark audio quality metrics calculation
fn bench_audio_quality_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("quality_metrics");

    let fft_size = 2048;
    let analyzer = FftAnalyzer::new(fft_size);

    // Generate test signal with known characteristics
    let fundamental_freq = 1000.0;
    let generator = SineGenerator::new(fundamental_freq).with_amplitude(1.0);
    let samples = generator.generate(TEST_DURATION, SAMPLE_RATE);
    let spectrum = analyzer.analyze(&samples);
    let fundamental_bin = ((fundamental_freq * fft_size as f32) / SAMPLE_RATE) as usize;

    group.bench_function("comprehensive_analysis", |b| {
        b.iter(|| {
            // RMS calculation
            let rms = calculate_rms(black_box(&samples));

            // Peak detection
            let peak = samples.iter().map(|&x| x.abs()).fold(0.0, f32::max);

            // THD calculation
            let thd = calculate_thd(
                black_box(&spectrum),
                black_box(fundamental_bin),
                black_box(SAMPLE_RATE),
                black_box(fft_size)
            );

            // Dynamic range
            let _dynamic_range = 20.0 * (peak / rms).log10();

            let _metrics = (rms, peak, thd);
        });
    });

    group.finish();
}

criterion_group! {
    name = audio_quality_benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));
    targets =
        bench_sine_generation,
        bench_rms_calculation,
        bench_fft_analysis,
        bench_thd_calculation,
        bench_noise_analysis,
        bench_frequency_response,
        bench_audio_quality_metrics
}

criterion_main!(audio_quality_benches);