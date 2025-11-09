// Comprehensive Unit Tests for Audio Processing Functions
// Mathematical verification with high precision testing

use approx::{assert_abs_diff_eq, assert_relative_eq};
use proptest::prelude::*;
use quickcheck::{QuickCheck, TestResult};
use quickcheck_macros::quickcheck;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rstest::{fixture, rstest};
use rustfft::{num_complex::Complex32, FftPlanner};
use statrs::statistics::{Data, Statistics};
use std::f64::consts::PI;
use test_case::test_case;

// Test constants with high precision
const SAMPLE_RATE: f32 = 48000.0;
const TOLERANCE: f32 = 1e-6;
const HIGH_PRECISION_TOLERANCE: f32 = 1e-9;
const AUDIO_TOLERANCE: f32 = 1e-3; // More lenient for audio processing
const FFT_SIZE: usize = 2048;
const TEST_DURATION: f32 = 1.0;

/// Mathematical test result with detailed analysis
#[derive(Debug, Clone)]
struct MathTestResult {
    test_name: String,
    passed: bool,
    expected: f64,
    actual: f64,
    tolerance: f64,
    error_magnitude: f64,
    relative_error: f64,
    significance: f64, // Statistical significance
}

impl MathTestResult {
    fn new(name: &str, expected: f64, actual: f64, tolerance: f64) -> Self {
        let error_magnitude = (expected - actual).abs();
        let relative_error = if expected != 0.0 {
            error_magnitude / expected.abs()
        } else {
            error_magnitude
        };
        let passed = error_magnitude <= tolerance;
        let significance = if tolerance != 0.0 {
            error_magnitude / tolerance
        } else {
            f64::INFINITY
        };

        Self {
            test_name: name.to_string(),
            passed,
            expected,
            actual,
            tolerance,
            error_magnitude,
            relative_error,
            significance,
        }
    }
}

/// Enhanced signal generator with mathematical precision
#[derive(Debug, Clone)]
struct PrecisionSineGenerator {
    frequency: f64,
    amplitude: f64,
    phase: f64,
    sample_rate: f64,
}

impl PrecisionSineGenerator {
    fn new(frequency: f64, sample_rate: f64) -> Self {
        Self {
            frequency,
            amplitude: 1.0,
            phase: 0.0,
            sample_rate,
        }
    }

    fn with_amplitude(mut self, amplitude: f64) -> Self {
        self.amplitude = amplitude;
        self
    }

    fn with_phase(mut self, phase: f64) -> Self {
        self.phase = phase;
        self
    }

    fn generate_f64(&self, duration: f64) -> Vec<f64> {
        let num_samples = (duration * self.sample_rate) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f64 / self.sample_rate;
            let sample = self.amplitude * (2.0 * PI * self.frequency * t + self.phase).sin();
            samples.push(sample);
        }

        samples
    }

    fn generate_f32(&self, duration: f64) -> Vec<f32> {
        self.generate_f64(duration)
            .into_iter()
            .map(|x| x as f32)
            .collect()
    }
}

/// High-precision FFT analyzer
struct PrecisionFftAnalyzer {
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    fft_size: usize,
    window: Vec<f64>,
    normalization_factor: f64,
}

impl PrecisionFftAnalyzer {
    fn new(fft_size: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);

        // Blackman-Harris window for better precision
        let window: Vec<f64> = (0..fft_size)
            .map(|i| {
                let n = i as f64;
                let N = fft_size as f64;
                let a0 = 0.35875;
                let a1 = 0.48829;
                let a2 = 0.14128;
                let a3 = 0.01168;

                a0 - a1 * (2.0 * PI * n / (N - 1.0)).cos() + a2 * (4.0 * PI * n / (N - 1.0)).cos()
                    - a3 * (6.0 * PI * n / (N - 1.0)).cos()
            })
            .collect();

        let normalization_factor = window.iter().sum::<f64>() / fft_size as f64;

