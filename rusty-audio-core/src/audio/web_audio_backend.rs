//! Web Audio API backend for WASM
//!
//! Implements the AudioBackend trait using the browser's Web Audio API.
//! This provides audio functionality for WASM builds.

#[cfg(target_arch = "wasm32")]
use super::backend::{
    AudioBackend, AudioBackendError, AudioConfig, AudioStream, DeviceInfo, InputCallback,
    OutputCallback, Result, SampleFormat, StreamDirection, StreamStatus,
};

#[cfg(target_arch = "wasm32")]
use parking_lot::Mutex;
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{AudioContext, AudioDestinationNode};

/// Thread-safe wrapper for AudioContext in WASM with multithreading support
///
/// With WASM multithreading enabled (SharedArrayBuffer + atomics), we use
/// Arc<Mutex<>> for true thread safety across the main thread and worker pool.
/// The AudioContext itself must stay on the main thread (Web Audio API restriction),
/// but this wrapper allows safe access from workers for coordination.
#[cfg(target_arch = "wasm32")]
struct WasmAudioContext {
    context: Arc<Mutex<Option<AudioContext>>>,
}

#[cfg(target_arch = "wasm32")]
impl WasmAudioContext {
    fn new() -> Self {
        Self {
            context: Arc::new(Mutex::new(None)),
        }
    }

    fn get_or_create(&self) -> Result<AudioContext> {
        let mut ctx = self.context.lock();
        if ctx.is_none() {
            let audio_ctx = AudioContext::new().map_err(|e| {
                AudioBackendError::InitializationFailed(format!(
                    "Failed to create AudioContext: {:?}",
                    e
                ))
            })?;
            *ctx = Some(audio_ctx);
        }

        // Clone the AudioContext (it's internally reference-counted in JS)
        Ok(ctx.as_ref().unwrap().clone())
    }

    fn get(&self) -> Option<AudioContext> {
        self.context.lock().as_ref().map(|c| c.clone())
    }
}

#[cfg(target_arch = "wasm32")]
impl Clone for WasmAudioContext {
    fn clone(&self) -> Self {
        Self {
            context: Arc::clone(&self.context),
        }
    }
}

// Thread safety is now guaranteed by Arc<Mutex<>>
// Send + Sync are automatically implemented

/// Web Audio API backend
#[cfg(target_arch = "wasm32")]
pub struct WebAudioBackend {
    context: WasmAudioContext,
    initialized: bool,
}

#[cfg(target_arch = "wasm32")]
impl WebAudioBackend {
    /// Create a new Web Audio API backend
    pub fn new() -> Self {
        Self {
            context: WasmAudioContext::new(),
            initialized: false,
        }
    }

    /// Get or create the audio context
    fn get_context(&self) -> Result<AudioContext> {
        self.context.get_or_create()
    }

    /// Get sample rate from context
    fn get_sample_rate(&self) -> Result<u32> {
        let context = self.get_context()?;
        Ok(context.sample_rate() as u32)
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for WebAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
impl AudioBackend for WebAudioBackend {
    fn name(&self) -> &'static str {
        "Web Audio API"
    }

    fn is_available(&self) -> bool {
        // Web Audio API is always available in browsers
        true
    }

    fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // Create audio context to verify it works
        self.get_context()?;
        self.initialized = true;

        log::info!("Web Audio API backend initialized");
        Ok(())
    }

    fn enumerate_devices(&self, _direction: StreamDirection) -> Result<Vec<DeviceInfo>> {
        // Web Audio API doesn't expose device enumeration in the same way
        // Return a single "default" device
        let sample_rate = if let Some(ctx) = self.context.get() {
            ctx.sample_rate() as u32
        } else {
            48000 // Default assumption
        };

        Ok(vec![DeviceInfo {
            id: "default".to_string(),
            name: "Web Audio Default".to_string(),
            is_default: true,
            supported_configs: vec![AudioConfig {
                sample_rate,
                channels: 2,
                sample_format: SampleFormat::F32,
                buffer_size: 512,
                exclusive_mode: false,
            }],
            min_sample_rate: 44100,
            max_sample_rate: 48000,
            max_input_channels: 2,
            max_output_channels: 2,
        }])
    }

    fn default_device(&self, direction: StreamDirection) -> Result<DeviceInfo> {
        let devices = self.enumerate_devices(direction)?;
        devices
            .into_iter()
            .next()
            .ok_or_else(|| AudioBackendError::DeviceNotFound("No default device".to_string()))
    }

