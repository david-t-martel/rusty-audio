//! Integrated Audio Manager
//!
//! Provides a high-level interface for the UI to interact with the audio routing system.
//! Manages audio sources, destinations, and routing for playback, recording, and signal generation.
//!
//! Supports both native (CPAL/ASIO) and WASM (Web Audio API) backends.

// Common imports (both platforms)
use super::audio::{
    AudioBackend, AudioConfig, AudioDestination, AudioRouter, AudioSource, DestId,
    LevelMeterDestination, Route, RouteId, SignalGeneratorSource, SourceId, StreamDirection,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

// Native-only imports
#[cfg(not(target_arch = "wasm32"))]
use super::audio::{BackendSelector, FileRecorderDestination, OutputDeviceDestination};

// WASM-only imports
#[cfg(target_arch = "wasm32")]
use super::audio::web_audio_backend::WebAudioBackend;
#[cfg(target_arch = "wasm32")]
use super::audio::web_audio_destination::WebAudioDestination;

/// Audio manager errors
#[derive(Debug, thiserror::Error)]
pub enum AudioManagerError {
    #[error("Backend error: {0}")]
    Backend(#[from] super::audio::AudioBackendError),

    #[error("No output device configured")]
    NoOutputDevice,

    #[error("No input device configured")]
    NoInputDevice,

    #[error("Source not found: {0:?}")]
    SourceNotFound(SourceId),

    #[error("Destination not found: {0:?}")]
    DestinationNotFound(DestId),

    #[error("Route not found: {0:?}")]
    RouteNotFound(RouteId),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

pub type Result<T> = std::result::Result<T, AudioManagerError>;

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Managed audio route information
#[derive(Debug, Clone)]
pub struct ManagedRoute {
    pub id: RouteId,
    pub source: SourceId,
    pub destination: DestId,
    pub route_type: RouteType,
}

/// Type of audio route
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteType {
    /// Signal generator to output
    SignalGeneratorPlayback,
    /// Input device to output (monitoring)
    InputMonitoring,
    /// Input device to recorder
    InputRecording,
    /// File playback to output
    FilePlayback,
}

/// Integrated audio manager
///
/// Manages all audio routing, playback, and recording in a unified system.
pub struct IntegratedAudioManager {
    router: AudioRouter,
    backend: Box<dyn AudioBackend>,
    config: AudioConfig,

    // Device destinations
    output_device: Option<DestId>,

    // Active sources
    signal_generator_source: Option<SourceId>,
    input_device_source: Option<SourceId>,
    file_playback_source: Option<SourceId>,

    // Active routes
    active_routes: HashMap<RouteType, RouteId>,

    // Playback state
    playback_state: PlaybackState,
    signal_generator_playing: bool,

    // Configuration
    default_sample_rate: u32,
    default_channels: u16,
}

impl IntegratedAudioManager {
    /// Create a new integrated audio manager (Native version)
    ///
    /// # Arguments
    /// * `buffer_size` - Router buffer size (typically 512 or 1024)
    /// * `config` - Audio configuration (sample rate, channels, etc.)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(buffer_size: usize, config: AudioConfig) -> Result<Self> {
        let router = AudioRouter::new(buffer_size);

        // Create backend using selector
        let selector = BackendSelector::new();
        let backend = selector.create_recommended_backend()?;

        Ok(Self {
            router,
            backend,
            config: config.clone(),
            output_device: None,
            signal_generator_source: None,
            input_device_source: None,
            file_playback_source: None,
            active_routes: HashMap::new(),
            playback_state: PlaybackState::Stopped,
            signal_generator_playing: false,
            default_sample_rate: config.sample_rate,
            default_channels: config.channels,
        })
    }

    /// Create a new integrated audio manager (WASM version)
    ///
    /// # Arguments
    /// * `buffer_size` - Router buffer size (typically 512 or 1024)
    /// * `config` - Audio configuration (sample rate, channels, etc.)
    #[cfg(target_arch = "wasm32")]
    pub fn new(buffer_size: usize, config: AudioConfig) -> Result<Self> {
        let router = AudioRouter::new(buffer_size);

        // Create Web Audio backend
        let mut backend = WebAudioBackend::new();
        backend.initialize()?;

        // Initialize 8-band EQ chain
        backend.create_eq_chain()?;

        // Initialize spectrum analyser (512 FFT size, 0.8 smoothing)
        backend.create_analyser(512, 0.8)?;

        Ok(Self {
            router,
            backend: Box::new(backend),
            config: config.clone(),
            output_device: None,
            signal_generator_source: None,
            input_device_source: None,
            file_playback_source: None,
            active_routes: HashMap::new(),
            playback_state: PlaybackState::Stopped,
            signal_generator_playing: false,
            default_sample_rate: config.sample_rate,
            default_channels: config.channels,
        })
    }

    /// Initialize output device
    pub fn initialize_output_device(&mut self, device_id: Option<&str>) -> Result<()> {
        // Get default device if none specified
        let device_info = if let Some(id) = device_id {
            self.backend
                .enumerate_devices(StreamDirection::Output)?
                .into_iter()
                .find(|d| d.id == id)
                .ok_or_else(|| {
                    AudioManagerError::Backend(super::audio::AudioBackendError::DeviceNotFound(
                        id.to_string(),
                    ))
                })?
        } else {
            self.backend.default_device(StreamDirection::Output)?
        };

        // Create output destination (Native)
        #[cfg(not(target_arch = "wasm32"))]
        {
            let output = OutputDeviceDestination::new(
                &mut *self.backend,
                &device_info.id,
                self.config.clone(),
                0, // auto buffer size
            )?;

            let dest_id = self.router.add_destination(Box::new(output));
            self.output_device = Some(dest_id);
        }

        // Create output destination (WASM)
        #[cfg(target_arch = "wasm32")]
        {
            // Create Web Audio context if not already done
            if let Ok(context) = web_sys::AudioContext::new() {
                let output = WebAudioDestination::new(
                    context,
                    self.config.sample_rate,
                    self.config.channels,
                    0.1, // 100ms buffer duration
                );

                let dest_id = self.router.add_destination(Box::new(output));
                self.output_device = Some(dest_id);
            } else {
                return Err(AudioManagerError::Backend(
                    super::audio::AudioBackendError::InitializationFailed(
                        "Failed to create Web Audio context".to_string(),
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Play a signal generator buffer
    ///
    /// # Arguments
    /// * `samples` - Audio samples to play
    /// * `sample_rate` - Sample rate of the audio
    /// * `looping` - Whether to loop the audio
    pub fn play_signal_generator(
        &mut self,
        samples: Vec<f32>,
        sample_rate: f32,
        looping: bool,
    ) -> Result<()> {
        // Stop any existing signal generator playback
        self.stop_signal_generator()?;

        let output_dest = self
            .output_device
            .ok_or(AudioManagerError::NoOutputDevice)?;

        // Create signal generator source
        let source = SignalGeneratorSource::from_buffer(samples, sample_rate, looping);
        let source_id = self.router.add_source(Box::new(source));
        self.signal_generator_source = Some(source_id);

        // Create route to output
        let route_id = self.router.create_route(source_id, output_dest, 1.0)?;
        self.active_routes
            .insert(RouteType::SignalGeneratorPlayback, route_id);

        self.signal_generator_playing = true;
        Ok(())
    }

    /// Stop signal generator playback
    pub fn stop_signal_generator(&mut self) -> Result<()> {
        if let Some(route_id) = self
            .active_routes
            .remove(&RouteType::SignalGeneratorPlayback)
        {
            self.router.remove_route(route_id);
        }

        if let Some(source_id) = self.signal_generator_source.take() {
            self.router.remove_source(source_id);
        }

        self.signal_generator_playing = false;
        Ok(())
    }

    /// Check if signal generator is playing
    pub fn is_signal_generator_playing(&self) -> bool {
        self.signal_generator_playing
    }

    /// Get playback state
    pub fn playback_state(&self) -> PlaybackState {
        self.playback_state
    }

    /// Get all active routes
    pub fn active_routes(&self) -> Vec<Route> {
        self.router.get_routes()
    }

    /// Process audio (should be called regularly from audio thread or timer)
    pub fn process(&self) -> Result<()> {
        Ok(self.router.process()?)
    }

    /// Get output device ID
    pub fn output_device_id(&self) -> Option<DestId> {
        self.output_device
    }

    /// Set route gain
    pub fn set_route_gain(&self, route_type: RouteType, gain: f32) -> Result<()> {
        let route_id = self
            .active_routes
            .get(&route_type)
            .ok_or(AudioManagerError::RouteNotFound(RouteId(0)))?;

        Ok(self.router.set_route_gain(*route_id, gain)?)
    }

    /// Get available output devices
    pub fn enumerate_output_devices(&mut self) -> Result<Vec<super::audio::DeviceInfo>> {
        Ok(self.backend.enumerate_devices(StreamDirection::Output)?)
    }

    /// Get available input devices
    pub fn enumerate_input_devices(&mut self) -> Result<Vec<super::audio::DeviceInfo>> {
        Ok(self.backend.enumerate_devices(StreamDirection::Input)?)
    }

    /// Set EQ band gain (WASM only)
    ///
    /// # Arguments
    /// * `band` - Band index (0-7)
    /// * `gain_db` - Gain in decibels (±12 dB range)
    ///
    /// Source: Based on WASM_CODE_BORROWING_GUIDE.md Phase 2 recommendations
    #[cfg(target_arch = "wasm32")]
    pub fn set_eq_band(&mut self, band: usize, gain_db: f32) -> Result<()> {
        // Downcast to WebAudioBackend (safe in WASM builds)
        let backend_name = self.backend.name();
        let backend = self
            .backend
            .as_any_mut()
            .downcast_mut::<WebAudioBackend>()
            .ok_or_else(|| {
                AudioManagerError::InvalidConfiguration(format!(
                    "Expected WebAudioBackend, found {}",
                    backend_name
                ))
            })?;

        Ok(backend.set_eq_band(band, gain_db)?)
    }

    /// Get EQ band gain (WASM only)
    ///
    /// # Arguments
    /// * `band` - Band index (0-7)
    #[cfg(target_arch = "wasm32")]
    pub fn get_eq_band(&self, band: usize) -> Result<f32> {
        // Downcast to WebAudioBackend (safe in WASM builds)
        let backend_name = self.backend.name();
        let backend = self
            .backend
            .as_any()
            .downcast_ref::<WebAudioBackend>()
            .ok_or_else(|| {
                AudioManagerError::InvalidConfiguration(format!(
                    "Expected WebAudioBackend, found {}",
                    backend_name
                ))
            })?;

        Ok(backend.get_eq_band(band)?)
    }

    /// Reset all EQ bands to 0 dB (WASM only)
    #[cfg(target_arch = "wasm32")]
    pub fn reset_eq(&mut self) -> Result<()> {
        // Downcast to WebAudioBackend (safe in WASM builds)
        let backend_name = self.backend.name();
        let backend = self
            .backend
            .as_any_mut()
            .downcast_mut::<WebAudioBackend>()
            .ok_or_else(|| {
                AudioManagerError::InvalidConfiguration(format!(
                    "Expected WebAudioBackend, found {}",
                    backend_name
                ))
            })?;

        Ok(backend.reset_eq()?)
    }

    /// Get spectrum frequency data (WASM only)
    ///
    /// Returns normalized frequency bin amplitudes (0.0 to 1.0)
    #[cfg(target_arch = "wasm32")]
    pub fn get_frequency_data(&self) -> Result<Vec<f32>> {
        // Downcast to WebAudioBackend (safe in WASM builds)
        let backend_name = self.backend.name();
        let backend = self
            .backend
            .as_any()
            .downcast_ref::<WebAudioBackend>()
            .ok_or_else(|| {
                AudioManagerError::InvalidConfiguration(format!(
                    "Expected WebAudioBackend, found {}",
                    backend_name
                ))
            })?;

        Ok(backend.get_frequency_data()?)
    }

    /// Get frequency bin count (WASM only)
    #[cfg(target_arch = "wasm32")]
    pub fn frequency_bin_count(&self) -> Option<u32> {
        // Downcast to WebAudioBackend (safe in WASM builds)
        self.backend
            .as_any()
            .downcast_ref::<WebAudioBackend>()
            .and_then(|b| b.frequency_bin_count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_manager_creation() {
        let config = AudioConfig::default();
        let manager = IntegratedAudioManager::new(512, config);

        // May fail if no audio device available, but structure should be valid
        match manager {
            Ok(mgr) => {
                assert_eq!(mgr.playback_state(), PlaybackState::Stopped);
                assert!(!mgr.is_signal_generator_playing());
            }
            Err(_) => {
                // Expected if no audio devices available in test environment
            }
        }
    }
}
