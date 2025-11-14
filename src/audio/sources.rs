//! Audio source implementations
//!
//! This module provides concrete implementations of the AudioSource trait
//! for various audio input types.

use super::backend::{AudioBackendError, Result};
use super::router::AudioSource;
use parking_lot::Mutex;
use std::sync::Arc;

/// Signal generator source
///
/// Produces test signals (sine, square, sawtooth, noise, etc.)
pub struct SignalGeneratorSource {
    samples: Vec<f32>,
    position: usize,
    sample_rate: u32,
    channels: u16,
    looping: bool,
}

impl SignalGeneratorSource {
    /// Create a new signal generator source from pre-generated samples
    ///
    /// # Arguments
    /// * `samples` - Pre-generated audio samples
    /// * `sample_rate` - Sample rate of the audio
    /// * `looping` - Whether to loop the audio when it reaches the end
    pub fn from_buffer(samples: Vec<f32>, sample_rate: f32, looping: bool) -> Self {
        Self {
            samples,
            position: 0,
            sample_rate: sample_rate as u32,
            channels: 1, // Mono
            looping,
        }
    }

    /// Create a new signal generator source with custom channel count
    pub fn from_buffer_with_channels(
        samples: Vec<f32>,
        sample_rate: f32,
        channels: u16,
        looping: bool,
    ) -> Self {
        Self {
            samples,
            position: 0,
            sample_rate: sample_rate as u32,
            channels,
            looping,
        }
    }

    /// Reset playback to the beginning
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Set looping mode
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// Get current playback progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.position as f32 / self.samples.len() as f32
        }
    }
}

impl AudioSource for SignalGeneratorSource {
    fn read_samples(&mut self, buffer: &mut [f32]) -> usize {
        if self.samples.is_empty() {
            buffer.fill(0.0);
            return buffer.len();
        }

        let mut samples_read = 0;

        while samples_read < buffer.len() {
            if self.position >= self.samples.len() {
                if self.looping {
                    self.position = 0;
                } else {
                    // Fill rest with silence
                    buffer[samples_read..].fill(0.0);
                    break;
                }
            }

            let available = (self.samples.len() - self.position).min(buffer.len() - samples_read);
            buffer[samples_read..samples_read + available]
                .copy_from_slice(&self.samples[self.position..self.position + available]);

            self.position += available;
            samples_read += available;
        }

        samples_read
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn has_more_samples(&self) -> bool {
        self.looping || self.position < self.samples.len()
    }

    fn seek(&mut self, sample: u64) -> Result<()> {
        let sample = sample as usize;
        if sample < self.samples.len() {
            self.position = sample;
            Ok(())
        } else {
            Err(AudioBackendError::Other(anyhow::anyhow!(
                "Seek position out of range"
            )))
        }
    }

    fn position(&self) -> Option<u64> {
        Some(self.position as u64)
    }

    fn length(&self) -> Option<u64> {
        Some(self.samples.len() as u64)
    }
}

/// Ring buffer source
///
/// Reads audio from a lock-free ring buffer (useful for input devices)
pub struct RingBufferSource {
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
}

impl RingBufferSource {
    /// Create a new ring buffer source
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate,
            channels,
        }
    }

    /// Get a handle to write to the ring buffer
    pub fn get_writer(&self) -> RingBufferWriter {
        RingBufferWriter {
            buffer: self.buffer.clone(),
        }
    }

    /// Get the current buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer.lock().len()
    }

    /// Clear the buffer
    pub fn clear(&self) {
        self.buffer.lock().clear();
    }
}

/// Writer handle for ring buffer source
pub struct RingBufferWriter {
    buffer: Arc<Mutex<Vec<f32>>>,
}

impl RingBufferWriter {
    /// Write samples to the ring buffer
    pub fn write(&self, samples: &[f32]) {
        let mut buffer = self.buffer.lock();
        buffer.extend_from_slice(samples);
    }
}

impl AudioSource for RingBufferSource {
    fn read_samples(&mut self, buffer: &mut [f32]) -> usize {
        let mut ring_buffer = self.buffer.lock();

        let available = ring_buffer.len().min(buffer.len());
        if available > 0 {
            buffer[..available].copy_from_slice(&ring_buffer[..available]);
            ring_buffer.drain(..available);
        }

        // Fill rest with silence
        if available < buffer.len() {
            buffer[available..].fill(0.0);
        }

        available
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn has_more_samples(&self) -> bool {
        true // Continuous stream
    }
}

/// Silence source
///
/// Produces silence (zeros)
pub struct SilenceSource {
    sample_rate: u32,
    channels: u16,
}

impl SilenceSource {
    /// Create a new silence source
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
        }
    }
}

impl AudioSource for SilenceSource {
    fn read_samples(&mut self, buffer: &mut [f32]) -> usize {
        buffer.fill(0.0);
        buffer.len()
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn has_more_samples(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_generator_source() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mut source = SignalGeneratorSource::from_buffer(samples.clone(), 44100.0, false);

        assert_eq!(source.sample_rate(), 44100);
        assert_eq!(source.channels(), 1);
        assert!(source.has_more_samples());

        let mut buffer = vec![0.0; 3];
        let read = source.read_samples(&mut buffer);

        assert_eq!(read, 3);
        assert_eq!(buffer, vec![1.0, 2.0, 3.0]);
        assert_eq!(source.progress(), 0.6); // 3/5
    }

    #[test]
    fn test_signal_generator_looping() {
        let samples = vec![1.0, 2.0, 3.0];
        let mut source = SignalGeneratorSource::from_buffer(samples, 44100.0, true);

        let mut buffer = vec![0.0; 5];
        source.read_samples(&mut buffer);

        // Should loop: [1, 2, 3, 1, 2]
        assert_eq!(buffer, vec![1.0, 2.0, 3.0, 1.0, 2.0]);
    }

    #[test]
    fn test_ring_buffer_source() {
        let mut source = RingBufferSource::new(44100, 2);
        let writer = source.get_writer();

        // Write some samples
        writer.write(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(source.buffer_size(), 4);

        // Read samples
        let mut buffer = vec![0.0; 2];
        let read = source.read_samples(&mut buffer);

        assert_eq!(read, 2);
        assert_eq!(buffer, vec![1.0, 2.0]);
        assert_eq!(source.buffer_size(), 2);
    }

    #[test]
    fn test_silence_source() {
        let mut source = SilenceSource::new(48000, 2);

        let mut buffer = vec![1.0; 100];
        source.read_samples(&mut buffer);

        assert!(buffer.iter().all(|&x| x == 0.0));
    }
}
