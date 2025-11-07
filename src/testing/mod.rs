// Mathematical Testing Framework for Rusty Audio
//
// This module provides comprehensive testing utilities for audio processing,
// including signal generators, mathematical verification, and test suites.

pub mod signal_generators;
pub mod spectrum_analysis;
pub mod equalizer_tests;
pub mod integration_tests;
pub mod property_tests;
pub mod ui_tests;
pub mod visual_regression;
pub mod audio_feature_tests;

use web_audio_api::context::{AudioContext, BaseAudioContext};
use signal_generators::SignalGenerator;

/// Mathematical constants for audio testing
pub const SAMPLE_RATE: f32 = 44100.0;
pub const TEST_DURATION: f32 = 1.0; // 1 second
pub const TOLERANCE: f32 = 0.001;

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
                    println!("{}: expected {:.6}, got {:.6} (error: {:.6})",
                        result.test_name,
                        result.expected,
                        result.actual,
                        result.error_magnitude
                    );
                }
            }
        }
    }
}

/// Utility function to create test audio buffer
pub fn create_test_buffer(context: &AudioContext, samples: Vec<f32>) {
    let mut buffer = context.create_buffer(1, samples.len(), SAMPLE_RATE);
    buffer.copy_to_channel(&samples, 0);
    // Note: Buffer is auto-dropped, this is mainly for testing the buffer creation
}

/// Utility function to compare floating point values with tolerance
pub fn approx_equal(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}

/// Convert frequency to bin index for FFT analysis
pub fn freq_to_bin(frequency: f32, fft_size: usize, sample_rate: f32) -> usize {
    ((frequency * fft_size as f32) / sample_rate) as usize
}

