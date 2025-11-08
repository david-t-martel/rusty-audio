//! Audio Recording Module
//!
//! Provides ASIO-like recording with routing isolation:
//! - Direct monitoring (input → output, zero-latency)
//! - Routed monitoring (input → processing → output)
//! - Recording to buffer (isolated from playback)

use super::backend::{AudioBackendError, AudioConfig, Result, SampleFormat, StreamStatus};
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Recording state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
    Stopped,
}

/// Recording configuration
#[derive(Debug, Clone)]
pub struct RecordingConfig {
    pub config: AudioConfig,
    pub format: RecordingFormat,
    pub monitoring: MonitoringMode,
    pub buffer_size_seconds: f32,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            config: AudioConfig::default(),
            format: RecordingFormat::Wav,
            monitoring: MonitoringMode::Off,
            buffer_size_seconds: 60.0, // 60 seconds default
        }
    }
}

/// Recording output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingFormat {
    Wav,
    Flac,
}

/// Monitoring mode for input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitoringMode {
    /// No monitoring
    Off,
    /// Direct monitoring (input → output, minimal latency)
    Direct,
    /// Routed monitoring (input → processing → output)
    Routed,
}

/// Recording buffer that isolates recording from playback
pub struct RecordingBuffer {
    buffer: Arc<RwLock<Vec<Vec<f32>>>>, // channels x samples
    write_position: Arc<Mutex<usize>>,
    capacity_samples: usize,
    channels: usize,
    sample_rate: u32,
    state: Arc<RwLock<RecordingState>>,
}

impl RecordingBuffer {
    pub fn new(channels: usize, capacity_seconds: f32, sample_rate: u32) -> Self {
        let capacity_samples = (capacity_seconds * sample_rate as f32) as usize;
        let buffer = vec![vec![0.0; capacity_samples]; channels];
        
        Self {
            buffer: Arc::new(RwLock::new(buffer)),
            write_position: Arc::new(Mutex::new(0)),
            capacity_samples,
            channels,
            sample_rate,
            state: Arc::new(RwLock::new(RecordingState::Idle)),
        }
    }
    
    /// Write samples to the recording buffer
    pub fn write(&self, samples: &[Vec<f32>]) -> Result<usize> {
        let state = *self.state.read();
        if state != RecordingState::Recording {
            return Ok(0);
        }
        
        let mut buffer = self.buffer.write();
        let mut position = self.write_position.lock();
        
        if samples.len() != self.channels {
            return Err(AudioBackendError::Other(anyhow::anyhow!(
                "Channel mismatch: expected {}, got {}",
                self.channels,
                samples.len()
            )));
        }
        
        let samples_to_write = samples[0].len().min(self.capacity_samples - *position);
        
        for (ch_idx, channel) in samples.iter().enumerate() {
            if ch_idx < self.channels {
                for (i, &sample) in channel.iter().take(samples_to_write).enumerate() {
                    buffer[ch_idx][*position + i] = sample;
                }
            }
        }
        
        *position += samples_to_write;
        
        // Stop recording if buffer is full
        if *position >= self.capacity_samples {
            *self.state.write() = RecordingState::Stopped;
        }
        
        Ok(samples_to_write)
    }
    
    /// Get current recording position in samples
    pub fn position(&self) -> usize {
        *self.write_position.lock()
    }
    
    /// Get recording duration in seconds
    pub fn duration(&self) -> f32 {
        self.position() as f32 / self.sample_rate as f32
    }
    
    /// Get current recording state
    pub fn state(&self) -> RecordingState {
        *self.state.read()
    }
    
    /// Set recording state
    pub fn set_state(&self, state: RecordingState) {
        *self.state.write() = state;
        
        // Reset position when starting new recording
        if state == RecordingState::Recording {
            *self.write_position.lock() = 0;
        }
    }
    
    /// Get recorded data
    pub fn get_data(&self) -> Vec<Vec<f32>> {
        let buffer = self.buffer.read();
        let position = self.position();
        
        buffer.iter()
            .map(|channel| channel[..position].to_vec())
            .collect()
    }
    
    /// Clear the recording buffer
    pub fn clear(&self) {
        let mut buffer = self.buffer.write();
        for channel in buffer.iter_mut() {
            channel.fill(0.0);
        }
        *self.write_position.lock() = 0;
        *self.state.write() = RecordingState::Idle;
    }
    
    /// Get peak level for a channel
    pub fn peak_level(&self, channel: usize) -> f32 {
        if channel >= self.channels {
            return 0.0;
        }
        
        let buffer = self.buffer.read();
        let position = self.position();
        
        if position == 0 {
            return 0.0;
        }
        
        // Get peak from last 100ms
        let samples_to_check = (0.1 * self.sample_rate as f32) as usize;
        let start = position.saturating_sub(samples_to_check);
        
        buffer[channel][start..position]
            .iter()
            .map(|&s| s.abs())
            .fold(0.0, f32::max)
    }
    