        Self {
            fft,
            fft_size,
            window,
            normalization_factor,
        }
    }

    fn analyze_with_precision(&self, samples: &[f32]) -> Vec<f64> {
        if samples.len() < self.fft_size {
            return vec![0.0; self.fft_size / 2];
        }

        // Apply window with high precision
        let mut buffer: Vec<Complex32> = samples[..self.fft_size]
            .iter()
            .zip(&self.window)
            .map(|(&sample, &window)| {
                let windowed = (sample as f64 * window) as f32;
                Complex32::new(windowed, 0.0)
            })
            .collect();

        // Perform FFT
        self.fft.process(&mut buffer);

        // Calculate magnitude spectrum with normalization
        buffer[..self.fft_size / 2]
            .iter()
            .map(|c| (c.norm() as f64) / self.normalization_factor)
            .collect()
    }

    fn find_peak_frequency(&self, spectrum: &[f64], sample_rate: f64) -> Option<(f64, f64)> {
        if spectrum.is_empty() {
            return None;
        }

        let (max_bin, &max_magnitude) = spectrum
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())?;

        if max_magnitude > 0.0 {
            let frequency = (max_bin as f64 * sample_rate) / (self.fft_size as f64);
            Some((frequency, max_magnitude))
        } else {
            None
        }
    }
}

/// Mathematical utility functions with high precision
mod math_utils {
    use super::*;

    pub fn calculate_rms_f64(samples: &[f64]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_squares: f64 = samples.iter().map(|&x| x * x).sum();
        (sum_squares / samples.len() as f64).sqrt()
    }

    pub fn calculate_rms_f32(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    pub fn calculate_peak_f64(samples: &[f64]) -> f64 {
        samples.iter().map(|&x| x.abs()).fold(0.0, f64::max)
    }

    pub fn calculate_peak_f32(samples: &[f32]) -> f32 {
        samples.iter().map(|&x| x.abs()).fold(0.0, f32::max)
    }

    pub fn calculate_snr_db(signal_power: f64, noise_power: f64) -> f64 {
        if noise_power == 0.0 {
            f64::INFINITY
        } else {
            10.0 * (signal_power / noise_power).log10()
        }
    }

    pub fn calculate_thd_plus_n(fundamental: f64, harmonics: &[f64], noise_power: f64) -> f64 {
        if fundamental == 0.0 {
            return f64::INFINITY;
        }

        let harmonic_power: f64 = harmonics.iter().map(|&h| h * h).sum();
        let total_distortion_plus_noise = harmonic_power + noise_power;
        (total_distortion_plus_noise.sqrt() / fundamental) * 100.0
    }

    pub fn calculate_sinad_db(signal_power: f64, noise_plus_distortion: f64) -> f64 {
        if noise_plus_distortion == 0.0 {
            f64::INFINITY
        } else {
            10.0 * (signal_power / noise_plus_distortion).log10()
        }
    }

    pub fn calculate_enob(sinad_db: f64) -> f64 {
        (sinad_db - 1.76) / 6.02
    }

    pub fn frequency_response_gain(input_amplitude: f64, output_amplitude: f64) -> f64 {
        if input_amplitude == 0.0 {
            f64::NEG_INFINITY
        } else {
            20.0 * (output_amplitude / input_amplitude).log10()
        }
    }

    pub fn phase_response(input_phase: f64, output_phase: f64) -> f64 {
        let mut phase_diff = output_phase - input_phase;
        // Normalize to [-π, π]
        while phase_diff > PI as f64 {
            phase_diff -= 2.0 * PI as f64;
        }
        while phase_diff < -(PI as f64) {
            phase_diff += 2.0 * PI as f64;
        }
        phase_diff
    }
}

// ============================================================================
// COMPREHENSIVE UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // 1. FUNDAMENTAL MATHEMATICAL FUNCTIONS
    // ========================================================================

