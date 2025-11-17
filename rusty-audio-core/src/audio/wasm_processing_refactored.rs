//! WASM-specific audio processing with worker pool support (REFACTORED)
//!
//! This module provides audio processing utilities optimized for WASM
//! with multithreading support via Web Workers and SharedArrayBuffer.
//!
//! ## P0 Fixes Applied:
//! - P0-5: Replaced infinite spin loop with timeout mechanism

#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use parking_lot::Mutex;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
#[cfg(target_arch = "wasm32")]
use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
use super::backend::Result;

/// Maximum time to wait for buffer readiness (milliseconds)
const BUFFER_READY_TIMEOUT_MS: u64 = 100;

/// Number of spin iterations before yielding
const MAX_SPIN_ITERATIONS: usize = 100;

/// Audio processing task that can be offloaded to workers
#[cfg(target_arch = "wasm32")]
pub enum AudioProcessingTask {
    /// Apply FFT for spectrum analysis
    FFT { samples: Vec<f32>, window_size: usize },
    /// Apply equalizer filter
    Equalizer { samples: Vec<f32>, gains: Vec<f32> },
    /// Apply audio effects
    Effects {
        samples: Vec<f32>,
        effect_type: EffectType,
    },
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

#[cfg(target_arch = "wasm32")]
/// Result of a buffer read operation
pub enum BufferReadResult<T> {
    /// Data is ready and returned
    Ready(T),
    /// Timeout occurred waiting for data
    Timeout,
    /// Buffer is empty or not initialized
    NotReady,
}

#[cfg(target_arch = "wasm32")]
/// Thread-safe audio buffer with atomic synchronization
///
/// **P0-5 FIX**: Replaced infinite spin loop with timeout mechanism
///
/// ## Before (INFINITE LOOP):
/// ```rust,ignore
/// pub fn read(&self, num_samples: usize) -> Vec<f32> {
///     let mut spin_count = 0;
///     while !self.is_ready.load(Ordering::Acquire) {
///         spin_count += 1;
///         if spin_count > 100 {
///             std::hint::spin_loop();  // ❌ Never yields in WASM!
///             spin_count = 0;
///         }
///     }
///     // ... read data
/// }
/// ```
///
/// ## After (TIMEOUT-BASED):
/// ```rust,ignore
/// pub fn read_with_timeout(&self, num_samples: usize, timeout: Duration) -> BufferReadResult<Vec<f32>> {
///     let start = Instant::now();
///     let mut spin_count = 0;
///
///     while !self.is_ready.load(Ordering::Acquire) {
///         if start.elapsed() > timeout {
///             return BufferReadResult::Timeout;  // ✅ Returns after timeout
///         }
///         spin_count += 1;
///         if spin_count > MAX_SPIN_ITERATIONS {
///             // In WASM, we can't truly yield, but we can return early
///             return BufferReadResult::NotReady;
///         }
///     }
///     BufferReadResult::Ready(/* data */)
/// }
/// ```
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
        self.write_position
            .store(copy_len as u32, Ordering::Release);
        self.is_ready.store(true, Ordering::Release);

        Ok(())
    }

    /// Read audio samples from the buffer (legacy method - kept for compatibility)
    ///
    /// **DEPRECATED**: Use `read_with_timeout` instead to avoid potential hangs
    ///
    /// This method uses a short timeout internally for safety.
    pub fn read(&self, num_samples: usize) -> Vec<f32> {
        match self.read_with_timeout(num_samples, Duration::from_millis(BUFFER_READY_TIMEOUT_MS))
        {
            BufferReadResult::Ready(data) => data,
            BufferReadResult::Timeout => {
                log::warn!("Buffer read timeout - returning silence");
                vec![0.0; num_samples.min(self.length * self.channels)]
            }
            BufferReadResult::NotReady => {
                log::debug!("Buffer not ready - returning silence");
                vec![0.0; num_samples.min(self.length * self.channels)]
            }
        }
    }

    /// Read audio samples from the buffer with timeout
    ///
    /// **P0-5 FIX**: Implements proper timeout mechanism
    ///
    /// # Arguments
    /// * `num_samples` - Number of samples to read
    /// * `timeout` - Maximum time to wait for data
    ///
    /// # Returns
    /// - `BufferReadResult::Ready(data)` if data is available
    /// - `BufferReadResult::Timeout` if timeout elapsed
    /// - `BufferReadResult::NotReady` if data not ready after max spins
    ///
    /// ## Performance Characteristics:
    /// - Fast path: O(1) if data is ready (< 1μs)
    /// - Slow path: O(n) with n = timeout / spin_interval
    /// - Memory: O(num_samples) for output buffer
    pub fn read_with_timeout(
        &self,
        num_samples: usize,
        timeout: Duration,
    ) -> BufferReadResult<Vec<f32>> {
        let start = Instant::now();
        let mut spin_count = 0;

        // Wait for data to be ready with timeout
        while !self.is_ready.load(Ordering::Acquire) {
            // Check timeout
            if start.elapsed() > timeout {
                log::debug!(
                    "Buffer read timeout after {:?} (spin_count: {})",
                    timeout,
                    spin_count
                );
                return BufferReadResult::Timeout;
            }

            spin_count += 1;

            // After max spins, return NotReady
            // In WASM, spin_loop() doesn't yield to the browser's event loop,
            // so we limit spinning and return early
            if spin_count > MAX_SPIN_ITERATIONS {
                log::trace!("Buffer not ready after {} spins", spin_count);
                return BufferReadResult::NotReady;
            }

            // Hint to CPU that we're spinning (does nothing in WASM, but kept for consistency)
            std::hint::spin_loop();
        }

        // Data is ready - read it
        let buffer = self.data.lock();
        let read_len = num_samples.min(buffer.len());

        // Update read position atomically
        self.read_position.store(read_len as u32, Ordering::Release);

        let data = buffer[..read_len].to_vec();

        log::trace!(
            "Buffer read completed: {} samples (spin_count: {}, elapsed: {:?})",
            read_len,
            spin_count,
            start.elapsed()
        );

        BufferReadResult::Ready(data)
    }

    /// Try to read audio samples without blocking
    ///
    /// **P0-5 FIX**: Non-blocking alternative to read()
    ///
    /// Returns immediately if data is not ready.
    pub fn try_read(&self, num_samples: usize) -> BufferReadResult<Vec<f32>> {
        if !self.is_ready.load(Ordering::Acquire) {
            return BufferReadResult::NotReady;
        }

        let buffer = self.data.lock();
        let read_len = num_samples.min(buffer.len());

        self.read_position.store(read_len as u32, Ordering::Release);

        BufferReadResult::Ready(buffer[..read_len].to_vec())
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

    /// Get current buffer utilization
    pub fn utilization(&self) -> f32 {
        let write_pos = self.write_position.load(Ordering::Acquire) as usize;
        let total_size = self.length * self.channels;

        if total_size == 0 {
            0.0
        } else {
            (write_pos as f32) / (total_size as f32)
        }
    }
}

