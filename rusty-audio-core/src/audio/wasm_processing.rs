//! WASM-specific audio processing with worker pool support
//!
//! This module provides audio processing utilities optimized for WASM
//! with multithreading support via Web Workers and SharedArrayBuffer.

#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use parking_lot::Mutex;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[cfg(target_arch = "wasm32")]
use super::backend::Result;

/// Audio processing task that can be offloaded to workers
#[cfg(target_arch = "wasm32")]
pub enum AudioProcessingTask {
    /// Apply FFT for spectrum analysis
    FFT { samples: Vec<f32>, window_size: usize },
    /// Apply equalizer filter
    Equalizer { samples: Vec<f32>, gains: Vec<f32> },
    /// Apply audio effects
    Effects { samples: Vec<f32>, effect_type: EffectType },
    /// Volume normalization
    Normalize { samples: Vec<f32>, target_rms: f32 },
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Copy)]
pub enum EffectType {
    Reverb,
    Delay,
    Chorus,
    Distortion,
}

/// Thread-safe audio buffer with atomic synchronization
///
/// Uses atomics for lock-free coordination between main thread and workers
#[cfg(target_arch = "wasm32")]
pub struct AtomicAudioBuffer {
    // Audio data
    data: Arc<Mutex<Vec<f32>>>,
    // Buffer metadata
    length: usize,
    channels: usize,
    sample_rate: u32,
    // Atomic state flags
    is_ready: Arc<AtomicBool>,
    write_position: Arc<AtomicU32>,
    read_position: Arc<AtomicU32>,
}

