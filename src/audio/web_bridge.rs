//! Web Audio API Bridge
//!
//! Bridges web-audio-api's ScriptProcessorNode with the native audio ring buffer
//! for hybrid playback mode.

use super::hybrid::HybridRingBuffer;
use std::sync::Arc;
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
    ring_buffer: Arc<HybridRingBuffer>,
    config: WebAudioBridgeConfig,
}

impl WebAudioBridge {
    /// Create a new bridge with the given ring buffer
    pub fn new(ring_buffer: Arc<HybridRingBuffer>, config: WebAudioBridgeConfig) -> Self {
        Self {
            ring_buffer,
            config,
        }
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
    pub fn connect_to_graph<N: AudioNode>(
        &self,
        _context: &AudioContext,
        _source_node: &N,
    ) {
        // TODO: Implement actual audio bridging once web-audio-api crate
        // exposes ScriptProcessorNode or AudioWorkletNode
        // 
        // For now, the ring buffer infrastructure is in place for when
        // the API becomes available.
        println!("ℹ️ WebAudioBridge: Infrastructure ready, waiting for ScriptProcessor API");
    }
    
    /// Get the current buffer fill level (0.0 to 1.0)
    pub fn buffer_fill_level(&self) -> f32 {
        let available = self.ring_buffer.available();
        let capacity = self.config.buffer_size * self.config.input_channels as usize * 8; // 8x buffer
        (available as f32) / (capacity as f32)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::hybrid::AudioRingBuffer;
    
    #[test]
    fn test_bridge_creation() {
        let ring_buffer = Arc::new(AudioRingBuffer::new(8192));
        let bridge = WebAudioBridge::new(ring_buffer, WebAudioBridgeConfig::default());
        assert!(bridge.buffer_fill_level() >= 0.0);
    }
    
    #[test]
    fn test_buffer_health() {
        let ring_buffer = Arc::new(AudioRingBuffer::new(8192));
        let bridge = WebAudioBridge::new(ring_buffer.clone(), WebAudioBridgeConfig::default());
        
        // Empty buffer should be unhealthy
        assert!(!bridge.is_buffer_healthy());
        
        // Fill to 50%
        let samples = vec![0.0; 4096];
        ring_buffer.write(&samples);
        
        // Should now be healthy
        assert!(bridge.is_buffer_healthy());
    }
}
