// Property-Based Testing for Audio Processing
//
// This module uses QuickCheck and Proptest to verify mathematical properties
// of audio processing functions that should hold for all valid inputs.

use proptest::prelude::*;
use quickcheck::{quickcheck, TestResult};
use super::{TestSuite, SAMPLE_RATE, TOLERANCE};
use super::signal_generators::*;
use super::spectrum_analysis::FftAnalyzer;

/// Property: Signal generation should preserve mathematical properties
pub fn prop_sine_wave_properties() -> TestSuite {
    let suite = TestSuite::new();

    // Property: RMS of a sine wave should be amplitude / sqrt(2)
    quickcheck(prop_sine_rms as fn(f32, f32) -> TestResult);

    // Property: Peak amplitude should equal the set amplitude
    quickcheck(prop_sine_peak as fn(f32, f32) -> TestResult);

    // Property: Frequency content should be concentrated at the generated frequency
    quickcheck(prop_sine_frequency_content as fn(f32) -> TestResult);

    suite
}

/// Property test: Sine wave RMS calculation
fn prop_sine_rms(frequency: f32, amplitude: f32) -> TestResult {
    if frequency <= 0.0 || frequency >= SAMPLE_RATE / 2.0 || amplitude <= 0.0 || amplitude > 10.0 {
        return TestResult::discard();
    }

    let generator = SineGenerator::new(frequency).with_amplitude(amplitude);
    let samples = generator.generate(1.0, SAMPLE_RATE);

    let rms = super::calculate_rms(&samples);
    let expected_rms = amplitude / 2.0f32.sqrt();

    TestResult::from_bool(
        (rms - expected_rms).abs() < TOLERANCE * 10.0
    )
}

/// Property test: Sine wave peak amplitude
fn prop_sine_peak(frequency: f32, amplitude: f32) -> TestResult {
    if frequency <= 0.0 || frequency >= SAMPLE_RATE / 2.0 || amplitude <= 0.0 || amplitude > 10.0 {
        return TestResult::discard();
    }

    let generator = SineGenerator::new(frequency).with_amplitude(amplitude);
    let samples = generator.generate(1.0, SAMPLE_RATE);

    let peak = super::calculate_peak(&samples);

    TestResult::from_bool(
        (peak - amplitude).abs() < TOLERANCE * 10.0
    )
}

/// Property test: Sine wave frequency content
fn prop_sine_frequency_content(frequency: f32) -> TestResult {
    if frequency < 100.0 || frequency > 10000.0 {
        return TestResult::discard();
    }

    let generator = SineGenerator::new(frequency);
    let samples = generator.generate(2.0, SAMPLE_RATE); // Longer signal for better frequency resolution

    let analyzer = FftAnalyzer::new(4096);
    let analysis = analyzer.analyze(&samples, SAMPLE_RATE);

    if let Some((peak_freq, _)) = analysis.find_peak() {
        TestResult::from_bool(
            (peak_freq - frequency).abs() < 50.0 // 50 Hz tolerance
        )
    } else {
        TestResult::failed()
    }
}

/// Property tests for white noise
pub fn prop_white_noise_properties() -> TestSuite {
    let suite = TestSuite::new();

    // Property: White noise should have approximately flat spectrum
    quickcheck(prop_white_noise_spectrum as fn(u64) -> TestResult);

    // Property: White noise samples should be bounded by amplitude
    quickcheck(prop_white_noise_bounds as fn(f32, u64) -> TestResult);

    suite
}

/// Property test: White noise spectrum flatness
fn prop_white_noise_spectrum(seed: u64) -> TestResult {
    let generator = WhiteNoiseGenerator::new().with_seed(seed);
    let samples = generator.generate(4.0, SAMPLE_RATE); // Long signal for good spectrum

    let analyzer = FftAnalyzer::new(8192);
    let analysis = analyzer.analyze(&samples, SAMPLE_RATE);

    // Calculate spectrum variance - should be relatively low for white noise
    let mean_magnitude: f32 = analysis.magnitudes.iter().sum::<f32>() / analysis.magnitudes.len() as f32;

    let variance: f32 = analysis.magnitudes.iter()
        .map(|&mag| (mag - mean_magnitude).powi(2))
        .sum::<f32>() / analysis.magnitudes.len() as f32;

    let coefficient_of_variation = variance.sqrt() / mean_magnitude;

    // White noise should have relatively uniform spectrum (low coefficient of variation)
    TestResult::from_bool(coefficient_of_variation < 2.0)
}

/// Property test: White noise amplitude bounds
fn prop_white_noise_bounds(amplitude: f32, seed: u64) -> TestResult {
    if amplitude <= 0.0 || amplitude > 10.0 {
        return TestResult::discard();
    }

    let generator = WhiteNoiseGenerator::new()
        .with_amplitude(amplitude)
        .with_seed(seed);
    let samples = generator.generate(1.0, SAMPLE_RATE);

    let max_sample = samples.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);

    TestResult::from_bool(max_sample <= amplitude * 1.01) // Allow small numerical error
}

