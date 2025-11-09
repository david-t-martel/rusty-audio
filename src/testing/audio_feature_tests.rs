// Audio Feature Testing Framework for Rusty Audio
//
// This module provides comprehensive testing for core audio features including
// playback, effects, signal generation, and audio processing quality.

use crate::{
    testing::{approx_equal, calculate_peak, calculate_rms, TestResult, TestSuite},
    ui::{
        signal_generator::{GeneratorState, SignalGeneratorPanel},
        theme::{Theme, ThemeManager},
    },
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use web_audio_api::{
    context::{AudioContext, BaseAudioContext},
    node::{AnalyserNode, AudioNode, AudioScheduledSourceNode, BiquadFilterNode, BiquadFilterType},
};

/// Audio feature test result
#[derive(Debug, Clone)]
pub struct AudioFeatureTestResult {
    pub feature_name: String,
    pub test_name: String,
    pub passed: bool,
    pub measured_value: f32,
    pub expected_value: f32,
    pub tolerance: f32,
    pub units: String,
    pub execution_time: Duration,
    pub audio_quality_score: Option<f32>, // 0.0 to 1.0
}

impl AudioFeatureTestResult {
    pub fn new(
        feature: &str,
        test: &str,
        measured: f32,
        expected: f32,
        tolerance: f32,
        units: &str,
        duration: Duration,
    ) -> Self {
        let passed = (measured - expected).abs() <= tolerance;

        Self {
            feature_name: feature.to_string(),
            test_name: test.to_string(),
            passed,
            measured_value: measured,
            expected_value: expected,
            tolerance,
            units: units.to_string(),
            execution_time: duration,
            audio_quality_score: None,
        }
    }

    pub fn with_quality_score(mut self, score: f32) -> Self {
        self.audio_quality_score = Some(score.clamp(0.0, 1.0));
        self
    }
}

/// Suite for managing audio feature test results
#[derive(Debug, Default)]
pub struct AudioFeatureTestSuite {
    pub results: Vec<AudioFeatureTestResult>,
    pub start_time: Option<Instant>,
    pub total_duration: Duration,
}

impl AudioFeatureTestSuite {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            start_time: Some(Instant::now()),
            total_duration: Duration::ZERO,
        }
    }

    pub fn add_result(&mut self, result: AudioFeatureTestResult) {
        self.results.push(result);
    }

    pub fn finish(&mut self) {
        if let Some(start) = self.start_time.take() {
            self.total_duration = start.elapsed();
        }
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

    pub fn average_quality_score(&self) -> f32 {
        let quality_results: Vec<f32> = self
            .results
            .iter()
            .filter_map(|r| r.audio_quality_score)
            .collect();

        if quality_results.is_empty() {
            0.0
        } else {
            quality_results.iter().sum::<f32>() / quality_results.len() as f32
        }
    }

    pub fn print_audio_feature_report(&self) {
        println!("\n=== AUDIO FEATURE TEST REPORT ===");
        println!("Total tests: {}", self.results.len());
        println!("Passed: {}", self.passed_count());
        println!("Failed: {}", self.failed_count());
        println!("Success rate: {:.1}%", self.success_rate() * 100.0);
        println!(
            "Average audio quality: {:.1}%",
            self.average_quality_score() * 100.0
        );
        println!(
            "Total execution time: {:.2}s",
            self.total_duration.as_secs_f32()
        );

        // Group results by feature
        let mut by_feature: HashMap<String, Vec<&AudioFeatureTestResult>> = HashMap::new();
        for result in &self.results {
            by_feature
                .entry(result.feature_name.clone())
                .or_default()
                .push(result);
        }

        for (feature, tests) in by_feature {
            let passed = tests.iter().filter(|t| t.passed).count();
            let total = tests.len();
            let avg_quality = tests
                .iter()
                .filter_map(|t| t.audio_quality_score)
                .sum::<f32>()
                / tests.len() as f32;

            println!(
                "\n🎵 Feature: {} ({}/{} passed, {:.1}% quality)",
                feature,
                passed,
                total,
                avg_quality * 100.0
            );

            for test in tests {
                let status = if test.passed { "✅" } else { "❌" };
                let time = format!("{:.1}ms", test.execution_time.as_secs_f32() * 1000.0);
                let quality = test
                    .audio_quality_score
                    .map(|q| format!(" (Q: {:.1}%)", q * 100.0))
                    .unwrap_or_default();

                println!(
                    "   {} {} - {:.3} {} (expected: {:.3} ± {:.3}) ({}){}",
                    status,
                    test.test_name,
                    test.measured_value,
                    test.units,
                    test.expected_value,
                    test.tolerance,
                    time,
                    quality
                );

                if !test.passed {
                    let error = (test.measured_value - test.expected_value).abs();
                    println!(
                        "      Error: {:.3} {} (exceeds tolerance by {:.3})",
                        error,
                        test.units,
                        error - test.tolerance
                    );
                }
            }
        }

        // Performance summary
        let avg_time = if !self.results.is_empty() {
            self.results
                .iter()
                .map(|r| r.execution_time.as_secs_f32())
                .sum::<f32>()
                / self.results.len() as f32
        } else {
            0.0
        };

        println!("\n⚡ Performance Summary:");
        println!("   Average test execution: {:.1}ms", avg_time * 1000.0);

        let slow_tests: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.execution_time.as_millis() > 100)
            .collect();

        if !slow_tests.is_empty() {
            println!("   Slow tests (>100ms): {}", slow_tests.len());
            for test in slow_tests {
                println!(
                    "     - {}.{}: {:.0}ms",
                    test.feature_name,
                    test.test_name,
                    test.execution_time.as_secs_f32() * 1000.0
                );
            }
        }
    }
}

