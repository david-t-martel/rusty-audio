//! High-Performance Signal Generators
//!
//! This module provides SIMD-optimized signal generators for audio synthesis
//! with focus on real-time performance and mathematical accuracy.

use std::f32::consts::PI;
use std::sync::Arc;
use parking_lot::RwLock;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Fast lookup table for trigonometric functions
pub struct TrigLookup {
    sin_table: Vec<f32>,
    cos_table: Vec<f32>,
    table_size: usize,
    scale_factor: f32,
}

impl TrigLookup {
    pub fn new(table_size: usize) -> Self {
        let mut sin_table = Vec::with_capacity(table_size);
        let mut cos_table = Vec::with_capacity(table_size);

        for i in 0..table_size {
            let angle = 2.0 * PI * i as f32 / table_size as f32;
            sin_table.push(angle.sin());
            cos_table.push(angle.cos());
        }

        Self {
            sin_table,
            cos_table,
            table_size,
            scale_factor: table_size as f32 / (2.0 * PI),
        }
    }

    #[inline(always)]
    pub fn sin(&self, x: f32) -> f32 {
        let index = ((x * self.scale_factor) as usize) % self.table_size;
        self.sin_table[index]
    }

    #[inline(always)]
    pub fn cos(&self, x: f32) -> f32 {
        let index = ((x * self.scale_factor) as usize) % self.table_size;
        self.cos_table[index]
    }
}

/// High-performance signal generator trait
pub trait OptimizedGenerator: Send + Sync {
    /// Generate a block of samples
    fn generate_block(&mut self, output: &mut [f32], sample_rate: f32);

    /// Reset generator state
    fn reset(&mut self);

    /// Get generator name
    fn name(&self) -> &str;

    /// Set frequency (if applicable)
    fn set_frequency(&mut self, frequency: f32);

    /// Set amplitude
    fn set_amplitude(&mut self, amplitude: f32);
}

/// SIMD-optimized sine wave generator
pub struct SimdSineGenerator {
    frequency: f32,
    amplitude: f32,
    phase: f32,
    phase_increment: f32,
    lookup_table: Arc<TrigLookup>,
    // Vectorized state for SIMD processing
    phases: [f32; 8], // AVX2 can process 8 floats at once
}

impl SimdSineGenerator {
    pub fn new(frequency: f32, lookup_table: Arc<TrigLookup>) -> Self {
        Self {
            frequency,
            amplitude: 1.0,
            phase: 0.0,
            phase_increment: 0.0,
            lookup_table,
            phases: [0.0; 8],
        }
    }

