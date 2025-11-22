//! Audio Engine Module
//!
//! This module contains all audio-related functionality separated from the UI.
//! It follows the Single Responsibility Principle by handling only audio operations.

use crate::error::{AudioError, ErrorContext, Result};
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{
    AnalyserNode, AudioNode, AudioScheduledSourceNode, BiquadFilterNode, BiquadFilterType,
};

/// Represents the current state of audio playback
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Audio engine trait for dependency inversion
pub trait AudioEngineInterface {
    /// Get as Any for downcasting (needed for complex systems)
    fn as_any(&self) -> &dyn Any;

    /// Get as Any mutable for downcasting (needed for complex systems)
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// Load an audio file from the given path
    fn load_audio_file(&mut self, path: &str) -> Result<Duration>;

    /// Start or resume playback
    fn play(&mut self) -> Result<()>;

    /// Pause playback
    fn pause(&mut self) -> Result<()>;

    /// Stop playback and reset position
    fn stop(&mut self) -> Result<()>;

    /// Seek to a specific position
    fn seek(&mut self, position: Duration) -> Result<()>;

    /// Set the volume (0.0 to 1.0)
    fn set_volume(&mut self, volume: f32) -> Result<()>;

    /// Get the current volume (0.0 to 1.0)
    fn get_volume(&self) -> f32;

    /// Get current playback position
    fn get_position(&self) -> Duration;

    /// Audition an in-memory buffer (e.g., generated signal) through the playback chain
    fn audition_buffer(
        &mut self,
        samples: &[f32],
        sample_rate: f32,
        channels: usize,
    ) -> Result<Duration>;

    /// Get current playback state
    fn get_state(&self) -> PlaybackState;

    /// Get spectrum data for visualization
    fn get_spectrum(&mut self) -> Vec<f32>;

    /// Set equalizer band gain
    fn set_eq_gain(&mut self, band: usize, gain: f32) -> Result<()>;

    /// Set equalizer band gain (alias for set_eq_gain for backwards compatibility)
    fn set_eq_band(&mut self, band: usize, gain: f32) -> Result<()> {
        self.set_eq_gain(band, gain)
    }

    /// Set whether playback should loop
    fn set_loop(&mut self, looping: bool) -> Result<()>;

    /// Load an audio buffer for playback
    fn load_buffer(
        &mut self,
        samples: Vec<f32>,
        sample_rate: f32,
        channels: usize,
    ) -> Result<Duration>;

    /// Get the audio context
    fn get_context(&self) -> &AudioContext;

    /// Get the total duration of the current audio
    fn get_duration(&self) -> Duration;

    /// Get the waveform data for visualization (downsampled)
    fn get_waveform(&self, samples: usize) -> Option<Vec<f32>>;
}

/// Web Audio API implementation of the audio engine
pub struct WebAudioEngine {
    audio_context: AudioContext,
    source_node: Option<web_audio_api::node::AudioBufferSourceNode>,
    gain_node: web_audio_api::node::GainNode,
    eq_bands: Vec<BiquadFilterNode>,
    analyser: AnalyserNode,
    playback_state: PlaybackState,
    volume: f32,
    playback_pos: Duration,
    total_duration: Duration,
    is_seeking: bool,
    spectrum: Vec<f32>,
    waveform_data: Option<Arc<Vec<f32>>>, // Cached full resolution waveform
}

impl WebAudioEngine {
    fn from_context(audio_context: AudioContext) -> Self {
        let analyser = audio_context.create_analyser();
        let gain_node = audio_context.create_gain();
        gain_node.gain().set_value(0.5);

        let mut eq_bands = Vec::new();
        for i in 0..8 {
            let mut band = audio_context.create_biquad_filter();
            band.set_type(BiquadFilterType::Peaking);
            band.frequency().set_value(60.0 * 2.0_f32.powi(i));
            band.q().set_value(1.0);
            band.gain().set_value(0.0);
            eq_bands.push(band);
        }

        debug!("Created {} EQ bands", eq_bands.len());

        Self {
            audio_context,
            source_node: None,
            gain_node,
            eq_bands,
            analyser,
            playback_state: PlaybackState::Stopped,
            volume: 0.5,
            playback_pos: Duration::ZERO,
            total_duration: Duration::ZERO,
            is_seeking: false,
            spectrum: vec![0.0; 1024],
            waveform_data: None,
        }
    }

    /// Create a new WebAudioEngine instance
    pub fn new() -> Result<Self> {
        info!("Initializing WebAudio engine");
        Ok(Self::from_context(AudioContext::default()))
    }

