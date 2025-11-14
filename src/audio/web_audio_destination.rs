//! Web Audio API destination for WASM
//!
//! Provides an AudioDestination that plays audio through the browser's
//! Web Audio API using AudioBufferSourceNode.

#[cfg(target_arch = "wasm32")]
use super::backend::Result;
#[cfg(target_arch = "wasm32")]
use super::router::AudioDestination;
#[cfg(target_arch = "wasm32")]
use parking_lot::Mutex;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext};

/// Web Audio destination
///
/// Plays audio through the browser's Web Audio API
#[cfg(target_arch = "wasm32")]
pub struct WebAudioDestination {
    context: AudioContext,
    sample_rate: u32,
    channels: u16,
    buffer: Arc<Mutex<Vec<f32>>>,
    buffer_duration: f32, // Duration in seconds
}

#[cfg(target_arch = "wasm32")]
impl WebAudioDestination {
    /// Create a new Web Audio destination
    ///
    /// # Arguments
    /// * `context` - Web Audio API AudioContext
    /// * `sample_rate` - Sample rate
    /// * `channels` - Number of channels
    /// * `buffer_duration` - Buffer duration in seconds (e.g., 0.1 for 100ms)
    pub fn new(
        context: AudioContext,
        sample_rate: u32,
        channels: u16,
        buffer_duration: f32,
    ) -> Self {
        Self {
            context,
            sample_rate,
            channels,
            buffer: Arc::new(Mutex::new(Vec::new())),
            buffer_duration,
        }
    }

    /// Play buffered audio
    pub fn flush_to_output(&mut self) -> Result<()> {
        let samples = {
            let mut buffer = self.buffer.lock();
            if buffer.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *buffer)
        };

        if samples.is_empty() {
            return Ok(());
        }

        // Calculate number of frames
        let num_frames = samples.len() / self.channels as usize;

        if num_frames == 0 {
            return Ok(());
        }

        // Create AudioBuffer
        let audio_buffer = self
            .context
            .create_buffer(
                self.channels as u32,
                num_frames as u32,
                self.sample_rate as f32,
            )
            .map_err(|e| {
                super::backend::AudioBackendError::Other(anyhow::anyhow!(
                    "Failed to create AudioBuffer: {:?}",
                    e
                ))
            })?;

        // Fill the buffer with samples
        for channel_num in 0..self.channels {
            let mut channel_data = vec![0.0f32; num_frames];

            // De-interleave samples
            for (frame_idx, sample_idx) in (channel_num as usize..samples.len())
                .step_by(self.channels as usize)
                .enumerate()
            {
                if frame_idx < num_frames {
                    channel_data[frame_idx] = samples[sample_idx];
                }
            }

            // Copy to AudioBuffer
            audio_buffer
                .copy_to_channel(&mut channel_data, channel_num as i32)
                .map_err(|e| {
                    super::backend::AudioBackendError::Other(anyhow::anyhow!(
                        "Failed to copy to channel: {:?}",
                        e
                    ))
                })?;
        }

        // Create source node
        let source = self.context.create_buffer_source().map_err(|e| {
            super::backend::AudioBackendError::Other(anyhow::anyhow!(
                "Failed to create buffer source: {:?}",
                e
            ))
        })?;

        source.set_buffer(Some(&audio_buffer));

        // Connect to destination
        source
            .connect_with_audio_node(&self.context.destination())
            .map_err(|e| {
                super::backend::AudioBackendError::Other(anyhow::anyhow!(
                    "Failed to connect source: {:?}",
                    e
                ))
            })?;

        // Play immediately
        source.start().map_err(|e| {
            super::backend::AudioBackendError::Other(anyhow::anyhow!(
                "Failed to start source: {:?}",
                e
            ))
        })?;

        log::debug!("Playing {} frames through Web Audio API", num_frames);

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl AudioDestination for WebAudioDestination {
    fn write_samples(&mut self, buffer: &[f32]) -> Result<()> {
        // Accumulate samples in internal buffer
        self.buffer.lock().extend_from_slice(buffer);

        // Flush if buffer is large enough
        let buffer_size = self.buffer.lock().len();
        let target_size =
            (self.sample_rate as f32 * self.buffer_duration) as usize * self.channels as usize;

        if buffer_size >= target_size {
            self.flush_to_output()?;
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
        self.flush_to_output()
    }
}

// Stub for non-WASM
#[cfg(not(target_arch = "wasm32"))]
pub struct WebAudioDestination;

#[cfg(not(target_arch = "wasm32"))]
impl WebAudioDestination {
    pub fn new(_sample_rate: u32, _channels: u16, _buffer_duration: f32) -> Self {
        Self
    }
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_web_audio_destination_creation() {
        let context = AudioContext::new().unwrap();
        let dest = WebAudioDestination::new(context, 48000, 2, 0.1);

        assert_eq!(dest.sample_rate(), 48000);
        assert_eq!(dest.channels(), 2);
    }

    #[wasm_bindgen_test]
    fn test_write_samples() {
        let context = AudioContext::new().unwrap();
        let mut dest = WebAudioDestination::new(context, 48000, 2, 0.1);

        let samples = vec![0.1, 0.2, 0.3, 0.4];
        let result = dest.write_samples(&samples);

        assert!(result.is_ok());
    }
}