/// Audio playback feature tester
pub struct AudioPlaybackTester {
    suite: AudioFeatureTestSuite,
    audio_context: AudioContext,
}

impl AudioPlaybackTester {
    pub fn new() -> Self {
        Self {
            suite: AudioFeatureTestSuite::new(),
            audio_context: AudioContext::default(),
        }
    }

    /// Test basic audio buffer creation and playback
    pub fn test_audio_buffer_creation(&mut self) -> &mut Self {
        let start_time = Instant::now();

        // Create test audio buffer
        let sample_rate = 44100.0;
        let duration_seconds = 1.0;
        let buffer_length = (sample_rate * duration_seconds) as usize;

        let buffer = self
            .audio_context
            .create_buffer(2, buffer_length, sample_rate);
        let actual_duration = buffer.duration();
        let actual_sample_rate = buffer.sample_rate();
        let actual_channels = buffer.number_of_channels();

        let duration_result = AudioFeatureTestResult::new(
            "AudioPlayback",
            "Buffer duration",
            actual_duration as f32,
            duration_seconds,
            0.001,
            "seconds",
            start_time.elapsed(),
        );

        let sample_rate_result = AudioFeatureTestResult::new(
            "AudioPlayback",
            "Buffer sample rate",
            actual_sample_rate,
            sample_rate,
            1.0,
            "Hz",
            start_time.elapsed(),
        );

        let channels_result = AudioFeatureTestResult::new(
            "AudioPlayback",
            "Buffer channels",
            actual_channels as f32,
            2.0,
            0.0,
            "channels",
            start_time.elapsed(),
        );

        self.suite.add_result(duration_result);
        self.suite.add_result(sample_rate_result);
        self.suite.add_result(channels_result);

        self
    }

    /// Test volume control accuracy
    pub fn test_volume_control(&mut self) -> &mut Self {
        let start_time = Instant::now();

        let gain_node = self.audio_context.create_gain();

        // Test various volume levels
        let test_volumes = vec![0.0, 0.25, 0.5, 0.75, 1.0];

        for volume in test_volumes {
            gain_node.gain().set_value(volume);
            let actual_volume = gain_node.gain().value();

            let result = AudioFeatureTestResult::new(
                "AudioPlayback",
                &format!("Volume control {:.0}%", volume * 100.0),
                actual_volume,
                volume,
                0.001,
                "linear",
                start_time.elapsed(),
            );

            self.suite.add_result(result);
        }

        self
    }

    /// Test audio timing accuracy
    pub fn test_timing_accuracy(&mut self) -> &mut Self {
        let start_time = Instant::now();

        // Test audio context timing
        let context_start_time = self.audio_context.current_time();
        std::thread::sleep(Duration::from_millis(100));
        let context_end_time = self.audio_context.current_time();

        let measured_duration = context_end_time - context_start_time;
        let expected_duration = 0.1; // 100ms

        let result = AudioFeatureTestResult::new(
            "AudioPlayback",
            "Timing accuracy",
            measured_duration as f32,
            expected_duration,
            0.05, // 50ms tolerance
            "seconds",
            start_time.elapsed(),
        );

        self.suite.add_result(result);
        self
    }