    /// Connect the audio chain: source -> EQ bands -> gain -> analyser -> output
    fn connect_audio_chain(&self) -> Result<()> {
        if let Some(source_node) = &self.source_node {
            source_node.connect(&self.gain_node);

            let mut previous_node: &dyn AudioNode = &self.gain_node;
            for band in &self.eq_bands {
                previous_node.connect(band);
                previous_node = band;
            }

            previous_node.connect(&self.analyser);
            self.analyser.connect(&self.audio_context.destination());

            debug!("Audio chain connected successfully");
            Ok(())
        } else {
            Err(AudioError::ConnectionFailed.into())
        }
    }

    /// Update spectrum data for visualization
    pub fn update_spectrum(&mut self) {
        let mut frequency_data = vec![0.0; self.analyser.frequency_bin_count()];
        self.analyser.get_float_frequency_data(&mut frequency_data);

        // Convert dB values to linear scale for visualization (0-1 range)
        self.spectrum = frequency_data
            .iter()
            .map(|&db| {
                // Convert from dB to linear scale
                // AnalyserNode typically returns values from -100 dB to 0 dB
                let linear = 10.0_f32.powf(db / 20.0);
                // Normalize to 0-1 range for visualization
                (linear * 100.0).clamp(0.0, 1.0)
            })
            .collect();
    }

    /// Update playback position
    pub fn update_position(&mut self) {
        if self.playback_state == PlaybackState::Playing && !self.is_seeking {
            self.playback_pos = Duration::from_secs_f64(self.audio_context.current_time());
        }
    }
}

impl AudioEngineInterface for WebAudioEngine {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn load_audio_file(&mut self, path: &str) -> Result<Duration> {
        info!("Loading audio file: {}", path);

        let file = std::fs::File::open(path).map_err(|e| AudioError::PlaybackFailed {
            reason: format!("Failed to open file: {}", e),
        })?;

        let buffer = self
            .audio_context
            .decode_audio_data_sync(file)
            .map_err(|_| AudioError::DecodeFailed)?;

        self.total_duration = Duration::from_secs_f64(buffer.duration());

        // Extract waveform data for visualization
        let channels = buffer.number_of_channels();
        let length = buffer.length();
        let mut waveform = vec![0.0; length];
        
        if channels > 0 {
            // Mix down to mono for visualization
            for i in 0..channels {
                let channel_data = buffer.get_channel_data(i);
                for (j, sample) in channel_data.iter().enumerate() {
                    waveform[j] += sample / channels as f32;
                }
            }
        }
        self.waveform_data = Some(Arc::new(waveform));

        let mut source_node = self.audio_context.create_buffer_source();
        source_node.set_buffer(buffer);

        self.source_node = Some(source_node);
        self.connect_audio_chain()?;

        info!(
            "Audio file loaded successfully. Duration: {:?}",
            self.total_duration
        );
        Ok(self.total_duration)
    }

    fn play(&mut self) -> Result<()> {
        match self.playback_state {
            PlaybackState::Playing => {
                debug!("Already playing");
                return Ok(());
            }
            PlaybackState::Paused => {
                info!("Resuming playback");
                self.audio_context.resume_sync();
                self.playback_state = PlaybackState::Playing;
            }
            PlaybackState::Stopped => {
                if let Some(source_node) = &mut self.source_node {
                    info!("Starting playback");
                    source_node.start();
                    self.playback_state = PlaybackState::Playing;
                    self.playback_pos = Duration::ZERO;
                } else {
                    warn!("No audio source loaded");
                    return Err(AudioError::PlaybackFailed {
                        reason: "No audio source loaded".to_string(),
                    }
                    .into());
                }
            }
        }
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        if self.playback_state == PlaybackState::Playing {
            info!("Pausing playback");
            self.audio_context.suspend_sync();
            self.playback_state = PlaybackState::Paused;
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(source_node) = &mut self.source_node {
            info!("Stopping playback");
            source_node.stop();
            self.playback_state = PlaybackState::Stopped;
            self.playback_pos = Duration::ZERO;
        }
        Ok(())
    }

    fn seek(&mut self, position: Duration) -> Result<()> {
        info!("Seeking to position: {:?}", position);
        self.is_seeking = true;
        self.playback_pos = position.min(self.total_duration);

        if let Some(source_node) = &mut self.source_node {
            // Note: This is a simplified seek implementation
            // In a real application, you might need to recreate the source node
            source_node.stop();
            source_node.start_at(self.audio_context.current_time() + position.as_secs_f64());
        }

        self.is_seeking = false;
        Ok(())
    }

    fn set_volume(&mut self, volume: f32) -> Result<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        debug!("Setting volume to: {}", clamped_volume);
        self.gain_node.gain().set_value(clamped_volume);
        self.volume = clamped_volume;
        Ok(())
    }