    /// Generate samples using SIMD optimization
    #[inline(always)]
    fn generate_simd_block(&mut self, output: &mut [f32], sample_rate: f32) {
        self.phase_increment = 2.0 * PI * self.frequency / sample_rate;
        let phase_inc_vec = self.phase_increment;

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe { self.generate_avx2(output, phase_inc_vec) };
                return;
            }
            if is_x86_feature_detected!("sse") {
                unsafe { self.generate_sse(output, phase_inc_vec) };
                return;
            }
        }

        // Fallback scalar implementation
        for sample in output.iter_mut() {
            *sample = self.amplitude * self.lookup_table.sin(self.phase);
            self.phase = (self.phase + self.phase_increment) % (2.0 * PI);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn generate_avx2(&mut self, output: &mut [f32], phase_inc: f32) {
        let len = output.len();
        let simd_len = len - (len % 8);

        let amplitude_vec = _mm256_set1_ps(self.amplitude);
        let phase_inc_vec = _mm256_set1_ps(phase_inc);
        let two_pi = _mm256_set1_ps(2.0 * PI);

        // Initialize phase vector
        let mut phase_offsets = _mm256_setr_ps(0.0, phase_inc, phase_inc * 2.0, phase_inc * 3.0,
                                             phase_inc * 4.0, phase_inc * 5.0, phase_inc * 6.0, phase_inc * 7.0);
        let mut current_phase = _mm256_set1_ps(self.phase);
        current_phase = _mm256_add_ps(current_phase, phase_offsets);

        let phase_inc_8 = _mm256_set1_ps(phase_inc * 8.0);

        for i in (0..simd_len).step_by(8) {
            // Calculate sine values using fast approximation
            let sine_values = self.fast_sin_avx2(current_phase);
            let result = _mm256_mul_ps(amplitude_vec, sine_values);

            _mm256_storeu_ps(output.as_mut_ptr().add(i), result);

            // Update phase
            current_phase = _mm256_add_ps(current_phase, phase_inc_8);

            // Wrap phase to [0, 2π]
            let mask = _mm256_cmp_ps(current_phase, two_pi, _CMP_GE_OQ);
            current_phase = _mm256_blendv_ps(current_phase,
                                           _mm256_sub_ps(current_phase, two_pi), mask);
        }

        // Extract final phase for next iteration
        let phases_array: [f32; 8] = std::mem::transmute(current_phase);
        self.phase = phases_array[0];

        // Handle remaining elements
        for i in simd_len..len {
            output[i] = self.amplitude * self.lookup_table.sin(self.phase);
            self.phase = (self.phase + phase_inc) % (2.0 * PI);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn fast_sin_avx2(&self, x: __m256) -> __m256 {
        // Fast sine approximation using polynomial
        // sin(x) ≈ x - x³/6 + x⁵/120 for x in [-π, π]

        // First normalize to [-π, π]
        let pi = _mm256_set1_ps(PI);
        let two_pi = _mm256_set1_ps(2.0 * PI);

        // Reduce to [-π, π] range
        let mut normalized = x;
        let mask_gt_pi = _mm256_cmp_ps(normalized, pi, _CMP_GT_OQ);
        normalized = _mm256_blendv_ps(normalized,
                                    _mm256_sub_ps(normalized, two_pi), mask_gt_pi);

        let x2 = _mm256_mul_ps(normalized, normalized);
        let x3 = _mm256_mul_ps(x2, normalized);
        let x5 = _mm256_mul_ps(x3, x2);

        let term1 = normalized;
        let term2 = _mm256_mul_ps(x3, _mm256_set1_ps(-1.0/6.0));
        let term3 = _mm256_mul_ps(x5, _mm256_set1_ps(1.0/120.0));

        _mm256_add_ps(_mm256_add_ps(term1, term2), term3)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse")]
    unsafe fn generate_sse(&mut self, output: &mut [f32], phase_inc: f32) {
        let len = output.len();
        let simd_len = len - (len % 4);

        let amplitude_vec = _mm_set1_ps(self.amplitude);
        let phase_inc_vec = _mm_set1_ps(phase_inc);

        for i in (0..simd_len).step_by(4) {
            let phases = _mm_setr_ps(
                self.phase,
                self.phase + phase_inc,
                self.phase + phase_inc * 2.0,
                self.phase + phase_inc * 3.0
            );

            // Use lookup table for sine values (simplified)
            let sin_vals = _mm_setr_ps(
                self.lookup_table.sin(self.phase),
                self.lookup_table.sin(self.phase + phase_inc),
                self.lookup_table.sin(self.phase + phase_inc * 2.0),
                self.lookup_table.sin(self.phase + phase_inc * 3.0)
            );

            let result = _mm_mul_ps(amplitude_vec, sin_vals);
            _mm_storeu_ps(output.as_mut_ptr().add(i), result);

            self.phase = (self.phase + phase_inc * 4.0) % (2.0 * PI);
        }

        // Handle remaining elements
        for i in simd_len..len {
            output[i] = self.amplitude * self.lookup_table.sin(self.phase);
            self.phase = (self.phase + phase_inc) % (2.0 * PI);
        }
    }
}

impl OptimizedGenerator for SimdSineGenerator {
    fn generate_block(&mut self, output: &mut [f32], sample_rate: f32) {
        self.generate_simd_block(output, sample_rate);
    }

    fn reset(&mut self) {
        self.phase = 0.0;
        self.phases = [0.0; 8];
    }

    fn name(&self) -> &str {
        "SIMD Sine Generator"
    }

    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }
}

/// Optimized multi-tone generator for complex waveforms
pub struct MultiToneGenerator {
    generators: Vec<SimdSineGenerator>,
    temp_buffer: Vec<f32>,
    mix_buffer: Vec<f32>,
    lookup_table: Arc<TrigLookup>,
}

impl MultiToneGenerator {
    pub fn new(frequencies: Vec<f32>, amplitudes: Vec<f32>) -> Self {
        let lookup_table = Arc::new(TrigLookup::new(4096)); // High resolution

        let generators: Vec<_> = frequencies.iter().zip(amplitudes.iter())
            .map(|(&freq, &amp)| {
                let mut gen = SimdSineGenerator::new(freq, lookup_table.clone());
                gen.set_amplitude(amp);
                gen
            })
            .collect();

        Self {
            generators,
            temp_buffer: Vec::new(),
            mix_buffer: Vec::new(),
            lookup_table,
        }
    }

    pub fn add_tone(&mut self, frequency: f32, amplitude: f32) {
        let mut gen = SimdSineGenerator::new(frequency, self.lookup_table.clone());
        gen.set_amplitude(amplitude);
        self.generators.push(gen);
    }

    pub fn remove_tone(&mut self, index: usize) {
        if index < self.generators.len() {
            self.generators.remove(index);
        }
    }

    pub fn update_tone(&mut self, index: usize, frequency: f32, amplitude: f32) {
        if let Some(gen) = self.generators.get_mut(index) {
            gen.set_frequency(frequency);
            gen.set_amplitude(amplitude);
        }
    }
}

impl OptimizedGenerator for MultiToneGenerator {
    fn generate_block(&mut self, output: &mut [f32], sample_rate: f32) {
        let block_size = output.len();

        // Resize buffers if needed
        if self.temp_buffer.len() < block_size {
            self.temp_buffer.resize(block_size, 0.0);
            self.mix_buffer.resize(block_size, 0.0);
        }

        // Clear output
        output.fill(0.0);

        // Generate and mix all tones
        for generator in &mut self.generators {
            // Generate into temp buffer
            self.temp_buffer.fill(0.0);
            generator.generate_block(&mut self.temp_buffer[..block_size], sample_rate);

            // Add to output using SIMD
            crate::audio_performance_optimized::simd_ops::add_vectors_simd(
                output,
                &self.temp_buffer[..block_size],
                output
            );
        }
    }

    fn reset(&mut self) {
        for gen in &mut self.generators {
            gen.reset();
        }
    }

    fn name(&self) -> &str {
        "Multi-Tone Generator"
    }

    fn set_frequency(&mut self, frequency: f32) {
        // Set fundamental frequency
        if let Some(gen) = self.generators.get_mut(0) {
            gen.set_frequency(frequency);
        }
    }

    fn set_amplitude(&mut self, amplitude: f32) {
        // Scale all generators proportionally
        for gen in &mut self.generators {
            let current_amp = gen.amplitude;
            gen.set_amplitude(current_amp * amplitude);
        }
    }
}

/// Optimized noise generator with multiple noise types
pub struct OptimizedNoiseGenerator {
    noise_type: NoiseType,
    amplitude: f32,
    seed: u64,
    rng_state: u64,
    // Pink noise filter state
    pink_state: [f32; 7],
}

#[derive(Debug, Clone, Copy)]
pub enum NoiseType {
    White,
    Pink,
    Brown,
}

impl OptimizedNoiseGenerator {
    pub fn new(noise_type: NoiseType) -> Self {
        Self {
            noise_type,
            amplitude: 1.0,
            seed: 12345,
            rng_state: 12345,
            pink_state: [0.0; 7],
        }
    }

    /// Fast linear congruential generator for real-time use
    #[inline(always)]
    fn fast_random(&mut self) -> f32 {
        self.rng_state = self.rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.rng_state >> 16) as f32 / 32768.0) - 1.0
    }

    /// Generate pink noise using Paul Kellett's filter
    #[inline(always)]
    fn pink_noise_sample(&mut self) -> f32 {
        let white = self.fast_random();

        self.pink_state[0] = 0.99886 * self.pink_state[0] + white * 0.0555179;
        self.pink_state[1] = 0.99332 * self.pink_state[1] + white * 0.0750759;
        self.pink_state[2] = 0.96900 * self.pink_state[2] + white * 0.1538520;
        self.pink_state[3] = 0.86650 * self.pink_state[3] + white * 0.3104856;
        self.pink_state[4] = 0.55000 * self.pink_state[4] + white * 0.5329522;
        self.pink_state[5] = -0.7616 * self.pink_state[5] - white * 0.0168980;

        let pink = self.pink_state[0] + self.pink_state[1] + self.pink_state[2] +
                  self.pink_state[3] + self.pink_state[4] + self.pink_state[5] +
                  self.pink_state[6] + white * 0.5362;

        self.pink_state[6] = white * 0.115926;
        pink * 0.11
    }

    /// Generate brown noise (integrated white noise)
    #[inline(always)]
    fn brown_noise_sample(&mut self) -> f32 {
        let white = self.fast_random() * 0.1;
        self.pink_state[0] += white;

        // Prevent DC buildup
        if self.pink_state[0] > 1.0 {
            self.pink_state[0] = 1.0;
        } else if self.pink_state[0] < -1.0 {
            self.pink_state[0] = -1.0;
        }

        self.pink_state[0]
    }
}

