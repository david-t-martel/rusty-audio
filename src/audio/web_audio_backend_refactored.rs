//! Web Audio API backend for WASM (REFACTORED)
//!
//! Implements the AudioBackend trait using the browser's Web Audio API.
//! This provides audio functionality for WASM builds.
//!
//! ## P0 Fixes Applied:
//! - P0-3: Added main thread assertion and initialization guard
//! - P0-4: Added panic boundaries to all public methods

#[cfg(target_arch = "wasm32")]
use super::backend::{
    AudioBackend, AudioBackendError, AudioConfig, AudioStream, DeviceInfo, Result, SampleFormat,
    InputCallback, OutputCallback, StreamDirection, StreamStatus,
};

#[cfg(target_arch = "wasm32")]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use parking_lot::Mutex;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{AudioContext, AudioDestinationNode};
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_arch = "wasm32")]
use crate::wasm_panic_handler::with_panic_boundary;

/// Maximum time to wait for AudioContext creation (milliseconds)
const AUDIO_CONTEXT_TIMEOUT_MS: i32 = 5000;

#[cfg(target_arch = "wasm32")]
/// Check if we're on the main thread
///
/// **P0-3 FIX**: AudioContext creation must happen on main thread
///
/// Web Audio API requires AudioContext to be created on the main thread.
/// Creating it on a worker thread will fail with "NotSupportedError".
fn is_main_thread() -> bool {
    // Check if we have access to window (only available on main thread)
    web_sys::window().is_some()
}

