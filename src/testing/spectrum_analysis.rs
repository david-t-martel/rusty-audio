// Spectrum Analysis Testing and Verification
//
// This module provides mathematical verification of FFT accuracy and
// spectrum analysis correctness for the audio processing pipeline.

use rustfft::{FftPlanner, Fft, num_complex::Complex32};
use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};
use super::{TestResult, TestSuite, SAMPLE_RATE, TOLERANCE};
use super::signal_generators::*;
use std::sync::Arc;

/// FFT analysis result for verification
#[derive(Debug, Clone)]
pub struct SpectrumAnalysis {
    pub frequencies: Vec<f32>,
    pub magnitudes: Vec<f32>,
    pub phases: Vec<f32>,
    pub sample_rate: f32,
    pub fft_size: usize,
}

impl SpectrumAnalysis {
    pub fn new(fft_size: usize, sample_rate: f32) -> Self {
        let frequencies = (0..fft_size/2)
            .map(|i| (i as f32 * sample_rate) / fft_size as f32)
            .collect();

        Self {
            frequencies,
            magnitudes: Vec::new(),
            phases: Vec::new(),
            sample_rate,
            fft_size,
        }
    }

    /// Find peak frequency and its magnitude
    pub fn find_peak(&self) -> Option<(f32, f32)> {
        if self.magnitudes.is_empty() {
            return None;
        }

        let (max_idx, &max_magnitude) = self.magnitudes
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())?;

        Some((self.frequencies[max_idx], max_magnitude))
    }

    /// Find all peaks above a threshold
    pub fn find_peaks(&self, threshold: f32) -> Vec<(f32, f32)> {
        let mut peaks = Vec::new();

        for i in 1..self.magnitudes.len() - 1 {
            let mag = self.magnitudes[i];
            if mag > threshold &&
               mag > self.magnitudes[i - 1] &&
               mag > self.magnitudes[i + 1] {
                peaks.push((self.frequencies[i], mag));
            }
        }

        peaks
    }

    /// Get magnitude at specific frequency (with interpolation)
    pub fn magnitude_at_frequency(&self, target_freq: f32) -> Option<f32> {
        if self.frequencies.is_empty() {
            return None;
        }

        // Find closest frequency bins
        let bin_resolution = self.sample_rate / self.fft_size as f32;
        let target_bin = target_freq / bin_resolution;
        let lower_bin = target_bin.floor() as usize;
        let upper_bin = target_bin.ceil() as usize;

        if upper_bin >= self.magnitudes.len() {
            return None;
        }

        if lower_bin == upper_bin {
            Some(self.magnitudes[lower_bin])
        } else {
            // Linear interpolation
            let fraction = target_bin - lower_bin as f32;
            let lower_mag = self.magnitudes[lower_bin];
            let upper_mag = self.magnitudes[upper_bin];
            Some(lower_mag + fraction * (upper_mag - lower_mag))
        }
    }

    /// Calculate total harmonic distortion
    pub fn calculate_thd(&self, fundamental_freq: f32, max_harmonic: usize) -> Option<f32> {
        let fundamental_mag = self.magnitude_at_frequency(fundamental_freq)?;
        if fundamental_mag == 0.0 {
            return Some(f32::INFINITY);
        }

        let mut harmonic_power_sum = 0.0;

        for harmonic in 2..=max_harmonic {
            let harmonic_freq = fundamental_freq * harmonic as f32;
            if let Some(harmonic_mag) = self.magnitude_at_frequency(harmonic_freq) {
                harmonic_power_sum += harmonic_mag * harmonic_mag;
            }
        }

        let thd = (harmonic_power_sum.sqrt() / fundamental_mag) * 100.0;
        Some(thd)
    }
}

