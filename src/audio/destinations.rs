//! Audio destination implementations
//!
//! This module provides concrete implementations of the AudioDestination trait
//! for various audio output types.

use super::backend::{AudioBackendError, Result};
use super::router::AudioDestination;
use parking_lot::Mutex;
use std::sync::Arc;

/// Ring buffer destination
///
/// Writes audio to a thread-safe, mutex-protected buffer (useful for output devices or processing chains)
pub struct RingBufferDestination {
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    max_size: usize,
}

impl RingBufferDestination {
    /// Create a new ring buffer destination
    ///
    /// # Arguments
    /// * `sample_rate` - Sample rate
    /// * `channels` - Number of channels
    /// * `max_size` - Maximum buffer size (0 = unlimited)
    pub fn new(sample_rate: u32, channels: u16, max_size: usize) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate,
            channels,
            max_size,
        }
    }

    /// Get a reader handle for the ring buffer
    pub fn get_reader(&self) -> RingBufferReader {
        RingBufferReader {
            buffer: self.buffer.clone(),
        }
    }

    /// Get current buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer.lock().len()
    }

    /// Clear the buffer
    pub fn clear(&self) {
        self.buffer.lock().clear();
    }
}

/// Reader handle for ring buffer destination
pub struct RingBufferReader {
    buffer: Arc<Mutex<Vec<f32>>>,
}

impl RingBufferReader {
    /// Read samples from the ring buffer
    ///
    /// Returns the number of samples actually read
    pub fn read(&self, output: &mut [f32]) -> usize {
        let mut buffer = self.buffer.lock();
        let available = buffer.len().min(output.len());

        if available > 0 {
            output[..available].copy_from_slice(&buffer[..available]);
            buffer.drain(..available);
        }

        available
    }

    /// Peek at samples without removing them
    pub fn peek(&self, output: &mut [f32]) -> usize {
        let buffer = self.buffer.lock();
        let available = buffer.len().min(output.len());

        if available > 0 {
            output[..available].copy_from_slice(&buffer[..available]);
        }

        available
    }

    /// Get the number of available samples
    pub fn available(&self) -> usize {
        self.buffer.lock().len()
    }
}

impl AudioDestination for RingBufferDestination {
    fn write_samples(&mut self, buffer: &[f32]) -> Result<()> {
        let mut ring_buffer = self.buffer.lock();

        // If max size is set, maintain it
        if self.max_size > 0 {
            let new_size = ring_buffer.len() + buffer.len();
            if new_size > self.max_size {
                let to_remove = new_size - self.max_size;
                ring_buffer.drain(..to_remove);
            }
        }

        ring_buffer.extend_from_slice(buffer);
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn flush(&mut self) -> Result<()> {
        // Ring buffer doesn't need explicit flushing
        Ok(())
    }
}

/// Level meter destination
///
/// Measures audio levels (peak and RMS) while passing through
pub struct LevelMeterDestination {
    inner: Option<Box<dyn AudioDestination>>,
    sample_rate: u32,
    channels: u16,
    peak_level: Arc<Mutex<f32>>,
    rms_level: Arc<Mutex<f32>>,
}

impl LevelMeterDestination {
    /// Create a new level meter destination
    ///
    /// # Arguments
    /// * `inner` - Optional inner destination to pass samples through
    /// * `sample_rate` - Sample rate
    /// * `channels` - Number of channels
    pub fn new(inner: Option<Box<dyn AudioDestination>>, sample_rate: u32, channels: u16) -> Self {
        Self {
            inner,
            sample_rate,
            channels,
            peak_level: Arc::new(Mutex::new(0.0)),
            rms_level: Arc::new(Mutex::new(0.0)),
        }
    }

    /// Get current peak level
    pub fn peak_level(&self) -> f32 {
        *self.peak_level.lock()
    }

    /// Get current RMS level
    pub fn rms_level(&self) -> f32 {
        *self.rms_level.lock()
    }

    /// Get peak level in dB
    pub fn peak_db(&self) -> f32 {
        let peak = self.peak_level();
        if peak > 0.0 {
            20.0 * peak.log10()
        } else {
            -std::f32::INFINITY
        }
    }

    /// Get RMS level in dB
    pub fn rms_db(&self) -> f32 {
        let rms = self.rms_level();
        if rms > 0.0 {
            20.0 * rms.log10()
        } else {
            -std::f32::INFINITY
        }
    }

    /// Reset levels
    pub fn reset(&self) {
        *self.peak_level.lock() = 0.0;
        *self.rms_level.lock() = 0.0;
    }
}

impl AudioDestination for LevelMeterDestination {
    fn write_samples(&mut self, buffer: &[f32]) -> Result<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        // Calculate peak
        let peak = buffer.iter().map(|&x| x.abs()).fold(0.0f32, f32::max);
        *self.peak_level.lock() = peak;

