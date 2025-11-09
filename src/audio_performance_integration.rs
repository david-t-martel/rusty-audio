//! Integration layer for audio performance optimizations
//!
//! This module provides a simplified API for integrating the optimized buffer pool
//! and SIMD processing into the spectrum visualizer and audio engine.
//!
//! Architecture:
//! - DESKTOP: Uses AVX2/SSE SIMD when available, falls back to scalar
//! - WASM: Uses scalar processing only (SIMD not available)
//! - BOTH: Benefit from buffer pooling and cache-line alignment

use crate::audio_performance_optimized::{
    OptimizedBufferPoolV2, PooledSpectrumProcessor, ParallelEqProcessor,
    ZeroCopyAudioPipeline, AlignedBuffer,
};
use std::sync::Arc;
use web_audio_api::node::AnalyserNode;

/// Simplified spectrum analyzer with integrated buffer pooling
///
/// This wraps the complex optimizations in a simple API suitable for the UI layer.
pub struct OptimizedSpectrumAnalyzer {
    processor: PooledSpectrumProcessor,
    buffer_pool: Arc<OptimizedBufferPoolV2>,
    fft_size: usize,
}

impl OptimizedSpectrumAnalyzer {
    /// Create a new optimized spectrum analyzer
    ///
    /// # Arguments
    /// * `fft_size` - FFT size (must be power of 2, typically 2048)
    /// * `pool_size` - Number of buffers to pre-allocate (typically 8-16)
    pub fn new(fft_size: usize, pool_size: usize) -> Self {
        let buffer_size = fft_size / 2; // Frequency bins
        let buffer_pool = Arc::new(OptimizedBufferPoolV2::new(pool_size, buffer_size));

        Self {
            processor: PooledSpectrumProcessor::new(Arc::clone(&buffer_pool), fft_size),
            buffer_pool,
            fft_size,
        }
    }

    /// Process spectrum data from an AnalyserNode
    ///
    /// Returns a slice to the smoothed spectrum data (no allocation).
    /// The data is owned by the processor and remains valid until next call.
    #[inline(always)]
    pub fn process_spectrum(&mut self, analyser: &mut AnalyserNode) -> &[f32] {
        self.processor.process_spectrum_pooled(analyser)
    }

    /// Get buffer pool statistics for monitoring
    ///
    /// Returns: (buffers_available, allocations_saved, cache_hits)
    pub fn get_stats(&self) -> (usize, usize, usize) {
        self.buffer_pool.stats()
    }

    /// Get the FFT size
    pub fn fft_size(&self) -> usize {
        self.fft_size
    }
}

/// Optimized audio pipeline for complete audio processing chain
///
/// Integrates EQ processing, spectrum analysis, and buffer management
/// in a zero-copy architecture.
pub struct OptimizedAudioPipeline {
    pipeline: ZeroCopyAudioPipeline,
    max_block_size: usize,
}

impl OptimizedAudioPipeline {
    /// Create a new optimized audio pipeline
    ///
    /// # Arguments
    /// * `max_block_size` - Maximum audio block size in samples
    /// * `num_eq_bands` - Number of EQ bands (typically 8)
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `fft_size` - FFT size for spectrum analysis
    pub fn new(max_block_size: usize, num_eq_bands: usize, sample_rate: f32, fft_size: usize) -> Self {
        Self {
            pipeline: ZeroCopyAudioPipeline::new(max_block_size, num_eq_bands, sample_rate, fft_size),
            max_block_size,
        }
    }

    /// Process audio through the complete pipeline
    ///
    /// # Arguments
    /// * `input` - Input audio samples
    /// * `output` - Output buffer (must be same size as input)
    /// * `analyser` - Web Audio API AnalyserNode for spectrum extraction
    ///
    /// Returns: Spectrum data slice (no allocation)
    #[inline(always)]
    pub fn process(&mut self, input: &[f32], output: &mut [f32], analyser: &mut AnalyserNode) -> &[f32] {
        assert!(input.len() <= self.max_block_size, "Input exceeds max block size");
        self.pipeline.process_zero_copy(input, output, analyser)
    }

    /// Update EQ band settings
    ///
    /// # Arguments
    /// * `band_idx` - Band index (0 to num_eq_bands-1)
    /// * `frequency` - Center frequency in Hz
    /// * `q` - Q factor (resonance)
    /// * `gain_db` - Gain in decibels
    pub fn update_eq_band(&mut self, band_idx: usize, frequency: f32, q: f32, gain_db: f32) {
        self.pipeline.eq_processor.update_band(band_idx, frequency, q, gain_db);
    }

    /// Get pipeline statistics
    pub fn get_stats(&self) -> String {
        self.pipeline.stats()
    }
}

/// Builder pattern for easy construction of optimized components
pub struct AudioOptimizationBuilder {
    fft_size: usize,
    pool_size: usize,
    max_block_size: usize,
    num_eq_bands: usize,
    sample_rate: f32,
}

impl Default for AudioOptimizationBuilder {
    fn default() -> Self {
        Self {
            fft_size: 2048,
            pool_size: 16,
            max_block_size: 4096,
            num_eq_bands: 8,
            sample_rate: 44100.0,
        }
    }
}

impl AudioOptimizationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fft_size(mut self, size: usize) -> Self {
        self.fft_size = size;
        self
    }

    pub fn pool_size(mut self, size: usize) -> Self {
        self.pool_size = size;
        self
    }

    pub fn max_block_size(mut self, size: usize) -> Self {
        self.max_block_size = size;
        self
    }

    pub fn num_eq_bands(mut self, bands: usize) -> Self {
        self.num_eq_bands = bands;
        self
    }

    pub fn sample_rate(mut self, rate: f32) -> Self {
        self.sample_rate = rate;
        self
    }

    /// Build an optimized spectrum analyzer
    pub fn build_spectrum_analyzer(self) -> OptimizedSpectrumAnalyzer {
        OptimizedSpectrumAnalyzer::new(self.fft_size, self.pool_size)
    }

    /// Build a complete optimized audio pipeline
    pub fn build_audio_pipeline(self) -> OptimizedAudioPipeline {
        OptimizedAudioPipeline::new(
            self.max_block_size,
            self.num_eq_bands,
            self.sample_rate,
            self.fft_size,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        let analyzer = AudioOptimizationBuilder::new()
            .fft_size(2048)
            .pool_size(16)
            .build_spectrum_analyzer();

        assert_eq!(analyzer.fft_size(), 2048);
        let (available, _, _) = analyzer.get_stats();
        assert_eq!(available, 16, "Pool should have 16 buffers available");
    }

    #[test]
    fn test_spectrum_analyzer_creation() {
        let analyzer = OptimizedSpectrumAnalyzer::new(2048, 16);
        assert_eq!(analyzer.fft_size(), 2048);
    }

    #[test]
    fn test_audio_pipeline_creation() {
        let pipeline = OptimizedAudioPipeline::new(4096, 8, 44100.0, 2048);
        let stats = pipeline.get_stats();
        assert!(stats.contains("Pipeline Stats"));
    }
}