/// FFT Analyzer for mathematical verification
pub struct FftAnalyzer {
    fft: Arc<dyn Fft<f32>>,
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
                let factor = 2.0 * std::f32::consts::PI * i as f32 / (fft_size - 1) as f32;
                0.5 * (1.0 - factor.cos())
            })
            .collect();

        Self {
            fft,
            fft_size,
            window,
        }
    }

    /// Analyze samples with FFT
    pub fn analyze(&self, samples: &[f32], sample_rate: f32) -> SpectrumAnalysis {
        let mut analysis = SpectrumAnalysis::new(self.fft_size, sample_rate);

        if samples.len() < self.fft_size {
            return analysis; // Return empty analysis
        }

        // Apply window and convert to complex
        let mut buffer: Vec<Complex32> = samples[..self.fft_size]
            .iter()
            .zip(&self.window)
            .map(|(&sample, &window)| Complex32::new(sample * window, 0.0))
            .collect();

        // Perform FFT
        self.fft.process(&mut buffer);

        // Calculate magnitudes and phases (only positive frequencies)
        let half_size = self.fft_size / 2;
        analysis.magnitudes.reserve(half_size);
        analysis.phases.reserve(half_size);

        for i in 0..half_size {
            let complex = buffer[i];
            let magnitude = complex.norm() / (self.fft_size as f32 / 2.0);
            let phase = complex.arg();

            analysis.magnitudes.push(magnitude);
            analysis.phases.push(phase);
        }

        analysis
    }

    /// Compare expected vs actual spectrum analysis
    pub fn verify_spectrum(
        &self,
        samples: &[f32],
        sample_rate: f32,
        expected_peaks: &[(f32, f32)], // (frequency, magnitude) pairs
        tolerance: f32,
    ) -> TestSuite {
        let mut suite = TestSuite::new();
        let analysis = self.analyze(samples, sample_rate);

        for &(expected_freq, expected_mag) in expected_peaks {
            if let Some(actual_mag) = analysis.magnitude_at_frequency(expected_freq) {
                let result = TestResult::new(
                    &format!("Peak at {:.1} Hz", expected_freq),
                    expected_mag,
                    actual_mag,
                    tolerance,
                );
                suite.add_result(result);
            } else {
                let result = TestResult::new(
                    &format!("Peak at {:.1} Hz (not found)", expected_freq),
                    expected_mag,
                    0.0,
                    tolerance,
                );
                suite.add_result(result);
            }
        }

        suite
    }
}

/// Web Audio API Analyser Node verification
pub struct AnalyserVerification {
    context: AudioContext,
    analyzer: FftAnalyzer,
}

impl AnalyserVerification {
    pub fn new(fft_size: usize) -> Self {
        let context = AudioContext::default();
        let analyzer = FftAnalyzer::new(fft_size);

        Self {
            context,
            analyzer,
        }
    }

    /// Test AnalyserNode accuracy against reference FFT implementation
    pub fn test_analyser_accuracy(&self, generator: &dyn SignalGenerator) -> TestSuite {
        let mut suite = TestSuite::new();
        let duration = 1.0; // 1 second test signal

        // Generate test signal
        let samples = generator.generate(duration, SAMPLE_RATE);

        // Reference analysis using our FFT implementation
        let reference_analysis = self.analyzer.analyze(&samples, SAMPLE_RATE);

        // Create audio buffer and connect to AnalyserNode
        let mut buffer = self.context.create_buffer(1, samples.len(), SAMPLE_RATE);
        buffer.copy_to_channel(&samples, 0);

        let mut source = self.context.create_buffer_source();
        source.set_buffer(buffer);

        let mut analyser = self.context.create_analyser();
        analyser.set_fft_size(self.analyzer.fft_size);

        source.connect(&analyser);
        analyser.connect(&self.context.destination());

        // Start playback and analyze
        source.start();

        // Wait a bit for the analyser to process
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Get frequency data from AnalyserNode
        let mut web_audio_data = vec![0.0f32; analyser.frequency_bin_count()];
        analyser.get_float_frequency_data(&mut web_audio_data);

        // Compare reference vs web audio API results
        for (i, (&reference_mag, &web_audio_db)) in reference_analysis.magnitudes
            .iter()
            .zip(&web_audio_data)
            .enumerate() {

            // Convert web audio dB to linear magnitude
            let web_audio_mag = if web_audio_db == -f32::INFINITY {
                0.0
            } else {
                10.0f32.powf(web_audio_db / 20.0)
            };

            let result = TestResult::new(
                &format!("Bin {} ({:.1} Hz)", i, reference_analysis.frequencies[i]),
                reference_mag,
                web_audio_mag,
                TOLERANCE * 10.0, // More lenient tolerance for web audio comparison
            );
            suite.add_result(result);
        }

        suite
    }