    /// Get RMS level for a channel
    pub fn rms_level(&self, channel: usize) -> f32 {
        if channel >= self.channels {
            return 0.0;
        }
        
        let buffer = self.buffer.read();
        let position = self.position();
        
        if position == 0 {
            return 0.0;
        }
        
        // Get RMS from last 100ms
        let samples_to_check = (0.1 * self.sample_rate as f32) as usize;
        let start = position.saturating_sub(samples_to_check);
        let samples = &buffer[channel][start..position];
        
        let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }
}

/// Audio recorder with routing control
pub struct AudioRecorder {
    buffer: RecordingBuffer,
    config: RecordingConfig,
    monitoring_gain: f32,
    start_time: Option<Instant>,
}

impl AudioRecorder {
    pub fn new(config: RecordingConfig) -> Self {
        let buffer = RecordingBuffer::new(
            config.config.channels as usize,
            config.buffer_size_seconds,
            config.config.sample_rate,
        );
        
        Self {
            buffer,
            config,
            monitoring_gain: 1.0,
            start_time: None,
        }
    }
    
    /// Start recording
    pub fn start(&mut self) -> Result<()> {
        self.buffer.set_state(RecordingState::Recording);
        self.start_time = Some(Instant::now());
        Ok(())
    }
    
    /// Stop recording
    pub fn stop(&mut self) -> Result<()> {
        self.buffer.set_state(RecordingState::Stopped);
        Ok(())
    }
    
    /// Pause recording
    pub fn pause(&mut self) -> Result<()> {
        self.buffer.set_state(RecordingState::Paused);
        Ok(())
    }
    
    /// Resume recording
    pub fn resume(&mut self) -> Result<()> {
        self.buffer.set_state(RecordingState::Recording);
        Ok(())
    }
    
    /// Get current recording state
    pub fn state(&self) -> RecordingState {
        self.buffer.state()
    }
    
    /// Get recording duration
    pub fn duration(&self) -> Duration {
        self.start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO)
    }
    
    /// Get recording buffer reference
    pub fn buffer(&self) -> &RecordingBuffer {
        &self.buffer
    }
    
    /// Process input samples (for monitoring routing)
    pub fn process_input(&self, input: &[Vec<f32>]) -> Vec<Vec<f32>> {
        match self.config.monitoring {
            MonitoringMode::Off => {
                // No monitoring, return silence
                vec![vec![0.0; input[0].len()]; input.len()]
            }
            MonitoringMode::Direct => {
                // Direct monitoring with gain
                input.iter()
                    .map(|channel| {
                        channel.iter()
                            .map(|&s| s * self.monitoring_gain)
                            .collect()
                    })
                    .collect()
            }
            MonitoringMode::Routed => {
                // Routed monitoring (can add effects here)
                input.iter()
                    .map(|channel| {
                        channel.iter()
                            .map(|&s| s * self.monitoring_gain)
                            .collect()
                    })
                    .collect()
            }
        }
    }
    
    /// Set monitoring gain (0.0 to 1.0)
    pub fn set_monitoring_gain(&mut self, gain: f32) {
        self.monitoring_gain = gain.clamp(0.0, 1.0);
    }
    
    /// Get monitoring gain
    pub fn monitoring_gain(&self) -> f32 {
        self.monitoring_gain
    }
    
    /// Set monitoring mode
    pub fn set_monitoring_mode(&mut self, mode: MonitoringMode) {
        self.config.monitoring = mode;
    }
    
    /// Get monitoring mode
    pub fn monitoring_mode(&self) -> MonitoringMode {
        self.config.monitoring
    }
    
    /// Get configuration
    pub fn config(&self) -> &RecordingConfig {
        &self.config
    }
    
    /// Save recording to file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        use hound::{WavWriter, WavSpec};
        
        let data = self.buffer.get_data();
        if data.is_empty() || data[0].is_empty() {
            return Err(AudioBackendError::Other(anyhow::anyhow!(
                "No data to save"
            )));
        }
        
        match self.config.format {
            RecordingFormat::Wav => {
                let spec = WavSpec {
                    channels: self.config.config.channels,
                    sample_rate: self.config.config.sample_rate,
                    bits_per_sample: 32,
                    sample_format: hound::SampleFormat::Float,
                };
                
                let mut writer = WavWriter::create(path, spec)
                    .map_err(|e| AudioBackendError::Other(anyhow::anyhow!(
                        "Failed to create WAV file: {}", e
                    )))?;
                
                // Interleave channels
                let num_samples = data[0].len();
                for i in 0..num_samples {
                    for channel in &data {
                        writer.write_sample(channel[i])
                            .map_err(|e| AudioBackendError::Other(anyhow::anyhow!(
                                "Failed to write sample: {}", e
                            )))?;
                    }
                }
                
                writer.finalize()
                    .map_err(|e| AudioBackendError::Other(anyhow::anyhow!(
                        "Failed to finalize WAV file: {}", e
                    )))?;
            }
            RecordingFormat::Flac => {
                // FLAC encoding would go here
                // For now, return error as it's not implemented
                return Err(AudioBackendError::Other(anyhow::anyhow!(
                    "FLAC recording not yet implemented"
                )));
            }
        }
        
        Ok(())
    }
}
