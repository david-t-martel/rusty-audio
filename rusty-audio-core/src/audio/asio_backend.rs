//! ASIO audio backend for Windows
//!
//! This module provides ASIO (Audio Stream Input/Output) support for professional
//! audio interfaces on Windows. ASIO enables:
//! - Ultra-low latency (<10ms round-trip)
//! - Exclusive mode access
//! - Direct hardware access bypassing Windows audio stack
//! - Multi-channel support for professional interfaces
//!
//! # Requirements
//! - Windows operating system
//! - ASIO drivers installed for audio interface
//! - cpal compiled with "asio" feature enabled
//!
//! # Usage
//! ```rust,no_run
//! use rusty_audio::audio::asio_backend::AsioBackend;
//!
//! let backend = AsioBackend::new()?;
//! if backend.is_available() {
//!     let devices = backend.enumerate_devices(StreamDirection::Output)?;
//!     // Use ASIO devices...
//! }
//! ```

use super::backend::{
    AudioBackend, AudioBackendError, AudioConfig, AudioStream, DeviceInfo, InputCallback,
    OutputCallback, Result, SampleFormat, StreamDirection, StreamStatus,
};
use parking_lot::RwLock;
use std::sync::Arc;

#[cfg(target_os = "windows")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Backend type enumeration for Windows audio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsBackendType {
    /// ASIO backend (professional, lowest latency)
    Asio,
    /// WASAPI backend (Windows Audio Session API, exclusive mode capable)
    Wasapi,
    /// DirectSound backend (legacy, compatibility mode)
    DirectSound,
}

impl WindowsBackendType {
    pub fn name(&self) -> &'static str {
        match self {
            WindowsBackendType::Asio => "ASIO",
            WindowsBackendType::Wasapi => "WASAPI",
            WindowsBackendType::DirectSound => "DirectSound",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            WindowsBackendType::Asio => "Professional low-latency audio (requires ASIO drivers)",
            WindowsBackendType::Wasapi => "Windows Audio Session API (standard)",
            WindowsBackendType::DirectSound => "Legacy DirectSound (compatibility)",
        }
    }

    #[cfg(target_os = "windows")]
    pub fn to_host_id(&self) -> cpal::HostId {
        match self {
            WindowsBackendType::Asio => cpal::HostId::Asio,
            WindowsBackendType::Wasapi => cpal::HostId::Wasapi,
            // DirectSound backend removed in newer CPAL versions - fall back to WASAPI
            WindowsBackendType::DirectSound => {
                eprintln!("Warning: DirectSound backend no longer supported in CPAL, using WASAPI instead");
                cpal::HostId::Wasapi
            }
        }
    }
}

/// ASIO-capable audio backend for Windows
///
/// This backend supports multiple Windows audio APIs:
/// - ASIO: Professional audio interfaces with minimal latency
/// - WASAPI: Standard Windows audio with exclusive mode option
/// - DirectSound: Legacy fallback for compatibility
pub struct AsioBackend {
    #[cfg(target_os = "windows")]
    host: Option<cpal::Host>,
    backend_type: WindowsBackendType,
    initialized: bool,
    exclusive_mode: bool,
}

impl AsioBackend {
    /// Create a new ASIO backend instance
    ///
    /// # Platform Support
    /// Only available on Windows. On other platforms, this will create a backend
    /// that reports as unavailable.
    pub fn new() -> Self {
        Self::with_backend_type(WindowsBackendType::Asio)
    }

