//! Audio device management
//!
//! This module handles device enumeration, selection, and configuration
//! using the cpal library for native audio device access.

use super::backend::{
    AudioBackend, AudioBackendError, AudioConfig, AudioStream, DeviceInfo, Result, SampleFormat,
    StreamDirection, StreamStatus,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::RwLock;
use std::sync::Arc;

/// CPAL-based audio backend implementation
pub struct CpalBackend {
    host: cpal::Host,
    initialized: bool,
}

impl CpalBackend {
    /// Create a new CPAL backend
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
            initialized: false,
        }
    }

    /// Create output stream with custom ring buffer callback
    pub fn create_output_stream_with_callback<F>(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: F,
    ) -> Result<Box<dyn AudioStream>>
    where
        F: FnMut(&mut [f32]) + Send + 'static,
    {
        let device = self
            .host
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
                    eprintln!("Stream error: {}", err);
                },
                None,
            )
            .map_err(|e| {
                AudioBackendError::StreamError(format!("Failed to build output stream: {}", e))
            })?;

        Ok(Box::new(CpalOutputStream {
            stream,
            config,
            status: StreamStatus::Stopped,
        }))
    }

    /// Create input stream with custom callback for recording
    pub fn create_input_stream_with_callback<F>(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: F,
    ) -> Result<Box<dyn AudioStream>>
    where
        F: FnMut(&[f32]) + Send + 'static,
    {
        let device = self
            .host
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

        // Wrap callback in Arc<Mutex> for thread-safe access
        let callback = Arc::new(parking_lot::Mutex::new(callback));
        let callback_clone = callback.clone();

        let stream = device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut cb = callback_clone.lock();
                    cb(data);
                },
                move |err| {
                    eprintln!("Input stream error: {}", err);
                },
                None,
            )
            .map_err(|e| {
                AudioBackendError::StreamError(format!("Failed to build input stream: {}", e))
            })?;

        Ok(Box::new(CpalInputStream {
            stream,
            config,
            status: StreamStatus::Stopped,
        }))
    }

    /// Convert cpal sample format to our SampleFormat
    fn convert_sample_format(format: cpal::SampleFormat) -> SampleFormat {
        match format {
            cpal::SampleFormat::I16 => SampleFormat::I16,
            cpal::SampleFormat::I32 => SampleFormat::I32,
            cpal::SampleFormat::F32 => SampleFormat::F32,
            _ => SampleFormat::F32, // Default to F32 for unsupported formats
        }
    }

    /// Convert our SampleFormat to cpal sample format
    fn to_cpal_sample_format(format: SampleFormat) -> cpal::SampleFormat {
        match format {
            SampleFormat::I16 => cpal::SampleFormat::I16,
            SampleFormat::I32 => cpal::SampleFormat::I32,
            SampleFormat::F32 => cpal::SampleFormat::F32,
        }
    }

    /// Build DeviceInfo from a cpal device
    fn build_device_info(&self, device: &cpal::Device, is_default: bool) -> Result<DeviceInfo> {
        let name = device.name().map_err(|e| {
            AudioBackendError::DeviceUnavailable(format!("Cannot get device name: {}", e))
        })?;

        // Get supported configurations
        let supported_configs: Vec<_> = device
            .supported_output_configs()
            .map_err(|e| {
                AudioBackendError::DeviceUnavailable(format!("Cannot query configs: {}", e))
            })?
            .collect();

        let mut configs = Vec::new();
        let mut min_rate = u32::MAX;
        let mut max_rate = 0u32;
        let mut max_channels = 0u16;

        for config_range in &supported_configs {
            min_rate = min_rate.min(config_range.min_sample_rate().0);
            max_rate = max_rate.max(config_range.max_sample_rate().0);
            max_channels = max_channels.max(config_range.channels());

            // Add a few common sample rates from this range
            for &rate in &[44100, 48000, 88200, 96000] {
                if rate >= config_range.min_sample_rate().0
                    && rate <= config_range.max_sample_rate().0
                {
                    configs.push(AudioConfig {
                        sample_rate: rate,
                        channels: config_range.channels(),
                        sample_format: Self::convert_sample_format(config_range.sample_format()),
                        buffer_size: 512, // Default buffer size
                    });
                }
            }
        }

        Ok(DeviceInfo {
            id: name.clone(),
            name,
            is_default,
            supported_configs: configs,
            min_sample_rate: min_rate,
            max_sample_rate: max_rate,
            max_input_channels: max_channels,
            max_output_channels: max_channels,
        })
    }
}