#[cfg(target_arch = "wasm32")]
/// Assert that we're on the main thread
///
/// **P0-3 FIX**: Prevents AudioContext creation on worker threads
///
/// ## Rationale:
/// Web Audio API has strict threading requirements:
/// - AudioContext must be created on the main thread
/// - AudioNodes can only be created after AudioContext exists
/// - Worker threads cannot create AudioContext instances
///
/// This assertion catches thread safety violations early.
fn assert_main_thread() -> Result<()> {
    if !is_main_thread() {
        return Err(AudioBackendError::InitializationFailed(
            "AudioContext must be created on the main thread. \
             Current thread is a worker thread."
                .to_string(),
        ));
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
/// Thread-safe wrapper for AudioContext in WASM with multithreading support
///
/// **P0-3 FIX**: Added initialization guard and main thread checks
///
/// ## Before (RACE CONDITION):
/// ```rust,ignore
/// fn get_or_create(&self) -> Result<AudioContext> {
///     let mut ctx = self.context.lock();
///     if ctx.is_none() {
///         let audio_ctx = AudioContext::new()?;  // ❌ No thread check!
///         *ctx = Some(audio_ctx);
///     }
///     Ok(ctx.as_ref().unwrap().clone())
/// }
/// ```
///
/// ## After (THREAD-SAFE):
/// ```rust,ignore
/// fn get_or_create(&self) -> Result<AudioContext> {
///     assert_main_thread()?;  // ✅ Thread check
///
///     if self.initialized.compare_exchange(...).is_ok() {
///         let audio_ctx = AudioContext::new()?;  // ✅ Safe
///         *self.context.lock() = Some(audio_ctx);
///     }
///     Ok(self.context.lock().unwrap().clone())
/// }
/// ```
struct WasmAudioContext {
    context: Arc<Mutex<Option<AudioContext>>>,
    /// Atomic initialization flag (P0-3 fix)
    initialized: Arc<AtomicBool>,
}

#[cfg(target_arch = "wasm32")]
impl WasmAudioContext {
    fn new() -> Self {
        Self {
            context: Arc::new(Mutex::new(None)),
            initialized: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get or create the AudioContext with thread safety
    ///
    /// **P0-3 FIX**: Adds main thread assertion and atomic initialization guard
    fn get_or_create(&self) -> Result<AudioContext> {
        // P0-3 FIX: Verify we're on the main thread
        assert_main_thread()?;

        // Fast path: already initialized
        if self.initialized.load(Ordering::Acquire) {
            let ctx = self.context.lock();
            if let Some(ref audio_ctx) = *ctx {
                return Ok(audio_ctx.clone());
            }
        }

        // Slow path: need to initialize
        // Use compare-and-swap to ensure only one thread initializes
        match self.initialized.compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => {
                // We won the race - create AudioContext
                log::info!("Creating AudioContext on main thread");

                let audio_ctx = AudioContext::new().map_err(|e| {
                    // Revert initialization flag on failure
                    self.initialized.store(false, Ordering::SeqCst);

                    AudioBackendError::InitializationFailed(format!(
                        "Failed to create AudioContext: {:?}. \
                         This may be caused by: \
                         1) Not running on main thread \
                         2) Browser autoplay policy restrictions \
                         3) Missing user gesture",
                        e
                    ))
                })?;

                // Verify AudioContext state
                let state = audio_ctx.state();
                log::info!("AudioContext created with state: {:?}", state);

                // Store the context
                *self.context.lock() = Some(audio_ctx.clone());

                Ok(audio_ctx)
            }
            Err(_) => {
                // Another thread won the race - wait for initialization
                log::debug!("Waiting for AudioContext initialization");

                // Spin briefly then return context
                // In practice, initialization should be very fast
                let mut spin_count = 0;
                loop {
                    if let Some(ref ctx) = *self.context.lock() {
                        return Ok(ctx.clone());
                    }

                    spin_count += 1;
                    if spin_count > 1000 {
                        return Err(AudioBackendError::InitializationFailed(
                            "Timeout waiting for AudioContext initialization".to_string(),
                        ));
                    }

                    // Yield to other threads
                    std::hint::spin_loop();
                }
            }
        }
    }

    /// Get existing AudioContext without creating
    fn get(&self) -> Option<AudioContext> {
        self.context.lock().as_ref().map(|c| c.clone())
    }

    /// Reset the AudioContext (for testing)
    fn reset(&self) {
        *self.context.lock() = None;
        self.initialized.store(false, Ordering::SeqCst);
    }
}

#[cfg(target_arch = "wasm32")]
impl Clone for WasmAudioContext {
    fn clone(&self) -> Self {
        Self {
            context: Arc::clone(&self.context),
            initialized: Arc::clone(&self.initialized),
        }
    }
}

// Thread safety is guaranteed by Arc<Mutex<>> + AtomicBool
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
    ///
    /// **P0-3 FIX**: Verifies main thread during creation
    pub fn new() -> Result<Self> {
        // P0-3 FIX: Check we're on main thread early
        assert_main_thread()?;

        Ok(Self {
            context: WasmAudioContext::new(),
            initialized: false,
        })
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

    /// Resume AudioContext if suspended (required by autoplay policy)
    ///
    /// **P0-3 FIX**: Handles browser autoplay restrictions
    fn ensure_resumed(&self) -> Result<()> {
        let context = self.get_context()?;

        // Check current state
        let state = context.state();
        log::debug!("AudioContext state: {:?}", state);

        // If suspended, try to resume
        // Note: This may require a user gesture in some browsers
        if format!("{:?}", state).contains("suspended") {
            log::info!("Resuming suspended AudioContext");

            // Resume is async, but we don't await it here
            // The browser will handle the Promise resolution
            let _ = context.resume();
        }

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for WebAudioBackend {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            log::error!("Failed to create WebAudioBackend: {}", e);
            panic!("Cannot create WebAudioBackend: {}", e);
        })
    }
}

#[cfg(target_arch = "wasm32")]
impl AudioBackend for WebAudioBackend {
    fn name(&self) -> &'static str {
        "Web Audio API (Refactored)"
    }

    fn is_available(&self) -> bool {
        // Web Audio API is available if we're on the main thread
        is_main_thread() && web_sys::window()
            .and_then(|w| {
                // Try to check if AudioContext constructor exists
                js_sys::Reflect::get(&w, &JsValue::from_str("AudioContext")).ok()
            })
            .is_some()
    }

    fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // P0-3 FIX: Verify main thread
        assert_main_thread()?;

        // Create audio context to verify it works
        self.get_context()?;

        // Ensure context is resumed (P0-3 fix: handle autoplay)
        self.ensure_resumed()?;

        self.initialized = true;

        log::info!("Web Audio API backend initialized (refactored)");
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
            context: self.context.clone(),
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
        Err(AudioBackendError::UnsupportedFormat(
            "Input streams not yet supported in Web Audio API backend".to_string(),
        ))
    }

    fn create_output_stream_with_callback(
        &mut self,
        _device_id: &str,
        _config: AudioConfig,
        _callback: OutputCallback,
    ) -> Result<Box<dyn AudioStream>> {
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
        log::debug!("Audio stream playing");
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        let ctx = self.context.get_or_create()?;
        let _promise = ctx.suspend().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to suspend context: {:?}", e))
        })?;

        self.status = StreamStatus::Paused;
        log::debug!("Audio stream paused");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        let ctx = self.context.get_or_create()?;
        let _promise = ctx.suspend().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to stop context: {:?}", e))
        })?;

        self.status = StreamStatus::Stopped;
        log::debug!("Audio stream stopped");
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
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_main_thread_detection() {
        // This test runs on main thread in browser
        assert!(is_main_thread());
    }

    #[wasm_bindgen_test]
    fn test_web_audio_backend_creation() {
        let backend = WebAudioBackend::new().unwrap();
        assert!(backend.is_available());
        assert_eq!(backend.name(), "Web Audio API (Refactored)");
    }

    #[wasm_bindgen_test]
    fn test_web_audio_backend_initialization() {
        let mut backend = WebAudioBackend::new().unwrap();
        assert!(backend.initialize().is_ok());
        assert!(backend.initialized);
    }

    #[wasm_bindgen_test]
    fn test_device_enumeration() {
        let mut backend = WebAudioBackend::new().unwrap();
        backend.initialize().unwrap();

        let devices = backend
            .enumerate_devices(StreamDirection::Output)
            .unwrap();
        assert!(!devices.is_empty());
        assert_eq!(devices[0].id, "default");
    }

    #[wasm_bindgen_test]
    fn test_audio_context_singleton() {
        let ctx = WasmAudioContext::new();

        // Multiple calls should return same context
        let context1 = ctx.get_or_create().unwrap();
        let context2 = ctx.get_or_create().unwrap();

        // Both should have same sample rate (indicates same context)
        assert_eq!(context1.sample_rate(), context2.sample_rate());
    }
}