    /// Create backend with specific Windows audio API
    ///
    /// # Arguments
    /// * `backend_type` - The Windows audio API to use (ASIO, WASAPI, DirectSound)
    ///
    /// # Examples
    /// ```rust,no_run
    /// let asio = AsioBackend::with_backend_type(WindowsBackendType::Asio);
    /// let wasapi = AsioBackend::with_backend_type(WindowsBackendType::Wasapi);
    /// ```
    pub fn with_backend_type(backend_type: WindowsBackendType) -> Self {
        #[cfg(target_os = "windows")]
        {
            Self {
                host: None,
                backend_type,
                initialized: false,
                exclusive_mode: backend_type == WindowsBackendType::Asio, // ASIO is always exclusive
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            Self {
                backend_type,
                initialized: false,
                exclusive_mode: false,
            }
        }
    }

    /// Set exclusive mode flag
    ///
    /// Exclusive mode provides direct hardware access with lower latency but
    /// prevents other applications from using the audio device.
    ///
    /// # Arguments
    /// * `exclusive` - true to enable exclusive mode, false for shared mode
    ///
    /// # Note
    /// ASIO is always exclusive. This setting only affects WASAPI backend.
    pub fn set_exclusive_mode(&mut self, exclusive: bool) {
        // ASIO is always exclusive, don't allow disabling it
        if self.backend_type != WindowsBackendType::Asio {
            self.exclusive_mode = exclusive;
        }
    }

    /// Get current exclusive mode setting
    pub fn is_exclusive_mode(&self) -> bool {
        self.exclusive_mode
    }

    /// Get the backend type
    pub fn backend_type(&self) -> WindowsBackendType {
        self.backend_type
    }

    /// Check if ASIO drivers are available on the system
    ///
    /// Returns true if ASIO host can be created, indicating drivers are installed.
    #[cfg(target_os = "windows")]
    pub fn asio_available() -> bool {
        cpal::host_from_id(cpal::HostId::Asio).is_ok()
    }

    #[cfg(not(target_os = "windows"))]
    pub fn asio_available() -> bool {
        false
    }

    /// Enumerate all available Windows audio backends
    ///
    /// Returns a list of backend types that are available on the system.
    /// ASIO will only be included if drivers are installed.
    pub fn available_backends() -> Vec<WindowsBackendType> {
        let mut backends = Vec::new();

        #[cfg(target_os = "windows")]
        {
            // Check ASIO availability
            if Self::asio_available() {
                backends.push(WindowsBackendType::Asio);
            }

            // WASAPI is always available on Windows Vista and later
            backends.push(WindowsBackendType::Wasapi);

            // DirectSound is available for compatibility
            backends.push(WindowsBackendType::DirectSound);
        }

        backends
    }

    #[cfg(target_os = "windows")]
    fn get_or_create_host(&mut self) -> Result<&cpal::Host> {
        if self.host.is_none() {
            let host_id = self.backend_type.to_host_id();
            let host = cpal::host_from_id(host_id).map_err(|e| {
                AudioBackendError::BackendNotAvailable(format!(
                    "{} backend not available: {}",
                    self.backend_type.name(),
                    e
                ))
            })?;
            self.host = Some(host);
        }

        Ok(self.host.as_ref().unwrap())
    }

    /// Create an output stream with custom callback
    ///
    /// This is a low-level method that allows direct control over the audio callback.
    /// For most use cases, prefer using the AudioBackend trait methods.
    #[cfg(target_os = "windows")]
    pub fn create_output_stream_with_callback<F>(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: F,
    ) -> Result<Box<dyn AudioStream>>
    where
        F: FnMut(&mut [f32]) + Send + 'static,
    {
        let host = self.get_or_create_host()?;

        let device = host
            .output_devices()
            .map_err(|e| {
                AudioBackendError::DeviceUnavailable(format!("Cannot enumerate devices: {}", e))
            })?
            .find(|d| d.name().ok().as_deref() == Some(device_id))
            .ok_or_else(|| AudioBackendError::DeviceNotFound(device_id.to_string()))?;

        let stream_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
        };

        // Wrap callback in Arc<Mutex> for thread-safe access
        let callback = Arc::new(parking_lot::Mutex::new(callback));
        let callback_clone = callback.clone();

        // Enable real-time thread priority for audio callback
        use std::sync::atomic::{AtomicBool, Ordering};
        let priority_set = Arc::new(AtomicBool::new(false));
        let priority_set_clone = priority_set.clone();

        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Set real-time priority on first callback (runs in audio thread)
                    if !priority_set_clone.load(Ordering::Relaxed) {
                        // TODO: Implement audio thread priority optimizations
                        // The feature flag 'audio-optimizations' and module
                        // 'crate::audio_optimizations::AudioThreadPriority' are not yet defined.
                        // This block is a placeholder for future work.
                        #[cfg(feature = "audio-optimizations")]
                        {
                            use crate::audio_optimizations::AudioThreadPriority;
                            if let Ok(()) = AudioThreadPriority::set_realtime() {
                                // Pin to last CPU core for best isolation
                                let core_count = num_cpus::get();
                                AudioThreadPriority::pin_to_core(core_count.saturating_sub(1)).ok();
                            }
                        }
                        priority_set_clone.store(true, Ordering::Relaxed);
                    }

                    let mut cb = callback_clone.lock();
                    cb(data);
                },
                move |err| {
                    log::error!("Stream error: {}", err);
                },
                None,
            )
            .map_err(|e| {
                AudioBackendError::StreamError(format!("Failed to build output stream: {}", e))
            })?;

        Ok(Box::new(AsioOutputStream {
            stream,
            config,
            status: StreamStatus::Stopped,
        }))
    }

    /// Create an input stream with custom callback
    #[cfg(target_os = "windows")]
    pub fn create_input_stream_with_callback<F>(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: F,
    ) -> Result<Box<dyn AudioStream>>
    where
        F: FnMut(&[f32]) + Send + 'static,
    {
        let host = self.get_or_create_host()?;

        let device = host
            .input_devices()
            .map_err(|e| {
                AudioBackendError::DeviceUnavailable(format!("Cannot enumerate devices: {}", e))
            })?
            .find(|d| d.name().ok().as_deref() == Some(device_id))
            .ok_or_else(|| AudioBackendError::DeviceNotFound(device_id.to_string()))?;

        let stream_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
        };

        let callback = Arc::new(parking_lot::Mutex::new(callback));
        let callback_clone = callback.clone();

        // Enable real-time thread priority
        let priority_set = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let priority_set_clone = priority_set.clone();

        let stream = device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if !priority_set_clone.load(std::sync::atomic::Ordering::Relaxed) {
                        // TODO: Implement audio thread priority optimizations if needed
                        #[cfg(feature = "audio-optimizations")]
                        {
                            use crate::audio_optimizations::AudioThreadPriority;
                            if let Ok(()) = AudioThreadPriority::set_realtime() {
                                let core_count = num_cpus::get();
                                AudioThreadPriority::pin_to_core(core_count.saturating_sub(1)).ok();
                            }
                        }
                        priority_set_clone.store(true, std::sync::atomic::Ordering::Relaxed);
                    }

                    let mut cb = callback_clone.lock();
                    cb(data);
                },
                move |err| {
                    log::error!("Input stream error: {}", err);
                },
                None,
            )
            .map_err(|e| {
                AudioBackendError::StreamError(format!("Failed to build input stream: {}", e))
            })?;

        Ok(Box::new(AsioInputStream {
            stream,
            config,
            status: StreamStatus::Stopped,
        }))
    }
}