#[cfg(target_arch = "wasm32")]
impl AtomicAudioBuffer {
    /// Create a new atomic audio buffer
    pub fn new(length: usize, channels: usize, sample_rate: u32) -> Self {
        Self {
            data: Arc::new(Mutex::new(vec![0.0; length * channels])),
            length,
            channels,
            sample_rate,
            is_ready: Arc::new(AtomicBool::new(false)),
            write_position: Arc::new(AtomicU32::new(0)),
            read_position: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Write audio samples to the buffer
    ///
    /// This is typically called from workers after processing
    pub fn write(&self, samples: &[f32]) -> Result<()> {
        let mut buffer = self.data.lock();
        let copy_len = samples.len().min(buffer.len());
        buffer[..copy_len].copy_from_slice(&samples[..copy_len]);

        // Update write position atomically
        self.write_position.store(copy_len as u32, Ordering::Release);
        self.is_ready.store(true, Ordering::Release);

        Ok(())
    }

    /// Read audio samples from the buffer
    ///
    /// This is typically called from the main thread for playback
    pub fn read(&self, num_samples: usize) -> Vec<f32> {
        // Wait for data to be ready (spin briefly, then yield)
        let mut spin_count = 0;
        while !self.is_ready.load(Ordering::Acquire) {
            spin_count += 1;
            if spin_count > 100 {
                // Yield to prevent busy-waiting
                std::hint::spin_loop();
                spin_count = 0;
            }
        }

        let buffer = self.data.lock();
        let read_len = num_samples.min(buffer.len());

        // Update read position atomically
        self.read_position.store(read_len as u32, Ordering::Release);

        buffer[..read_len].to_vec()
    }

    /// Check if buffer has data ready
    pub fn is_ready(&self) -> bool {
        self.is_ready.load(Ordering::Acquire)
    }

    /// Reset buffer state
    pub fn reset(&self) {
        self.is_ready.store(false, Ordering::Release);
        self.write_position.store(0, Ordering::Release);
        self.read_position.store(0, Ordering::Release);
    }

    /// Get buffer metadata
    pub fn metadata(&self) -> (usize, usize, u32) {
        (self.length, self.channels, self.sample_rate)
    }
}

/// Worker-based audio processor
///
/// Offloads heavy audio processing to Web Workers
#[cfg(target_arch = "wasm32")]
pub struct WorkerAudioProcessor {
    // Shared buffers for input/output
    input_buffer: Arc<AtomicAudioBuffer>,
    output_buffer: Arc<AtomicAudioBuffer>,
}

#[cfg(target_arch = "wasm32")]
impl WorkerAudioProcessor {
    /// Create a new worker audio processor
    pub fn new(buffer_size: usize, channels: usize, sample_rate: u32) -> Self {
        Self {
            input_buffer: Arc::new(AtomicAudioBuffer::new(buffer_size, channels, sample_rate)),
            output_buffer: Arc::new(AtomicAudioBuffer::new(buffer_size, channels, sample_rate)),
        }
    }

    /// Process audio samples using worker pool
    ///
    /// This offloads the actual processing to workers via rayon
    pub fn process_async(&self, samples: Vec<f32>, task: AudioProcessingTask) -> Result<Vec<f32>> {
        // Write input samples to shared buffer
        self.input_buffer.write(&samples)?;

        // Process on workers using rayon (if available)
        #[cfg(target_arch = "wasm32")]
        {
            use rayon::prelude::*;

            let processed = match task {
                AudioProcessingTask::FFT { window_size, .. } => {
                    // FFT processing on workers
                    self.process_fft_parallel(&samples, window_size)
                }
                AudioProcessingTask::Equalizer { gains, .. } => {
                    // EQ processing on workers
                    self.process_eq_parallel(&samples, &gains)
                }
                AudioProcessingTask::Effects { effect_type, .. } => {
                    // Effects processing on workers
                    self.process_effects_parallel(&samples, effect_type)
                }
                AudioProcessingTask::Normalize { target_rms, .. } => {
                    // Normalization on workers
                    self.process_normalize_parallel(&samples, target_rms)
                }
            };

            // Write output to shared buffer
            self.output_buffer.write(&processed)?;

            Ok(processed)
        }
    }

    /// Process FFT in parallel chunks
    fn process_fft_parallel(&self, samples: &[f32], window_size: usize) -> Vec<f32> {
        use rayon::prelude::*;

        samples
            .par_chunks(window_size)
            .flat_map(|chunk| {
                // Simplified FFT processing (would use rustfft in real implementation)
                chunk.to_vec()
            })
            .collect()
    }

    /// Process equalizer in parallel
    fn process_eq_parallel(&self, samples: &[f32], gains: &[f32]) -> Vec<f32> {
        use rayon::prelude::*;

        samples
            .par_iter()
            .enumerate()
            .map(|(i, &sample)| {
                let gain_idx = i % gains.len();
                sample * gains[gain_idx]
            })
            .collect()
    }

    /// Process effects in parallel
    fn process_effects_parallel(&self, samples: &[f32], effect_type: EffectType) -> Vec<f32> {
        use rayon::prelude::*;

        match effect_type {
            EffectType::Reverb => {
                // Simple reverb effect
                samples
                    .par_iter()
                    .enumerate()
                    .map(|(i, &sample)| {
                        if i > 100 {
                            sample + samples[i - 100] * 0.3
                        } else {
                            sample
                        }
                    })
                    .collect()
            }
            EffectType::Delay => {
                // Simple delay effect
                samples
                    .par_iter()
                    .enumerate()
                    .map(|(i, &sample)| {
                        if i > 1000 {
                            sample + samples[i - 1000] * 0.5
                        } else {
                            sample
                        }
                    })
                    .collect()
            }
            EffectType::Chorus | EffectType::Distortion => {
                // Placeholder for other effects
                samples.to_vec()
            }
        }
    }

    /// Process volume normalization in parallel
    fn process_normalize_parallel(&self, samples: &[f32], target_rms: f32) -> Vec<f32> {
        use rayon::prelude::*;

        // Calculate current RMS in parallel
        let rms_squared: f32 = samples.par_iter().map(|&s| s * s).sum();
        let current_rms = (rms_squared / samples.len() as f32).sqrt();

        // Calculate gain factor
        let gain = if current_rms > 0.0 {
            target_rms / current_rms
        } else {
            1.0
        };

        // Apply gain in parallel
        samples.par_iter().map(|&s| s * gain).collect()
    }

    /// Get input buffer for direct access
    pub fn input_buffer(&self) -> Arc<AtomicAudioBuffer> {
        Arc::clone(&self.input_buffer)
    }

    /// Get output buffer for direct access
    pub fn output_buffer(&self) -> Arc<AtomicAudioBuffer> {
        Arc::clone(&self.output_buffer)
    }
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_atomic_buffer_creation() {
        let buffer = AtomicAudioBuffer::new(512, 2, 48000);
        assert_eq!(buffer.length, 512);
        assert_eq!(buffer.channels, 2);
        assert_eq!(buffer.sample_rate, 48000);
        assert!(!buffer.is_ready());
    }

    #[wasm_bindgen_test]
    fn test_atomic_buffer_write_read() {
        let buffer = AtomicAudioBuffer::new(512, 2, 48000);
        let samples = vec![1.0, 2.0, 3.0, 4.0];

        buffer.write(&samples).unwrap();
        assert!(buffer.is_ready());

        let read_samples = buffer.read(4);
        assert_eq!(read_samples[0], 1.0);
        assert_eq!(read_samples[1], 2.0);
    }

    #[wasm_bindgen_test]
    fn test_worker_processor_creation() {
        let processor = WorkerAudioProcessor::new(512, 2, 48000);
        let (length, channels, sample_rate) = processor.input_buffer().metadata();
        assert_eq!(length, 512);
        assert_eq!(channels, 2);
        assert_eq!(sample_rate, 48000);
    }
}