impl OptimizedGenerator for OptimizedNoiseGenerator {
    fn generate_block(&mut self, output: &mut [f32], _sample_rate: f32) {
        match self.noise_type {
            NoiseType::White => {
                for sample in output.iter_mut() {
                    *sample = self.amplitude * self.fast_random();
                }
            },
            NoiseType::Pink => {
                for sample in output.iter_mut() {
                    *sample = self.amplitude * self.pink_noise_sample();
                }
            },
            NoiseType::Brown => {
                for sample in output.iter_mut() {
                    *sample = self.amplitude * self.brown_noise_sample();
                }
            },
        }
    }

    fn reset(&mut self) {
        self.rng_state = self.seed;
        self.pink_state = [0.0; 7];
    }

    fn name(&self) -> &str {
        match self.noise_type {
            NoiseType::White => "White Noise Generator",
            NoiseType::Pink => "Pink Noise Generator",
            NoiseType::Brown => "Brown Noise Generator",
        }
    }

    fn set_frequency(&mut self, _frequency: f32) {
        // Noise generators don't use frequency
    }

    fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }
}

/// Band-limited impulse train generator for accurate digital synthesis
pub struct BandLimitedImpulseGenerator {
    frequency: f32,
    amplitude: f32,
    phase: f32,
    phase_increment: f32,
    harmonics: usize,
    lookup_table: Arc<TrigLookup>,
}

