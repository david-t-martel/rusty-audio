//! Device-based audio sources
//!
//! This module provides AudioSource implementations that read from audio
//! input devices (microphones, line inputs, etc.) via CPAL/ASIO.

use super::backend::{AudioBackend, AudioBackendError, AudioConfig, Result, StreamDirection};
use super::router::AudioSource;
use super::sources::RingBufferSource;
use parking_lot::Mutex;
use std::sync::Arc;

/// Input device source
///
/// Reads audio from a physical input device (microphone, line in, etc.)
/// using the CPAL/ASIO backend.
pub struct InputDeviceSource {
    ring_buffer: RingBufferSource,
    device_id: String,
    config: AudioConfig,
    _stream: Arc<Mutex<Option<Box<dyn super::backend::AudioStream>>>>,
}

impl InputDeviceSource {
    /// Create a new input device source
    ///
    /// # Arguments
    /// * `backend` - Audio backend to use (CPAL, ASIO, etc.)
    /// * `device_id` - ID of the input device
    /// * `config` - Audio configuration
    ///
    /// # Returns
    /// A new input device source that continuously reads from the device
    pub fn new(
        backend: &mut dyn AudioBackend,
        device_id: &str,
        config: AudioConfig,
    ) -> Result<Self> {
        // Create ring buffer for audio data
        let ring_buffer = RingBufferSource::new(config.sample_rate, config.channels);
        let writer = ring_buffer.get_writer();

        // Create input stream with callback that writes to ring buffer
        let stream = backend.create_input_stream_with_callback(
            device_id,
            config.clone(),
            move |data: &[f32]| {
                // Write samples from input device to ring buffer
                writer.write(data);
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

    /// Get the current buffer size (number of samples waiting to be read)
    pub fn buffer_size(&self) -> usize {
        self.ring_buffer.buffer_size()
    }

    /// Clear the internal buffer
    pub fn clear_buffer(&self) {
        self.ring_buffer.clear();
    }
}

impl AudioSource for InputDeviceSource {
    fn read_samples(&mut self, buffer: &mut [f32]) -> usize {
        self.ring_buffer.read_samples(buffer)
    }

    fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    fn channels(&self) -> u16 {
        self.config.channels
    }

    fn has_more_samples(&self) -> bool {
        true // Continuous input stream
    }
}

// Note: Downcast helper trait no longer needed since we added
// create_input_stream_with_callback to the AudioBackend trait

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_device_source_config() {
        // Test that we can access the configuration
        // Actual device creation would require hardware
        let config = AudioConfig {
            sample_rate: 44100,
            channels: 2,
            buffer_size: 512,
            exclusive_mode: false,
            ..Default::default()
        };

        // Would need actual backend for full test
        // This just verifies the structure
        assert_eq!(config.sample_rate, 44100);
    }
}