    #[test]
    fn test_rms_calculation_precision() {
        // Test with known analytical values
        let test_cases = vec![
            // DC signal
            (vec![1.0, 1.0, 1.0, 1.0], 1.0),
            // Square wave
            (vec![1.0, -1.0, 1.0, -1.0], 1.0),
            // Sine wave (approximate)
            (
                (0..1000)
                    .map(|i| (i as f64 * 2.0 * PI / 1000.0).sin())
                    .collect::<Vec<_>>(),
                1.0 / (2.0f64).sqrt(),
            ),
            // Zero signal
            (vec![0.0; 100], 0.0),
            // 3-4-5 triangle
            (vec![3.0, 4.0], (25.0f64 / 2.0).sqrt()),
        ];

        for (samples, expected_rms) in test_cases {
            let rms = math_utils::calculate_rms_f64(&samples);
            let result = MathTestResult::new(
                "RMS calculation",
                expected_rms,
                rms,
                HIGH_PRECISION_TOLERANCE as f64,
            );

            assert!(
                result.passed,
                "RMS test failed: {} - Expected: {:.10}, Got: {:.10}, Error: {:.2e}",
                result.test_name, result.expected, result.actual, result.error_magnitude
            );
        }
    }

    #[test_case(vec![1.0, -2.0, 0.5, -0.8], 2.0; "mixed_positive_negative")]
    #[test_case(vec![0.0, 0.0, 0.0], 0.0; "all_zeros")]
    #[test_case(vec![-5.0, -3.0, -1.0], 5.0; "all_negative")]
    #[test_case(vec![0.1, 0.2, 0.15], 0.2; "small_positive")]
    fn test_peak_calculation(samples: Vec<f64>, expected: f64) {
        let peak = math_utils::calculate_peak_f64(&samples);
        assert_abs_diff_eq!(peak, expected, epsilon = HIGH_PRECISION_TOLERANCE as f64);
    }

    #[rstest]
    #[case(1.0, &[0.1, 0.05, 0.02], 0.0, 11.18033988749895)] // THD only
    #[case(1.0, &[0.1, 0.05], 0.01, 12.20655562093723)] // THD+N
    #[case(1.0, &[], 0.05, 5.0)] // Noise only
    fn test_thd_plus_n_calculation(
        #[case] fundamental: f64,
        #[case] harmonics: &[f64],
        #[case] noise_power: f64,
        #[case] expected: f64,
    ) {
        let thd_n = math_utils::calculate_thd_plus_n(fundamental, harmonics, noise_power);
        assert_relative_eq!(thd_n, expected, epsilon = AUDIO_TOLERANCE as f64);
    }

    // ========================================================================
    // 2. SIGNAL GENERATOR VERIFICATION
    // ========================================================================

    #[test]
    fn test_sine_generator_mathematical_properties() {
        let frequencies = vec![100.0, 440.0, 1000.0, 2000.0, 8000.0];

        for freq in frequencies {
            let generator = PrecisionSineGenerator::new(freq, SAMPLE_RATE as f64);
            let samples = generator.generate_f64(TEST_DURATION as f64);

            // Test sample count
            let expected_samples = (TEST_DURATION as f64 * SAMPLE_RATE as f64) as usize;
            assert_eq!(
                samples.len(),
                expected_samples,
                "Sample count mismatch for {} Hz",
                freq
            );

            // Test RMS value (theoretical RMS of unit sine wave is 1/√2)
            let rms = math_utils::calculate_rms_f64(&samples);
            let expected_rms = generator.amplitude / 2.0f64.sqrt();
            assert_relative_eq!(
                rms,
                expected_rms,
                epsilon = AUDIO_TOLERANCE as f64,
                "RMS mismatch for {} Hz: expected {:.6}, got {:.6}",
                freq,
                expected_rms,
                rms
            );

            // Test peak value
            let peak = math_utils::calculate_peak_f64(&samples);
            assert_relative_eq!(
                peak,
                generator.amplitude,
                epsilon = AUDIO_TOLERANCE as f64,
                "Peak mismatch for {} Hz: expected {:.6}, got {:.6}",
                freq,
                generator.amplitude,
                peak
            );

            // Test DC component (should be near zero)
            let dc_component: f64 = samples.iter().sum::<f64>() / samples.len() as f64;
            assert!(
                dc_component.abs() < TOLERANCE as f64,
                "DC component too high for {} Hz: {:.2e}",
                freq,
                dc_component
            );
        }
    }

