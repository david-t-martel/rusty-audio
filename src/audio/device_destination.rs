//! Device-based audio destinations
//!
//! This module provides AudioDestination implementations that write to audio
//! output devices (speakers, headphones, etc.) via CPAL/ASIO.

use super::backend::{AudioBackend, AudioBackendError, AudioConfig, Result};
use super::destinations::RingBufferDestination;
use super::router::AudioDestination;
use parking_lot::Mutex;
use std::sync::Arc;

/// Output device destination
///
/// Writes audio to a physical output device (speakers, headphones, etc.)
/// using the CPAL/ASIO backend.
pub struct OutputDeviceDestination {
    ring_buffer: RingBufferDestination,
    device_id: String,
    config: AudioConfig,
    _stream: Arc<Mutex<Option<Box<dyn super::backend::AudioStream>>>>,
}

impl OutputDeviceDestination {
    /// Create a new output device destination
    ///
    /// # Arguments
    /// * `backend` - Audio backend to use (CPAL, ASIO, etc.)
    /// * `device_id` - ID of the output device
    /// * `config` - Audio configuration
    /// * `buffer_size` - Size of internal ring buffer (in samples, 0 = 4x audio buffer)
    ///
    /// # Returns
    /// A new output device destination that continuously writes to the device
    pub fn new(
        _backend: &mut dyn AudioBackend,
        device_id: &str,
        config: AudioConfig,
        buffer_size: usize,
    ) -> Result<Self> {
        // Use 4x the audio buffer size if not specified
        let ring_buffer_size = if buffer_size == 0 {
            config.buffer_size * 4
        } else {
            buffer_size
        };

        // Create ring buffer for audio data
        let ring_buffer =
            RingBufferDestination::new(config.sample_rate, config.channels, ring_buffer_size);
        let reader = ring_buffer.get_reader();

        // Create output stream with callback that reads from ring buffer
        // TODO: Implement proper backend integration
        // For now, we'll use a placeholder stream
        let stream = Arc::new(Mutex::new(None));

        Ok(Self {
            ring_buffer,
            device_id: device_id.to_string(),
            config,
            _stream: stream,
        })
    }

    /// Get the device ID
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Get the audio configuration
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }

    /// Get the current buffer size (number of samples waiting to be written)
    pub fn buffer_size(&self) -> usize {
        self.ring_buffer.buffer_size()
    }

    /// Clear the internal buffer
    pub fn clear_buffer(&self) {
        self.ring_buffer.clear();
    }
}

impl AudioDestination for OutputDeviceDestination {
    fn write_samples(&mut self, buffer: &[f32]) -> Result<()> {
        self.ring_buffer.write_samples(buffer)
    }

    fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    fn channels(&self) -> u16 {
        self.config.channels
    }

    fn flush(&mut self) -> Result<()> {
        self.ring_buffer.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_device_destination_config() {
        let config = AudioConfig {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 256,
            exclusive_mode: true,
            ..Default::default()
        };

        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.buffer_size, 256);
        assert!(config.exclusive_mode);
    }
}
