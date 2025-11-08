//! High-level audio device management
//!
//! Provides a convenient API for managing audio devices, streams, and playback

use super::backend::{
    AudioBackend, AudioBackendError, AudioConfig, AudioStream, DeviceInfo, Result,
    StreamDirection, StreamStatus,
};
use super::device::CpalBackend;
use parking_lot::RwLock;
use std::sync::Arc;

/// High-level audio device manager
pub struct AudioDeviceManager {
    backend: Arc<RwLock<Box<dyn AudioBackend>>>,
    selected_output_device: Arc<RwLock<Option<DeviceInfo>>>,
    selected_input_device: Arc<RwLock<Option<DeviceInfo>>>,
    current_stream: Arc<RwLock<Option<Box<dyn AudioStream>>>>,
}

impl AudioDeviceManager {
    /// Create a new AudioDeviceManager with the default backend (CPAL)
    pub fn new() -> Result<Self> {
        let mut backend: Box<dyn AudioBackend> = Box::new(CpalBackend::new());
        backend.initialize()?;
        
        let backend = Arc::new(RwLock::new(backend));
        
        // Try to get default output device
        let default_output = {
            let backend_lock = backend.read();
            backend_lock.default_device(StreamDirection::Output).ok()
        };
        
        Ok(Self {
            backend,
            selected_output_device: Arc::new(RwLock::new(default_output)),
            selected_input_device: Arc::new(RwLock::new(None)),
            current_stream: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Get all available output devices
    pub fn enumerate_output_devices(&self) -> Result<Vec<DeviceInfo>> {
        let backend = self.backend.read();
        backend.enumerate_devices(StreamDirection::Output)
    }
    
    /// Get all available input devices
    pub fn enumerate_input_devices(&self) -> Result<Vec<DeviceInfo>> {
        let backend = self.backend.read();
        backend.enumerate_devices(StreamDirection::Input)
    }
    
    /// Get the currently selected output device
    pub fn selected_output_device(&self) -> Option<DeviceInfo> {
        self.selected_output_device.read().clone()
    }
    
    /// Get the currently selected input device
    pub fn selected_input_device(&self) -> Option<DeviceInfo> {
        self.selected_input_device.read().clone()
    }
    
    /// Select an output device by ID
    pub fn select_output_device(&self, device_id: &str) -> Result<()> {
        let backend = self.backend.read();
        let devices = backend.enumerate_devices(StreamDirection::Output)?;
        
        let device = devices
            .into_iter()
            .find(|d| d.id == device_id)
            .ok_or_else(|| AudioBackendError::DeviceNotFound(device_id.to_string()))?;
        
        *self.selected_output_device.write() = Some(device);
        Ok(())
    }
    
    /// Select an input device by ID
    pub fn select_input_device(&self, device_id: &str) -> Result<()> {
        let backend = self.backend.read();
        let devices = backend.enumerate_devices(StreamDirection::Input)?;
        
        let device = devices
            .into_iter()
            .find(|d| d.id == device_id)
            .ok_or_else(|| AudioBackendError::DeviceNotFound(device_id.to_string()))?;
        
        *self.selected_input_device.write() = Some(device);
        Ok(())
    }
    
    /// Create an output stream with the currently selected device
    pub fn create_output_stream(&self, config: AudioConfig) -> Result<()> {
        let device = self
            .selected_output_device
            .read()
            .clone()
            .ok_or_else(|| AudioBackendError::DeviceNotFound("No output device selected".to_string()))?;
        
        let mut backend = self.backend.write();
        let stream = backend.create_output_stream(&device.id, config)?;
        
        *self.current_stream.write() = Some(stream);
        Ok(())
    }
    
    /// Create an input stream with the currently selected device
    pub fn create_input_stream(&self, config: AudioConfig) -> Result<()> {
        let device = self
            .selected_input_device
            .read()
            .clone()
            .ok_or_else(|| AudioBackendError::DeviceNotFound("No input device selected".to_string()))?;
        
        let mut backend = self.backend.write();
        let stream = backend.create_input_stream(&device.id, config)?;
        
        *self.current_stream.write() = Some(stream);
        Ok(())
    }
    
    /// Start playback/recording on the current stream
    pub fn play(&self) -> Result<()> {
        let mut stream_lock = self.current_stream.write();
        if let Some(stream) = stream_lock.as_mut() {
            stream.play()
        } else {
            Err(AudioBackendError::StreamError("No stream available".to_string()))
        }
    }
    
    /// Pause playback/recording on the current stream
    pub fn pause(&self) -> Result<()> {
        let mut stream_lock = self.current_stream.write();
        if let Some(stream) = stream_lock.as_mut() {
            stream.pause()
        } else {
            Err(AudioBackendError::StreamError("No stream available".to_string()))
        }
    }
    
    /// Stop playback/recording on the current stream
    pub fn stop(&self) -> Result<()> {
        let mut stream_lock = self.current_stream.write();
        if let Some(stream) = stream_lock.as_mut() {
            stream.stop()
        } else {
            Err(AudioBackendError::StreamError("No stream available".to_string()))
        }
    }
    
    /// Get the current stream status
    pub fn stream_status(&self) -> Option<StreamStatus> {
        let stream_lock = self.current_stream.read();
        stream_lock.as_ref().map(|s| s.status())
    }
    
    /// Get the current stream latency in milliseconds
    pub fn stream_latency_ms(&self) -> Option<f32> {
        let stream_lock = self.current_stream.read();
        stream_lock.as_ref().and_then(|s| s.latency_ms())
    }
    
    /// Test if a device is available and functional
    pub fn test_device(&self, device_id: &str) -> Result<bool> {
        let backend = self.backend.read();
        backend.test_device(device_id)
    }
    
    /// Get supported configurations for a device
    pub fn get_device_configs(&self, device_id: &str) -> Result<Vec<AudioConfig>> {
        let backend = self.backend.read();
        backend.supported_configs(device_id)
    }
    
    /// Get information about the current backend
    pub fn backend_name(&self) -> String {
        let backend = self.backend.read();
        backend.name().to_string()
    }
}

impl Default for AudioDeviceManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize audio device manager")
    }
}

// Make it Send + Sync for use in egui apps
unsafe impl Send for AudioDeviceManager {}
unsafe impl Sync for AudioDeviceManager {}
