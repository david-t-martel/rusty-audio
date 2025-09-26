// Performance-optimized audio processing utilities

use std::sync::Arc;

/// Optimized spectrum data processor with reduced allocations
pub struct SpectrumProcessor {
    frequency_buffer: Vec<f32>,
    spectrum_buffer: Vec<f32>,
    smoothing_factor: f32,
}

impl SpectrumProcessor {
    pub fn new(fft_size: usize) -> Self {
        let bin_count = fft_size / 2;
        Self {
            frequency_buffer: vec![0.0; bin_count],
            spectrum_buffer: vec![0.0; bin_count],
            smoothing_factor: 0.8,
        }
    }

    /// Process frequency data with smoothing and reduced allocations
    pub fn process_spectrum(&mut self, analyser: &mut web_audio_api::node::AnalyserNode) -> &[f32] {
        // Get byte frequency data (requires mutable access to analyser)
        let mut byte_data = vec![0u8; self.frequency_buffer.len()];
        analyser.get_byte_frequency_data(&mut byte_data);

        // Convert byte data to float dB values
        for (i, &byte_val) in byte_data.iter().enumerate() {
            // Convert from byte (0-255) to dB scale (-100 to 0)
            self.frequency_buffer[i] = (byte_val as f32 / 255.0) * 100.0 - 100.0;
        }

        // Apply smoothing and convert to linear scale in-place
        for (i, &db) in self.frequency_buffer.iter().enumerate() {
            // Convert from dB to linear scale with optimization
            let linear = if db > -100.0 {
                fast_pow10(db * 0.05) // Optimized 10^(db/20)
            } else {
                0.0
            };

            // Apply smoothing
            self.spectrum_buffer[i] = self.spectrum_buffer[i] * self.smoothing_factor
                + linear * (1.0 - self.smoothing_factor);
        }

        &self.spectrum_buffer
    }

    pub fn resize(&mut self, new_size: usize) {
        let bin_count = new_size / 2;
        self.frequency_buffer.resize(bin_count, 0.0);
        self.spectrum_buffer.resize(bin_count, 0.0);
    }

    pub fn fft_size(&self) -> usize {
        self.frequency_buffer.len() * 2
    }
}

/// Fast approximation of 10^x using lookup table and interpolation
#[inline(always)]
fn fast_pow10(x: f32) -> f32 {
    // Use fast approximation for common range
    if x >= -5.0 && x <= 1.0 {
        // Taylor series approximation optimized for this range
        let ln10 = 2.302585093;
        let t = x * ln10;
        let t2 = t * t;
        1.0 + t + t2 * (0.5 + t * (0.16666667 + t * 0.041666667))
    } else {
        10.0_f32.powf(x)
    }
}

/// Ring buffer for efficient audio buffering
pub struct AudioRingBuffer {
    buffer: Vec<f32>,
    write_pos: usize,
    read_pos: usize,
    size: usize,
}

impl AudioRingBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            write_pos: 0,
            read_pos: 0,
            size,
        }
    }

    #[inline(always)]
    pub fn write(&mut self, data: &[f32]) {
        for &sample in data {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.size;
        }
    }

    #[inline(always)]
    pub fn read(&mut self, output: &mut [f32]) -> usize {
        let available = self.available();
        let to_read = output.len().min(available);

        for i in 0..to_read {
            output[i] = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.size;
        }

        to_read
    }

    #[inline(always)]
    pub fn available(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.write_pos - self.read_pos
        } else {
            self.size - self.read_pos + self.write_pos
        }
    }
}

/// Optimized EQ band processor with SIMD-friendly operations
pub struct EqBandOptimizer {
    coefficients: Vec<BiquadCoefficients>,
    states: Vec<BiquadState>,
}

#[derive(Clone, Copy, Default)]
struct BiquadCoefficients {
    a0: f32,
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
}