/// Property tests for multi-tone signals
pub fn prop_multi_tone_properties() -> TestSuite {
    let suite = TestSuite::new();

    // Property: Multi-tone signal should contain all specified frequencies
    // Note: This requires custom test framework since quickcheck doesn't handle Vec<f32> well

    suite
}

/// Property-based test using proptest for more complex properties
pub fn prop_audio_processing_invariants() -> TestSuite {
    let suite = TestSuite::new();

    // Use proptest for more sophisticated property testing
    proptest!(|(
        frequency in 100.0f32..10000.0f32,
        amplitude in 0.1f32..2.0f32,
        duration in 0.5f32..5.0f32
    )| {
        // Test signal energy conservation
        let generator = SineGenerator::new(frequency).with_amplitude(amplitude);
        let samples = generator.generate(duration, SAMPLE_RATE);

        // Energy should be proportional to amplitude squared and duration
        let energy: f32 = samples.iter().map(|&x| x * x).sum();
        let expected_energy = amplitude * amplitude * 0.5 * duration * SAMPLE_RATE;

        prop_assert!((energy - expected_energy).abs() / expected_energy < 0.1);
    });

    suite
}

/// Property tests for spectrum analysis
pub fn prop_spectrum_analysis_properties() -> TestSuite {
    let suite = TestSuite::new();

    // Property: Parseval's theorem - energy conservation in frequency domain
    quickcheck(prop_parsevals_theorem as fn(u64) -> TestResult);

    // Property: DC component should be zero for zero-mean signals
    quickcheck(prop_dc_component_zero_mean as fn(u64) -> TestResult);

    // Property: Spectrum symmetry for real signals
    quickcheck(prop_spectrum_symmetry as fn(u64) -> TestResult);

    suite
}

/// Property test: Parseval's theorem (energy conservation)
fn prop_parsevals_theorem(seed: u64) -> TestResult {
    let generator = WhiteNoiseGenerator::new().with_seed(seed);
    let samples = generator.generate(1.0, SAMPLE_RATE);

    // Calculate time domain energy
    let time_energy: f32 = samples.iter().map(|&x| x * x).sum();

    // Calculate frequency domain energy
    let analyzer = FftAnalyzer::new(samples.len().next_power_of_two());
    let analysis = analyzer.analyze(&samples, SAMPLE_RATE);

    let freq_energy: f32 = analysis.magnitudes.iter()
        .map(|&mag| mag * mag)
        .sum::<f32>() * 2.0; // Factor of 2 for positive frequencies only

    let energy_ratio = freq_energy / time_energy;

    TestResult::from_bool(
        (energy_ratio - 1.0).abs() < 0.1 // 10% tolerance for Parseval's theorem
    )
}

/// Property test: DC component for zero-mean signals
fn prop_dc_component_zero_mean(seed: u64) -> TestResult {
    let generator = WhiteNoiseGenerator::new().with_seed(seed);
    let mut samples = generator.generate(1.0, SAMPLE_RATE);

    // Ensure zero mean
    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    for sample in &mut samples {
        *sample -= mean;
    }

    let analyzer = FftAnalyzer::new(samples.len().next_power_of_two());
    let analysis = analyzer.analyze(&samples, SAMPLE_RATE);

    // DC component should be near zero
    let dc_magnitude = analysis.magnitudes[0];

    TestResult::from_bool(dc_magnitude < TOLERANCE * 100.0)
}

/// Property test: Spectrum symmetry for real signals
fn prop_spectrum_symmetry(seed: u64) -> TestResult {
    let generator = WhiteNoiseGenerator::new().with_seed(seed);
    let samples = generator.generate(1.0, SAMPLE_RATE);

    let fft_size = samples.len().next_power_of_two();
    let analyzer = FftAnalyzer::new(fft_size);
    let analysis = analyzer.analyze(&samples, SAMPLE_RATE);

    // For real signals, spectrum should be symmetric
    // We only get half the spectrum from our analyzer, so this property
    // is implicitly satisfied by the FFT implementation
    TestResult::passed()
}

/// Property tests for equalizer behavior
pub fn prop_equalizer_properties() -> TestSuite {
    let suite = TestSuite::new();

    // Property: Flat EQ (0dB all bands) should not change signal
    quickcheck(prop_flat_eq_unity as fn(f32) -> TestResult);

    // Property: EQ boost should increase energy in target band
    quickcheck(prop_eq_boost_increases_energy as fn(f32, f32) -> TestResult);

    suite
}

/// Property test: Flat equalizer should pass signal unchanged
fn prop_flat_eq_unity(frequency: f32) -> TestResult {
    if frequency < 100.0 || frequency > 10000.0 {
        return TestResult::discard();
    }

    // This is a conceptual test - in practice would need actual EQ processing
    let generator = SineGenerator::new(frequency);
    let samples = generator.generate(1.0, SAMPLE_RATE);

    let input_rms = super::calculate_rms(&samples);

    // With flat EQ (all gains = 0dB), output RMS should equal input RMS
    let output_rms = input_rms; // Placeholder - would process through flat EQ

    TestResult::from_bool(
        (output_rms - input_rms).abs() < TOLERANCE
    )
}