    /// Test frequency resolution and accuracy
    pub fn test_frequency_resolution(&self) -> TestSuite {
        let mut suite = TestSuite::new();

        // Test with known frequencies
        let test_frequencies = vec![440.0, 1000.0, 2000.0, 5000.0, 10000.0];

        for &freq in &test_frequencies {
            let generator = SineGenerator::new(freq);
            let samples = generator.generate(2.0, SAMPLE_RATE); // 2 second signal for good resolution

            let analysis = self.analyzer.analyze(&samples, SAMPLE_RATE);

            if let Some((detected_freq, magnitude)) = analysis.find_peak() {
                let freq_error = (detected_freq - freq).abs();
                let bin_resolution = SAMPLE_RATE / self.analyzer.fft_size as f32;

                let result = TestResult::new(
                    &format!("Frequency detection: {:.1} Hz", freq),
                    freq,
                    detected_freq,
                    bin_resolution / 2.0, // Allow half-bin accuracy
                );
                suite.add_result(result);

                // Test that magnitude is significant
                let mag_result = TestResult::new(
                    &format!("Magnitude at {:.1} Hz", freq),
                    0.5, // Expected magnitude for unit amplitude sine
                    magnitude,
                    0.1,
                );
                suite.add_result(mag_result);
            }
        }

        suite
    }

    /// Test harmonic analysis
    pub fn test_harmonic_analysis(&self) -> TestSuite {
        let mut suite = TestSuite::new();

        // Generate signal with fundamental and harmonics
        let fundamental = 200.0; // Low frequency for multiple harmonics
        let generator = presets::harmonic_test_signal(fundamental);
        let samples = generator.generate(2.0, SAMPLE_RATE);

        let analysis = self.analyzer.analyze(&samples, SAMPLE_RATE);

        // Test detection of fundamental and harmonics
        let expected_harmonics = vec![
            (fundamental, 1.0),
            (fundamental * 2.0, 0.5),
            (fundamental * 3.0, 0.33),
            (fundamental * 4.0, 0.25),
            (fundamental * 5.0, 0.2),
        ];

        for (harmonic_freq, expected_amplitude) in expected_harmonics {
            if let Some(detected_magnitude) = analysis.magnitude_at_frequency(harmonic_freq) {
                let result = TestResult::new(
                    &format!("Harmonic at {:.1} Hz", harmonic_freq),
                    expected_amplitude,
                    detected_magnitude,
                    0.1, // 10% tolerance for harmonic amplitudes
                );
                suite.add_result(result);
            }
        }

        // Test THD calculation
        if let Some(thd) = analysis.calculate_thd(fundamental, 5) {
            // Expected THD for the harmonic series: sqrt(0.5^2 + 0.33^2 + 0.25^2 + 0.2^2) * 100
            let expected_thd = (0.5f32*0.5 + 0.33*0.33 + 0.25*0.25 + 0.2*0.2).sqrt() * 100.0;

            let result = TestResult::new(
                "Total Harmonic Distortion",
                expected_thd,
                thd,
                5.0, // 5% tolerance for THD
            );
            suite.add_result(result);
        }

        suite
    }

    /// Test noise floor and dynamic range
    pub fn test_dynamic_range(&self) -> TestSuite {
        let mut suite = TestSuite::new();

        // Generate quiet sine wave
        let generator = SineGenerator::new(1000.0).with_amplitude(0.001); // -60 dB
        let samples = generator.generate(2.0, SAMPLE_RATE);

        let analysis = self.analyzer.analyze(&samples, SAMPLE_RATE);

        if let Some((peak_freq, peak_magnitude)) = analysis.find_peak() {
            // Check that we can detect the quiet signal
            let result = TestResult::new(
                "Dynamic Range: Quiet signal detection",
                1000.0,
                peak_freq,
                10.0, // 10 Hz tolerance
            );
            suite.add_result(result);

            // Check magnitude is approximately correct
            let mag_result = TestResult::new(
                "Dynamic Range: Signal amplitude",
                0.001,
                peak_magnitude,
                0.0005, // 0.05% tolerance
            );
            suite.add_result(mag_result);
        }

        // Test noise floor with pure silence
        let silence = vec![0.0; (2.0 * SAMPLE_RATE) as usize];
        let silence_analysis = self.analyzer.analyze(&silence, SAMPLE_RATE);

        let max_noise = silence_analysis.magnitudes.iter()
            .skip(1) // Skip DC bin
            .fold(0.0f32, |max, &mag| max.max(mag));

        let noise_result = TestResult::new(
            "Noise floor",
            0.0,
            max_noise,
            1e-6, // Very low noise floor expected
        );
        suite.add_result(noise_result);

        suite
    }
}