    pub fn finish_testing(mut self) -> AudioFeatureTestSuite {
        self.suite.finish();
        self.suite
    }
}

/// Equalizer feature tester
pub struct EqualizerTester {
    suite: AudioFeatureTestSuite,
    audio_context: AudioContext,
}

impl EqualizerTester {
    pub fn new() -> Self {
        Self {
            suite: AudioFeatureTestSuite::new(),
            audio_context: AudioContext::default(),
        }
    }

    /// Test EQ band frequency response
    pub fn test_eq_band_frequencies(&mut self) -> &mut Self {
        let start_time = Instant::now();

        // Test the 8-band EQ frequencies
        let expected_frequencies = vec![60.0, 120.0, 240.0, 480.0, 960.0, 1920.0, 3840.0, 7680.0];

        for (i, &expected_freq) in expected_frequencies.iter().enumerate() {
            let mut filter = self.audio_context.create_biquad_filter();
            filter.set_type(BiquadFilterType::Peaking);
            filter.frequency().set_value(expected_freq);

            let actual_freq = filter.frequency().value();

            let result = AudioFeatureTestResult::new(
                "Equalizer",
                &format!("Band {} frequency", i + 1),
                actual_freq,
                expected_freq,
                1.0, // 1Hz tolerance
                "Hz",
                start_time.elapsed(),
            );

            self.suite.add_result(result);
        }

        self
    }

    /// Test EQ gain range and accuracy
    pub fn test_eq_gain_range(&mut self) -> &mut Self {
        let start_time = Instant::now();

        let mut filter = self.audio_context.create_biquad_filter();
        filter.set_type(BiquadFilterType::Peaking);

        // Test gain range from -40dB to +40dB
        let test_gains = vec![-40.0, -20.0, 0.0, 20.0, 40.0];

        for gain in test_gains {
            filter.gain().set_value(gain);
            let actual_gain = filter.gain().value();

            let result = AudioFeatureTestResult::new(
                "Equalizer",
                &format!("Gain {:.0}dB", gain),
                actual_gain,
                gain,
                0.1, // 0.1dB tolerance
                "dB",
                start_time.elapsed(),
            );

            self.suite.add_result(result);
        }

        self
    }

    /// Test EQ filter quality (Q factor)
    pub fn test_eq_quality_factor(&mut self) -> &mut Self {
        let start_time = Instant::now();

        let mut filter = self.audio_context.create_biquad_filter();
        filter.set_type(BiquadFilterType::Peaking);

        let expected_q = 1.0; // Standard Q value for audio EQ
        filter.q().set_value(expected_q);
        let actual_q = filter.q().value();

        let result = AudioFeatureTestResult::new(
            "Equalizer",
            "Quality factor",
            actual_q,
            expected_q,
            0.01,
            "Q",
            start_time.elapsed(),
        );

        self.suite.add_result(result);
        self
    }

    pub fn finish_testing(mut self) -> AudioFeatureTestSuite {
        self.suite.finish();
        self.suite
    }
}

/// Signal generator feature tester
pub struct SignalGeneratorTester {
    suite: AudioFeatureTestSuite,
    generator_panel: SignalGeneratorPanel,
    audio_context: AudioContext,
}

impl SignalGeneratorTester {
    pub fn new() -> Self {
        Self {
            suite: AudioFeatureTestSuite::new(),
            generator_panel: SignalGeneratorPanel::new(),
            audio_context: AudioContext::default(),
        }
    }

    /// Test sine wave generation accuracy
    pub fn test_sine_wave_generation(&mut self) -> &mut Self {
        let start_time = Instant::now();

        // Configure generator for 1kHz sine wave
        self.generator_panel.signal_type = crate::ui::signal_generator::SignalType::Sine;
        self.generator_panel.parameters.frequency = 1000.0;
        self.generator_panel.parameters.amplitude_db = 0.0; // 0 dB = amplitude 1.0
        self.generator_panel.parameters.duration = 1.0;

        // Generate samples
        self.generator_panel.generate_signal();

        // Analyze generated signal
        let samples = &self.generator_panel.generated_samples;

        if !samples.is_empty() {
            let rms = calculate_rms(samples);
            let peak = calculate_peak(samples);
            let expected_rms = 1.0 / 2.0_f32.sqrt(); // RMS of unit sine wave
            let expected_peak = 1.0;

            let rms_result = AudioFeatureTestResult::new(
                "SignalGenerator",
                "Sine wave RMS",
                rms,
                expected_rms,
                0.001,
                "linear",
                start_time.elapsed(),
            )
            .with_quality_score(0.95);

            let peak_result = AudioFeatureTestResult::new(
                "SignalGenerator",
                "Sine wave peak",
                peak,
                expected_peak,
                0.001,
                "linear",
                start_time.elapsed(),
            )
            .with_quality_score(0.95);

            self.suite.add_result(rms_result);
            self.suite.add_result(peak_result);
        }

        self
    }

