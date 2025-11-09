// Signal Generators for Mathematical Audio Testing
//
// This module provides mathematically accurate test signal generators
// for verifying audio processing algorithms.

use rand::{rngs::StdRng, Rng, SeedableRng};
use std::f32::consts::PI;

/// Type alias for audio samples
pub type Samples = Vec<f32>;

/// Trait for signal generators
pub trait SignalGenerator {
    fn generate(&self, duration: f32, sample_rate: f32) -> Samples;
    fn name(&self) -> &str;
}

/// Pure sine wave generator
#[derive(Debug, Clone)]
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

    pub fn with_phase(mut self, phase: f32) -> Self {
        self.phase = phase;
        self
    }
}

impl SignalGenerator for SineGenerator {
    fn generate(&self, duration: f32, sample_rate: f32) -> Samples {
        let num_samples = (duration * sample_rate) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            let sample = self.amplitude * (2.0 * PI * self.frequency * t + self.phase).sin();
            samples.push(sample);
        }

        samples
    }

    fn name(&self) -> &str {
        "SineGenerator"
    }
}

/// Square wave generator
#[derive(Debug, Clone)]
pub struct SquareGenerator {
    pub frequency: f32,
    pub amplitude: f32,
    pub duty_cycle: f32, // 0.5 for symmetric square wave
}

impl SquareGenerator {
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            amplitude: 1.0,
            duty_cycle: 0.5,
        }
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }

    pub fn with_duty_cycle(mut self, duty_cycle: f32) -> Self {
        self.duty_cycle = duty_cycle.clamp(0.0, 1.0);
        self
    }
}

impl SignalGenerator for SquareGenerator {
    fn generate(&self, duration: f32, sample_rate: f32) -> Samples {
        let num_samples = (duration * sample_rate) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            let phase = (self.frequency * t) % 1.0;
            let sample = if phase < self.duty_cycle {
                self.amplitude
            } else {
                -self.amplitude
            };
            samples.push(sample);
        }

        samples
    }

    fn name(&self) -> &str {
        "SquareGenerator"
    }
}

/// Sawtooth wave generator
#[derive(Debug, Clone)]
pub struct SawtoothGenerator {
    pub frequency: f32,
    pub amplitude: f32,
}

impl SawtoothGenerator {
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            amplitude: 1.0,
        }
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }
}

impl SignalGenerator for SawtoothGenerator {
    fn generate(&self, duration: f32, sample_rate: f32) -> Samples {
        let num_samples = (duration * sample_rate) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            let phase = (self.frequency * t) % 1.0;
            let sample = self.amplitude * (2.0 * phase - 1.0);
            samples.push(sample);
        }

        samples
    }

    fn name(&self) -> &str {
        "SawtoothGenerator"
    }
}

/// White noise generator
#[derive(Debug, Clone)]
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

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
}

impl SignalGenerator for WhiteNoiseGenerator {
    fn generate(&self, duration: f32, sample_rate: f32) -> Samples {
        let num_samples = (duration * sample_rate) as usize;
        let mut samples = Vec::with_capacity(num_samples);
        let mut rng = StdRng::seed_from_u64(self.seed);

        for _ in 0..num_samples {
            let sample = self.amplitude * (rng.gen::<f32>() * 2.0 - 1.0);
            samples.push(sample);
        }

        samples
    }

    fn name(&self) -> &str {
        "WhiteNoiseGenerator"
    }
}

/// Pink noise generator (1/f noise)
#[derive(Debug, Clone)]
pub struct PinkNoiseGenerator {
    pub amplitude: f32,
    pub seed: u64,
}

