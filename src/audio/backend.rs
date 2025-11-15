//! Audio backend abstraction layer
//!
//! This module provides a trait-based abstraction over different audio backends,
//! allowing the application to work with multiple audio systems (cpal, web-audio-api, etc.)

use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during audio backend operations
#[derive(Error, Debug)]
pub enum AudioBackendError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Device unavailable: {0}")]
    DeviceUnavailable(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Backend not available: {0}")]
    BackendNotAvailable(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, AudioBackendError>;

/// Represents an audio sample format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    I16,
    I32,
    F32,
}

/// Audio configuration for streams
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_format: SampleFormat,
    pub buffer_size: usize,
    /// Exclusive mode flag (Windows WASAPI, macOS CoreAudio, Linux ALSA)
    /// - true: Exclusive mode for lowest latency, blocks other applications
    /// - false: Shared mode, allows other applications to use audio device
    /// Note: ASIO is always exclusive
    pub exclusive_mode: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            sample_format: SampleFormat::F32,
            buffer_size: 512,
            exclusive_mode: false, // Default to shared mode for compatibility
        }
    }
}

impl AudioConfig {
    /// Create a low-latency configuration for professional audio work
    /// - 48kHz sample rate
    /// - Stereo (2 channels)
    /// - 128 sample buffer (≈2.7ms latency at 48kHz)
    /// - Exclusive mode enabled
    pub fn low_latency() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            sample_format: SampleFormat::F32,
            buffer_size: 128,
            exclusive_mode: true,
        }
    }

    /// Create an ultra-low-latency configuration for ASIO interfaces
    /// - 48kHz sample rate
    /// - Stereo (2 channels)
    /// - 64 sample buffer (≈1.3ms latency at 48kHz)
    /// - Exclusive mode enabled
    pub fn ultra_low_latency() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            sample_format: SampleFormat::F32,
            buffer_size: 64,
            exclusive_mode: true,
        }
    }

    /// Calculate theoretical latency in milliseconds
    pub fn latency_ms(&self) -> f32 {
        (self.buffer_size as f32 / self.sample_rate as f32) * 1000.0
    }

    /// Check if configuration is suitable for real-time audio
    /// (latency < 10ms)
    pub fn is_realtime(&self) -> bool {
        self.latency_ms() < 10.0
    }
}

/// Information about an audio device
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub supported_configs: Vec<AudioConfig>,
    pub min_sample_rate: u32,
    pub max_sample_rate: u32,
    pub max_input_channels: u16,
    pub max_output_channels: u16,
}

/// Direction of audio flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamDirection {
    Input,
    Output,
}

/// Status of an audio stream
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamStatus {
    Playing,
    Paused,
    Stopped,
    Error,
}

/// The audio backend trait that all backends must implement
pub trait AudioBackend: Send + Sync {
    /// Get the backend name (e.g., "cpal", "web-audio-api")
    fn name(&self) -> &'static str;

    /// Check if this backend is available on the current platform
    fn is_available(&self) -> bool;

    /// Initialize the backend
    fn initialize(&mut self) -> Result<()>;

    /// Enumerate available audio devices
    fn enumerate_devices(&self, direction: StreamDirection) -> Result<Vec<DeviceInfo>>;

    /// Get the default device for the specified direction
    fn default_device(&self, direction: StreamDirection) -> Result<DeviceInfo>;

    /// Test if a device is available and functional
    fn test_device(&self, device_id: &str) -> Result<bool>;

    /// Get supported configurations for a device
    fn supported_configs(&self, device_id: &str, direction: StreamDirection) -> Result<Vec<AudioConfig>>;

    /// Create an output stream with the specified device and config
    fn create_output_stream(
        &mut self,
        device_id: &str,
        config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>>;

    /// Create an input stream with the specified device and config
    fn create_input_stream(
        &mut self,
        device_id: &str,
        config: AudioConfig,
    ) -> Result<Box<dyn AudioStream>>;

    /// Create an output stream with custom callback
    ///
    /// The callback receives a mutable buffer to fill with audio samples
    fn create_output_stream_with_callback<F>(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: F,
    ) -> Result<Box<dyn AudioStream>>
    where
        F: FnMut(&mut [f32]) + Send + 'static;

    /// Create an input stream with custom callback
    ///
    /// The callback receives audio samples from the input device
    fn create_input_stream_with_callback<F>(
        &mut self,
        device_id: &str,
        config: AudioConfig,
        callback: F,
    ) -> Result<Box<dyn AudioStream>>
    where
        F: FnMut(&[f32]) + Send + 'static;
}

/// Trait for audio streams (playback or recording)
pub trait AudioStream: Send {
    /// Start the stream
    fn play(&mut self) -> Result<()>;

    /// Pause the stream
    fn pause(&mut self) -> Result<()>;

    /// Stop the stream
    fn stop(&mut self) -> Result<()>;

    /// Get current stream status
    fn status(&self) -> StreamStatus;

    /// Get the stream configuration
    fn config(&self) -> &AudioConfig;

    /// Get current latency in samples
    fn latency_samples(&self) -> Option<usize>;

    /// Get current latency in milliseconds
    fn latency_ms(&self) -> Option<f32> {
        self.latency_samples().map(|samples| {
            let config = self.config();
            (samples as f32 / config.sample_rate as f32) * 1000.0
        })
    }
}

/// Audio buffer for processing
#[derive(Clone)]
pub struct AudioBuffer {
    pub channels: Vec<Vec<f32>>,
    pub sample_rate: u32,
}

impl AudioBuffer {
    pub fn new(num_channels: usize, num_samples: usize, sample_rate: u32) -> Self {
        Self {
            channels: vec![vec![0.0; num_samples]; num_channels],
            sample_rate,
        }
    }

    pub fn num_channels(&self) -> usize {
        self.channels.len()
    }

    pub fn num_samples(&self) -> usize {
        self.channels.first().map(|c| c.len()).unwrap_or(0)
    }

    pub fn duration_ms(&self) -> f32 {
        (self.num_samples() as f32 / self.sample_rate as f32) * 1000.0
    }
}