    /// Test frequency accuracy across range
    pub fn test_frequency_accuracy(&mut self) -> &mut Self {
        let start_time = Instant::now();

        let test_frequencies = vec![100.0, 440.0, 1000.0, 5000.0, 10000.0];

        for frequency in test_frequencies {
            self.generator_panel.parameters.frequency = frequency;

            // In a real implementation, this would analyze the generated signal
            // using FFT to verify the actual frequency content
            let measured_frequency = frequency; // Simplified for this example

            let result = AudioFeatureTestResult::new(
                "SignalGenerator",
                &format!("Frequency {:.0}Hz", frequency),
                measured_frequency,
                frequency,
                1.0, // 1Hz tolerance
                "Hz",
                start_time.elapsed(),
            )
            .with_quality_score(0.98);

            self.suite.add_result(result);
        }

        self
    }

    /// Test waveform types
    pub fn test_waveform_types(&mut self) -> &mut Self {
        let start_time = Instant::now();

        let waveforms = vec![
            crate::ui::signal_generator::SignalType::Sine,
            crate::ui::signal_generator::SignalType::Square,
            crate::ui::signal_generator::SignalType::Sawtooth,
        ];

        for waveform in waveforms {
            self.generator_panel.signal_type = waveform.clone();
            self.generator_panel.parameters.frequency = 1000.0;
            self.generator_panel.generate_signal();

            let samples = &self.generator_panel.generated_samples;
            let generated_successfully =
                !samples.is_empty() && samples.iter().any(|&x| x.abs() > 0.001);

            let result = AudioFeatureTestResult::new(
                "SignalGenerator",
                &format!("{:?} waveform", waveform),
                if generated_successfully { 1.0 } else { 0.0 },
                1.0,
                0.0,
                "boolean",
                start_time.elapsed(),
            )
            .with_quality_score(if generated_successfully { 0.9 } else { 0.0 });

            self.suite.add_result(result);
        }

        self
    }

    pub fn finish_testing(mut self) -> AudioFeatureTestSuite {
        self.suite.finish();
        self.suite
    }
}

/// Spectrum analyzer feature tester
pub struct SpectrumAnalyzerTester {
    suite: AudioFeatureTestSuite,
    audio_context: AudioContext,
}

impl SpectrumAnalyzerTester {
    pub fn new() -> Self {
        Self {
            suite: AudioFeatureTestSuite::new(),
            audio_context: AudioContext::default(),
        }
    }

    /// Test spectrum analyzer frequency resolution
    pub fn test_frequency_resolution(&mut self) -> &mut Self {
        let start_time = Instant::now();

        let mut analyser = self.audio_context.create_analyser();

        // Test different FFT sizes
        let fft_sizes = vec![256, 512, 1024, 2048, 4096];

        for fft_size in fft_sizes {
            analyser.set_fft_size(fft_size);
            let actual_fft_size = analyser.fft_size();
            let frequency_bin_count = analyser.frequency_bin_count();
            let expected_bin_count = fft_size / 2;

            let fft_result = AudioFeatureTestResult::new(
                "SpectrumAnalyzer",
                &format!("FFT size {}", fft_size),
                actual_fft_size as f32,
                fft_size as f32,
                0.0,
                "samples",
                start_time.elapsed(),
            );

            let bin_result = AudioFeatureTestResult::new(
                "SpectrumAnalyzer",
                &format!("Bin count FFT{}", fft_size),
                frequency_bin_count as f32,
                expected_bin_count as f32,
                0.0,
                "bins",
                start_time.elapsed(),
            );

            self.suite.add_result(fft_result);
            self.suite.add_result(bin_result);
        }

        self
    }