    fn test_device(&self, _device_id: &str) -> Result<bool> {
        // In Web Audio API, if the context exists, the device works
        Ok(self.context.get().is_some())
    }

    fn supported_configs(
        &self,
        _device_id: &str,
        _direction: StreamDirection,
    ) -> Result<Vec<AudioConfig>> {
        let sample_rate = if let Some(ctx) = self.context.get() {
            ctx.sample_rate() as u32
        } else {
            48000
        };

        Ok(vec![
            AudioConfig {
                sample_rate,
                channels: 2,
                sample_format: SampleFormat::F32,
                buffer_size: 512,
                exclusive_mode: false,
            },
            AudioConfig {
                sample_rate,
                channels: 1,
                sample_format: SampleFormat::F32,
                buffer_size: 512,
                exclusive_mode: false,
            },
        ])
    }

    fn create_output_stream(
        &mut self,
        _device_id: &str,
        config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>> {
        let context = self.get_context()?;

        Ok(Box::new(WebAudioOutputStream {
            context: WasmAudioContext::new(),
            config,
            status: StreamStatus::Stopped,
        }))
    }

    fn create_input_stream(
        &mut self,
        _device_id: &str,
        _config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>> {
        // Input streams require getUserMedia, which is more complex
        // For now, return an error
        Err(AudioBackendError::UnsupportedFormat(
            "Input streams not yet supported in Web Audio API backend".to_string(),
        ))
    }

    fn create_output_stream_with_callback(
        &mut self,
        _device_id: &str,
        config: AudioConfig,
        _callback: OutputCallback,
    ) -> Result<Box<dyn AudioStream>> {
        // For now, return error - full implementation requires ScriptProcessorNode
        Err(AudioBackendError::UnsupportedFormat(
            "Callback-based Web Audio streams not yet implemented".to_string(),
        ))
    }

    fn create_input_stream_with_callback(
        &mut self,
        _device_id: &str,
        _config: AudioConfig,
        _callback: InputCallback,
    ) -> Result<Box<dyn AudioStream>> {
        Err(AudioBackendError::UnsupportedFormat(
            "Input streams require getUserMedia API".to_string(),
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Web Audio output stream
#[cfg(target_arch = "wasm32")]
struct WebAudioOutputStream {
    context: WasmAudioContext,
    config: AudioConfig,
    status: StreamStatus,
}

#[cfg(target_arch = "wasm32")]
impl AudioStream for WebAudioOutputStream {
    fn play(&mut self) -> Result<()> {
        // Resume the audio context if suspended
        let ctx = self.context.get_or_create()?;
        let _promise = ctx.resume().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to resume context: {:?}", e))
        })?;

        self.status = StreamStatus::Playing;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        let ctx = self.context.get_or_create()?;
        let _promise = ctx.suspend().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to suspend context: {:?}", e))
        })?;

        self.status = StreamStatus::Paused;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        let ctx = self.context.get_or_create()?;
        let _promise = ctx.suspend().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to stop context: {:?}", e))
        })?;

        self.status = StreamStatus::Stopped;
        Ok(())
    }

    fn status(&self) -> StreamStatus {
        self.status
    }

    fn config(&self) -> &AudioConfig {
        &self.config
    }

    fn latency_samples(&self) -> Option<usize> {
        // Web Audio API has built-in buffering
        Some(self.config.buffer_size)
    }
}

// Stub implementation for non-WASM platforms
#[cfg(not(target_arch = "wasm32"))]
pub struct WebAudioBackend;

#[cfg(not(target_arch = "wasm32"))]
impl WebAudioBackend {
    pub fn new() -> Self {
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
    fn test_web_audio_backend_creation() {
        let backend = WebAudioBackend::new();
        assert!(backend.is_available());
        assert_eq!(backend.name(), "Web Audio API");
    }

    #[wasm_bindgen_test]
    fn test_web_audio_backend_initialization() {
        let mut backend = WebAudioBackend::new();
        assert!(backend.initialize().is_ok());
        assert!(backend.initialized);
    }

    #[wasm_bindgen_test]
    fn test_device_enumeration() {
        let mut backend = WebAudioBackend::new();
        backend.initialize().unwrap();

        let devices = backend.enumerate_devices(StreamDirection::Output).unwrap();
        assert!(!devices.is_empty());
        assert_eq!(devices[0].id, "default");
    }
}