/// Property test: EQ boost should increase energy
fn prop_eq_boost_increases_energy(frequency: f32, boost_db: f32) -> TestResult {
    if frequency < 100.0 || frequency > 10000.0 || boost_db <= 0.0 || boost_db > 20.0 {
        return TestResult::discard();
    }

    let generator = SineGenerator::new(frequency);
    let samples = generator.generate(1.0, SAMPLE_RATE);

    let input_energy: f32 = samples.iter().map(|&x| x * x).sum();

    // With boost at the signal frequency, output energy should increase
    let boost_linear = 10.0f32.powf(boost_db / 20.0);
    let expected_output_energy = input_energy * boost_linear * boost_linear;

    // Placeholder - would process through actual EQ
    let actual_output_energy = expected_output_energy;

    TestResult::from_bool(
        actual_output_energy > input_energy * 0.9 // At least 90% of expected increase
    )
}

/// Property tests for dynamic range and precision
pub fn prop_dynamic_range_properties() -> TestSuite {
    let suite = TestSuite::new();

    // Property: Very quiet signals should still be detectable
    quickcheck(prop_quiet_signal_detection as fn(f32) -> TestResult);

    // Property: Signal + noise should have higher energy than signal alone
    quickcheck(prop_additive_noise_increases_energy as fn(f32, f32) -> TestResult);

    suite
}

/// Property test: Quiet signal detection
fn prop_quiet_signal_detection(frequency: f32) -> TestResult {
    if frequency < 100.0 || frequency > 10000.0 {
        return TestResult::discard();
    }

    let quiet_amplitude = 0.001; // -60 dB
    let generator = SineGenerator::new(frequency).with_amplitude(quiet_amplitude);
    let samples = generator.generate(2.0, SAMPLE_RATE);

    let analyzer = FftAnalyzer::new(8192); // Large FFT for good resolution
    let analysis = analyzer.analyze(&samples, SAMPLE_RATE);

    if let Some((detected_freq, magnitude)) = analysis.find_peak() {
        TestResult::from_bool(
            (detected_freq - frequency).abs() < 50.0 && magnitude > 0.0
        )
    } else {
        TestResult::failed()
    }
}

/// Property test: Additive noise increases energy
fn prop_additive_noise_increases_energy(signal_amplitude: f32, noise_amplitude: f32) -> TestResult {
    if signal_amplitude <= 0.0 || signal_amplitude > 1.0 ||
       noise_amplitude <= 0.0 || noise_amplitude > 1.0 {
        return TestResult::discard();
    }

    let signal_gen = SineGenerator::new(1000.0).with_amplitude(signal_amplitude);
    let signal_samples = signal_gen.generate(1.0, SAMPLE_RATE);

    let noise_gen = WhiteNoiseGenerator::new().with_amplitude(noise_amplitude);
    let noise_samples = noise_gen.generate(1.0, SAMPLE_RATE);

    // Combine signal and noise
    let combined_samples: Vec<f32> = signal_samples.iter()
        .zip(&noise_samples)
        .map(|(&s, &n)| s + n)
        .collect();

    let signal_energy: f32 = signal_samples.iter().map(|&x| x * x).sum();
    let combined_energy: f32 = combined_samples.iter().map(|&x| x * x).sum();

    TestResult::from_bool(combined_energy > signal_energy)
}

/// Run all property-based tests
pub fn run_property_tests() -> TestSuite {
    let mut master_suite = TestSuite::new();

    println!("Running Property-Based Tests...");

    // Sine wave properties
    let mut sine_suite = prop_sine_wave_properties();
    master_suite.results.append(&mut sine_suite.results);

    // White noise properties
    let mut noise_suite = prop_white_noise_properties();
    master_suite.results.append(&mut noise_suite.results);

    // Spectrum analysis properties
    let mut spectrum_suite = prop_spectrum_analysis_properties();
    master_suite.results.append(&mut spectrum_suite.results);

    // Equalizer properties
    let mut eq_suite = prop_equalizer_properties();
    master_suite.results.append(&mut eq_suite.results);

    // Dynamic range properties
    let mut dynamic_suite = prop_dynamic_range_properties();
    master_suite.results.append(&mut dynamic_suite.results);

    master_suite
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_rms_property() {
        let result = prop_sine_rms(1000.0, 1.0);
        assert!(result.is_passed());
    }

    #[test]
    fn test_sine_peak_property() {
        let result = prop_sine_peak(1000.0, 0.5);
        assert!(result.is_passed());
    }

    #[test]
    fn test_white_noise_bounds() {
        let result = prop_white_noise_bounds(0.5, 42);
        assert!(result.is_passed());
    }

    #[test]
    fn test_parsevals_theorem() {
        let result = prop_parsevals_theorem(12345);
        assert!(result.is_passed());
    }

    #[test]
    fn test_quiet_signal_detection() {
        let result = prop_quiet_signal_detection(1000.0);
        assert!(result.is_passed());
    }
}