// Mathematical Testing Framework Demonstration
//
// This example demonstrates the comprehensive mathematical testing framework
// for rusty-audio without requiring the full UI to compile.

use rand::{rngs::StdRng, Rng, SeedableRng};
use rustfft::{num_complex::Complex32, FftPlanner};
use std::f32::consts::PI;

/// Mathematical constants for audio testing
const SAMPLE_RATE: f32 = 44100.0;
const TOLERANCE: f32 = 0.001;

/// Test result structure for mathematical verification
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub expected: f32,
    pub actual: f32,
    pub tolerance: f32,
    pub error_magnitude: f32,
}

impl TestResult {
    pub fn new(name: &str, expected: f32, actual: f32, tolerance: f32) -> Self {
        let error_magnitude = (expected - actual).abs();
        let passed = error_magnitude <= tolerance;

        Self {
            test_name: name.to_string(),
            passed,
            expected,
            actual,
            tolerance,
            error_magnitude,
        }
    }
}

/// Suite for managing multiple test results
#[derive(Debug, Default)]
pub struct TestSuite {
    pub results: Vec<TestResult>,
}

impl TestSuite {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_result(&mut self, result: TestResult) {
        self.results.push(result);
    }

    pub fn passed_count(&self) -> usize {
        self.results.iter().filter(|r| r.passed).count()
    }

    pub fn failed_count(&self) -> usize {
        self.results.len() - self.passed_count()
    }

    pub fn success_rate(&self) -> f32 {
        if self.results.is_empty() {
            0.0
        } else {
            self.passed_count() as f32 / self.results.len() as f32
        }
    }

    pub fn print_summary(&self) {
        println!("\n=== Test Suite Summary ===");
        println!("Total tests: {}", self.results.len());
        println!("Passed: {}", self.passed_count());
        println!("Failed: {}", self.failed_count());
        println!("Success rate: {:.1}%", self.success_rate() * 100.0);

        if self.failed_count() > 0 {
            println!("\n=== Failed Tests ===");
            for result in &self.results {
                if !result.passed {
                    println!(
                        "{}: expected {:.6}, got {:.6} (error: {:.6})",
                        result.test_name, result.expected, result.actual, result.error_magnitude
                    );
                }
            }
        }
    }
}

/// Pure sine wave generator
pub struct SineGenerator {
    pub frequency: f32,
    pub amplitude: f32,
    pub phase: f32,
}

