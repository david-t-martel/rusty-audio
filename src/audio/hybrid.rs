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

use super::backend::{AudioBackend, AudioBackendError, AudioConfig, AudioStream, Result, StreamStatus};
use super::device::CpalBackend;
use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Ring buffer for passing audio from web-audio-api to cpal
pub struct AudioRingBuffer {
    buffer: Arc<RwLock<Vec<f32>>>,
    write_pos: Arc<RwLock<usize>>,
    read_pos: Arc<RwLock<usize>>,
    capacity: usize,
}

impl AudioRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(vec![0.0; capacity])),
            write_pos: Arc::new(RwLock::new(0)),
            read_pos: Arc::new(RwLock::new(0)),
            capacity,
        }
    }
    
    /// Write samples to the ring buffer (called by web-audio-api ScriptProcessor)
    pub fn write(&self, samples: &[f32]) -> usize {
        let mut buffer = self.buffer.write();
        let mut write_pos = self.write_pos.write();
        let read_pos = *self.read_pos.read();
        
        let mut written = 0;
        for &sample in samples {
            let next_write_pos = (*write_pos + 1) % self.capacity;
            // Don't overwrite unread data
            if next_write_pos == read_pos {
                break; // Buffer full
            }
            buffer[*write_pos] = sample;
            *write_pos = next_write_pos;
            written += 1;
        }
        
        written
    }
    
    /// Read samples from ring buffer (called by cpal audio callback)
    pub fn read(&self, output: &mut [f32]) -> usize {
        let buffer = self.buffer.read();
        let write_pos = *self.write_pos.read();
        let mut read_pos = self.read_pos.write();
        
        let mut read = 0;
        for sample in output.iter_mut() {
            if *read_pos == write_pos {
                *sample = 0.0; // Underrun, output silence
            } else {
                *sample = buffer[*read_pos];
                *read_pos = (*read_pos + 1) % self.capacity;
                read += 1;
            }
        }
        
        read
    }
    
    /// Get number of samples available to read
    pub fn available(&self) -> usize {
        let write_pos = *self.write_pos.read();
        let read_pos = *self.read_pos.read();
        
        if write_pos >= read_pos {
            write_pos - read_pos
        } else {
            self.capacity - read_pos + write_pos
        }
    }
}

/// Hybrid audio backend mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HybridMode {
    /// Use web-audio-api exclusively (WASM/browser mode)
    WebAudioOnly,
    /// Use cpal for output, web-audio-api for routing/effects
    HybridNative,
    /// Use cpal exclusively (future: for maximum performance)
    CpalOnly,
}

/// Hybrid audio backend that combines web-audio-api routing with native audio
pub struct HybridAudioBackend {
    mode: HybridMode,
    
    #[cfg(not(target_arch = "wasm32"))]
    cpal_backend: Option<CpalBackend>,
    
    ring_buffer: Option<Arc<AudioRingBuffer>>,
    config: AudioConfig,
}

impl HybridAudioBackend {
    /// Create a new hybrid backend
    pub fn new() -> Self {
        // Auto-detect best mode
        let mode = if cfg!(target_arch = "wasm32") {
            HybridMode::WebAudioOnly
        } else {
            HybridMode::HybridNative
        };
        
        Self::with_mode(mode)
    }
    
    /// Create hybrid backend with specific mode
    pub fn with_mode(mode: HybridMode) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let cpal_backend = if mode != HybridMode::WebAudioOnly {
            Some(CpalBackend::new())
        } else {
            None
        };
        
        Self {
            mode,
            #[cfg(not(target_arch = "wasm32"))]
            cpal_backend,
            ring_buffer: None,
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
            // Update cpal backend availability
            match mode {
                HybridMode::WebAudioOnly => {
                    self.cpal_backend = None;
                }
                HybridMode::HybridNative | HybridMode::CpalOnly => {
                    if self.cpal_backend.is_none() {
                        let mut backend = CpalBackend::new();
                        backend.initialize()?;
                        self.cpal_backend = Some(backend);
                    }
                }
            }
        }
        
        self.mode = mode;
        Ok(())
    }
    
    /// Create ring buffer for hybrid mode
    fn create_ring_buffer(&mut self, buffer_size: usize) {
        // Use 8x buffer size for ring buffer to avoid underruns
        let capacity = buffer_size * 8;
        self.ring_buffer = Some(Arc::new(AudioRingBuffer::new(capacity)));
    }
    
    /// Get the ring buffer for connecting web-audio-api
    pub fn ring_buffer(&self) -> Option<Arc<AudioRingBuffer>> {
        self.ring_buffer.clone()
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
                            "CPAL backend not initialized".to_string()
                        ))
                    }
                }
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
        }
    }
    
    fn is_available(&self) -> bool {
        match self.mode {
            HybridMode::WebAudioOnly => true, // web-audio-api always available
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                self.cpal_backend.as_ref().map(|b| b.is_available()).unwrap_or(false)
            }
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
                        "CPAL backend not initialized".to_string()
                    ))
                }
            }
        }
    }
    
    fn enumerate_devices(&self, direction: super::backend::StreamDirection) -> Result<Vec<super::backend::DeviceInfo>> {
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
                        "CPAL backend not available".to_string()
                    ))
                }
            }
        }
    }
    
    fn default_device(&self, direction: super::backend::StreamDirection) -> Result<super::backend::DeviceInfo> {
        match self.mode {
            HybridMode::WebAudioOnly => {
                Ok(super::backend::DeviceInfo {
                    id: "browser-audio".to_string(),
                    name: "Browser Audio API".to_string(),
                    is_default: true,
                    supported_configs: vec![self.config.clone()],
                    min_sample_rate: 44100,
                    max_sample_rate: 48000,
                    max_input_channels: 2,
                    max_output_channels: 2,
                })
            }
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                if let Some(backend) = &self.cpal_backend {
                    backend.default_device(direction)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not available".to_string()
                    ))
                }
            }
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
        }
    }
    
    fn supported_configs(&self, device_id: &str) -> Result<Vec<AudioConfig>> {
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
                    backend.supported_configs(device_id)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not available".to_string()
                    ))
                }
            }
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
                    let ring_buffer = self.ring_buffer.as_ref().unwrap().clone();
                    
                    // Create stream with callback that reads from ring buffer
                    let stream = backend.create_output_stream_with_callback(
                        device_id,
                        config.clone(),
                        move |output: &mut [f32]| {
                            // Read from ring buffer into output
                            ring_buffer.read(output);
                        }
                    )?;
                    
                    Ok(stream)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not available".to_string()
                    ))
                }
            }
            HybridMode::CpalOnly => {
                if let Some(backend) = &mut self.cpal_backend {
                    backend.create_output_stream(device_id, config)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not available".to_string()
                    ))
                }
            }
        }
    }
    
    fn create_input_stream(
        &mut self,
        device_id: &str,
        config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>> {
        match self.mode {
            HybridMode::WebAudioOnly => {
                Ok(Box::new(NoOpStream {
                    config,
                    status: StreamStatus::Stopped,
                }))
            }
            HybridMode::HybridNative | HybridMode::CpalOnly => {
                if let Some(backend) = &mut self.cpal_backend {
                    backend.create_input_stream(device_id, config)
                } else {
                    Err(AudioBackendError::BackendNotAvailable(
                        "CPAL backend not available".to_string()
                    ))
                }
            }
        }
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

// Make ring buffer thread-safe
unsafe impl Send for AudioRingBuffer {}
unsafe impl Sync for AudioRingBuffer {}
unsafe impl Send for NoOpStream {}

// Re-export for convenience
pub use AudioRingBuffer as HybridRingBuffer;