/// Convert bin index to frequency for FFT analysis
pub fn bin_to_freq(bin: usize, fft_size: usize, sample_rate: f32) -> f32 {
    (bin as f32 * sample_rate) / fft_size as f32
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

/// Calculate signal-to-noise ratio in dB
pub fn calculate_snr_db(signal_rms: f32, noise_rms: f32) -> f32 {
    if noise_rms == 0.0 {
        f32::INFINITY
    } else {
        20.0 * (signal_rms / noise_rms).log10()
    }
}

/// Calculate Total Harmonic Distortion (THD)
pub fn calculate_thd(fundamental_amplitude: f32, harmonic_amplitudes: &[f32]) -> f32 {
    if fundamental_amplitude == 0.0 {
        return f32::INFINITY;
    }

    let harmonic_sum_squares: f32 = harmonic_amplitudes.iter()
        .map(|&h| h * h)
        .sum();

    (harmonic_sum_squares.sqrt() / fundamental_amplitude) * 100.0
}

/// Main test runner that executes all test suites
pub fn run_all_tests() -> TestSuite {
    let mut master_suite = TestSuite::new();

    println!("\n🔬 RUSTY AUDIO - COMPREHENSIVE MATHEMATICAL TESTING FRAMEWORK");
    println!("================================================================");

    // Test 1: Spectrum Analysis Tests
    println!("\n1️⃣  Running Spectrum Analysis Tests...");
    let mut spectrum_suite = spectrum_analysis::run_spectrum_tests();
    let spectrum_passed = spectrum_suite.passed_count();
    let spectrum_total = spectrum_suite.results.len();
    master_suite.results.append(&mut spectrum_suite.results);
    println!("   ✅ Spectrum Analysis: {}/{} tests passed", spectrum_passed, spectrum_total);

    // Test 2: Equalizer Tests
    println!("\n2️⃣  Running Equalizer Mathematical Tests...");
    let mut equalizer_suite = equalizer_tests::run_equalizer_tests();
    let eq_passed = equalizer_suite.passed_count();
    let eq_total = equalizer_suite.results.len();
    master_suite.results.append(&mut equalizer_suite.results);
    println!("   ✅ Equalizer Tests: {}/{} tests passed", eq_passed, eq_total);

    // Test 3: Property-Based Tests
    println!("\n3️⃣  Running Property-Based Tests...");
    let mut property_suite = property_tests::run_property_tests();
    let prop_passed = property_suite.passed_count();
    let prop_total = property_suite.results.len();
    master_suite.results.append(&mut property_suite.results);
    println!("   ✅ Property Tests: {}/{} tests passed", prop_passed, prop_total);

    // Test 4: Integration Tests
    println!("\n4️⃣  Running Audio Pipeline Integration Tests...");
    let mut integration_suite = integration_tests::run_integration_tests();
    let int_passed = integration_suite.passed_count();
    let int_total = integration_suite.results.len();
    master_suite.results.append(&mut integration_suite.results);
    println!("   ✅ Integration Tests: {}/{} tests passed", int_passed, int_total);

    // Print final summary
    println!("\n================================================================");
    println!("🎯 FINAL MATHEMATICAL VERIFICATION RESULTS");
    master_suite.print_summary();

    if master_suite.success_rate() >= 0.95 {
        println!("🎉 EXCELLENT: Mathematical accuracy > 95% - Audio processing is mathematically sound!");
    } else if master_suite.success_rate() >= 0.85 {
        println!("✅ GOOD: Mathematical accuracy > 85% - Audio processing is reliable with minor issues.");
    } else if master_suite.success_rate() >= 0.70 {
        println!("⚠️  WARNING: Mathematical accuracy < 85% - Review failed tests for audio processing issues.");
    } else {
        println!("❌ CRITICAL: Mathematical accuracy < 70% - Significant audio processing problems detected!");
    }

    println!("================================================================\n");

    master_suite
}

/// Quick test runner for essential functionality
pub fn run_quick_tests() -> TestSuite {
    let mut suite = TestSuite::new();

    println!("🚀 RUSTY AUDIO - QUICK MATHEMATICAL TESTS");
    println!("==========================================");

    // Test signal generators
    println!("Testing signal generators...");
    let generator = signal_generators::presets::sine_1khz();
    let samples = generator.generate(1.0, SAMPLE_RATE);

    let rms = calculate_rms(&samples);
    let expected_rms = 1.0 / 2.0f32.sqrt(); // RMS of unit sine wave
    let result = TestResult::new("Sine wave RMS", expected_rms, rms, TOLERANCE * 10.0);
    suite.add_result(result);

    // Test FFT analysis
    println!("Testing FFT analysis...");
    let analyzer = spectrum_analysis::FftAnalyzer::new(2048);
    let analysis = analyzer.analyze(&samples, SAMPLE_RATE);

    if let Some((detected_freq, _)) = analysis.find_peak() {
        let result = TestResult::new("FFT frequency detection", 1000.0, detected_freq, 50.0);
        suite.add_result(result);
    }

    suite.print_summary();
    suite
}

/// Test runner specifically for real-time performance validation
pub fn run_realtime_tests() -> TestSuite {
    println!("⚡ RUSTY AUDIO - REAL-TIME PERFORMANCE TESTS");
    println!("=============================================");

    let config = integration_tests::PipelineTestConfig {
        test_duration: 5.0, // 5 second test
        ..Default::default()
    };

    let integration = integration_tests::AudioPipelineIntegration::new(config);
    let suite = integration.test_realtime_performance();

    suite.print_summary();
    suite
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approx_equal() {
        assert!(approx_equal(1.0, 1.0001, 0.001));
        assert!(!approx_equal(1.0, 1.1, 0.01));
    }

    #[test]
    fn test_freq_to_bin() {
        let bin = freq_to_bin(1000.0, 2048, 44100.0);
        let expected = (1000.0 * 2048.0 / 44100.0) as usize;
        assert_eq!(bin, expected);
    }

    #[test]
    fn test_calculate_rms() {
        let samples = vec![1.0, -1.0, 1.0, -1.0];
        let rms = calculate_rms(&samples);
        assert!(approx_equal(rms, 1.0, 0.001));
    }

    #[test]
    fn test_calculate_peak() {
        let samples = vec![0.5, -0.8, 0.3, -0.9];
        let peak = calculate_peak(&samples);
        assert_eq!(peak, 0.9);
    }

    #[test]
    fn test_test_suite_operations() {
        let mut suite = TestSuite::new();

        let result1 = TestResult::new("Test 1", 1.0, 1.0, 0.1);
        let result2 = TestResult::new("Test 2", 2.0, 2.5, 0.1); // This will fail

        suite.add_result(result1);
        suite.add_result(result2);

        assert_eq!(suite.passed_count(), 1);
        assert_eq!(suite.failed_count(), 1);
        assert_eq!(suite.success_rate(), 0.5);
    }

    #[test]
    fn test_quick_tests_run() {
        let suite = run_quick_tests();
        // Should have at least some test results
        assert!(!suite.results.is_empty());
    }
}