impl PinkNoiseGenerator {
    pub fn new() -> Self {
        Self {
            amplitude: 1.0,
            seed: 42,
        }
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
}

impl SignalGenerator for PinkNoiseGenerator {
    fn generate(&self, duration: f32, sample_rate: f32) -> Samples {
        let num_samples = (duration * sample_rate) as usize;
        let mut samples = Vec::with_capacity(num_samples);
        let mut rng = StdRng::seed_from_u64(self.seed);

        // Simple pink noise approximation using running sum filter
        let mut b = [0.0; 7];

        for _ in 0..num_samples {
            let white = rng.gen::<f32>() * 2.0 - 1.0;

            // Paul Kellet's economy pink noise filter
            b[0] = 0.99886 * b[0] + white * 0.0555179;
            b[1] = 0.99332 * b[1] + white * 0.0750759;
            b[2] = 0.96900 * b[2] + white * 0.1538520;
            b[3] = 0.86650 * b[3] + white * 0.3104856;
            b[4] = 0.55000 * b[4] + white * 0.5329522;
            b[5] = -0.7616 * b[5] - white * 0.0168980;

            let pink = b[0] + b[1] + b[2] + b[3] + b[4] + b[5] + b[6] + white * 0.5362;
            b[6] = white * 0.115926;

            samples.push(self.amplitude * pink * 0.11);
        }

        samples
    }

    fn name(&self) -> &str {
        "PinkNoiseGenerator"
    }
}

/// Impulse generator (Dirac delta approximation)
#[derive(Debug, Clone)]
pub struct ImpulseGenerator {
    pub amplitude: f32,
    pub delay: f32, // Delay in seconds
}

impl ImpulseGenerator {
    pub fn new() -> Self {
        Self {
            amplitude: 1.0,
            delay: 0.0,
        }
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }

    pub fn with_delay(mut self, delay: f32) -> Self {
        self.delay = delay.max(0.0);
        self
    }
}

impl SignalGenerator for ImpulseGenerator {
    fn generate(&self, duration: f32, sample_rate: f32) -> Samples {
        let num_samples = (duration * sample_rate) as usize;
        let mut samples = vec![0.0; num_samples];

        let impulse_sample = (self.delay * sample_rate) as usize;
        if impulse_sample < num_samples {
            samples[impulse_sample] = self.amplitude;
        }

        samples
    }

    fn name(&self) -> &str {
        "ImpulseGenerator"
    }
}

/// Frequency sweep generator (chirp)
#[derive(Debug, Clone)]
pub struct SweepGenerator {
    pub start_freq: f32,
    pub end_freq: f32,
    pub amplitude: f32,
}

impl SweepGenerator {
    pub fn new(start_freq: f32, end_freq: f32) -> Self {
        Self {
            start_freq,
            end_freq,
            amplitude: 1.0,
        }
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }
}

impl SignalGenerator for SweepGenerator {
    fn generate(&self, duration: f32, sample_rate: f32) -> Samples {
        let num_samples = (duration * sample_rate) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            let t_norm = t / duration;

            // Linear frequency sweep
            let freq = self.start_freq + (self.end_freq - self.start_freq) * t_norm;

            // Instantaneous phase (integral of frequency)
            let phase = 2.0
                * PI
                * (self.start_freq * t
                    + 0.5 * (self.end_freq - self.start_freq) * t * t / duration);

            let sample = self.amplitude * phase.sin();
            samples.push(sample);
        }

        samples
    }

    fn name(&self) -> &str {
        "SweepGenerator"
    }
}

/// Multi-tone generator for testing frequency separation
#[derive(Debug, Clone)]
pub struct MultiToneGenerator {
    pub frequencies: Vec<f32>,
    pub amplitudes: Vec<f32>,
    pub phases: Vec<f32>,
}

impl MultiToneGenerator {
    pub fn new(frequencies: Vec<f32>) -> Self {
        let len = frequencies.len();
        Self {
            frequencies,
            amplitudes: vec![1.0; len],
            phases: vec![0.0; len],
        }
    }

    pub fn with_amplitudes(mut self, amplitudes: Vec<f32>) -> Self {
        if amplitudes.len() == self.frequencies.len() {
            self.amplitudes = amplitudes;
        }
        self
    }

    pub fn with_phases(mut self, phases: Vec<f32>) -> Self {
        if phases.len() == self.frequencies.len() {
            self.phases = phases;
        }
        self
    }
}