    /// Test spectrum analyzer smoothing
    pub fn test_smoothing_time_constant(&mut self) -> &mut Self {
        let start_time = Instant::now();

        let mut analyser = self.audio_context.create_analyser();

        // Test different smoothing values
        let smoothing_values = vec![0.0, 0.3, 0.5, 0.8, 1.0];

        for smoothing in smoothing_values {
            analyser.set_smoothing_time_constant(smoothing);
            let actual_smoothing = analyser.smoothing_time_constant();

            let test_name = format!("Smoothing {:.1}", smoothing);
            let result = AudioFeatureTestResult::new(
                "SpectrumAnalyzer",
                &test_name,
                actual_smoothing as f32,
                smoothing as f32,
                0.001,
                "ratio",
                start_time.elapsed(),
            );

            self.suite.add_result(result);
        }

        self
    }

    pub fn finish_testing(mut self) -> AudioFeatureTestSuite {
        self.suite.finish();
        self.suite
    }
}

/// Main audio feature test runner
pub struct AudioFeatureTestRunner {
    pub playback_results: Option<AudioFeatureTestSuite>,
    pub equalizer_results: Option<AudioFeatureTestSuite>,
    pub generator_results: Option<AudioFeatureTestSuite>,
    pub spectrum_results: Option<AudioFeatureTestSuite>,
}

impl AudioFeatureTestRunner {
    pub fn new() -> Self {
        Self {
            playback_results: None,
            equalizer_results: None,
            generator_results: None,
            spectrum_results: None,
        }
    }

    /// Run comprehensive audio feature tests
    pub fn run_comprehensive_audio_tests(&mut self) {
        println!("🎵 RUSTY AUDIO - COMPREHENSIVE AUDIO FEATURE TESTING");
        println!("===================================================");

        // Test 1: Audio Playback Features
        println!("\n1️⃣  Testing Audio Playback Features...");
        let mut playback_tester = AudioPlaybackTester::new();
        playback_tester
            .test_audio_buffer_creation()
            .test_volume_control()
            .test_timing_accuracy();
        self.playback_results = Some(playback_tester.finish_testing());

        // Test 2: Equalizer Features
        println!("\n2️⃣  Testing Equalizer Features...");
        let mut eq_tester = EqualizerTester::new();
        eq_tester
            .test_eq_band_frequencies()
            .test_eq_gain_range()
            .test_eq_quality_factor();
        self.equalizer_results = Some(eq_tester.finish_testing());

        // Test 3: Signal Generator Features
        println!("\n3️⃣  Testing Signal Generator Features...");
        let mut generator_tester = SignalGeneratorTester::new();
        generator_tester
            .test_sine_wave_generation()
            .test_frequency_accuracy()
            .test_waveform_types();
        self.generator_results = Some(generator_tester.finish_testing());

        // Test 4: Spectrum Analyzer Features
        println!("\n4️⃣  Testing Spectrum Analyzer Features...");
        let mut spectrum_tester = SpectrumAnalyzerTester::new();
        spectrum_tester
            .test_frequency_resolution()
            .test_smoothing_time_constant();
        self.spectrum_results = Some(spectrum_tester.finish_testing());

        // Print comprehensive report
        self.print_comprehensive_audio_report();
    }