    #[test]
    fn test_sine_generator_frequency_accuracy() {
        let test_frequencies = vec![50.0, 100.0, 440.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0];
        let analyzer = PrecisionFftAnalyzer::new(FFT_SIZE);

        for &freq in &test_frequencies {
            let generator = PrecisionSineGenerator::new(freq, SAMPLE_RATE as f64);
            let samples = generator.generate_f32(2.0); // 2 seconds for better resolution
            let spectrum = analyzer.analyze_with_precision(&samples);

            if let Some((detected_freq, magnitude)) =
                analyzer.find_peak_frequency(&spectrum, SAMPLE_RATE as f64)
            {
                // Frequency should be detected within tight tolerance
                let freq_error = (detected_freq - freq).abs();
                let relative_freq_error = freq_error / freq;

                assert!(relative_freq_error < 0.001, // 0.1% tolerance
                    "Frequency detection failed for {:.1} Hz: detected {:.3} Hz (error: {:.3} Hz, {:.3}%)",
                    freq, detected_freq, freq_error, relative_freq_error * 100.0);

                // Magnitude should be significant
                assert!(
                    magnitude > 0.1,
                    "Magnitude too low for {:.1} Hz: {:.6}",
                    freq,
                    magnitude
                );
            } else {
                panic!("Failed to detect frequency for {:.1} Hz", freq);
            }
        }
    }

    #[test]
    fn test_sine_generator_amplitude_linearity() {
        let amplitudes = vec![0.01, 0.1, 0.5, 1.0, 2.0, 5.0];
        let frequency = 1000.0;

        for &amplitude in &amplitudes {
            let generator = PrecisionSineGenerator::new(frequency, SAMPLE_RATE as f64)
                .with_amplitude(amplitude);
            let samples = generator.generate_f64(TEST_DURATION as f64);

            let rms = math_utils::calculate_rms_f64(&samples);
            let expected_rms = amplitude / 2.0f64.sqrt();
            let peak = math_utils::calculate_peak_f64(&samples);

            assert_relative_eq!(
                rms,
                expected_rms,
                epsilon = AUDIO_TOLERANCE as f64,
                "RMS linearity failed for amplitude {:.3}: expected {:.6}, got {:.6}",
                amplitude,
                expected_rms,
                rms
            );

            assert_relative_eq!(
                peak,
                amplitude,
                epsilon = AUDIO_TOLERANCE as f64,
                "Peak linearity failed for amplitude {:.3}: expected {:.6}, got {:.6}",
                amplitude,
                amplitude,
                peak
            );
        }
    }

    #[test]
    fn test_sine_generator_phase_accuracy() {
        let phases = vec![
            0.0,
            PI as f64 / 4.0,
            PI as f64 / 2.0,
            PI as f64,
            3.0 * PI as f64 / 2.0,
        ];
        let frequency = 1000.0;

        for &phase in &phases {
            let generator =
                PrecisionSineGenerator::new(frequency, SAMPLE_RATE as f64).with_phase(phase);
            let samples = generator.generate_f64(1.0 / frequency); // One period

            // First sample should match expected phase
            let first_sample = samples[0];
            let expected_first = phase.sin();

            assert_relative_eq!(
                first_sample,
                expected_first,
                epsilon = AUDIO_TOLERANCE as f64,
                "Phase accuracy failed for phase {:.3}: expected {:.6}, got {:.6}",
                phase,
                expected_first,
                first_sample
            );
        }
    }

    // ========================================================================
    // 3. FFT AND SPECTRAL ANALYSIS VERIFICATION
    // ========================================================================

    #[test]
    fn test_fft_analyzer_known_frequencies() {
        let analyzer = PrecisionFftAnalyzer::new(FFT_SIZE);
        let test_cases = vec![
            (100.0, 50.0),   // Low frequency, wider tolerance due to resolution
            (440.0, 25.0),   // A4 note
            (1000.0, 25.0),  // 1 kHz reference
            (2000.0, 50.0),  // Higher frequency
            (8000.0, 100.0), // High frequency, wider tolerance
        ];

        for (freq, tolerance) in test_cases {
            let generator = PrecisionSineGenerator::new(freq, SAMPLE_RATE as f64);
            let samples = generator.generate_f32(4.0); // 4 seconds for high resolution
            let spectrum = analyzer.analyze_with_precision(&samples);

            if let Some((detected_freq, magnitude)) =
                analyzer.find_peak_frequency(&spectrum, SAMPLE_RATE as f64)
            {
                assert!(
                    (detected_freq - freq).abs() < tolerance,
                    "FFT frequency detection failed for {:.1} Hz: detected {:.3} Hz ± {:.1} Hz",
                    freq,
                    detected_freq,
                    tolerance
                );

                // Verify magnitude is reasonable for unit amplitude sine
                assert!(
                    magnitude > 0.1 && magnitude < 10.0,
                    "FFT magnitude out of range for {:.1} Hz: {:.6}",
                    freq,
                    magnitude
                );
            } else {
                panic!("FFT failed to detect {:.1} Hz signal", freq);
            }
        }
    }