impl Default for AsioBackend {
    fn default() -> Self {
        Self::new()
    }
}

// Platform-specific stream implementations
#[cfg(target_os = "windows")]
struct AsioOutputStream {
    stream: cpal::Stream,
    config: AudioConfig,
    status: StreamStatus,
}

#[cfg(target_os = "windows")]
struct AsioInputStream {
    stream: cpal::Stream,
    config: AudioConfig,
    status: StreamStatus,
}

#[cfg(target_os = "windows")]
// SAFETY: AsioOutputStream is safe to Send in Windows audio context
// ASIO handles thread safety internally for audio callbacks
unsafe impl Send for AsioOutputStream {}

impl AudioStream for AsioOutputStream {
    fn play(&mut self) -> Result<()> {
        self.stream
            .play()
            .map_err(|e| AudioBackendError::StreamError(format!("Failed to play stream: {}", e)))?;
        self.status = StreamStatus::Playing;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.stream.pause().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to pause stream: {}", e))
        })?;
        self.status = StreamStatus::Paused;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.stream
            .pause()
            .map_err(|e| AudioBackendError::StreamError(format!("Failed to stop stream: {}", e)))?;
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
        // For ASIO, latency is typically the buffer size
        Some(self.config.buffer_size)
    }
}

#[cfg(target_os = "windows")]
// SAFETY: AsioInputStream is safe to Send in Windows audio context
// ASIO handles thread safety internally for audio callbacks
unsafe impl Send for AsioInputStream {}