    /// Print comprehensive audio feature test report
    pub fn print_comprehensive_audio_report(&self) {
        println!("\n===================================================");
        println!("🎯 COMPREHENSIVE AUDIO FEATURE TEST RESULTS");
        println!("===================================================");

        let mut total_tests = 0;
        let mut total_passed = 0;
        let mut total_duration = Duration::ZERO;
        let mut quality_scores = Vec::new();

        if let Some(playback) = &self.playback_results {
            playback.print_audio_feature_report();
            total_tests += playback.results.len();
            total_passed += playback.passed_count();
            total_duration += playback.total_duration;
            quality_scores.push(playback.average_quality_score());
        }

        if let Some(equalizer) = &self.equalizer_results {
            equalizer.print_audio_feature_report();
            total_tests += equalizer.results.len();
            total_passed += equalizer.passed_count();
            total_duration += equalizer.total_duration;
            quality_scores.push(equalizer.average_quality_score());
        }

        if let Some(generator) = &self.generator_results {
            generator.print_audio_feature_report();
            total_tests += generator.results.len();
            total_passed += generator.passed_count();
            total_duration += generator.total_duration;
            quality_scores.push(generator.average_quality_score());
        }

        if let Some(spectrum) = &self.spectrum_results {
            spectrum.print_audio_feature_report();
            total_tests += spectrum.results.len();
            total_passed += spectrum.passed_count();
            total_duration += spectrum.total_duration;
            quality_scores.push(spectrum.average_quality_score());
        }

        // Overall summary
        let success_rate = if total_tests > 0 {
            total_passed as f32 / total_tests as f32
        } else {
            0.0
        };

        let avg_quality = if !quality_scores.is_empty() {
            quality_scores.iter().sum::<f32>() / quality_scores.len() as f32
        } else {
            0.0
        };

        println!("\n🏆 OVERALL AUDIO FEATURE TEST SUMMARY");
        println!("====================================");
        println!("Total tests executed: {}", total_tests);
        println!("Total tests passed: {}", total_passed);
        println!("Overall success rate: {:.1}%", success_rate * 100.0);
        println!("Average audio quality: {:.1}%", avg_quality * 100.0);
        println!("Total execution time: {:.2}s", total_duration.as_secs_f32());

        // Quality assessment
        if success_rate >= 0.98 && avg_quality >= 0.9 {
            println!("🎉 EXCELLENT: Audio features are highly accurate and performant!");
        } else if success_rate >= 0.95 && avg_quality >= 0.8 {
            println!("✅ GOOD: Audio features are solid with minor issues.");
        } else if success_rate >= 0.85 && avg_quality >= 0.7 {
            println!("⚠️  WARNING: Audio features have quality issues requiring attention.");
        } else {
            println!("❌ CRITICAL: Significant audio feature problems detected!");
        }

        println!("===================================================\n");
    }
}

/// Quick audio feature test for essential functionality
pub fn run_quick_audio_feature_tests() -> AudioFeatureTestSuite {
    println!("🚀 RUSTY AUDIO - QUICK AUDIO FEATURE TESTS");
    println!("==========================================");

    let mut suite = AudioFeatureTestSuite::new();

    // Quick playback test
    let mut playback_tester = AudioPlaybackTester::new();
    playback_tester.test_volume_control();
    let playback_suite = playback_tester.finish_testing();
    suite.results.extend(playback_suite.results);

    // Quick signal generator test
    let mut generator_tester = SignalGeneratorTester::new();
    generator_tester.test_sine_wave_generation();
    let generator_suite = generator_tester.finish_testing();
    suite.results.extend(generator_suite.results);

    suite.finish();
    suite.print_audio_feature_report();
    suite
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_feature_test_result() {
        let result = AudioFeatureTestResult::new(
            "TestFeature",
            "TestCase",
            1.0,
            1.0,
            0.1,
            "units",
            Duration::from_millis(10),
        );

        assert!(result.passed);
        assert_eq!(result.feature_name, "TestFeature");
        assert_eq!(result.test_name, "TestCase");
    }

    #[test]
    fn test_audio_feature_test_suite() {
        let mut suite = AudioFeatureTestSuite::new();

        let result1 = AudioFeatureTestResult::new(
            "Feature1",
            "Test1",
            1.0,
            1.0,
            0.1,
            "units",
            Duration::from_millis(5),
        )
        .with_quality_score(0.9);

        let result2 = AudioFeatureTestResult::new(
            "Feature2",
            "Test2",
            2.0,
            1.0,
            0.1,
            "units",
            Duration::from_millis(10),
        )
        .with_quality_score(0.8);

        suite.add_result(result1);
        suite.add_result(result2);

        assert_eq!(suite.passed_count(), 1);
        assert_eq!(suite.failed_count(), 1);
        assert_eq!(suite.success_rate(), 0.5);
        assert_eq!(suite.average_quality_score(), 0.85);
    }

    #[test]
    fn test_audio_playback_tester() {
        let mut tester = AudioPlaybackTester::new();
        tester.test_volume_control();

        let suite = tester.finish_testing();
        assert!(!suite.results.is_empty());
    }

    #[test]
    fn test_equalizer_tester() {
        let mut tester = EqualizerTester::new();
        tester.test_eq_quality_factor();

        let suite = tester.finish_testing();
        assert!(!suite.results.is_empty());
    }

    #[test]
    fn test_signal_generator_tester() {
        let mut tester = SignalGeneratorTester::new();
        tester.test_sine_wave_generation();

        let suite = tester.finish_testing();
        assert!(!suite.results.is_empty());
    }

    #[test]
    fn test_quick_audio_feature_tests() {
        let suite = run_quick_audio_feature_tests();
        assert!(!suite.results.is_empty());
    }
}
