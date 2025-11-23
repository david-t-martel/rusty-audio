//! Hybrid audio system
//!
//! Bridges web-audio-api's routing graph capabilities with native hardware audio via cpal.
//! This allows:
//! - Using web-audio-api's powerful routing/effects graph
//! - Native low-latency hardware output via cpal
//! - WASM/PWA compatibility through conditional compilation
//!
//! Architecture:
//! ```
//! [Web Audio Graph] → [ScriptProcessorNode] → [Ring Buffer] → [CPAL Stream] → [Hardware]
//!     (effects, EQ, routing)         ↓                             ↑
//!                              (process audio)              (native output)
//! ```

use super::backend::{
    AudioBackend, AudioBackendError, AudioConfig, AudioStream, InputCallback, OutputCallback,
    Result, StreamStatus,
};
use super::device::CpalBackend;
#[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
use super::asio_backend::AsioBackend;
use anyhow::anyhow;
use parking_lot::Mutex;
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Hybrid audio backend mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HybridMode {
    /// Use web-audio-api exclusively (WASM/browser mode)
    WebAudioOnly,
    /// Use cpal for output, web-audio-api for routing/effects
    HybridNative,
    /// Use cpal exclusively (future: for maximum performance)
    CpalOnly,
    /// Use ASIO exclusively (Windows only)
    AsioOnly,
}

/// Fallback policy for automatic backend switching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackPolicy {
    /// User controls mode manually (no automatic fallback)
    Manual,
    /// Automatically switch mode on errors (e.g., device disconnect)
    AutoOnError,
    /// Try preferred mode first, fallback to alternative if unavailable
    AutoWithPreference(HybridMode),
}

impl Default for FallbackPolicy {
    fn default() -> Self {
        FallbackPolicy::AutoOnError
    }
}

/// Audio backend health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendHealth {
    Healthy,
    Degraded,
    Failed,
}

/// Error types that can trigger fallback
#[derive(Debug, Clone)]
pub enum FallbackTrigger {
    DeviceDisconnected,
    StreamUnderrun { consecutive_count: u32 },
    BufferHealthCritical { fill_level: f32 },
    InitializationFailed,
    UnknownError(String),
}

/// Hybrid audio backend that combines web-audio-api routing with native audio
pub struct HybridAudioBackend {
    mode: HybridMode,
    fallback_policy: FallbackPolicy,
    health: BackendHealth,
    underrun_count: u32,

    #[cfg(not(target_arch = "wasm32"))]
    cpal_backend: Option<CpalBackend>,

    #[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
    asio_backend: Option<AsioBackend>,

    // Lock-free ring buffer components (wrapped in Mutex for Sync, but taken out for use)
    ring_producer: Mutex<Option<Producer<f32>>>,
    ring_consumer: Mutex<Option<Consumer<f32>>>,
    
    config: AudioConfig,
}

impl HybridAudioBackend {
    /// Create a new hybrid backend
    pub fn new() -> Self {
        // Auto-detect best mode
        let mode = if cfg!(target_arch = "wasm32") {
            HybridMode::WebAudioOnly
        } else {
            // On Windows, prefer ASIO if available, otherwise hybrid native (WASAPI)
            #[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
            {
                if AsioBackend::asio_available() {
                    HybridMode::AsioOnly
                } else {
                    HybridMode::HybridNative
                }
            }
            #[cfg(not(target_os = "windows"))]
            HybridMode::HybridNative
        };

        Self::with_mode(mode)
    }

    /// Create hybrid backend with specific mode
    pub fn with_mode(mode: HybridMode) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let cpal_backend = if mode == HybridMode::HybridNative || mode == HybridMode::CpalOnly {
            Some(CpalBackend::new())
        } else {
            None
        };

        #[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
        let asio_backend = if mode == HybridMode::AsioOnly {
            Some(AsioBackend::new())
        } else {
            None
        };