    #[test]
    fn test_fft_window_function_properties() {
        let analyzer = PrecisionFftAnalyzer::new(FFT_SIZE);

        // Test window normalization
        let window_sum: f64 = analyzer.window.iter().sum();
        let expected_sum = analyzer.normalization_factor * FFT_SIZE as f64;
        assert_relative_eq!(
            window_sum,
            expected_sum,
            epsilon = HIGH_PRECISION_TOLERANCE as f64,
            "Window normalization failed: sum = {:.6}, expected = {:.6}",
            window_sum,
            expected_sum
        );

        // Test window symmetry (Blackman-Harris is symmetric)
        for i in 0..FFT_SIZE / 2 {
            let left = analyzer.window[i];
            let right = analyzer.window[FFT_SIZE - 1 - i];
            assert_relative_eq!(
                left,
                right,
                epsilon = HIGH_PRECISION_TOLERANCE as f64,
                "Window symmetry failed at index {}: left = {:.10}, right = {:.10}",
                i,
                left,
                right
            );
        }
    }

    #[test]
    fn test_fft_noise_floor() {
        let analyzer = PrecisionFftAnalyzer::new(FFT_SIZE);

        // Test with zero signal
        let zero_samples = vec![0.0; FFT_SIZE * 2];
        let spectrum = analyzer.analyze_with_precision(&zero_samples);

        // All bins should be very close to zero
        for (i, &magnitude) in spectrum.iter().enumerate() {
            assert!(
                magnitude < 1e-10,
                "Noise floor too high at bin {}: {:.2e}",
                i,
                magnitude
            );
        }

        // Test with very low amplitude signal
        let generator =
            PrecisionSineGenerator::new(1000.0, SAMPLE_RATE as f64).with_amplitude(1e-6);
        let samples = generator.generate_f32(2.0);
        let spectrum = analyzer.analyze_with_precision(&samples);

        if let Some((detected_freq, magnitude)) =
            analyzer.find_peak_frequency(&spectrum, SAMPLE_RATE as f64)
        {
            assert!(
                (detected_freq - 1000.0).abs() < 50.0,
                "Low amplitude signal detection failed: {:.1} Hz",
                detected_freq
            );
            assert!(
                magnitude > 1e-8 && magnitude < 1e-4,
                "Low amplitude magnitude out of range: {:.2e}",
                magnitude
            );
        }
    }

    // ========================================================================
    // 4. AUDIO QUALITY METRICS VERIFICATION
    // ========================================================================

    #[test]
    fn test_signal_to_noise_ratio_calculation() {
        let test_cases = vec![
            (1.0, 0.1, 20.0),   // 20 dB SNR
            (1.0, 0.01, 40.0),  // 40 dB SNR
            (1.0, 0.001, 60.0), // 60 dB SNR
            (2.0, 0.1, 26.02),  // Higher signal level
            (0.5, 0.05, 20.0),  // Lower signal level
        ];

        for (signal_power, noise_power, expected_snr_db) in test_cases {
            let snr_db = math_utils::calculate_snr_db(signal_power, noise_power);
            assert_relative_eq!(
                snr_db,
                expected_snr_db,
                epsilon = 0.01,
                "SNR calculation failed: expected {:.2} dB, got {:.2} dB",
                expected_snr_db,
                snr_db
            );
        }
    }