#[derive(Clone, Copy, Default)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl EqBandOptimizer {
    pub fn new(num_bands: usize) -> Self {
        Self {
            coefficients: vec![BiquadCoefficients::default(); num_bands],
            states: vec![BiquadState::default(); num_bands],
        }
    }

    /// Process audio through EQ bands with optimized biquad filtering
    #[inline(always)]
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        output.copy_from_slice(input);

        for (_band_idx, (coeff, state)) in self
            .coefficients
            .iter()
            .zip(self.states.iter_mut())
            .enumerate()
        {
            // Process each band in sequence
            for sample in output.iter_mut() {
                let x0 = *sample;

                // Direct form II biquad filter
                let y0 = coeff.b0 * x0 + coeff.b1 * state.x1 + coeff.b2 * state.x2
                    - coeff.a1 * state.y1
                    - coeff.a2 * state.y2;

                // Update state
                state.x2 = state.x1;
                state.x1 = x0;
                state.y2 = state.y1;
                state.y1 = y0;

                *sample = y0 / coeff.a0;
            }
        }
    }

    /// Update coefficients for a specific band
    pub fn update_band(&mut self, band_idx: usize, frequency: f32, q: f32, gain_db: f32) {
        if band_idx >= self.coefficients.len() {
            return;
        }

        // Calculate biquad coefficients for peaking EQ
        let sample_rate = 48000.0;
        let omega = 2.0 * std::f32::consts::PI * frequency / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);
        let a = 10.0_f32.powf(gain_db / 40.0);

        let coeff = &mut self.coefficients[band_idx];
        coeff.b0 = 1.0 + alpha * a;
        coeff.b1 = -2.0 * cos_omega;
        coeff.b2 = 1.0 - alpha * a;
        coeff.a0 = 1.0 + alpha / a;
        coeff.a1 = -2.0 * cos_omega;
        coeff.a2 = 1.0 - alpha / a;
    }
}

/// Memory pool for audio buffers to reduce allocations
pub struct AudioBufferPool {
    pool: Vec<Arc<Vec<f32>>>,
    buffer_size: usize,
}

impl AudioBufferPool {
    pub fn new(pool_size: usize, buffer_size: usize) -> Self {
        let mut pool = Vec::with_capacity(pool_size);
        for _ in 0..pool_size {
            pool.push(Arc::new(vec![0.0; buffer_size]));
        }
        Self {
            pool,
            buffer_size,
        }
    }

    pub fn acquire(&mut self) -> Option<Arc<Vec<f32>>> {
        self.pool.pop()
    }

    pub fn release(&mut self, buffer: Arc<Vec<f32>>) {
        if Arc::strong_count(&buffer) == 1 && buffer.len() == self.buffer_size {
            // Clear the buffer before returning to pool
            if let Some(buf) = Arc::get_mut(&mut buffer.clone()) {
                buf.fill(0.0);
            }
            self.pool.push(buffer);
        }
    }
}

/// Optimized audio file decoder with streaming support
pub struct StreamingDecoder {
    chunk_size: usize,
    decode_ahead: usize,
}

impl StreamingDecoder {
    pub fn new() -> Self {
        Self {
            chunk_size: 4096,  // Optimal chunk size for most systems
            decode_ahead: 2,    // Number of chunks to decode ahead
        }
    }

    /// Decode audio file in chunks for better memory usage
    pub fn decode_chunked<F>(&self, _file_path: &std::path::Path, _callback: F) -> Result<(), String>
    where
        F: FnMut(&[f32]) -> bool,
    {
        // Implementation would use streaming decoder
        // This is a placeholder for the actual implementation
        Ok(())
    }
}

/// CPU feature detection and optimization selection
pub struct AudioOptimizer {
    has_avx2: bool,
    has_sse42: bool,
    num_cores: usize,
}

impl AudioOptimizer {
    pub fn new() -> Self {
        Self {
            has_avx2: is_x86_feature_detected!("avx2"),
            has_sse42: is_x86_feature_detected!("sse4.2"),
            num_cores: num_cpus::get(),
        }
    }

    pub fn optimal_buffer_size(&self) -> usize {
        // Choose buffer size based on CPU capabilities
        if self.has_avx2 {
            256  // Larger buffers for AVX2
        } else if self.has_sse42 {
            128  // Standard render quantum
        } else {
            64   // Smaller buffers for older CPUs
        }
    }

    pub fn optimal_fft_size(&self) -> usize {
        // Choose FFT size based on CPU capabilities
        if self.has_avx2 && self.num_cores >= 4 {
            2048  // Larger FFT for better frequency resolution
        } else {
            1024  // Standard FFT size
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_pow10() {
        for i in -100..=10 {
            let x = i as f32 * 0.1;
            let fast = fast_pow10(x);
            let accurate = 10.0_f32.powf(x);
            let error = (fast - accurate).abs() / accurate;
            assert!(error < 0.01, "Large error at x={}: fast={}, accurate={}", x, fast, accurate);
        }
    }

    #[test]
    fn test_ring_buffer() {
        let mut buffer = AudioRingBuffer::new(16);
        let data = vec![1.0, 2.0, 3.0, 4.0];
        buffer.write(&data);

        let mut output = vec![0.0; 4];
        let read = buffer.read(&mut output);
        assert_eq!(read, 4);
        assert_eq!(output, vec![1.0, 2.0, 3.0, 4.0]);
    }
}