// Comprehensive Unit Test Suite for Rusty Audio Mathematical Testing Framework
//
// This test suite focuses on mathematical accuracy, signal processing verification,
// and audio processing correctness using the testing framework we built.

use std::f32::consts::PI;
use rustfft::{FftPlanner, num_complex::Complex32};
use rand::{Rng, SeedableRng, rngs::StdRng};
use approx::assert_relative_eq;

// Test constants
const SAMPLE_RATE: f32 = 44100.0;
const TOLERANCE: f32 = 0.001;
const FFT_SIZE: usize = 2048;

/// Pure sine wave generator for testing
pub struct TestSineGenerator {
    pub frequency: f32,
    pub amplitude: f32,
    pub phase: f32,
}

impl TestSineGenerator {
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

/// White noise generator with fixed seed for reproducible tests
pub struct TestNoiseGenerator {
    pub amplitude: f32,
    pub seed: u64,
}

impl TestNoiseGenerator {
    pub fn new() -> Self {
        Self {
            amplitude: 1.0,
            seed: 42,
        }
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

/// FFT Analyzer for frequency domain verification
pub struct TestFftAnalyzer {
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    fft_size: usize,
    window: Vec<f32>,
}

impl TestFftAnalyzer {
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

        // Find peak in magnitude spectrum
        let half_size = self.fft_size / 2;
        let mut max_magnitude = 0.0;
        let mut max_bin = 0;

        for i in 1..half_size {
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

/// Calculate THD (Total Harmonic Distortion)
pub fn calculate_thd(fundamental_amplitude: f32, harmonic_amplitudes: &[f32]) -> f32 {
    if fundamental_amplitude == 0.0 {
        return f32::INFINITY;
    }

    let harmonic_sum_squares: f32 = harmonic_amplitudes.iter()
        .map(|&h| h * h)
        .sum();

    (harmonic_sum_squares.sqrt() / fundamental_amplitude) * 100.0
}

// COMPREHENSIVE TEST SUITE
// ========================

#[cfg(test)]
mod tests {
    use super::*;

    // 1. SIGNAL GENERATOR TESTS
    // =========================

    #[test]
    fn test_sine_generator_mathematical_properties() {
        let generator = TestSineGenerator::new(1000.0);
        let samples = generator.generate(1.0, SAMPLE_RATE);

        // Test sample count
        assert_eq!(samples.len(), SAMPLE_RATE as usize);

        // Test RMS value (theoretical RMS of unit sine wave is 1/√2)
        let rms = calculate_rms(&samples);
        let expected_rms = 1.0 / 2.0f32.sqrt();
        assert_relative_eq!(rms, expected_rms, epsilon = TOLERANCE * 10.0);

        // Test peak value
        let peak = calculate_peak(&samples);
        assert_relative_eq!(peak, 1.0, epsilon = TOLERANCE * 10.0);

        // Test DC component (should be near zero)
        let dc_component: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
        assert!(dc_component.abs() < TOLERANCE);
    }

    #[test]
    fn test_sine_generator_frequency_accuracy() {
        let test_frequencies = vec![100.0, 440.0, 1000.0, 2000.0, 8000.0];
        let analyzer = TestFftAnalyzer::new(FFT_SIZE);

        for &freq in &test_frequencies {
            let generator = TestSineGenerator::new(freq);
            let samples = generator.generate(2.0, SAMPLE_RATE); // 2 seconds for better resolution

            if let Some((detected_freq, magnitude)) = analyzer.analyze_and_find_peak(&samples, SAMPLE_RATE) {
                // Frequency should be detected within 25 Hz tolerance
                assert!((detected_freq - freq).abs() < 25.0,
                    "Expected {:.1} Hz, got {:.1} Hz", freq, detected_freq);

                // Magnitude should be significant (> 0.3 for unit amplitude sine)
                assert!(magnitude > 0.3,
                    "Magnitude too low: {} for {:.1} Hz", magnitude, freq);
            } else {
                panic!("Failed to detect frequency for {:.1} Hz", freq);
            }
        }
    }

    #[test]
    fn test_sine_generator_amplitude_scaling() {
        let amplitudes = vec![0.1, 0.5, 1.0, 2.0];

        for &amp in &amplitudes {
            let generator = TestSineGenerator::new(1000.0).with_amplitude(amp);
            let samples = generator.generate(1.0, SAMPLE_RATE);

            let rms = calculate_rms(&samples);
            let expected_rms = amp / 2.0f32.sqrt();
            assert_relative_eq!(rms, expected_rms, epsilon = TOLERANCE * 10.0);

            let peak = calculate_peak(&samples);
            assert_relative_eq!(peak, amp, epsilon = TOLERANCE * 10.0);
        }
    }

    // 2. NOISE GENERATOR TESTS
    // =========================

    #[test]
    fn test_noise_generator_statistical_properties() {
        let generator = TestNoiseGenerator::new();
        let samples = generator.generate(10.0, SAMPLE_RATE); // 10 seconds for good statistics

        // Test that noise is roughly uniform distribution
        let rms = calculate_rms(&samples);
        let peak = calculate_peak(&samples);

        // For uniform white noise, RMS should be approximately amplitude/√3
        let expected_rms = 1.0 / 3.0f32.sqrt();
        let rms_tolerance = expected_rms * 0.2; // 20% tolerance due to randomness
        assert!((rms - expected_rms).abs() < rms_tolerance,
            "RMS {} not within tolerance of expected {}", rms, expected_rms);

        // Peak should be close to amplitude
        assert!(peak <= 1.0, "Peak {} exceeds amplitude", peak);
        assert!(peak > 0.8, "Peak {} too low for white noise", peak);

        // Test that DC component is near zero
        let dc_component: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
        assert!(dc_component.abs() < 0.1, "DC component {} too high", dc_component);
    }

    #[test]
    fn test_noise_generator_reproducibility() {
        let generator1 = TestNoiseGenerator::new();
        let generator2 = TestNoiseGenerator::new();

        let samples1 = generator1.generate(1.0, SAMPLE_RATE);
        let samples2 = generator2.generate(1.0, SAMPLE_RATE);

        // With same seed, should produce identical results
        assert_eq!(samples1.len(), samples2.len());
        for (s1, s2) in samples1.iter().zip(samples2.iter()) {
            assert_eq!(s1, s2, "Noise generators with same seed should be identical");
        }
    }

    // 3. FFT ANALYSIS TESTS
    // =====================

    #[test]
    fn test_fft_analyzer_known_frequencies() {
        let analyzer = TestFftAnalyzer::new(FFT_SIZE);
        let test_cases = vec![
            (100.0, 25.0),   // Low frequency
            (440.0, 25.0),   // A4 note
            (1000.0, 25.0),  // 1 kHz
            (5000.0, 50.0),  // High frequency (wider tolerance)
        ];

        for (freq, tolerance) in test_cases {
            let generator = TestSineGenerator::new(freq);
            let samples = generator.generate(2.0, SAMPLE_RATE);

            if let Some((detected_freq, _)) = analyzer.analyze_and_find_peak(&samples, SAMPLE_RATE) {
                assert!((detected_freq - freq).abs() < tolerance,
                    "FFT detected {:.1} Hz, expected {:.1} Hz ± {:.1} Hz",
                    detected_freq, freq, tolerance);
            } else {
                panic!("FFT failed to detect {:.1} Hz signal", freq);
            }
        }
    }

    #[test]
    fn test_fft_analyzer_magnitude_accuracy() {
        let analyzer = TestFftAnalyzer::new(FFT_SIZE);
        let generator = TestSineGenerator::new(1000.0);
        let samples = generator.generate(2.0, SAMPLE_RATE);

        if let Some((_, magnitude)) = analyzer.analyze_and_find_peak(&samples, SAMPLE_RATE) {
            // For unit amplitude sine wave, FFT magnitude should be around 0.5
            // (due to windowing and normalization)
            assert!(magnitude > 0.3 && magnitude < 0.7,
                "FFT magnitude {} not in expected range [0.3, 0.7]", magnitude);
        } else {
            panic!("FFT failed to analyze magnitude");
        }
    }

    #[test]
    fn test_fft_analyzer_noise_floor() {
        let analyzer = TestFftAnalyzer::new(FFT_SIZE);
        let zero_samples = vec![0.0; (2.0 * SAMPLE_RATE) as usize];

        let result = analyzer.analyze_and_find_peak(&zero_samples, SAMPLE_RATE);
        assert!(result.is_none(), "FFT should return None for zero signal");
    }

    // 4. MATHEMATICAL UTILITY TESTS
    // ==============================

    #[test]
    fn test_rms_calculation_accuracy() {
        // Test known RMS values
        let test_cases = vec![
            (vec![1.0, -1.0, 1.0, -1.0], 1.0),           // Square wave
            (vec![0.0, 0.0, 0.0, 0.0], 0.0),             // Silence
            (vec![3.0, 4.0], 5.0f32.sqrt() / 2.0),       // 3-4-5 triangle
            (vec![1.0, 1.0, 1.0, 1.0], 1.0),             // DC signal
        ];

        for (samples, expected_rms) in test_cases {
            let rms = calculate_rms(&samples);
            assert_relative_eq!(rms, expected_rms, epsilon = TOLERANCE);
        }
    }

    #[test]
    fn test_peak_calculation_accuracy() {
        let test_cases = vec![
            (vec![1.0, -2.0, 0.5, -0.8], 2.0),
            (vec![0.0, 0.0, 0.0], 0.0),
            (vec![-5.0, -3.0, -1.0], 5.0),
            (vec![0.1, 0.2, 0.15], 0.2),
        ];

        for (samples, expected_peak) in test_cases {
            let peak = calculate_peak(&samples);
            assert_eq!(peak, expected_peak);
        }
    }

    #[test]
    fn test_thd_calculation() {
        // Test THD calculation with known values
        let fundamental = 1.0;
        let harmonics = vec![0.1, 0.05]; // 2nd and 3rd harmonics

        let thd = calculate_thd(fundamental, &harmonics);
        let expected_thd = ((0.1f32.powi(2) + 0.05f32.powi(2)).sqrt() / 1.0) * 100.0;

        assert_relative_eq!(thd, expected_thd, epsilon = TOLERANCE);
    }

    // 5. EDGE CASE TESTS
    // ===================

    #[test]
    fn test_edge_cases() {
        // Empty samples
        assert_eq!(calculate_rms(&[]), 0.0);
        assert_eq!(calculate_peak(&[]), 0.0);

        // Single sample
        assert_eq!(calculate_rms(&[5.0]), 5.0);
        assert_eq!(calculate_peak(&[5.0]), 5.0);

        // THD with zero fundamental
        assert_eq!(calculate_thd(0.0, &[0.1, 0.2]), f32::INFINITY);
    }

    // 6. PROPERTY-BASED TESTS
    // ========================

    #[test]
    fn test_sine_wave_invariants() {
        let frequencies = vec![100.0, 440.0, 1000.0, 2000.0];

        for freq in frequencies {
            let generator = TestSineGenerator::new(freq);
            let samples = generator.generate(1.0, SAMPLE_RATE);

            // Property: RMS should always be amplitude/√2 for sine waves
            let rms = calculate_rms(&samples);
            let expected_rms = generator.amplitude / 2.0f32.sqrt();
            assert_relative_eq!(rms, expected_rms, epsilon = TOLERANCE * 10.0);

            // Property: Peak should always equal amplitude for sine waves
            let peak = calculate_peak(&samples);
            assert_relative_eq!(peak, generator.amplitude, epsilon = TOLERANCE * 10.0);

            // Property: Energy should be conserved
            let energy: f32 = samples.iter().map(|&x| x * x).sum();
            let expected_energy = (generator.amplitude * generator.amplitude) / 2.0 * samples.len() as f32;
            assert_relative_eq!(energy, expected_energy, epsilon = TOLERANCE * samples.len() as f32);
        }
    }

    #[test]
    fn test_scaling_invariants() {
        let base_freq = 1000.0;
        let base_generator = TestSineGenerator::new(base_freq);
        let base_samples = base_generator.generate(1.0, SAMPLE_RATE);
        let base_rms = calculate_rms(&base_samples);

        let scales = vec![0.5, 2.0, 0.1, 10.0];

        for scale in scales {
            let scaled_generator = TestSineGenerator::new(base_freq).with_amplitude(scale);
            let scaled_samples = scaled_generator.generate(1.0, SAMPLE_RATE);
            let scaled_rms = calculate_rms(&scaled_samples);

            // Property: RMS should scale linearly with amplitude
            assert_relative_eq!(scaled_rms, base_rms * scale, epsilon = TOLERANCE * scale);
        }
    }

    // 7. INTEGRATION TESTS
    // ====================

    #[test]
    fn test_complete_analysis_chain() {
        // Test complete chain: generation -> analysis -> verification
        let test_freq = 880.0; // A5 note
        let test_amplitude = 0.7;

        // Generate signal
        let generator = TestSineGenerator::new(test_freq).with_amplitude(test_amplitude);
        let samples = generator.generate(2.0, SAMPLE_RATE);

        // Analyze signal
        let analyzer = TestFftAnalyzer::new(FFT_SIZE);
        let (detected_freq, magnitude) = analyzer.analyze_and_find_peak(&samples, SAMPLE_RATE)
            .expect("Should detect generated frequency");

        // Verify mathematical properties
        let rms = calculate_rms(&samples);
        let peak = calculate_peak(&samples);

        // All properties should match expected values
        assert!((detected_freq - test_freq).abs() < 25.0);
        assert!(magnitude > 0.2);
        assert_relative_eq!(rms, test_amplitude / 2.0f32.sqrt(), epsilon = TOLERANCE * 10.0);
        assert_relative_eq!(peak, test_amplitude, epsilon = TOLERANCE * 10.0);
    }

    #[test]
    fn test_multi_tone_analysis() {
        // Generate signal with multiple tones
        let freq1 = 440.0;
        let freq2 = 880.0;
        let amp1 = 0.5;
        let amp2 = 0.3;

        let gen1 = TestSineGenerator::new(freq1).with_amplitude(amp1);
        let gen2 = TestSineGenerator::new(freq2).with_amplitude(amp2);

        let samples1 = gen1.generate(2.0, SAMPLE_RATE);
        let samples2 = gen2.generate(2.0, SAMPLE_RATE);

        // Mix signals
        let mixed_samples: Vec<f32> = samples1.iter()
            .zip(samples2.iter())
            .map(|(s1, s2)| s1 + s2)
            .collect();

        // Verify properties of mixed signal
        let total_energy_expected = amp1 * amp1 / 2.0 + amp2 * amp2 / 2.0;
        let actual_rms = calculate_rms(&mixed_samples);
        let expected_rms = total_energy_expected.sqrt();

        assert_relative_eq!(actual_rms, expected_rms, epsilon = TOLERANCE * 10.0);

        // Peak should be less than or equal to sum of amplitudes
        let peak = calculate_peak(&mixed_samples);
        assert!(peak <= amp1 + amp2 + TOLERANCE);
    }
}