impl BandLimitedImpulseGenerator {
    pub fn new(frequency: f32, harmonics: usize) -> Self {
        Self {
            frequency,
            amplitude: 1.0,
            phase: 0.0,
            phase_increment: 0.0,
            harmonics,
            lookup_table: Arc::new(TrigLookup::new(4096)),
        }
    }

    fn calculate_bandlimited_impulse(&self, phase: f32, sample_rate: f32) -> f32 {
        let max_harmonic = (sample_rate / (2.0 * self.frequency)).floor() as usize;
        let actual_harmonics = self.harmonics.min(max_harmonic);

        let mut sum = 1.0; // DC component

        for h in 1..=actual_harmonics {
            let harmonic_phase = phase * h as f32;
            sum += 2.0 * self.lookup_table.cos(harmonic_phase);
        }

        sum / (2.0 * actual_harmonics as f32 + 1.0)
    }
}

impl OptimizedGenerator for BandLimitedImpulseGenerator {
    fn generate_block(&mut self, output: &mut [f32], sample_rate: f32) {
        self.phase_increment = 2.0 * PI * self.frequency / sample_rate;

        for sample in output.iter_mut() {
            *sample = self.amplitude * self.calculate_bandlimited_impulse(self.phase, sample_rate);
            self.phase = (self.phase + self.phase_increment) % (2.0 * PI);
        }
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }

    fn name(&self) -> &str {
        "Band-Limited Impulse Generator"
    }

    fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }
}

/// Generator pool for efficient memory management
pub struct GeneratorPool {
    available_sine: RwLock<Vec<Box<dyn OptimizedGenerator>>>,
    available_noise: RwLock<Vec<Box<dyn OptimizedGenerator>>>,
    lookup_table: Arc<TrigLookup>,
}