        Self {
            mode,
            fallback_policy: FallbackPolicy::default(),
            health: BackendHealth::Healthy,
            underrun_count: 0,
            #[cfg(not(target_arch = "wasm32"))]
            cpal_backend,
            #[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
            asio_backend,
            ring_producer: Mutex::new(None),
            ring_consumer: Mutex::new(None),
            config: AudioConfig::default(),
        }
    }

    /// Get the current hybrid mode
    pub fn mode(&self) -> HybridMode {
        self.mode
    }

    /// Set the hybrid mode (may require reinitialization)
    pub fn set_mode(&mut self, mode: HybridMode) -> Result<()> {
        if self.mode == mode {
            return Ok(());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Update backend availability
            match mode {
                HybridMode::WebAudioOnly => {
                    self.cpal_backend = None;
                    #[cfg(target_os = "windows")]
                    {
                        self.asio_backend = None;
                    }
                }
                HybridMode::HybridNative | HybridMode::CpalOnly => {
                    #[cfg(target_os = "windows")]
                    {
                        self.asio_backend = None;
                    }
                    if self.cpal_backend.is_none() {
                        let mut backend = CpalBackend::new();
                        backend.initialize()?;
                        self.cpal_backend = Some(backend);
                    }
                }
                #[cfg(target_os = "windows")]
                HybridMode::AsioOnly => {
                    self.cpal_backend = None;
                    if self.asio_backend.is_none() {
                        let mut backend = AsioBackend::new();
                        backend.initialize()?;
                        self.asio_backend = Some(backend);
                    }
                }
                #[cfg(not(target_os = "windows"))]
                HybridMode::AsioOnly => {
                    return Err(AudioBackendError::BackendNotAvailable(
                        "ASIO not supported on this platform".to_string(),
                    ));
                }
            }
        }

        self.mode = mode;
        Ok(())
    }

    /// Create lock-free ring buffer for hybrid mode
    fn create_ring_buffer(&mut self, buffer_size: usize) {
        // Use 8x buffer size for ring buffer to avoid underruns
        let capacity = buffer_size * 8;
        let (producer, consumer) = RingBuffer::new(capacity);
        *self.ring_producer.lock() = Some(producer);
        *self.ring_consumer.lock() = Some(consumer);
    }

    /// Take the producer for the ring buffer (to be given to WebAudioEngine)
    pub fn take_ring_producer(&mut self) -> Option<Producer<f32>> {
        self.ring_producer.lock().take()
    }

    /// Get the current fallback policy
    pub fn fallback_policy(&self) -> FallbackPolicy {
        self.fallback_policy
    }

    /// Set the fallback policy
    pub fn set_fallback_policy(&mut self, policy: FallbackPolicy) {
        self.fallback_policy = policy;
    }

    /// Get the current backend health status
    pub fn health(&self) -> BackendHealth {
        self.health
    }

    /// Report a buffer underrun (called from audio callback)
    pub fn report_underrun(&mut self) {
        self.underrun_count += 1;

        // Degrade health after 3 consecutive underruns
        if self.underrun_count >= 3 {
            self.health = BackendHealth::Degraded;

            // Trigger fallback after 10 consecutive underruns
            if self.underrun_count >= 10 {
                self.health = BackendHealth::Failed;
                let trigger = FallbackTrigger::StreamUnderrun {
                    consecutive_count: self.underrun_count,
                };
                let _ = self.trigger_fallback(trigger);
            }
        }
    }

    /// Reset underrun counter (called when audio is healthy)
    pub fn reset_underrun_count(&mut self) {
        if self.underrun_count > 0 {
            self.underrun_count = 0;
            self.health = BackendHealth::Healthy;
        }
    }

    /// Trigger automatic fallback based on error condition
    pub fn trigger_fallback(&mut self, trigger: FallbackTrigger) -> Result<()> {
        // Only trigger fallback if policy allows it
        match self.fallback_policy {
            FallbackPolicy::Manual => {
                // Manual mode: just log the error but don't switch
                let msg = format!("Fallback triggered but policy is Manual: {:?}", trigger);
                return Err(AudioBackendError::Other(anyhow::anyhow!(msg)));
            }
            FallbackPolicy::AutoOnError | FallbackPolicy::AutoWithPreference(_) => {
                // Automatic fallback enabled
            }
        }

        // Determine fallback mode
        let fallback_mode = match self.mode {
            HybridMode::WebAudioOnly => {
                // Can't fallback from web-audio-only
                return Err(AudioBackendError::Other(anyhow::anyhow!(
                    "Cannot fallback from WebAudioOnly mode"
                )));
            }
            HybridMode::HybridNative => HybridMode::WebAudioOnly,
            HybridMode::CpalOnly => HybridMode::HybridNative,
            #[cfg(target_os = "windows")]
            HybridMode::AsioOnly => HybridMode::HybridNative,
            #[cfg(not(target_os = "windows"))]
            HybridMode::AsioOnly => HybridMode::HybridNative,
        };

        // Attempt to switch to fallback mode
        self.set_mode(fallback_mode)?;
        self.health = BackendHealth::Healthy;
        self.underrun_count = 0;

        Ok(())
    }

    /// Initialize the hybrid backend (public method for all platforms)
    pub fn initialize(&mut self) -> Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match self.mode {
                HybridMode::WebAudioOnly => Ok(()),
                HybridMode::HybridNative | HybridMode::CpalOnly => {
                    if let Some(backend) = &mut self.cpal_backend {
                        backend.initialize()
                    } else {
                        Err(AudioBackendError::BackendNotAvailable(
                            "CPAL backend not initialized".to_string(),
                        ))
                    }
                }
                #[cfg(target_os = "windows")]
                HybridMode::AsioOnly => {
                    if let Some(backend) = &mut self.asio_backend {
                        backend.initialize()
                    } else {
                        Err(AudioBackendError::BackendNotAvailable(
                            "ASIO backend not initialized".to_string(),
                        ))
                    }
                }
                #[cfg(not(target_os = "windows"))]
                HybridMode::AsioOnly => Err(AudioBackendError::BackendNotAvailable(
                    "ASIO backend only available on Windows".to_string(),
                )),
            }
        }
        #[cfg(target_arch = "wasm32")]
        Ok(())
    }
}

