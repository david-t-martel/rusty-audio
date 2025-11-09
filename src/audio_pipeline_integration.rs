//! Integration module for optimized audio pipeline
//!
//! This module provides the integration layer between the main application
//! and the optimized audio processing components.

use crate::audio_performance_optimized::{
    OptimizedBufferPoolV2, ParallelEqProcessor, PooledSpectrumProcessor, ZeroCopyAudioPipeline,
};
use std::sync::Arc;
use web_audio_api::node::AnalyserNode;

/// Main audio processing coordinator with performance optimizations
pub struct OptimizedAudioProcessor {
    /// Zero-copy pipeline for audio processing
    pipeline: ZeroCopyAudioPipeline,
    /// Statistics tracking
    frames_processed: u64,
    total_processing_time_ns: u64,
}

impl OptimizedAudioProcessor {
    /// Create a new optimized audio processor
    pub fn new(
        max_block_size: usize,
        num_eq_bands: usize,
        sample_rate: f32,
        fft_size: usize,
    ) -> Self {
        Self {
            pipeline: ZeroCopyAudioPipeline::new(
                max_block_size,
                num_eq_bands,
                sample_rate,
                fft_size,
            ),
            frames_processed: 0,
            total_processing_time_ns: 0,
        }
    }

    /// Process audio frame with performance tracking
    pub fn process_frame(
        &mut self,
        input: &[f32],
        output: &mut [f32],
        analyser: &mut AnalyserNode,
    ) -> ProcessingResult {
        let start = std::time::Instant::now();

        // Process through zero-copy pipeline
        let spectrum = self.pipeline.process_zero_copy(input, output, analyser);

        let elapsed = start.elapsed();
        self.frames_processed += 1;
        self.total_processing_time_ns += elapsed.as_nanos() as u64;

        ProcessingResult {
            spectrum: spectrum.to_vec(),
            processing_time_us: elapsed.as_micros() as u32,
            pipeline_stats: self.pipeline.stats(),
        }
    }

    /// Update EQ band parameters
    pub fn update_eq_band(&mut self, band_idx: usize, frequency: f32, q: f32, gain_db: f32) {
        self.pipeline
            .eq_processor
            .update_band(band_idx, frequency, q, gain_db);
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> PerformanceStats {
        let avg_processing_time_us = if self.frames_processed > 0 {
            (self.total_processing_time_ns / self.frames_processed / 1000) as u32
        } else {
            0
        };

        PerformanceStats {
            frames_processed: self.frames_processed,
            average_processing_time_us: avg_processing_time_us,
            pipeline_info: self.pipeline.stats(),
        }
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.frames_processed = 0;
        self.total_processing_time_ns = 0;
    }
}

/// Result from audio processing
pub struct ProcessingResult {
    /// Processed spectrum data
    pub spectrum: Vec<f32>,
    /// Processing time in microseconds
    pub processing_time_us: u32,
    /// Pipeline statistics
    pub pipeline_stats: String,
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Total frames processed
    pub frames_processed: u64,
    /// Average processing time per frame in microseconds
    pub average_processing_time_us: u32,
    /// Pipeline information
    pub pipeline_info: String,
}

/// Integration helper for transitioning from old to new system
pub mod migration {
    use super::*;
    use crate::audio_performance::OptimizedSpectrumProcessor;

    /// Create a compatibility wrapper for gradual migration
    pub struct CompatibilityWrapper {
        old_processor: OptimizedSpectrumProcessor,
        new_processor: Option<OptimizedAudioProcessor>,
        use_new_pipeline: bool,
    }

    impl CompatibilityWrapper {
        /// Create a new compatibility wrapper
        pub fn new(fft_size: usize) -> Self {
            Self {
                old_processor: OptimizedSpectrumProcessor::new(fft_size),
                new_processor: None,
                use_new_pipeline: false,
            }
        }

        /// Enable the new optimized pipeline
        pub fn enable_optimized_pipeline(
            &mut self,
            max_block_size: usize,
            num_eq_bands: usize,
            sample_rate: f32,
            fft_size: usize,
        ) {
            self.new_processor = Some(OptimizedAudioProcessor::new(
                max_block_size,
                num_eq_bands,
                sample_rate,
                fft_size,
            ));
            self.use_new_pipeline = true;
        }

        /// Process spectrum using appropriate processor
        pub fn process_spectrum(&mut self, analyser: &mut AnalyserNode) -> Vec<f32> {
            if self.use_new_pipeline {
                if let Some(ref mut processor) = self.new_processor {
                    // For spectrum-only processing
                    let dummy_input = vec![0.0; 1024];
                    let mut dummy_output = vec![0.0; 1024];
                    let result = processor.process_frame(&dummy_input, &mut dummy_output, analyser);
                    return result.spectrum;
                }
            }

            // Fallback to old processor
            self.old_processor.process_spectrum(analyser).to_vec()
        }

        /// Toggle between old and new pipeline
        pub fn toggle_pipeline(&mut self) -> bool {
            if self.new_processor.is_some() {
                self.use_new_pipeline = !self.use_new_pipeline;
                self.use_new_pipeline
            } else {
                false
            }
        }

        /// Get current pipeline status
        pub fn is_optimized_enabled(&self) -> bool {
            self.use_new_pipeline && self.new_processor.is_some()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = OptimizedAudioProcessor::new(1024, 8, 44100.0, 2048);
        let stats = processor.get_stats();
        assert_eq!(stats.frames_processed, 0);
    }

    #[test]
    fn test_compatibility_wrapper() {
        let mut wrapper = migration::CompatibilityWrapper::new(2048);
        assert!(!wrapper.is_optimized_enabled());

        wrapper.enable_optimized_pipeline(1024, 8, 44100.0, 2048);
        assert!(wrapper.is_optimized_enabled());

        wrapper.toggle_pipeline();
        assert!(!wrapper.is_optimized_enabled());
    }
}