    #[test]
    fn test_sinad_and_enob_calculation() {
        let test_cases = vec![
            (1.0, 0.001, 60.0, 9.68), // 60 dB SINAD ≈ 9.68 ENOB
            (1.0, 0.01, 40.0, 6.35),  // 40 dB SINAD ≈ 6.35 ENOB
            (1.0, 0.1, 20.0, 3.02),   // 20 dB SINAD ≈ 3.02 ENOB
        ];

        for (signal_power, noise_distortion, expected_sinad, expected_enob) in test_cases {
            let sinad_db = math_utils::calculate_sinad_db(signal_power, noise_distortion);
            let enob = math_utils::calculate_enob(sinad_db);

            assert_relative_eq!(
                sinad_db,
                expected_sinad,
                epsilon = 0.01,
                "SINAD calculation failed: expected {:.2} dB, got {:.2} dB",
                expected_sinad,
                sinad_db
            );

            assert_relative_eq!(
                enob,
                expected_enob,
                epsilon = 0.01,
                "ENOB calculation failed: expected {:.2} bits, got {:.2} bits",
                expected_enob,
                enob
            );
        }
    }

    #[test]
    fn test_frequency_response_analysis() {
        let frequencies = vec![20.0, 100.0, 1000.0, 10000.0, 20000.0];
        let analyzer = PrecisionFftAnalyzer::new(FFT_SIZE);

        for &freq in &frequencies {
            // Generate input signal
            let input_generator = PrecisionSineGenerator::new(freq, SAMPLE_RATE as f64);
            let input_samples = input_generator.generate_f32(2.0);
            let input_spectrum = analyzer.analyze_with_precision(&input_samples);

            // Simulate output with known gain
            let gain_db = -3.0; // -3 dB gain
            let gain_linear = 10.0f64.powf(gain_db / 20.0);
            let output_generator =
                PrecisionSineGenerator::new(freq, SAMPLE_RATE as f64).with_amplitude(gain_linear);
            let output_samples = output_generator.generate_f32(2.0);
            let output_spectrum = analyzer.analyze_with_precision(&output_samples);

            // Find peak magnitudes
            if let (Some((_, input_mag)), Some((_, output_mag))) = (
                analyzer.find_peak_frequency(&input_spectrum, SAMPLE_RATE as f64),
                analyzer.find_peak_frequency(&output_spectrum, SAMPLE_RATE as f64),
            ) {
                let measured_gain = math_utils::frequency_response_gain(input_mag, output_mag);

                assert_relative_eq!(
                    measured_gain,
                    gain_db,
                    epsilon = 0.1,
                    "Frequency response failed at {:.0} Hz: expected {:.2} dB, got {:.2} dB",
                    freq,
                    gain_db,
                    measured_gain
                );
            } else {
                panic!("Failed to measure frequency response at {:.0} Hz", freq);
            }
        }
    }

    // ========================================================================
    // 5. PROPERTY-BASED TESTS
    // ========================================================================

    #[quickcheck]
    fn qc_rms_properties(samples: Vec<f32>) -> TestResult {
        if samples.is_empty() || samples.len() > 10000 {
            return TestResult::discard();
        }

        let rms = math_utils::calculate_rms_f32(&samples);

        // Property 1: RMS is always non-negative
        if rms < 0.0 {
            return TestResult::failed();
        }

        // Property 2: RMS of zero signal is zero
        if samples.iter().all(|&x| x == 0.0) && rms != 0.0 {
            return TestResult::failed();
        }

        // Property 3: RMS ≤ peak
        let peak = math_utils::calculate_peak_f32(&samples);
        if rms > peak + f32::EPSILON {
            return TestResult::failed();
        }

        TestResult::passed()
    }