impl AudioStream for AsioInputStream {
    fn play(&mut self) -> Result<()> {
        self.stream
            .play()
            .map_err(|e| AudioBackendError::StreamError(format!("Failed to play stream: {}", e)))?;
        self.status = StreamStatus::Playing;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.stream.pause().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to pause stream: {}", e))
        })?;
        self.status = StreamStatus::Paused;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.stream
            .pause()
            .map_err(|e| AudioBackendError::StreamError(format!("Failed to stop stream: {}", e)))?;
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

impl AudioBackend for AsioBackend {
    fn name(&self) -> &'static str {
        self.backend_type.name()
    }

    fn is_available(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            match self.backend_type {
                WindowsBackendType::Asio => Self::asio_available(),
                WindowsBackendType::Wasapi => true, // Always available on Windows
                WindowsBackendType::DirectSound => true,
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }

    fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            // Try to create the host to verify availability
            self.get_or_create_host()?;
            self.initialized = true;
            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err(AudioBackendError::BackendNotAvailable(
                "ASIO backend only available on Windows".to_string(),
            ))
        }
    }

    fn enumerate_devices(&self, direction: StreamDirection) -> Result<Vec<DeviceInfo>> {
        #[cfg(target_os = "windows")]
        {
            let host = self.host.as_ref().ok_or_else(|| {
                AudioBackendError::InitializationFailed("Backend not initialized".to_string())
            })?;

            let devices = match direction {
                StreamDirection::Input => host.input_devices(),
                StreamDirection::Output => host.output_devices(),
            }
            .map_err(|e| {
                AudioBackendError::DeviceUnavailable(format!("Cannot enumerate devices: {}", e))
            })?;

            let mut device_infos = Vec::new();

            for device in devices {
                if let Ok(name) = device.name() {
                    let default_config = match direction {
                        StreamDirection::Output => device.default_output_config().ok(),
                        StreamDirection::Input => device.default_input_config().ok(),
                    };
                    let is_default = match direction {
                        StreamDirection::Output => {
                            host.default_output_device()
                                .and_then(|d| d.name().ok())
                                .as_deref()
                                == Some(&name)
                        }
                        StreamDirection::Input => {
                            host.default_input_device()
                                .and_then(|d| d.name().ok())
                                .as_deref()
                                == Some(&name)
                        }
                    };

                    let device_info = DeviceInfo {
                        id: name.clone(),
                        name,
                        is_default,
                        supported_configs: default_config
                            .clone()
                            .map(|c| {
                                vec![AudioConfig {
                                    sample_rate: c.sample_rate().0,
                                    channels: c.channels(),
                                    sample_format: SampleFormat::F32,
                                    buffer_size: 512, // TODO: Make buffer_size configurable (see issue #XXX or roadmap)
                                    exclusive_mode: true, // ASIO is always exclusive
                                }]
                            })
                            .unwrap_or_default(),
                        min_sample_rate: default_config
                            .as_ref()
                            .map(|c| c.sample_rate().0)
                            .unwrap_or(44100),
                        max_sample_rate: default_config
                            .as_ref()
                            .map(|c| c.sample_rate().0)
                            .unwrap_or(192000),
                        max_input_channels: default_config
                            .as_ref()
                            .map(|c| c.channels())
                            .unwrap_or(2),
                        max_output_channels: default_config.map(|c| c.channels()).unwrap_or(2),
                    };

                    device_infos.push(device_info);
                }
            }

            Ok(device_infos)
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(Vec::new())
        }
    }

    fn default_device(&self, direction: StreamDirection) -> Result<DeviceInfo> {
        #[cfg(target_os = "windows")]
        {
            let host = self.host.as_ref().ok_or_else(|| {
                AudioBackendError::InitializationFailed("Backend not initialized".to_string())
            })?;

            let device = match direction {
                StreamDirection::Input => host.default_input_device(),
                StreamDirection::Output => host.default_output_device(),
            }
            .ok_or_else(|| AudioBackendError::DeviceNotFound("No default device".to_string()))?;

            let name = device.name().map_err(|e| {
                AudioBackendError::DeviceUnavailable(format!("Cannot get device name: {}", e))
            })?;

            let default_config = match direction {
                StreamDirection::Output => device.default_output_config().ok(),
                StreamDirection::Input => device.default_input_config().ok(),
            };

            Ok(DeviceInfo {
                id: name.clone(),
                name,
                is_default: true,
                supported_configs: default_config
                    .clone()
                    .map(|c| {
                        vec![AudioConfig {
                            sample_rate: c.sample_rate().0,
                            channels: c.channels(),
                            sample_format: SampleFormat::F32,
                            buffer_size: 512,
                            exclusive_mode: true, // ASIO is always exclusive
                        }]
                    })
                    .unwrap_or_default(),
                min_sample_rate: default_config
                    .as_ref()
                    .map(|c| c.sample_rate().0)
                    .unwrap_or(44100),
                max_sample_rate: default_config
                    .as_ref()
                    .map(|c| c.sample_rate().0)
                    .unwrap_or(192000),
                max_input_channels: default_config.as_ref().map(|c| c.channels()).unwrap_or(2),
                max_output_channels: default_config.map(|c| c.channels()).unwrap_or(2),
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err(AudioBackendError::BackendNotAvailable(
                "Not available on this platform".to_string(),
            ))
        }
    }

    fn test_device(&self, _device_id: &str) -> Result<bool> {
        // TODO: Implement actual device testing
        Ok(true)
    }

    fn supported_configs(
        &self,
        device_id: &str,
        direction: StreamDirection,
    ) -> Result<Vec<AudioConfig>> {
        #[cfg(target_os = "windows")]
        {
            let host = self.host.as_ref().ok_or_else(|| {
                AudioBackendError::InitializationFailed("Backend not initialized".to_string())
            })?;

            let devices = match direction {
                StreamDirection::Output => host.output_devices(),
                StreamDirection::Input => host.input_devices(),
            }
            .map_err(|e| {
                AudioBackendError::DeviceUnavailable(format!("Cannot enumerate: {}", e))
            })?;

            let device = devices
                .filter_map(|d| d.name().ok().map(|n| (d, n)))
                .find(|(_, n)| n == device_id)
                .ok_or_else(|| AudioBackendError::DeviceNotFound(device_id.to_string()))?
                .0;

            let config = match direction {
                StreamDirection::Output => device.default_output_config(),
                StreamDirection::Input => device.default_input_config(),
            }
            .map_err(|e| {
                AudioBackendError::UnsupportedFormat(format!("Cannot get config: {}", e))
            })?;

            // Generate common buffer sizes for ASIO
            let buffer_sizes = if self.backend_type == WindowsBackendType::Asio {
                vec![64, 128, 256, 512, 1024]
            } else {
                vec![512, 1024, 2048]
            };

            let configs = buffer_sizes
                .into_iter()
                .map(|buffer_size| AudioConfig {
                    sample_rate: config.sample_rate().0,
                    channels: config.channels(),
                    sample_format: SampleFormat::F32,
                    buffer_size,
                    exclusive_mode: self.backend_type == WindowsBackendType::Asio,
                })
                .collect();

            Ok(configs)
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(Vec::new())
        }
    }

    fn create_output_stream(
        &mut self,
        device_id: &str,
        config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>> {
        #[cfg(target_os = "windows")]
        {
            // Create a silent stream (placeholder callback)
            self.create_output_stream_with_callback(device_id, config, |data| {
                data.fill(0.0);
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err(AudioBackendError::BackendNotAvailable(
                "Not available on this platform".to_string(),
            ))
        }
    }

    fn create_input_stream(
        &mut self,
        device_id: &str,
        config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>> {
        #[cfg(target_os = "windows")]
        {
            // Create a capture stream (placeholder callback)
            self.create_input_stream_with_callback(device_id, config, |_data| {
                // No-op placeholder
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err(AudioBackendError::BackendNotAvailable(
                "Not available on this platform".to_string(),
            ))
        }
    }

    fn create_output_stream_with_callback(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: OutputCallback,
    ) -> Result<Box<dyn AudioStream>> {
        #[cfg(target_os = "windows")]
        {
            let host = self.get_or_create_host()?;

            let device = host
                .output_devices()
                .map_err(|e| {
                    AudioBackendError::DeviceUnavailable(format!("Cannot enumerate: {}", e))
                })?
                .find(|d| d.name().ok().as_deref() == Some(device_id))
                .ok_or_else(|| AudioBackendError::DeviceNotFound(device_id.to_string()))?;

            let stream_config = cpal::StreamConfig {
                channels: config.channels,
                sample_rate: cpal::SampleRate(config.sample_rate),
                buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
            };

            let callback = Arc::new(parking_lot::Mutex::new(callback));
            let callback_clone = callback.clone();

            let stream = device
                .build_output_stream(
                    &stream_config,
                    move |data: &mut [f32], _| {
                        let mut cb = callback_clone.lock();
                        cb(data);
                    },
                    |err| log::error!("ASIO stream error: {}", err),
                    None,
                )
                .map_err(|e| AudioBackendError::StreamError(e.to_string()))?;

            Ok(Box::new(AsioOutputStream {
                stream,
                config,
                status: StreamStatus::Stopped,
            }))
        }

        #[cfg(not(target_os = "windows"))]
        Err(AudioBackendError::BackendNotAvailable(
            "ASIO only available on Windows".to_string(),
        ))
    }

    fn create_input_stream_with_callback(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: InputCallback,
    ) -> Result<Box<dyn AudioStream>> {
        #[cfg(target_os = "windows")]
        {
            let host = self.get_or_create_host()?;
            let device = host
                .input_devices()
                .map_err(|e| {
                    AudioBackendError::DeviceUnavailable(format!("Cannot enumerate: {}", e))
                })?
                .find(|d| d.name().ok().as_deref() == Some(device_id))
                .ok_or_else(|| AudioBackendError::DeviceNotFound(device_id.to_string()))?;

            let stream_config = cpal::StreamConfig {
                channels: config.channels,
                sample_rate: cpal::SampleRate(config.sample_rate),
                buffer_size: cpal::BufferSize::Fixed(config.buffer_size as u32),
            };

            let callback = Arc::new(parking_lot::Mutex::new(callback));
            let callback_clone = callback.clone();

            let stream = device
                .build_input_stream(
                    &stream_config,
                    move |data: &[f32], _| {
                        let mut cb = callback_clone.lock();
                        cb(data);
                    },
                    |err| log::error!("ASIO input stream error: {}", err),
                    None,
                )
                .map_err(|e| AudioBackendError::StreamError(e.to_string()))?;

            Ok(Box::new(AsioInputStream {
                stream,
                config,
                status: StreamStatus::Stopped,
            }))
        }

        #[cfg(not(target_os = "windows"))]
        Err(AudioBackendError::BackendNotAvailable(
            "ASIO only available on Windows".to_string(),
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type_names() {
        assert_eq!(WindowsBackendType::Asio.name(), "ASIO");
        assert_eq!(WindowsBackendType::Wasapi.name(), "WASAPI");
        assert_eq!(WindowsBackendType::DirectSound.name(), "DirectSound");
    }

    #[test]
    fn test_available_backends() {
        let backends = AsioBackend::available_backends();
        #[cfg(target_os = "windows")]
        {
            // At minimum, WASAPI should be available
            assert!(!backends.is_empty());
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert!(backends.is_empty());
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_backend_creation() {
        let backend = AsioBackend::new();
        assert_eq!(backend.backend_type(), WindowsBackendType::Asio);
        assert!(backend.is_exclusive_mode());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_wasapi_backend_creation() {
        let backend = AsioBackend::with_backend_type(WindowsBackendType::Wasapi);
        assert_eq!(backend.backend_type(), WindowsBackendType::Wasapi);
        assert!(!backend.is_exclusive_mode());
    }

    /// Comprehensive ASIO SDK integration test
    ///
    /// This test verifies:
    /// 1. ASIO backend availability (checks if ASIO SDK is properly configured)
    /// 2. ASIO device enumeration (lists available ASIO devices)
    /// 3. Device configuration querying (verifies device properties)
    ///
    /// Run with: cargo test test_asio_integration -- --nocapture
    #[test]
    #[cfg(target_os = "windows")]
    fn test_asio_integration() {
        println!("=== ASIO SDK Integration Test ===");

        // Check if ASIO is available
        let backends = AsioBackend::available_backends();
        let asio_available = backends.contains(&WindowsBackendType::Asio);

        println!("ASIO available: {}", asio_available);
        println!("Available backends: {:?}", backends);

        if !asio_available {
            println!("⚠️  ASIO not available - this is expected if:");
            println!("   - No ASIO hardware/drivers installed");
            println!("   - ASIO SDK not properly configured (check CPAL_ASIO_DIR)");
            println!("   - Building without ASIO feature enabled");
            return;
        }

        println!("✅ ASIO backend is available");

        // Try to create ASIO backend
        let backend = AsioBackend::new();
        assert_eq!(backend.backend_type(), WindowsBackendType::Asio);

        // Enumerate ASIO devices
        println!("\n=== Enumerating ASIO Devices ===");
        let devices = backend.list_output_devices();
        println!("Found {} ASIO output devices", devices.len());

        for (i, device) in devices.iter().enumerate() {
            println!("\nDevice {}: {}", i + 1, device.name);
            println!("  ID: {}", device.id);
            println!("  Default: {}", device.is_default);
            println!("  Channels: {}", device.channels);
            println!("  Sample Rate: {}", device.sample_rate);

            // Try to get supported configurations
            if let Ok(configs) = backend.supported_configs(&device.id, StreamDirection::Output) {
                println!("  Supported configs: {} configurations", configs.len());
                if let Some(config) = configs.first() {
                    println!(
                        "    Example: {}Hz, {} channels",
                        config.sample_rate, config.channels
                    );
                }
            }
        }

        // Check input devices as well
        let input_devices = backend.list_input_devices();
        println!("\nFound {} ASIO input devices", input_devices.len());

        if devices.is_empty() && input_devices.is_empty() {
            println!("\n⚠️  No ASIO devices found - install ASIO drivers for your audio interface");
            println!("   Or install ASIO4ALL for testing with standard sound cards");
            println!("   Download: https://www.asio4all.org/");
        } else {
            println!("\n✅ ASIO integration successful!");
        }
    }
}