impl SignalGenerator for MultiToneGenerator {
    fn generate(&self, duration: f32, sample_rate: f32) -> Samples {
        let num_samples = (duration * sample_rate) as usize;
        let mut samples = vec![0.0; num_samples];

        for (freq_idx, &frequency) in self.frequencies.iter().enumerate() {
            let amplitude = self.amplitudes[freq_idx];
            let phase = self.phases[freq_idx];

            for i in 0..num_samples {
                let t = i as f32 / sample_rate;
                let sample = amplitude * (2.0 * PI * frequency * t + phase).sin();
                samples[i] += sample;
            }
        }

        samples
    }

    fn name(&self) -> &str {
        "MultiToneGenerator"
    }
}

/// Convenience functions for common test signals
pub mod presets {
    use super::*;

    /// Generate 1kHz sine wave at 0 dBFS
    pub fn sine_1khz() -> SineGenerator {
        SineGenerator::new(1000.0)
    }

    /// Generate 440Hz sine wave (A4 note)
    pub fn sine_a4() -> SineGenerator {
        SineGenerator::new(440.0)
    }

    /// Generate quiet white noise at -20 dBFS
    pub fn quiet_white_noise() -> WhiteNoiseGenerator {
        WhiteNoiseGenerator::new().with_amplitude(0.1)
    }

    /// Generate test sweep from 20Hz to 20kHz
    pub fn full_range_sweep() -> SweepGenerator {
        SweepGenerator::new(20.0, 20000.0)
    }

    /// Generate impulse response test signal
    pub fn unit_impulse() -> ImpulseGenerator {
        ImpulseGenerator::new()
    }

    /// Generate multi-tone signal for IMD testing
    pub fn imd_test_signal() -> MultiToneGenerator {
        MultiToneGenerator::new(vec![1000.0, 1001.0]).with_amplitudes(vec![0.707, 0.707])
        // -3 dBFS each
    }

    /// Generate harmonic test signal (fundamental + harmonics)
    pub fn harmonic_test_signal(fundamental: f32) -> MultiToneGenerator {
        let frequencies = vec![
            fundamental,
            fundamental * 2.0,
            fundamental * 3.0,
            fundamental * 4.0,
            fundamental * 5.0,
        ];
        let amplitudes = vec![1.0, 0.5, 0.33, 0.25, 0.2]; // Decreasing harmonics

        MultiToneGenerator::new(frequencies).with_amplitudes(amplitudes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_sine_generator_frequency() {
        let gen = SineGenerator::new(1000.0);
        let samples = gen.generate(0.01, 44100.0); // 10ms at 44.1kHz
        assert_eq!(samples.len(), 441);

        // Check first few samples for correct frequency
        let dt = 1.0 / 44100.0;
        let expected_0 = (2.0 * PI * 1000.0 * 0.0).sin();
        let expected_1 = (2.0 * PI * 1000.0 * dt).sin();

        assert_abs_diff_eq!(samples[0], expected_0, epsilon = 1e-6);
        assert_abs_diff_eq!(samples[1], expected_1, epsilon = 1e-6);
    }

    #[test]
    fn test_white_noise_amplitude() {
        let gen = WhiteNoiseGenerator::new().with_amplitude(0.5);
        let samples = gen.generate(1.0, 44100.0);

        // Check that all samples are within amplitude bounds
        for sample in samples {
            assert!(sample.abs() <= 0.5);
        }
    }

    #[test]
    fn test_impulse_generator() {
        let gen = ImpulseGenerator::new().with_delay(0.001); // 1ms delay
        let samples = gen.generate(0.01, 44100.0);

        // Should be zero except at impulse location
        let impulse_idx = (0.001 * 44100.0) as usize;

        for (i, sample) in samples.iter().enumerate() {
            if i == impulse_idx {
                assert_eq!(*sample, 1.0);
            } else {
                assert_eq!(*sample, 0.0);
            }
        }
    }

    #[test]
    fn test_multi_tone_generator() {
        let gen = MultiToneGenerator::new(vec![1000.0, 2000.0]).with_amplitudes(vec![0.5, 0.3]);
        let samples = gen.generate(0.001, 44100.0);

        // At t=0, both sines are 0, so sum should be 0
        assert_abs_diff_eq!(samples[0], 0.0, epsilon = 1e-6);

        // Check that the signal contains both frequencies
        assert!(!samples.iter().all(|&x| x == 0.0));
    }
}
