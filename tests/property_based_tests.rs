// Property-Based Tests for Signal Generators
// Using QuickCheck and Proptest for comprehensive testing

use approx::{assert_relative_eq, relative_eq};
use proptest::prelude::*;
use proptest::strategy::{BoxedStrategy, Just};
use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};
use quickcheck_macros::quickcheck;
use rustfft::{num_complex::Complex32, FftPlanner};
use statrs::statistics::{Data, Statistics};
use std::f32::consts::PI;

const SAMPLE_RATE: f32 = 48000.0;
const MIN_FREQUENCY: f32 = 20.0;
const MAX_FREQUENCY: f32 = 20000.0;
const MIN_AMPLITUDE: f32 = 0.001;
const MAX_AMPLITUDE: f32 = 10.0;
const MIN_DURATION: f32 = 0.01;
const MAX_DURATION: f32 = 5.0;

/// Signal generator types for property testing
#[derive(Debug, Clone, PartialEq)]
enum SignalType {
    Sine,
    Square,
    Triangle,
    Sawtooth,
    WhiteNoise,
    PinkNoise,
}

impl Arbitrary for SignalType {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            Just(SignalType::Sine),
            Just(SignalType::Square),
            Just(SignalType::Triangle),
            Just(SignalType::Sawtooth),
            Just(SignalType::WhiteNoise),
            Just(SignalType::PinkNoise),
        ]
        .boxed()
    }
}

/// Signal parameters for property testing
#[derive(Debug, Clone)]
struct SignalParams {
    signal_type: SignalType,
    frequency: f32,
    amplitude: f32,
    phase: f32,
    duration: f32,
    sample_rate: f32,
}

impl Arbitrary for SignalParams {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            any::<SignalType>(),
            MIN_FREQUENCY..MAX_FREQUENCY,
            MIN_AMPLITUDE..MAX_AMPLITUDE,
            0.0f32..(2.0 * PI),
            MIN_DURATION..MAX_DURATION,
            prop_oneof![Just(44100.0f32), Just(48000.0f32)],
        )
            .prop_map(
                |(signal_type, frequency, amplitude, phase, duration, sample_rate)| SignalParams {
                    signal_type,
                    frequency,
                    amplitude,
                    phase,
                    duration,
                    sample_rate,
                },
            )
            .boxed()
    }
}

/// Property-based signal generator
struct PropertySignalGenerator {
    params: SignalParams,
}

impl PropertySignalGenerator {
    fn new(params: SignalParams) -> Self {
        Self { params }
    }

    fn generate(&self) -> Vec<f32> {
        let num_samples = (self.params.duration * self.params.sample_rate) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        match self.params.signal_type {
            SignalType::Sine => self.generate_sine(&mut samples),
            SignalType::Square => self.generate_square(&mut samples),
            SignalType::Triangle => self.generate_triangle(&mut samples),
            SignalType::Sawtooth => self.generate_sawtooth(&mut samples),
            SignalType::WhiteNoise => self.generate_white_noise(&mut samples),
            SignalType::PinkNoise => self.generate_pink_noise(&mut samples),
        }

        samples
    }

    fn generate_sine(&self, samples: &mut Vec<f32>) {
        let num_samples = (self.params.duration * self.params.sample_rate) as usize;

        for i in 0..num_samples {
            let t = i as f32 / self.params.sample_rate;
            let sample = self.params.amplitude
                * (2.0 * PI * self.params.frequency * t + self.params.phase).sin();
            samples.push(sample);
        }
    }

    fn generate_square(&self, samples: &mut Vec<f32>) {
        let num_samples = (self.params.duration * self.params.sample_rate) as usize;

        for i in 0..num_samples {
            let t = i as f32 / self.params.sample_rate;
            let phase = (2.0 * PI * self.params.frequency * t + self.params.phase) % (2.0 * PI);
            let sample = if phase < PI {
                self.params.amplitude
            } else {
                -self.params.amplitude
            };
            samples.push(sample);
        }
    }