    proptest! {
        #[test]
        fn prop_sine_rms_invariant(
            frequency in 1.0f64..20000.0,
            amplitude in 0.001f64..10.0,
            duration in 0.1f64..5.0
        ) {
            let generator = PrecisionSineGenerator::new(frequency, SAMPLE_RATE as f64)
                .with_amplitude(amplitude);
            let samples = generator.generate_f64(duration);

            let rms = math_utils::calculate_rms_f64(&samples);
            let expected_rms = amplitude / 2.0f64.sqrt();

            // Allow some tolerance due to finite precision and duration
            let relative_error = (rms - expected_rms).abs() / expected_rms;
            prop_assert!(relative_error < 0.01,
                "RMS invariant failed: freq={:.1}, amp={:.3}, expected_rms={:.6}, actual_rms={:.6}, error={:.3}%",
                frequency, amplitude, expected_rms, rms, relative_error * 100.0);
        }

        #[test]
        fn prop_fft_energy_conservation(
            frequency in 100.0f64..8000.0,
            amplitude in 0.1f64..2.0
        ) {
            let generator = PrecisionSineGenerator::new(frequency, SAMPLE_RATE as f64)
                .with_amplitude(amplitude);
            let samples = generator.generate_f32(2.0);

            // Calculate time domain energy
            let time_energy: f32 = samples.iter().map(|&x| x * x).sum();

            // Calculate frequency domain energy (Parseval's theorem)
            let analyzer = PrecisionFftAnalyzer::new(FFT_SIZE);
            let spectrum = analyzer.analyze_with_precision(&samples);
            let freq_energy: f64 = spectrum.iter().map(|&x| x * x).sum::<f64>() * 2.0; // Factor of 2 for one-sided spectrum

            let relative_error = (time_energy as f64 - freq_energy).abs() / time_energy as f64;
            prop_assert!(relative_error < 0.1,
                "Energy conservation failed: time_energy={:.6}, freq_energy={:.6}, error={:.3}%",
                time_energy, freq_energy, relative_error * 100.0);
        }
    }

    // ========================================================================
    // 6. EDGE CASES AND BOUNDARY CONDITIONS
    // ========================================================================

    #[test]
    fn test_edge_cases() {
        // Empty samples
        assert_eq!(math_utils::calculate_rms_f32(&[]), 0.0);
        assert_eq!(math_utils::calculate_peak_f32(&[]), 0.0);

        // Single sample
        assert_eq!(math_utils::calculate_rms_f32(&[5.0]), 5.0);
        assert_eq!(math_utils::calculate_peak_f32(&[5.0]), 5.0);

        // Very small values
        let tiny = vec![1e-10f32; 1000];
        let rms = math_utils::calculate_rms_f32(&tiny);
        assert_relative_eq!(rms, 1e-10, epsilon = 1e-12);

        // Very large values
        let large = vec![1e6f32; 100];
        let rms = math_utils::calculate_rms_f32(&large);
        assert_relative_eq!(rms, 1e6, epsilon = 1e3);

        // Mixed precision tests
        let mixed: Vec<f32> = vec![1.0, 1e-6, 1e6, -1e6, 1e-6, 1.0];
        let rms = math_utils::calculate_rms_f32(&mixed);
        assert!(rms.is_finite() && rms > 0.0);
    }

    #[test]
    fn test_numerical_stability() {
        // Test with values that might cause overflow/underflow
        let near_infinity = vec![f32::MAX / 2.0; 10];
        let rms = math_utils::calculate_rms_f32(&near_infinity);
        assert!(rms.is_finite());

        let near_zero = vec![f32::MIN_POSITIVE; 10];
        let rms = math_utils::calculate_rms_f32(&near_zero);
        assert!(rms > 0.0 && rms.is_finite());

        // Test alternating large values
        let alternating: Vec<f32> = (0..1000)
            .map(|i| if i % 2 == 0 { 1e6 } else { -1e6 })
            .collect();
        let rms = math_utils::calculate_rms_f32(&alternating);
        assert!(rms.is_finite());
        assert_relative_eq!(rms, 1e6, epsilon = 1e3);
    }

    // ========================================================================
    // 7. INTEGRATION TESTS FOR COMPLETE ANALYSIS CHAIN
    // ========================================================================