impl Default for CpalBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for CpalBackend {
    fn name(&self) -> &'static str {
        "cpal"
    }

    fn is_available(&self) -> bool {
        // CPAL is available on all supported platforms
        true
    }

    fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // Test that we can access the default device
        self.host.default_output_device().ok_or_else(|| {
            AudioBackendError::InitializationFailed(
                "No default output device available".to_string(),
            )
        })?;

        self.initialized = true;
        Ok(())
    }

    fn enumerate_devices(&self, direction: StreamDirection) -> Result<Vec<DeviceInfo>> {
        let devices: Vec<_> = match direction {
            StreamDirection::Output => self.host.output_devices().map_err(|e| {
                AudioBackendError::DeviceUnavailable(format!(
                    "Cannot enumerate output devices: {}",
                    e
                ))
            })?,
            StreamDirection::Input => self.host.input_devices().map_err(|e| {
                AudioBackendError::DeviceUnavailable(format!(
                    "Cannot enumerate input devices: {}",
                    e
                ))
            })?,
        }
        .collect();

        let default_device = match direction {
            StreamDirection::Output => self.host.default_output_device(),
            StreamDirection::Input => self.host.default_input_device(),
        };

        let default_name = default_device.as_ref().and_then(|d| d.name().ok());

        let mut device_infos = Vec::new();
        for device in devices {
            let device_name = device.name().ok();
            let is_default = device_name.as_ref() == default_name.as_ref();

            match self.build_device_info(&device, is_default) {
                Ok(info) => device_infos.push(info),
                Err(e) => {
                    // Log error but continue with other devices
                    eprintln!("Failed to build device info: {}", e);
                }
            }
        }

        Ok(device_infos)
    }

    fn default_device(&self, direction: StreamDirection) -> Result<DeviceInfo> {
        let device = match direction {
            StreamDirection::Output => self.host.default_output_device().ok_or_else(|| {
                AudioBackendError::DeviceNotFound("No default output device".to_string())
            })?,
            StreamDirection::Input => self.host.default_input_device().ok_or_else(|| {
                AudioBackendError::DeviceNotFound("No default input device".to_string())
            })?,
        };

        self.build_device_info(&device, true)
    }

    fn test_device(&self, device_id: &str) -> Result<bool> {
        let devices = match self.host.output_devices() {
            Ok(d) => d,
            Err(_) => return Ok(false),
        };

        for device in devices {
            if let Ok(name) = device.name() {
                if name == device_id {
                    // Try to get supported configs as a basic test
                    return Ok(device.supported_output_configs().is_ok());
                }
            }
        }

        Ok(false)
    }

    fn supported_configs(&self, device_id: &str) -> Result<Vec<AudioConfig>> {
        let devices = self.host.output_devices().map_err(|e| {
            AudioBackendError::DeviceUnavailable(format!("Cannot enumerate devices: {}", e))
        })?;

        for device in devices {
            if let Ok(name) = device.name() {
                if name == device_id {
                    let info = self.build_device_info(&device, false)?;
                    return Ok(info.supported_configs);
                }
            }
        }

        Err(AudioBackendError::DeviceNotFound(device_id.to_string()))
    }

    fn create_output_stream(
        &mut self,
        device_id: &str,
        config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>> {
        let device = self
            .host
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

        // Enable real-time thread priority for audio callback
        use std::sync::atomic::{AtomicBool, Ordering};
        let priority_set = Arc::new(AtomicBool::new(false));
        let priority_set_clone = priority_set.clone();

        // For now, create a silent stream - we'll implement actual playback later
        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Set real-time priority on first callback (runs in audio thread)
                    if !priority_set_clone.load(Ordering::Relaxed) {
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

                    // Fill with silence for now
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                    }
                },
                move |err| {
                    eprintln!("Stream error: {}", err);
                },
                None,
            )
            .map_err(|e| {
                AudioBackendError::StreamError(format!("Failed to build output stream: {}", e))
            })?;

        Ok(Box::new(CpalOutputStream {
            stream,
            config,
            status: StreamStatus::Stopped,
        }))
    }

    fn create_input_stream(
        &mut self,
        device_id: &str,
        config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>> {
        let device = self
            .host
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

        // Create a simple input stream that captures data
        let stream = device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Process input data (placeholder for now)
                    let _sample_count = data.len();
                },
                move |err| {
                    eprintln!("Input stream error: {}", err);
                },
                None,
            )
            .map_err(|e| {
                AudioBackendError::StreamError(format!("Failed to build input stream: {}", e))
            })?;

        Ok(Box::new(CpalInputStream {
            stream,
            config,
            status: StreamStatus::Stopped,
        }))
    }
}

/// CPAL output stream
struct CpalOutputStream {
    stream: cpal::Stream,
    config: AudioConfig,
    status: StreamStatus,
}

// SAFETY: cpal::Stream is thread-safe despite containing raw pointers
// The underlying platform audio APIs properly synchronize access
unsafe impl Send for CpalOutputStream {}

impl AudioStream for CpalOutputStream {
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
        // CPAL doesn't provide direct latency info, estimate from buffer size
        Some(self.config.buffer_size)
    }
}

/// CPAL input stream
struct CpalInputStream {
    stream: cpal::Stream,
    config: AudioConfig,
    status: StreamStatus,
}

// SAFETY: cpal::Stream is thread-safe despite containing raw pointers
// The underlying platform audio APIs properly synchronize access
unsafe impl Send for CpalInputStream {}

impl AudioStream for CpalInputStream {
    fn play(&mut self) -> Result<()> {
        self.stream.play().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to start recording: {}", e))
        })?;
        self.status = StreamStatus::Playing;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.stream.pause().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to pause recording: {}", e))
        })?;
        self.status = StreamStatus::Paused;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.stream.pause().map_err(|e| {
            AudioBackendError::StreamError(format!("Failed to stop recording: {}", e))
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
        Some(self.config.buffer_size)
    }
}