    fn get_position(&self) -> Duration {
        self.playback_pos
    }

    fn get_state(&self) -> PlaybackState {
        self.playback_state.clone()
    }

    fn get_spectrum(&mut self) -> Vec<f32> {
        self.update_spectrum();
        self.spectrum.clone()
    }

    fn audition_buffer(
        &mut self,
        samples: &[f32],
        sample_rate: f32,
        channels: usize,
    ) -> Result<Duration> {
        if samples.is_empty() {
            return Err(AudioError::InvalidParameters {
                details: "Cannot audition an empty buffer".to_string(),
            }
            .into());
        }

        let channels = channels.max(1);
        let frame_count = samples.len() / channels;
        if frame_count == 0 {
            return Err(AudioError::InvalidParameters {
                details: "Buffer must contain at least one full frame".to_string(),
            }
            .into());
        }

        // Stop any existing playback before replacing the source node
        let _ = self.stop();

        let mut buffer = self
            .audio_context
            .create_buffer(channels, frame_count, sample_rate);

        // Copy interleaved samples into per-channel buffers
        for ch in 0..channels {
            let mut channel_data = vec![0.0f32; frame_count];
            for (frame_idx, sample) in samples.iter().skip(ch).step_by(channels).enumerate() {
                channel_data[frame_idx] = *sample;
            }
            buffer.copy_to_channel(&channel_data, ch);
        }

        let mut source_node = self.audio_context.create_buffer_source();
        source_node.set_buffer(buffer);
        self.source_node = Some(source_node);
        self.connect_audio_chain()?;

        if let Some(node) = &mut self.source_node {
            node.start();
        }

        self.playback_state = PlaybackState::Playing;
        self.playback_pos = Duration::ZERO;
        self.total_duration = Duration::from_secs_f64(frame_count as f64 / sample_rate as f64);
        Ok(self.total_duration)
    }

    fn set_eq_gain(&mut self, band: usize, gain: f32) -> Result<()> {
        if band >= self.eq_bands.len() {
            return Err(AudioError::InvalidParameters {
                details: format!("Invalid EQ band: {}", band),
            }
            .into());
        }

        let clamped_gain = gain.clamp(-40.0, 40.0);
        debug!("Setting EQ band {} gain to: {}", band, clamped_gain);
        self.eq_bands[band].gain().set_value(clamped_gain);
        Ok(())
    }

    fn get_duration(&self) -> Duration {
        self.total_duration
    }

    fn get_volume(&self) -> f32 {
        self.volume
    }

    fn set_loop(&mut self, looping: bool) -> Result<()> {
        debug!("Setting loop mode to: {}", looping);
        if let Some(source_node) = &mut self.source_node {
            source_node.set_loop(looping);
        }
        Ok(())
    }

    fn load_buffer(
        &mut self,
        samples: Vec<f32>,
        sample_rate: f32,
        channels: usize,
    ) -> Result<Duration> {
        // Reuse the audition_buffer logic for loading a buffer
        self.audition_buffer(&samples, sample_rate, channels)
    }

    fn get_context(&self) -> &AudioContext {
        &self.audio_context
    }

    fn get_waveform(&self, target_points: usize) -> Option<Vec<f32>> {
        let data = self.waveform_data.as_ref()?;
        if data.is_empty() || target_points == 0 {
            return Some(Vec::new());
        }

        let step = (data.len() / target_points).max(1);
        let mut waveform = Vec::with_capacity(target_points);

        for chunk in data.chunks(step).take(target_points) {
            // Find max absolute value in chunk (peak detection)
            let peak = chunk.iter().fold(0.0f32, |max, &x| max.max(x.abs()));
            waveform.push(peak);
        }

        // Normalize
        let max_peak = waveform.iter().fold(0.0f32, |max, &x| max.max(x));
        if max_peak > 0.0 {
            for sample in &mut waveform {
                *sample /= max_peak;
            }
        }

        Some(waveform)
    }
}

impl Default for WebAudioEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|err| {
            error!("Failed to create default WebAudioEngine: {}", err);
            Self::from_context(AudioContext::default())
        })
    }
}