impl GeneratorPool {
    pub fn new(pool_size: usize) -> Self {
        let lookup_table = Arc::new(TrigLookup::new(4096));

        let mut sine_generators = Vec::with_capacity(pool_size);
        let mut noise_generators = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            sine_generators.push(Box::new(SimdSineGenerator::new(440.0, lookup_table.clone())) as Box<dyn OptimizedGenerator>);
            noise_generators.push(Box::new(OptimizedNoiseGenerator::new(NoiseType::White)) as Box<dyn OptimizedGenerator>);
        }

        Self {
            available_sine: RwLock::new(sine_generators),
            available_noise: RwLock::new(noise_generators),
            lookup_table,
        }
    }

    pub fn acquire_sine_generator(&self) -> Option<Box<dyn OptimizedGenerator>> {
        self.available_sine.write().pop()
    }

    pub fn acquire_noise_generator(&self) -> Option<Box<dyn OptimizedGenerator>> {
        self.available_noise.write().pop()
    }

    pub fn release_generator(&self, mut generator: Box<dyn OptimizedGenerator>) {
        generator.reset();

        match generator.name() {
            "SIMD Sine Generator" => {
                self.available_sine.write().push(generator);
            },
            name if name.contains("Noise") => {
                self.available_noise.write().push(generator);
            },
            _ => {
                // Unknown generator type, just drop it
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trig_lookup() {
        let lookup = TrigLookup::new(1024);

        // Test known values
        assert!((lookup.sin(0.0) - 0.0).abs() < 0.01);
        assert!((lookup.sin(PI / 2.0) - 1.0).abs() < 0.01);
        assert!((lookup.cos(0.0) - 1.0).abs() < 0.01);
        assert!((lookup.cos(PI) + 1.0).abs() < 0.01);
    }

    #[test]
    fn test_simd_sine_generator() {
        let lookup = Arc::new(TrigLookup::new(1024));
        let mut gen = SimdSineGenerator::new(440.0, lookup);

        let mut output = vec![0.0; 1024];
        gen.generate_block(&mut output, 44100.0);

        // Check that we have non-zero output
        assert!(output.iter().any(|&x| x.abs() > 0.1));

        // Check that amplitude is approximately correct
        let max_amplitude = output.iter().map(|&x| x.abs()).fold(0.0, f32::max);
        assert!(max_amplitude > 0.8 && max_amplitude <= 1.0);
    }

    #[test]
    fn test_multi_tone_generator() {
        let frequencies = vec![440.0, 880.0];
        let amplitudes = vec![0.5, 0.5];
        let mut gen = MultiToneGenerator::new(frequencies, amplitudes);

        let mut output = vec![0.0; 1024];
        gen.generate_block(&mut output, 44100.0);

        // Should have higher amplitude than single tone due to mixing
        let rms = (output.iter().map(|&x| x * x).sum::<f32>() / output.len() as f32).sqrt();
        assert!(rms > 0.3);
    }

    #[test]
    fn test_noise_generators() {
        for noise_type in [NoiseType::White, NoiseType::Pink, NoiseType::Brown] {
            let mut gen = OptimizedNoiseGenerator::new(noise_type);
            let mut output = vec![0.0; 1024];
            gen.generate_block(&mut output, 44100.0);

            // Check that we have noise (non-zero variance)
            let mean = output.iter().sum::<f32>() / output.len() as f32;
            let variance = output.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / output.len() as f32;
            assert!(variance > 0.01);
        }
    }

    #[test]
    fn test_generator_pool() {
        let pool = GeneratorPool::new(4);

        // Acquire generators
        let gen1 = pool.acquire_sine_generator();
        let gen2 = pool.acquire_noise_generator();

        assert!(gen1.is_some());
        assert!(gen2.is_some());

        // Release them back
        pool.release_generator(gen1.unwrap());
        pool.release_generator(gen2.unwrap());

        // Should be able to acquire again
        let gen3 = pool.acquire_sine_generator();
        assert!(gen3.is_some());
    }
}