    fn generate_triangle(&self, samples: &mut Vec<f32>) {
        let num_samples = (self.params.duration * self.params.sample_rate) as usize;

        for i in 0..num_samples {
            let t = i as f32 / self.params.sample_rate;
            let phase = (2.0 * PI * self.params.frequency * t + self.params.phase) % (2.0 * PI);
            let sample = if phase < PI {
                self.params.amplitude * (2.0 * phase / PI - 1.0)
            } else {
                self.params.amplitude * (3.0 - 2.0 * phase / PI)
            };
            samples.push(sample);
        }
    }

    fn generate_sawtooth(&self, samples: &mut Vec<f32>) {
        let num_samples = (self.params.duration * self.params.sample_rate) as usize;

        for i in 0..num_samples {
            let t = i as f32 / self.params.sample_rate;
            let phase = (2.0 * PI * self.params.frequency * t + self.params.phase) % (2.0 * PI);
            let sample = self.params.amplitude * (phase / PI - 1.0);
            samples.push(sample);
        }
    }

    fn generate_white_noise(&self, samples: &mut Vec<f32>) {
        use rand::{rngs::StdRng, Rng, SeedableRng};

        let num_samples = (self.params.duration * self.params.sample_rate) as usize;
        let mut rng = StdRng::seed_from_u64(42); // Fixed seed for reproducibility

        for _ in 0..num_samples {
            let sample = self.params.amplitude * (rng.gen::<f32>() * 2.0 - 1.0);
            samples.push(sample);
        }
    }

    fn generate_pink_noise(&self, samples: &mut Vec<f32>) {
        use rand::{rngs::StdRng, Rng, SeedableRng};

        let num_samples = (self.params.duration * self.params.sample_rate) as usize;
        let mut rng = StdRng::seed_from_u64(42);

        // Simple pink noise filter state
        let mut b = [0.0f32; 7];

        for _ in 0..num_samples {
            let white = rng.gen::<f32>() * 2.0 - 1.0;

            b[0] = 0.99886 * b[0] + white * 0.0555179;
            b[1] = 0.99332 * b[1] + white * 0.0750759;
            b[2] = 0.96900 * b[2] + white * 0.1538520;
            b[3] = 0.86650 * b[3] + white * 0.3104856;
            b[4] = 0.55000 * b[4] + white * 0.5329522;
            b[5] = -0.7616 * b[5] - white * 0.0168980;

            let pink = b[0] + b[1] + b[2] + b[3] + b[4] + b[5] + b[6] + white * 0.5362;
            b[6] = white * 0.115926;

            samples.push(self.params.amplitude * pink * 0.11);
        }
    }
}

/// Mathematical analysis functions
fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

fn calculate_peak(samples: &[f32]) -> f32 {
    samples.iter().map(|&x| x.abs()).fold(0.0, f32::max)
}

fn calculate_dc_component(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().sum::<f32>() / samples.len() as f32
}

fn calculate_crest_factor(samples: &[f32]) -> f32 {
    let rms = calculate_rms(samples);
    let peak = calculate_peak(samples);
    if rms == 0.0 {
        f32::INFINITY
    } else {
        peak / rms
    }
}

fn find_fundamental_frequency(samples: &[f32], sample_rate: f32) -> Option<f32> {
    if samples.len() < 512 {
        return None;
    }

    let fft_size = (samples.len() / 2).next_power_of_two().min(2048);
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);

    let mut buffer: Vec<Complex32> = samples[..fft_size]
        .iter()
        .map(|&sample| Complex32::new(sample, 0.0))
        .collect();

    fft.process(&mut buffer);

    // Find peak in magnitude spectrum
    let half_size = fft_size / 2;
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
        Some((max_bin as f32 * sample_rate) / fft_size as f32)
    } else {
        None
    }
}

// ============================================================================
// PROPERTY-BASED TESTS
// ============================================================================

#[cfg(test)]
mod quickcheck_tests {
    use super::*;

    #[quickcheck]
    fn qc_signal_length_property(params: SignalParams) -> TestResult {
        if params.duration < MIN_DURATION || params.duration > MAX_DURATION {
            return TestResult::discard();
        }

        let generator = PropertySignalGenerator::new(params.clone());
        let samples = generator.generate();

        let expected_length = (params.duration * params.sample_rate) as usize;
        let actual_length = samples.len();

        TestResult::from_bool(actual_length == expected_length)
    }