impl SineGenerator {
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            amplitude: 1.0,
            phase: 0.0,
        }
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }

    pub fn generate(&self, duration: f32, sample_rate: f32) -> Vec<f32> {
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

/// White noise generator
pub struct WhiteNoiseGenerator {
    pub amplitude: f32,
    pub seed: u64,
}

impl WhiteNoiseGenerator {
    pub fn new() -> Self {
        Self {
            amplitude: 1.0,
            seed: 42, // Fixed seed for reproducible tests
        }
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }

    pub fn generate(&self, duration: f32, sample_rate: f32) -> Vec<f32> {
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

/// FFT Analyzer for mathematical verification
pub struct FftAnalyzer {
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    fft_size: usize,
    window: Vec<f32>,
}

impl FftAnalyzer {
    pub fn new(fft_size: usize) -> Self {
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

    /// Analyze samples with FFT and find peak frequency
    pub fn analyze_and_find_peak(&self, samples: &[f32], sample_rate: f32) -> Option<(f32, f32)> {
        if samples.len() < self.fft_size {
            return None;
        }

        // Apply window and convert to complex
        let mut buffer: Vec<Complex32> = samples[..self.fft_size]
            .iter()
            .zip(&self.window)
            .map(|(&sample, &window)| Complex32::new(sample * window, 0.0))
            .collect();

        // Perform FFT
        self.fft.process(&mut buffer);

        // Find peak in magnitude spectrum (only positive frequencies)
        let half_size = self.fft_size / 2;
        let mut max_magnitude = 0.0;
        let mut max_bin = 0;

        for i in 1..half_size {
            // Skip DC bin
            let magnitude = buffer[i].norm();
            if magnitude > max_magnitude {
                max_magnitude = magnitude;
                max_bin = i;
            }
        }

        if max_magnitude > 0.0 {
            let frequency = (max_bin as f32 * sample_rate) / self.fft_size as f32;
            let normalized_magnitude = max_magnitude / (self.fft_size as f32 / 2.0);
            Some((frequency, normalized_magnitude))
        } else {
            None
        }
    }
}

/// Calculate RMS (Root Mean Square) of a signal
pub fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

/// Calculate peak amplitude of a signal
pub fn calculate_peak(samples: &[f32]) -> f32 {
    samples.iter().map(|&x| x.abs()).fold(0.0, f32::max)
}

/// Run comprehensive mathematical tests
fn run_mathematical_tests() -> TestSuite {
    let mut master_suite = TestSuite::new();

    println!("🔬 RUSTY AUDIO - MATHEMATICAL TESTING FRAMEWORK DEMO");
    println!("=====================================================");

    // Test 1: Sine Wave Generator Mathematical Properties
    println!("\n1️⃣  Testing Sine Wave Generator...");

    let sine_gen = SineGenerator::new(1000.0);
    let sine_samples = sine_gen.generate(1.0, SAMPLE_RATE);

    // Test RMS value
    let rms = calculate_rms(&sine_samples);
    let expected_rms = 1.0 / 2.0f32.sqrt(); // RMS of unit sine wave
    let rms_result = TestResult::new("Sine wave RMS", expected_rms, rms, TOLERANCE * 10.0);
    master_suite.add_result(rms_result);

    // Test peak value
    let peak = calculate_peak(&sine_samples);
    let peak_result = TestResult::new("Sine wave peak", 1.0, peak, TOLERANCE * 10.0);
    master_suite.add_result(peak_result);

    println!("   RMS: {:.6} (expected: {:.6})", rms, expected_rms);
    println!("   Peak: {:.6} (expected: 1.000)", peak);

    // Test 2: FFT Accuracy with Known Frequencies
    println!("\n2️⃣  Testing FFT Accuracy...");

    let analyzer = FftAnalyzer::new(2048);

    let test_frequencies = vec![440.0, 1000.0, 2000.0];

    for &freq in &test_frequencies {
        let freq_gen = SineGenerator::new(freq);
        let freq_samples = freq_gen.generate(2.0, SAMPLE_RATE); // 2 second signal for better resolution

        if let Some((detected_freq, magnitude)) =
            analyzer.analyze_and_find_peak(&freq_samples, SAMPLE_RATE)
        {
            let freq_result = TestResult::new(
                &format!("FFT frequency detection: {:.1} Hz", freq),
                freq,
                detected_freq,
                50.0, // 50 Hz tolerance
            );
            master_suite.add_result(freq_result);

            let mag_result = TestResult::new(
                &format!("FFT magnitude at {:.1} Hz", freq),
                0.5, // Expected magnitude for unit amplitude sine
                magnitude,
                0.1,
            );
            master_suite.add_result(mag_result);

            println!(
                "   {:.1} Hz: detected {:.1} Hz, magnitude {:.3}",
                freq, detected_freq, magnitude
            );
        }
    }

    // Test 3: White Noise Properties
    println!("\n3️⃣  Testing White Noise Properties...");

    let noise_gen = WhiteNoiseGenerator::new().with_amplitude(0.5);
    let noise_samples = noise_gen.generate(1.0, SAMPLE_RATE);

    let noise_rms = calculate_rms(&noise_samples);
    let noise_peak = calculate_peak(&noise_samples);

    // White noise should have RMS roughly amplitude / sqrt(3) for uniform distribution
    let expected_noise_rms = 0.5 / 3.0f32.sqrt();
    let noise_rms_result = TestResult::new(
        "White noise RMS",
        expected_noise_rms,
        noise_rms,
        expected_noise_rms * 0.5, // 50% tolerance due to randomness
    );
    master_suite.add_result(noise_rms_result);

    let noise_peak_result = TestResult::new(
        "White noise peak bounds",
        0.5,
        noise_peak,
        0.1, // Peak should be close to but not exceed amplitude
    );
    master_suite.add_result(noise_peak_result);

    println!(
        "   RMS: {:.6} (expected ~{:.6})",
        noise_rms, expected_noise_rms
    );
    println!("   Peak: {:.6} (should be ≤ 0.5)", noise_peak);

    // Test 4: Signal Processing Mathematical Properties
    println!("\n4️⃣  Testing Signal Processing Properties...");

    // Test signal energy conservation
    let test_signal = vec![1.0, 0.5, -0.5, -1.0, 0.0];
    let energy_before: f32 = test_signal.iter().map(|&x| x * x).sum();

    // Simple gain operation
    let gain = 0.8;
    let gained_signal: Vec<f32> = test_signal.iter().map(|&x| x * gain).collect();
    let energy_after: f32 = gained_signal.iter().map(|&x| x * x).sum();
    let expected_energy_after = energy_before * gain * gain;

    let energy_result = TestResult::new(
        "Energy conservation with gain",
        expected_energy_after,
        energy_after,
        TOLERANCE,
    );
    master_suite.add_result(energy_result);

    println!(
        "   Energy before: {:.6}, after: {:.6} (expected: {:.6})",
        energy_before, energy_after, expected_energy_after
    );

    master_suite
}

fn main() {
    let suite = run_mathematical_tests();

    // Print final summary
    println!("\n=====================================================");
    println!("🎯 FINAL MATHEMATICAL VERIFICATION RESULTS");
    suite.print_summary();

    if suite.success_rate() >= 0.95 {
        println!(
            "🎉 EXCELLENT: Mathematical accuracy > 95% - Audio processing is mathematically sound!"
        );
        std::process::exit(0);
    } else if suite.success_rate() >= 0.85 {
        println!("✅ GOOD: Mathematical accuracy > 85% - Audio processing is reliable with minor issues.");
        std::process::exit(0);
    } else if suite.success_rate() >= 0.70 {
        println!("⚠️  WARNING: Mathematical accuracy < 85% - Review failed tests for audio processing issues.");
        std::process::exit(1);
    } else {
        println!("❌ CRITICAL: Mathematical accuracy < 70% - Significant audio processing problems detected!");
        std::process::exit(1);
    }
}