impl Default for HybridAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl AudioBackend for HybridAudioBackend {
    fn name(&self) -> &'static str {
        match self.mode {
            HybridMode::WebAudioOnly => "web-audio-api",
            HybridMode::HybridNative => "hybrid(web-audio + cpal)",
            HybridMode::CpalOnly => "cpal",
            #[cfg(target_os = "windows")]
            HybridMode::AsioOnly => "asio",
            #[cfg(not(target_os = "windows"))]
            HybridMode::AsioOnly => "asio (unavailable)",
        }
    }

    fn is_available(&self) -> bool {
        match self.mode {
            HybridMode::WebAudioOnly => true, // web-audio-api always available
            HybridMode::HybridNative | HybridMode::CpalOnly => self
                .cpal_backend
                .as_ref()
                .map(|b| b.is_available())
                .unwrap_or(false),
            #[cfg(target_os = "windows")]
            HybridMode::AsioOnly => self
                .asio_backend
                .as_ref()
                .map(|b| b.is_available())
                .unwrap_or(false),
            #[cfg(not(target_os = "windows"))]
            HybridMode::AsioOnly => false,
        }
    }

    fn initialize(&mut self) -> Result<()> {
        match self.mode {
            HybridMode::WebAudioOnly => Ok(()),
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                if let Some(backend) = &mut self.cpal_backend {
                    backend.initialize()
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not initialized".to_string(),
                    ))
                }
            }
            #[cfg(target_os = "windows")]
            HybridMode::AsioOnly => {
                if let Some(backend) = &mut self.asio_backend {
                    backend.initialize()
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "ASIO backend not initialized".to_string(),
                    ))
                }
            }
            #[cfg(not(target_os = "windows"))]
            HybridMode::AsioOnly => Err(AudioBackendError::BackendNotAvailable(
                "ASIO backend only available on Windows".to_string(),
            )),
        }
    }

    fn enumerate_devices(
        &self,
        direction: super::backend::StreamDirection,
    ) -> Result<Vec<super::backend::DeviceInfo>> {
        match self.mode {
            HybridMode::WebAudioOnly => {
                // Return a dummy "Browser Audio" device for web-audio-api mode
                Ok(vec![super::backend::DeviceInfo {
                    id: "browser-audio".to_string(),
                    name: "Browser Audio API".to_string(),
                    is_default: true,
                    supported_configs: vec![self.config.clone()],
                    min_sample_rate: 44100,
                    max_sample_rate: 48000,
                    max_input_channels: 2,
                    max_output_channels: 2,
                }])
            }
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                if let Some(backend) = &self.cpal_backend {
                    backend.enumerate_devices(direction)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not available".to_string(),
                    ))
                }
            }
            #[cfg(target_os = "windows")]
            HybridMode::AsioOnly => {
                if let Some(backend) = &self.asio_backend {
                    backend.enumerate_devices(direction)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "ASIO backend not available".to_string(),
                    ))
                }
            }
            #[cfg(not(target_os = "windows"))]
            HybridMode::AsioOnly => Ok(Vec::new()),
        }
    }

    fn default_device(
        &self,
        direction: super::backend::StreamDirection,
    ) -> Result<super::backend::DeviceInfo> {
        match self.mode {
            HybridMode::WebAudioOnly => Ok(super::backend::DeviceInfo {
                id: "browser-audio".to_string(),
                name: "Browser Audio API".to_string(),
                is_default: true,
                supported_configs: vec![self.config.clone()],
                min_sample_rate: 44100,
                max_sample_rate: 48000,
                max_input_channels: 2,
                max_output_channels: 2,
            }),
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                if let Some(backend) = &self.cpal_backend {
                    backend.default_device(direction)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not available".to_string(),
                    ))
                }
            }
            #[cfg(target_os = "windows")]
            HybridMode::AsioOnly => {
                if let Some(backend) = &self.asio_backend {
                    backend.default_device(direction)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "ASIO backend not available".to_string(),
                    ))
                }
            }
            #[cfg(not(target_os = "windows"))]
            HybridMode::AsioOnly => Err(AudioBackendError::BackendNotAvailable(
                "Not available on this platform".to_string(),
            )),
        }
    }

    fn test_device(&self, device_id: &str) -> Result<bool> {
        match self.mode {
            HybridMode::WebAudioOnly => Ok(device_id == "browser-audio"),
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                if let Some(backend) = &self.cpal_backend {
                    backend.test_device(device_id)
                } else {
                    Ok(false)
                }
            }
            #[cfg(target_os = "windows")]
            HybridMode::AsioOnly => {
                if let Some(backend) = &self.asio_backend {
                    backend.test_device(device_id)
                } else {
                    Ok(false)
                }
            }
            #[cfg(not(target_os = "windows"))]
            HybridMode::AsioOnly => Ok(false),
        }
    }

    fn supported_configs(
        &self,
        device_id: &str,
        direction: super::backend::StreamDirection,
    ) -> Result<Vec<AudioConfig>> {
        match self.mode {
            HybridMode::WebAudioOnly => {
                if device_id == "browser-audio" {
                    Ok(vec![self.config.clone()])
                } else {
                    Err(AudioBackendError::DeviceNotFound(device_id.to_string()))
                }
            }
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                if let Some(backend) = &self.cpal_backend {
                    backend.supported_configs(device_id, direction)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not available".to_string(),
                    ))
                }
            }
            #[cfg(target_os = "windows")]
            HybridMode::AsioOnly => {
                if let Some(backend) = &self.asio_backend {
                    backend.supported_configs(device_id, direction)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "ASIO backend not available".to_string(),
                    ))
                }
            }
            #[cfg(not(target_os = "windows"))]
            HybridMode::AsioOnly => Ok(Vec::new()),
        }
    }

    fn create_output_stream(
        &mut self,
        device_id: &str,
        config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>> {
        self.config = config.clone();

        match self.mode {
            HybridMode::WebAudioOnly => {
                // Create a no-op stream for web-audio-api mode
                Ok(Box::new(NoOpStream {
                    config,
                    status: StreamStatus::Stopped,
                }))
            }
            HybridMode::HybridNative => {
                // Create ring buffer for hybrid mode
                self.create_ring_buffer(config.buffer_size);

                // Create cpal stream that reads from ring buffer
                if let Some(backend) = &mut self.cpal_backend {
                    let mut ring_consumer = self.ring_consumer.lock().take().ok_or_else(|| {
                        AudioBackendError::Other(anyhow!(
                            "Ring buffer not initialized for hybrid output stream"
                        ))
                    })?;

                    // Create stream with callback that reads from ring buffer
                    let stream = backend.create_output_stream_with_callback(
                        device_id,
                        config.clone(),
                        Box::new(move |output: &mut [f32]| {
                            // Read from ring buffer into output
                            let mut read = 0;
                            let chunk_size = output.len();
                            if let Ok(chunk) = ring_consumer.read_chunk(chunk_size) {
                                // Copy available data
                                let (s1, s2) = chunk.as_slices();
                                let len1 = s1.len();
                                output[..len1].copy_from_slice(s1);
                                output[len1..len1 + s2.len()].copy_from_slice(s2);
                                read = len1 + s2.len();
                                chunk.commit_all();
                            }
                            
                            // Fill remaining with silence
                            if read < chunk_size {
                                output[read..].fill(0.0);
                            }
                        }),
                    )?;

                    Ok(stream)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not available".to_string(),
                    ))
                }
            }
            HybridMode::CpalOnly => {
                if let Some(backend) = &mut self.cpal_backend {
                    backend.create_output_stream(device_id, config)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not available".to_string(),
                    ))
                }
            }
            #[cfg(target_os = "windows")]
            HybridMode::AsioOnly => {
                if let Some(backend) = &mut self.asio_backend {
                    backend.create_output_stream(device_id, config)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "ASIO backend not available".to_string(),
                    ))
                }
            }
            #[cfg(not(target_os = "windows"))]
            HybridMode::AsioOnly => Err(AudioBackendError::BackendNotAvailable(
                "Not available on this platform".to_string(),
            )),
        }
    }

    fn create_input_stream(
        &mut self,
        device_id: &str,
        config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>> {
        match self.mode {
            HybridMode::WebAudioOnly => Ok(Box::new(NoOpStream {
                config,
                status: StreamStatus::Stopped,
            })),
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                if let Some(backend) = &mut self.cpal_backend {
                    backend.create_input_stream(device_id, config)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not available".to_string(),
                    ))
                }
            }
            #[cfg(target_os = "windows")]
            HybridMode::AsioOnly => {
                if let Some(backend) = &mut self.asio_backend {
                    backend.create_input_stream(device_id, config)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "ASIO backend not available".to_string(),
                    ))
                }
            }
            #[cfg(not(target_os = "windows"))]
            HybridMode::AsioOnly => Err(AudioBackendError::BackendNotAvailable(
                "Not available on this platform".to_string(),
            )),
        }
    }

    fn create_output_stream_with_callback(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: OutputCallback,
    ) -> Result<Box<dyn AudioStream>> {
        match self.mode {
            HybridMode::WebAudioOnly => Err(AudioBackendError::UnsupportedFormat(
                "Callback streams not supported in WebAudioOnly mode".to_string(),
            )),
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                if let Some(backend) = &mut self.cpal_backend {
                    backend.create_output_stream_with_callback(device_id, config, callback)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not initialized".to_string(),
                    ))
                }
            }
            #[cfg(target_os = "windows")]
            HybridMode::AsioOnly => {
                if let Some(backend) = &mut self.asio_backend {
                    backend.create_output_stream_with_callback(device_id, config, callback)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "ASIO backend not initialized".to_string(),
                    ))
                }
            }
            #[cfg(not(target_os = "windows"))]
            HybridMode::AsioOnly => Err(AudioBackendError::BackendNotAvailable(
                "ASIO backend not available".to_string(),
            )),
        }
    }

    fn create_input_stream_with_callback(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: InputCallback,
    ) -> Result<Box<dyn AudioStream>> {
        match self.mode {
            HybridMode::WebAudioOnly => Err(AudioBackendError::UnsupportedFormat(
                "Input callback streams not supported in WebAudioOnly mode".to_string(),
            )),
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                if let Some(backend) = &mut self.cpal_backend {
                    backend.create_input_stream_with_callback(device_id, config, callback)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not initialized".to_string(),
                    ))
                }
            }
            #[cfg(target_os = "windows")]
            HybridMode::AsioOnly => {
                if let Some(backend) = &mut self.asio_backend {
                    backend.create_input_stream_with_callback(device_id, config, callback)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "ASIO backend not initialized".to_string(),
                    ))
                }
            }
            #[cfg(not(target_os = "windows"))]
            HybridMode::AsioOnly => Err(AudioBackendError::BackendNotAvailable(
                "ASIO backend not available".to_string(),
            )),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// No-op stream for web-audio-only mode
struct NoOpStream {
    config: AudioConfig,
    status: StreamStatus,
}

impl AudioStream for NoOpStream {
    fn play(&mut self) -> Result<()> {
        self.status = StreamStatus::Playing;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.status = StreamStatus::Paused;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
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
        Some(self.config.buffer_size)
    }
}

// Make no-op stream thread-safe
unsafe impl Send for NoOpStream {}