/// Comprehensive spectrum analysis test suite
pub fn run_spectrum_tests() -> TestSuite {
    let mut master_suite = TestSuite::new();

    println!("Running FFT Accuracy Tests...");

    // Test different FFT sizes
    let fft_sizes = vec![512, 1024, 2048, 4096];

    for &fft_size in &fft_sizes {
        println!("Testing FFT size: {}", fft_size);

        let verifier = AnalyserVerification::new(fft_size);

        // Test frequency resolution
        let mut resolution_suite = verifier.test_frequency_resolution();
        master_suite.results.append(&mut resolution_suite.results);

        // Test with different signal types
        let generators: Vec<Box<dyn SignalGenerator>> = vec![
            Box::new(presets::sine_1khz()),
            Box::new(presets::sine_a4()),
            Box::new(presets::quiet_white_noise()),
            Box::new(presets::imd_test_signal()),
        ];

        for generator in generators {
            let mut accuracy_suite = verifier.test_analyser_accuracy(generator.as_ref());
            master_suite.results.append(&mut accuracy_suite.results);
        }

        // Test harmonic analysis
        let mut harmonic_suite = verifier.test_harmonic_analysis();
        master_suite.results.append(&mut harmonic_suite.results);

        // Test dynamic range
        let mut dynamic_suite = verifier.test_dynamic_range();
        master_suite.results.append(&mut dynamic_suite.results);
    }

    master_suite
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_fft_analyzer_sine_wave() {
        let analyzer = FftAnalyzer::new(1024);
        let generator = SineGenerator::new(1000.0);
        let samples = generator.generate(1.0, 44100.0);

        let analysis = analyzer.analyze(&samples, 44100.0);

        // Should detect peak near 1000 Hz
        if let Some((peak_freq, _)) = analysis.find_peak() {
            assert_abs_diff_eq!(peak_freq, 1000.0, epsilon = 50.0);
        } else {
            panic!("No peak detected");
        }
    }

    #[test]
    fn test_spectrum_analysis_magnitude() {
        let analyzer = FftAnalyzer::new(2048);
        let generator = SineGenerator::new(1000.0).with_amplitude(0.5);
        let samples = generator.generate(2.0, 44100.0);

        let analysis = analyzer.analyze(&samples, 44100.0);

        if let Some(magnitude) = analysis.magnitude_at_frequency(1000.0) {
            // Should be approximately 0.5 for a 0.5 amplitude sine wave
            assert_abs_diff_eq!(magnitude, 0.5, epsilon = 0.1);
        } else {
            panic!("Could not measure magnitude at 1000 Hz");
        }
    }

    #[test]
    fn test_multi_tone_detection() {
        let analyzer = FftAnalyzer::new(4096);
        let generator = MultiToneGenerator::new(vec![440.0, 880.0, 1320.0])
            .with_amplitudes(vec![1.0, 0.5, 0.25]);
        let samples = generator.generate(2.0, 44100.0);

        let analysis = analyzer.analyze(&samples, 44100.0);
        let peaks = analysis.find_peaks(0.1);

        // Should detect all three peaks
        assert!(peaks.len() >= 3, "Expected at least 3 peaks, found {}", peaks.len());

        // Check that major peaks are detected
        let peak_frequencies: Vec<f32> = peaks.iter().map(|(f, _)| *f).collect();
        assert!(peak_frequencies.iter().any(|&f| (f - 440.0).abs() < 20.0));
        assert!(peak_frequencies.iter().any(|&f| (f - 880.0).abs() < 20.0));
        assert!(peak_frequencies.iter().any(|&f| (f - 1320.0).abs() < 20.0));
    }
}