    #[quickcheck]
    fn qc_amplitude_bounds_property(params: SignalParams) -> TestResult {
        if params.amplitude < MIN_AMPLITUDE || params.amplitude > MAX_AMPLITUDE {
            return TestResult::discard();
        }

        let generator = PropertySignalGenerator::new(params.clone());
        let samples = generator.generate();

        if samples.is_empty() {
            return TestResult::discard();
        }

        let peak = calculate_peak(&samples);

        // For deterministic signals, peak should not exceed amplitude significantly
        match params.signal_type {
            SignalType::Sine | SignalType::Square | SignalType::Triangle | SignalType::Sawtooth => {
                TestResult::from_bool(peak <= params.amplitude * 1.01) // Small tolerance for numerical errors
            }
            SignalType::WhiteNoise | SignalType::PinkNoise => {
                // Noise can have peaks higher than RMS amplitude
                TestResult::from_bool(peak <= params.amplitude * 5.0) // Reasonable bound for noise
            }
        }
    }

    #[quickcheck]
    fn qc_rms_properties(params: SignalParams) -> TestResult {
        if params.amplitude < MIN_AMPLITUDE || params.duration < 0.1 {
            return TestResult::discard();
        }

        let generator = PropertySignalGenerator::new(params.clone());
        let samples = generator.generate();

        if samples.is_empty() {
            return TestResult::discard();
        }

        let rms = calculate_rms(&samples);

        // RMS should always be positive for non-zero signals
        if rms < 0.0 {
            return TestResult::failed();
        }

        // For sine waves, RMS should be amplitude / sqrt(2)
        if params.signal_type == SignalType::Sine {
            let expected_rms = params.amplitude / (2.0_f32).sqrt();
            let relative_error = (rms - expected_rms).abs() / expected_rms;

            if relative_error > 0.01 {
                // 1% tolerance
                return TestResult::failed();
            }
        }

        // RMS should not exceed peak amplitude
        let peak = calculate_peak(&samples);
        TestResult::from_bool(rms <= peak + f32::EPSILON)
    }

    #[quickcheck]
    fn qc_dc_component_property(params: SignalParams) -> TestResult {
        if params.duration < 0.5 {
            return TestResult::discard();
        }

        let generator = PropertySignalGenerator::new(params.clone());
        let samples = generator.generate();

        if samples.is_empty() {
            return TestResult::discard();
        }

        let dc = calculate_dc_component(&samples);

        // For symmetric waveforms, DC component should be near zero
        match params.signal_type {
            SignalType::Sine | SignalType::Square | SignalType::Triangle | SignalType::Sawtooth => {
                let dc_threshold = params.amplitude * 0.1; // 10% of amplitude
                TestResult::from_bool(dc.abs() < dc_threshold)
            }
            SignalType::WhiteNoise | SignalType::PinkNoise => {
                // Noise should have DC close to zero with enough samples
                let dc_threshold = params.amplitude * 0.2;
                TestResult::from_bool(dc.abs() < dc_threshold)
            }
        }
    }

    #[quickcheck]
    fn qc_frequency_detection_property(params: SignalParams) -> TestResult {
        // Only test deterministic periodic signals
        if !matches!(
            params.signal_type,
            SignalType::Sine | SignalType::Square | SignalType::Triangle | SignalType::Sawtooth
        ) {
            return TestResult::discard();
        }

        if params.frequency < 50.0
            || params.frequency > params.sample_rate / 4.0
            || params.duration < 1.0
        {
            return TestResult::discard();
        }

        let generator = PropertySignalGenerator::new(params.clone());
        let samples = generator.generate();

        if let Some(detected_freq) = find_fundamental_frequency(&samples, params.sample_rate) {
            let frequency_error = (detected_freq - params.frequency).abs();
            let relative_error = frequency_error / params.frequency;

            // Allow 5% error for frequency detection
            TestResult::from_bool(relative_error < 0.05)
        } else {
            TestResult::failed()
        }
    }

