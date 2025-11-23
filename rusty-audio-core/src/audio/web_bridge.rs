//! Web Audio API Bridge
//!
//! Bridges web-audio-api's ScriptProcessorNode with the native audio ring buffer
//! for hybrid playback mode.

use rtrb::Producer;
use web_audio_api::context::AudioContext;
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};

/// Configuration for web audio bridge
pub struct WebAudioBridgeConfig {
    pub buffer_size: usize,
    pub input_channels: u16,
    pub output_channels: u16,
}

impl Default for WebAudioBridgeConfig {
    fn default() -> Self {
        Self {
            buffer_size: 4096,
            input_channels: 2,
            output_channels: 2,
        }
    }
}

/// Web Audio Bridge
///
/// Connects web-audio-api output to native CPAL hardware via ring buffer
pub struct WebAudioBridge {
    producer: Producer<f32>,
    config: WebAudioBridgeConfig,
}

impl WebAudioBridge {
    /// Create a new bridge with the given ring buffer producer
    pub fn new(producer: Producer<f32>, config: WebAudioBridgeConfig) -> Self {
        Self { producer, config }
    }

    /// Connect the bridge to the audio graph
    ///
    /// NOTE: The current web-audio-api crate doesn't expose ScriptProcessorNode.
    /// This is a placeholder for future implementation.
    ///
    /// For now, the bridge is set up but audio routing happens through
    /// the existing web-audio-api destination.
    ///
    /// # Usage
    /// ```ignore
    /// bridge.connect_to_graph(&audio_context, &analyser);
    /// ```
    pub fn connect_to_graph<N: AudioNode>(&self, _context: &AudioContext, _source_node: &N) {
        // TODO: Implement actual audio bridging once web-audio-api crate
        // exposes ScriptProcessorNode or AudioWorkletNode
        //
        // For now, the ring buffer infrastructure is in place for when
        // the API becomes available.
        println!("ℹ️ WebAudioBridge: Infrastructure ready, waiting for ScriptProcessor API");
    }

    /// Get the current buffer fill level (0.0 to 1.0)
    pub fn buffer_fill_level(&self) -> f32 {
        // For a producer, we care about how much space is *used* (capacity - slots)
        // Capacity is 8x buffer_size as defined in HybridAudioBackend
        let capacity = self.config.buffer_size * 8;
        let available = self.producer.slots();
        let used = capacity.saturating_sub(available);

        (used as f32) / (capacity as f32)
    }

    /// Check if the buffer is healthy (not too empty or too full)
    pub fn is_buffer_healthy(&self) -> bool {
        let fill = self.buffer_fill_level();
        fill > 0.1 && fill < 0.9 // Healthy range: 10% to 90%
    }

    /// Get buffer health status message
    pub fn buffer_health_message(&self) -> &'static str {
        let fill = self.buffer_fill_level();
        if fill < 0.05 {
            "Buffer underrun risk"
        } else if fill < 0.1 {
            "Buffer low"
        } else if fill > 0.95 {
            "Buffer overrun risk"
        } else if fill > 0.9 {
            "Buffer high"
        } else {
            "Buffer healthy"
        }
    }

    /// Push samples into the ring buffer
    pub fn push_samples(&mut self, samples: &[f32]) -> usize {
        if let Ok(mut chunk) = self.producer.write_chunk(samples.len()) {
            let (s1, s2) = chunk.as_mut_slices();
            let len1 = s1.len();
            s1.copy_from_slice(&samples[..len1]);
            s2.copy_from_slice(&samples[len1..len1 + s2.len()]);
            chunk.commit_all();
            return samples.len();
        }
        // If buffer is full, we drop samples (overrun)
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rtrb::RingBuffer;

    #[test]
    fn test_bridge_creation() {
        let (producer, _consumer) = RingBuffer::new(8192);
        let bridge = WebAudioBridge::new(producer, WebAudioBridgeConfig::default());
        assert!(bridge.buffer_fill_level() <= 0.05); // Should be empty
    }

    #[test]
    fn test_buffer_health() {
        let (mut producer, _consumer) = RingBuffer::new(8192);
        let mut bridge = WebAudioBridge::new(producer, WebAudioBridgeConfig::default());

        // Empty buffer should be unhealthy (underrun risk)
        assert!(!bridge.is_buffer_healthy());

        // Fill to 50%
        let samples = vec![0.0; 4096];
        bridge.push_samples(&samples);

        // Should now be healthy
        assert!(bridge.is_buffer_healthy());
    }
}