    #[test]
    fn test_complete_analysis_pipeline() {
        let test_frequency = 1000.0;
        let test_amplitude = 0.7;
        let test_duration = 2.0;

        // Generate high-quality test signal
        let generator = PrecisionSineGenerator::new(test_frequency, SAMPLE_RATE as f64)
            .with_amplitude(test_amplitude);
        let samples_f64 = generator.generate_f64(test_duration);
        let samples_f32 = generator.generate_f32(test_duration);

        // Analyze in time domain
        let rms_f64 = math_utils::calculate_rms_f64(&samples_f64);
        let rms_f32 = math_utils::calculate_rms_f32(&samples_f32);
        let peak_f64 = math_utils::calculate_peak_f64(&samples_f64);
        let peak_f32 = math_utils::calculate_peak_f32(&samples_f32);

        // Analyze in frequency domain
        let analyzer = PrecisionFftAnalyzer::new(FFT_SIZE);
        let spectrum = analyzer.analyze_with_precision(&samples_f32);
        let (detected_freq, detected_magnitude) = analyzer
            .find_peak_frequency(&spectrum, SAMPLE_RATE as f64)
            .expect("Should detect generated frequency");

        // Verify all measurements are consistent
        let expected_rms = test_amplitude / 2.0f64.sqrt();
        assert_relative_eq!(rms_f64, expected_rms, epsilon = AUDIO_TOLERANCE as f64);
        assert_relative_eq!(
            rms_f32 as f64,
            expected_rms,
            epsilon = AUDIO_TOLERANCE as f64
        );

        assert_relative_eq!(peak_f64, test_amplitude, epsilon = AUDIO_TOLERANCE as f64);
        assert_relative_eq!(
            peak_f32 as f64,
            test_amplitude,
            epsilon = AUDIO_TOLERANCE as f64
        );

        assert!((detected_freq - test_frequency).abs() < 25.0);
        assert!(detected_magnitude > 0.1);

        // Calculate and verify audio quality metrics
        let signal_power = rms_f64 * rms_f64;
        let noise_power = 1e-6; // Assume very low noise floor
        let snr_db = math_utils::calculate_snr_db(signal_power, noise_power);

        assert!(snr_db > 40.0, "SNR too low: {:.1} dB", snr_db);

        println!("Complete analysis pipeline test passed:");
        println!(
            "  Generated: {:.1} Hz, {:.3} amplitude",
            test_frequency, test_amplitude
        );
        println!(
            "  Detected: {:.1} Hz, {:.6} magnitude",
            detected_freq, detected_magnitude
        );
        println!("  RMS (f64): {:.6}, Peak (f64): {:.6}", rms_f64, peak_f64);
        println!("  RMS (f32): {:.6}, Peak (f32): {:.6}", rms_f32, peak_f32);
        println!("  SNR: {:.1} dB", snr_db);
    }
}

// ============================================================================
// STATISTICAL ANALYSIS FOR TEST QUALITY
// ============================================================================

#[cfg(test)]
mod statistical_tests {
    use super::*;

    #[test]
    fn test_measurement_repeatability() {
        let frequency = 1000.0;
        let amplitude = 1.0;
        let num_trials = 100;

        let mut rms_measurements = Vec::new();
        let mut peak_measurements = Vec::new();

        for _ in 0..num_trials {
            let generator = PrecisionSineGenerator::new(frequency, SAMPLE_RATE as f64)
                .with_amplitude(amplitude);
            let samples = generator.generate_f64(1.0);

            rms_measurements.push(math_utils::calculate_rms_f64(&samples));
            peak_measurements.push(math_utils::calculate_peak_f64(&samples));
        }

        // Calculate statistics
        let rms_data = Data::new(rms_measurements);
        let peak_data = Data::new(peak_measurements);

        let rms_mean = rms_data.mean().unwrap();
        let rms_std = rms_data.std_dev().unwrap();
        let peak_mean = peak_data.mean().unwrap();
        let peak_std = peak_data.std_dev().unwrap();

        // Expected values
        let expected_rms = amplitude / 2.0f64.sqrt();
        let expected_peak = amplitude;

        // Verify means are close to expected
        assert_relative_eq!(rms_mean, expected_rms, epsilon = 1e-10);
        assert_relative_eq!(peak_mean, expected_peak, epsilon = 1e-10);

        // Verify low standard deviation (high repeatability)
        assert!(
            rms_std < 1e-10,
            "RMS measurement not repeatable: std = {:.2e}",
            rms_std
        );
        assert!(
            peak_std < 1e-10,
            "Peak measurement not repeatable: std = {:.2e}",
            peak_std
        );

        println!("Repeatability test passed:");
        println!("  RMS: mean = {:.10}, std = {:.2e}", rms_mean, rms_std);
        println!("  Peak: mean = {:.10}, std = {:.2e}", peak_mean, peak_std);
    }
}