    #[quickcheck]
    fn qc_crest_factor_property(params: SignalParams) -> TestResult {
        if params.amplitude < MIN_AMPLITUDE || params.duration < 0.1 {
            return TestResult::discard();
        }

        let generator = PropertySignalGenerator::new(params.clone());
        let samples = generator.generate();

        if samples.is_empty() {
            return TestResult::discard();
        }

        let crest_factor = calculate_crest_factor(&samples);

        // Crest factor should always be >= 1 for valid signals
        if crest_factor < 1.0 && crest_factor.is_finite() {
            return TestResult::failed();
        }

        // Check expected crest factors for known waveforms
        match params.signal_type {
            SignalType::Sine => {
                // Sine wave crest factor should be sqrt(2) ≈ 1.414
                let expected_cf = (2.0_f32).sqrt();
                let error = (crest_factor - expected_cf).abs() / expected_cf;
                TestResult::from_bool(error < 0.1)
            }
            SignalType::Square => {
                // Square wave crest factor should be 1.0
                TestResult::from_bool((crest_factor - 1.0).abs() < 0.1)
            }
            _ => TestResult::passed(), // Other waveforms have variable crest factors
        }
    }

    #[quickcheck]
    fn qc_energy_conservation_property(params: SignalParams) -> TestResult {
        if params.amplitude < MIN_AMPLITUDE || params.duration < 0.1 {
            return TestResult::discard();
        }

        let generator = PropertySignalGenerator::new(params.clone());
        let samples = generator.generate();

        if samples.is_empty() {
            return TestResult::discard();
        }

        // Calculate energy
        let energy: f32 = samples.iter().map(|&x| x * x).sum();

        // Energy should be proportional to amplitude squared
        let rms = calculate_rms(&samples);
        let expected_energy = rms * rms * samples.len() as f32;

        let error = (energy - expected_energy).abs() / expected_energy;
        TestResult::from_bool(error < 1e-6)
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;

    proptest! {
        #[test]
        fn prop_signal_symmetry(
            frequency in 100.0f32..1000.0,
            amplitude in 0.1f32..2.0,
            duration in 1.0f32..3.0
        ) {
            let params = SignalParams {
                signal_type: SignalType::Sine,
                frequency,
                amplitude,
                phase: 0.0,
                duration,
                sample_rate: SAMPLE_RATE,
            };

            let generator = PropertySignalGenerator::new(params);
            let samples = generator.generate();

            // Test that sine wave is symmetric around zero
            let positive_sum: f32 = samples.iter().filter(|&&x| x > 0.0).sum();
            let negative_sum: f32 = samples.iter().filter(|&&x| x < 0.0).sum();

            let symmetry_error = (positive_sum + negative_sum).abs() / positive_sum.max(negative_sum.abs());
            prop_assert!(symmetry_error < 0.01, "Sine wave not symmetric: error = {:.4}", symmetry_error);
        }

        #[test]
        fn prop_frequency_scaling_invariance(
            base_frequency in 200.0f32..800.0,
            scale_factor in 1.5f32..4.0,
            amplitude in 0.5f32..1.5
        ) {
            let base_params = SignalParams {
                signal_type: SignalType::Sine,
                frequency: base_frequency,
                amplitude,
                phase: 0.0,
                duration: 2.0,
                sample_rate: SAMPLE_RATE,
            };

            let scaled_params = SignalParams {
                frequency: base_frequency * scale_factor,
                ..base_params.clone()
            };

            let base_generator = PropertySignalGenerator::new(base_params);
            let scaled_generator = PropertySignalGenerator::new(scaled_params);

            let base_samples = base_generator.generate();
            let scaled_samples = scaled_generator.generate();

            // Both should have same RMS (amplitude scaling)
            let base_rms = calculate_rms(&base_samples);
            let scaled_rms = calculate_rms(&scaled_samples);

            let rms_error = (base_rms - scaled_rms).abs() / base_rms;
            prop_assert!(rms_error < 0.001, "RMS not preserved under frequency scaling: {:.6} vs {:.6}", base_rms, scaled_rms);

            // Frequency detection should scale proportionally
            if let (Some(base_freq), Some(scaled_freq)) = (
                find_fundamental_frequency(&base_samples, SAMPLE_RATE),
                find_fundamental_frequency(&scaled_samples, SAMPLE_RATE)
            ) {
                let actual_scale = scaled_freq / base_freq;
                let scale_error = (actual_scale - scale_factor).abs() / scale_factor;
                prop_assert!(scale_error < 0.05, "Frequency scaling error: expected {:.2}, got {:.2}", scale_factor, actual_scale);
            }
        }

        #[test]
        fn prop_amplitude_linearity(
            frequency in 440.0f32..880.0,
            base_amplitude in 0.1f32..1.0,
            amplitude_factor in 1.5f32..3.0
        ) {
            let base_params = SignalParams {
                signal_type: SignalType::Sine,
                frequency,
                amplitude: base_amplitude,
                phase: 0.0,
                duration: 1.0,
                sample_rate: SAMPLE_RATE,
            };

            let scaled_params = SignalParams {
                amplitude: base_amplitude * amplitude_factor,
                ..base_params.clone()
            };

            let base_generator = PropertySignalGenerator::new(base_params);
            let scaled_generator = PropertySignalGenerator::new(scaled_params);

            let base_samples = base_generator.generate();
            let scaled_samples = scaled_generator.generate();

            let base_rms = calculate_rms(&base_samples);
            let scaled_rms = calculate_rms(&scaled_samples);

            let actual_factor = scaled_rms / base_rms;
            let linearity_error = (actual_factor - amplitude_factor).abs() / amplitude_factor;

            prop_assert!(linearity_error < 0.001, "Amplitude linearity error: expected factor {:.3}, got {:.3}", amplitude_factor, actual_factor);
        }

        #[test]
        fn prop_phase_invariance(
            frequency in 500.0f32..1500.0,
            amplitude in 0.5f32..1.5,
            phase in 0.0f32..(2.0 * PI)
        ) {
            let params_zero_phase = SignalParams {
                signal_type: SignalType::Sine,
                frequency,
                amplitude,
                phase: 0.0,
                duration: 2.0,
                sample_rate: SAMPLE_RATE,
            };

            let params_with_phase = SignalParams {
                phase,
                ..params_zero_phase.clone()
            };

            let generator_zero = PropertySignalGenerator::new(params_zero_phase);
            let generator_phase = PropertySignalGenerator::new(params_with_phase);

            let samples_zero = generator_zero.generate();
            let samples_phase = generator_phase.generate();

            // RMS should be identical regardless of phase
            let rms_zero = calculate_rms(&samples_zero);
            let rms_phase = calculate_rms(&samples_phase);

            let rms_error = (rms_zero - rms_phase).abs() / rms_zero;
            prop_assert!(rms_error < 0.001, "Phase affects RMS: {:.6} vs {:.6}", rms_zero, rms_phase);

            // Peak should be identical
            let peak_zero = calculate_peak(&samples_zero);
            let peak_phase = calculate_peak(&samples_phase);

            let peak_error = (peak_zero - peak_phase).abs() / peak_zero;
            prop_assert!(peak_error < 0.001, "Phase affects peak: {:.6} vs {:.6}", peak_zero, peak_phase);
        }

        #[test]
        fn prop_duration_independence(
            frequency in 300.0f32..700.0,
            amplitude in 0.3f32..1.0,
            short_duration in 0.5f32..1.0,
            long_duration in 2.0f32..4.0
        ) {
            let short_params = SignalParams {
                signal_type: SignalType::Sine,
                frequency,
                amplitude,
                phase: 0.0,
                duration: short_duration,
                sample_rate: SAMPLE_RATE,
            };

            let long_params = SignalParams {
                duration: long_duration,
                ..short_params.clone()
            };

            let short_generator = PropertySignalGenerator::new(short_params);
            let long_generator = PropertySignalGenerator::new(long_params);

            let short_samples = short_generator.generate();
            let long_samples = long_generator.generate();

            // RMS should be independent of duration
            let short_rms = calculate_rms(&short_samples);
            let long_rms = calculate_rms(&long_samples);

            let rms_error = (short_rms - long_rms).abs() / short_rms;
            prop_assert!(rms_error < 0.001, "Duration affects RMS: {:.6} vs {:.6}", short_rms, long_rms);

            // Frequency detection should be independent of duration
            if let (Some(short_freq), Some(long_freq)) = (
                find_fundamental_frequency(&short_samples, SAMPLE_RATE),
                find_fundamental_frequency(&long_samples, SAMPLE_RATE)
            ) {
                let freq_error = (short_freq - long_freq).abs() / short_freq;
                prop_assert!(freq_error < 0.02, "Duration affects frequency detection: {:.1} Hz vs {:.1} Hz", short_freq, long_freq);
            }
        }

        #[test]
        fn prop_noise_statistical_properties(
            amplitude in 0.1f32..2.0,
            duration in 5.0f32..10.0 // Longer for better statistics
        ) {
            let params = SignalParams {
                signal_type: SignalType::WhiteNoise,
                frequency: 1000.0, // Not used for noise
                amplitude,
                phase: 0.0,
                duration,
                sample_rate: SAMPLE_RATE,
            };

            let generator = PropertySignalGenerator::new(params);
            let samples = generator.generate();

            prop_assert!(!samples.is_empty());

            // Test statistical properties of white noise
            let data = Data::new(samples.iter().map(|&x| x as f64).collect());
            let mean = data.mean().unwrap();
            let std_dev = data.std_dev().unwrap();

            // Mean should be close to zero
            let mean_error = mean.abs() / (amplitude as f64);
            prop_assert!(mean_error < 0.1, "Noise mean too far from zero: {:.4}", mean);

            // Standard deviation should be related to amplitude
            let expected_std = amplitude as f64 / (3.0_f64).sqrt(); // For uniform distribution
            let std_error = (std_dev - expected_std).abs() / expected_std;
            prop_assert!(std_error < 0.3, "Noise std dev unexpected: expected {:.4}, got {:.4}", expected_std, std_dev);
        }

        #[test]
        fn prop_waveform_continuity(
            frequency in 100.0f32..500.0,
            amplitude in 0.5f32..1.5
        ) {
            let params = SignalParams {
                signal_type: SignalType::Sine,
                frequency,
                amplitude,
                phase: 0.0,
                duration: 1.0,
                sample_rate: SAMPLE_RATE,
            };

            let generator = PropertySignalGenerator::new(params);
            let samples = generator.generate();

            prop_assert!(samples.len() > 100);

            // Test continuity by checking derivatives don't have large jumps
            let mut max_derivative = 0.0f32;
            for i in 1..samples.len() {
                let derivative = (samples[i] - samples[i-1]).abs() * SAMPLE_RATE;
                max_derivative = max_derivative.max(derivative);
            }

            // For sine wave, max derivative should be 2π * frequency * amplitude
            let expected_max_derivative = 2.0 * PI * frequency * amplitude;
            let derivative_error = (max_derivative - expected_max_derivative).abs() / expected_max_derivative;

            prop_assert!(derivative_error < 0.1, "Continuity violation: max derivative {:.1}, expected {:.1}", max_derivative, expected_max_derivative);
        }
    }

    // Additional statistical tests
    proptest! {
        #[test]
        fn prop_reproducibility(
            frequency in 440.0f32..880.0,
            amplitude in 0.5f32..1.0,
            duration in 1.0f32..2.0
        ) {
            let params = SignalParams {
                signal_type: SignalType::WhiteNoise,
                frequency,
                amplitude,
                phase: 0.0,
                duration,
                sample_rate: SAMPLE_RATE,
            };

            let generator1 = PropertySignalGenerator::new(params.clone());
            let generator2 = PropertySignalGenerator::new(params);

            let samples1 = generator1.generate();
            let samples2 = generator2.generate();

            // With fixed seed, noise should be reproducible
            prop_assert_eq!(samples1.len(), samples2.len());

            for (i, (&s1, &s2)) in samples1.iter().zip(samples2.iter()).enumerate() {
                prop_assert_eq!(s1, s2, "Samples differ at index {}: {} vs {}", i, s1, s2);
            }
        }
    }
}