#[cfg(target_arch = "wasm32")]
/// Worker-based audio processor
///
/// Offloads heavy audio processing to Web Workers
pub struct WorkerAudioProcessor {
    // Shared buffers for input/output
    input_buffer: Arc<AtomicAudioBuffer>,
    output_buffer: Arc<AtomicAudioBuffer>,
    // Processing timeout
    processing_timeout: Duration,
}

#[cfg(target_arch = "wasm32")]
impl WorkerAudioProcessor {
    /// Create a new worker audio processor
    pub fn new(buffer_size: usize, channels: usize, sample_rate: u32) -> Self {
        Self::with_timeout(
            buffer_size,
            channels,
            sample_rate,
            Duration::from_millis(BUFFER_READY_TIMEOUT_MS),
        )
    }

    /// Create a new worker audio processor with custom timeout
    pub fn with_timeout(
        buffer_size: usize,
        channels: usize,
        sample_rate: u32,
        timeout: Duration,
    ) -> Self {
        Self {
            input_buffer: Arc::new(AtomicAudioBuffer::new(buffer_size, channels, sample_rate)),
            output_buffer: Arc::new(AtomicAudioBuffer::new(buffer_size, channels, sample_rate)),
            processing_timeout: timeout,
        }
    }

    /// Process audio samples using worker pool
    ///
    /// **P0-5 FIX**: Uses timeout-based buffer reads
    ///
    /// This offloads the actual processing to workers via rayon
    pub fn process_async(
        &self,
        samples: Vec<f32>,
        task: AudioProcessingTask,
    ) -> Result<Option<Vec<f32>>> {
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

            Ok(Some(processed))
        }
    }

    /// Read output with timeout
    ///
    /// **P0-5 FIX**: Safe timeout-based read
    pub fn read_output(&self, num_samples: usize) -> BufferReadResult<Vec<f32>> {
        self.output_buffer
            .read_with_timeout(num_samples, self.processing_timeout)
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

    /// Get processing timeout
    pub fn timeout(&self) -> Duration {
        self.processing_timeout
    }

    /// Set processing timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.processing_timeout = timeout;
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
    fn test_atomic_buffer_write_read_with_timeout() {
        let buffer = AtomicAudioBuffer::new(512, 2, 48000);
        let samples = vec![1.0, 2.0, 3.0, 4.0];

        buffer.write(&samples).unwrap();
        assert!(buffer.is_ready());

        let result = buffer.read_with_timeout(4, Duration::from_millis(100));
        match result {
            BufferReadResult::Ready(data) => {
                assert_eq!(data[0], 1.0);
                assert_eq!(data[1], 2.0);
            }
            _ => panic!("Expected Ready result"),
        }
    }

    #[wasm_bindgen_test]
    fn test_atomic_buffer_timeout() {
        let buffer = AtomicAudioBuffer::new(512, 2, 48000);

        // Don't write any data - buffer should timeout
        let result = buffer.read_with_timeout(4, Duration::from_millis(10));
        assert!(matches!(
            result,
            BufferReadResult::Timeout | BufferReadResult::NotReady
        ));
    }

    #[wasm_bindgen_test]
    fn test_atomic_buffer_try_read() {
        let buffer = AtomicAudioBuffer::new(512, 2, 48000);

        // Should return NotReady immediately
        let result = buffer.try_read(4);
        assert!(matches!(result, BufferReadResult::NotReady));

        // Write data
        buffer.write(&vec![1.0, 2.0, 3.0, 4.0]).unwrap();

        // Should return Ready immediately
        let result = buffer.try_read(4);
        assert!(matches!(result, BufferReadResult::Ready(_)));
    }

    #[wasm_bindgen_test]
    fn test_worker_processor_creation() {
        let processor = WorkerAudioProcessor::new(512, 2, 48000);
        let (length, channels, sample_rate) = processor.input_buffer().metadata();
        assert_eq!(length, 512);
        assert_eq!(channels, 2);
        assert_eq!(sample_rate, 48000);
    }

    #[wasm_bindgen_test]
    fn test_buffer_utilization() {
        let buffer = AtomicAudioBuffer::new(100, 2, 48000);
        assert_eq!(buffer.utilization(), 0.0);

        buffer.write(&vec![1.0; 100]).unwrap();
        assert!(buffer.utilization() > 0.0);
    }
}