        // Calculate RMS
        let sum_squares: f32 = buffer.iter().map(|&x| x * x).sum();
        let rms = (sum_squares / buffer.len() as f32).sqrt();
        *self.rms_level.lock() = rms;

        // Pass through to inner destination if present
        if let Some(ref mut inner) = self.inner {
            inner.write_samples(buffer)?;
        }

        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn flush(&mut self) -> Result<()> {
        if let Some(ref mut inner) = self.inner {
            inner.flush()?;
        }
        Ok(())
    }
}

/// Null destination (discards audio)
///
/// Useful for testing or when you want to process audio without output
pub struct NullDestination {
    sample_rate: u32,
    channels: u16,
    samples_written: usize,
}

impl NullDestination {
    /// Create a new null destination
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
            samples_written: 0,
        }
    }

    /// Get the total number of samples written
    pub fn samples_written(&self) -> usize {
        self.samples_written
    }

    /// Reset the sample counter
    pub fn reset(&mut self) {
        self.samples_written = 0;
    }
}

impl AudioDestination for NullDestination {
    fn write_samples(&mut self, buffer: &[f32]) -> Result<()> {
        self.samples_written += buffer.len();
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }
}

/// Multi-destination splitter
///
/// Writes to multiple destinations simultaneously
pub struct SplitterDestination {
    destinations: Vec<Box<dyn AudioDestination>>,
    sample_rate: u32,
    channels: u16,
}

impl SplitterDestination {
    /// Create a new splitter destination
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            destinations: Vec::new(),
            sample_rate,
            channels,
        }
    }

    /// Add a destination to the splitter
    pub fn add_destination(&mut self, dest: Box<dyn AudioDestination>) {
        self.destinations.push(dest);
    }

    /// Get the number of destinations
    pub fn destination_count(&self) -> usize {
        self.destinations.len()
    }
}

impl AudioDestination for SplitterDestination {
    fn write_samples(&mut self, buffer: &[f32]) -> Result<()> {
        // Write to all destinations
        for dest in &mut self.destinations {
            dest.write_samples(buffer)?;
        }
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn flush(&mut self) -> Result<()> {
        for dest in &mut self.destinations {
            dest.flush()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_destination() {
        let mut dest = RingBufferDestination::new(44100, 2, 0);
        let reader = dest.get_reader();

        // Write samples
        dest.write_samples(&[1.0, 2.0, 3.0, 4.0]).unwrap();
        assert_eq!(dest.buffer_size(), 4);

        // Read samples
        let mut buffer = vec![0.0; 2];
        let read = reader.read(&mut buffer);

        assert_eq!(read, 2);
        assert_eq!(buffer, vec![1.0, 2.0]);
        assert_eq!(reader.available(), 2);
    }

    #[test]
    fn test_ring_buffer_max_size() {
        let mut dest = RingBufferDestination::new(44100, 2, 4);

        // Write more than max size
        dest.write_samples(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();

        // Should only keep last 4 samples
        assert_eq!(dest.buffer_size(), 4);

        let reader = dest.get_reader();
        let mut buffer = vec![0.0; 4];
        reader.read(&mut buffer);

        assert_eq!(buffer, vec![3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_level_meter() {
        let mut meter = LevelMeterDestination::new(None, 44100, 1);

        // Write samples with known peak and RMS
        let samples = vec![0.5, -0.8, 0.3, -0.4];
        meter.write_samples(&samples).unwrap();

        assert_eq!(meter.peak_level(), 0.8);

        // Check RMS (should be sqrt(mean(squares)))
        let expected_rms = ((0.5 * 0.5 + 0.8 * 0.8 + 0.3 * 0.3 + 0.4 * 0.4) / 4.0).sqrt();
        assert!((meter.rms_level() - expected_rms).abs() < 0.001);
    }

    #[test]
    fn test_null_destination() {
        let mut dest = NullDestination::new(48000, 2);

        dest.write_samples(&[1.0, 2.0, 3.0, 4.0]).unwrap();
        assert_eq!(dest.samples_written(), 4);

        dest.write_samples(&[5.0, 6.0]).unwrap();
        assert_eq!(dest.samples_written(), 6);
    }

    #[test]
    fn test_splitter_destination() {
        let mut splitter = SplitterDestination::new(44100, 2);

        let dest1 = NullDestination::new(44100, 2);
        let dest2 = NullDestination::new(44100, 2);

        splitter.add_destination(Box::new(dest1));
        splitter.add_destination(Box::new(dest2));

        assert_eq!(splitter.destination_count(), 2);

        splitter.write_samples(&[1.0, 2.0, 3.0]).unwrap();
    }
}
