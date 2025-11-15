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
        backend: &mut dyn AudioBackend,
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
        let stream = backend.create_output_stream_with_callback(
            device_id,
            config.clone(),
            move |output_buffer: &mut [f32]| {
                // Read samples from ring buffer and write to output
                let samples_read = reader.read(output_buffer);

                // Fill remaining buffer with silence if not enough samples
                if samples_read < output_buffer.len() {
                    output_buffer[samples_read..].fill(0.0);
                }
            },
        )?;

        // Store stream to keep it alive
        let stream = Arc::new(Mutex::new(Some(stream)));

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
