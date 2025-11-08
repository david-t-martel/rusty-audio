//! Audio Recording Module
//!
//! Provides professional audio recording capabilities with:
//! - Real-time level metering (peak and RMS)
//! - Multi-channel support (stereo by default)
//! - State management (Idle, Recording, Paused, Stopped)
//! - Monitoring modes (Off, Direct, Routed)
//! - WAV file export (32-bit float)

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use anyhow::{Result, Context};

/// Recording format for file export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingFormat {
    /// WAV format with 32-bit float samples
    Wav,
    /// FLAC format (lossless compression) - TODO Phase 4.1
    Flac,
}

/// Recording state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingState {
    /// Ready to record, no data captured
    Idle,
    /// Actively recording audio
    Recording,
    /// Recording paused, can be resumed
    Paused,
    /// Recording stopped, data available for export
    Stopped,
}

/// Monitoring mode for input signal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitoringMode {
    /// No monitoring - silent recording
    Off,
    /// Direct monitoring - zero-latency hardware monitoring
    Direct,
    /// Routed monitoring - through effects chain
    Routed,
}

/// Recording configuration
#[derive(Debug, Clone)]
pub struct RecordingConfig {
    /// Sample rate in Hz (e.g., 44100, 48000)
    pub sample_rate: u32,
    /// Number of channels (1 for mono, 2 for stereo)
    pub channels: u16,
    /// Buffer size in samples per channel
    pub buffer_size: usize,
    /// Maximum recording duration in seconds (0 = unlimited)
    pub max_duration_secs: u64,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,  // Professional audio standard
            channels: 2,         // Stereo
            buffer_size: 1024 * 1024 * 10,  // ~10MB buffer (~3.5 minutes stereo)
            max_duration_secs: 0,  // Unlimited
        }
    }
}

/// Recording buffer with circular buffer and level metering
pub struct RecordingBuffer {
    /// Interleaved audio samples (LRLRLR...)
    samples: Vec<f32>,
    /// Write position in the buffer
    write_pos: usize,
    /// Total samples written (for duration calculation)
    total_samples: usize,
    /// Number of channels
    channels: usize,
    /// Sample rate
    sample_rate: u32,
    /// Peak levels per channel
    peak_levels: Vec<f32>,
    /// RMS levels per channel
    rms_levels: Vec<f32>,
    /// Last update time for level decay
    last_update: Instant,
}

impl RecordingBuffer {
    pub fn new(capacity: usize, channels: usize, sample_rate: u32) -> Self {
        Self {
            samples: vec![0.0; capacity * channels],
            write_pos: 0,
            total_samples: 0,
            channels,
            sample_rate,
            peak_levels: vec![0.0; channels],
            rms_levels: vec![0.0; channels],
            last_update: Instant::now(),
        }
    }

    /// Write interleaved audio samples to the buffer
    pub fn write(&mut self, data: &[f32]) {
        for &sample in data {
            self.samples[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.samples.len();
            self.total_samples += 1;
        }

        // Update level meters
        self.update_levels(data);
    }

    /// Update peak and RMS levels from audio data
    fn update_levels(&mut self, data: &[f32]) {
        if data.is_empty() {
            return;
        }

        // Decay old levels (60dB per second)
        let now = Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32();
        let decay_factor = 0.001_f32.powf(dt);  // -60dB per second
        
        for level in &mut self.peak_levels {
            *level *= decay_factor;
        }
        for level in &mut self.rms_levels {
            *level *= decay_factor;
        }
        self.last_update = now;

        // Process samples per channel
        let mut sum_squares = vec![0.0f32; self.channels];
        let mut sample_counts = vec![0usize; self.channels];

        for (i, &sample) in data.iter().enumerate() {
            let ch = i % self.channels;
            let abs_sample = sample.abs();

            // Update peak
            if abs_sample > self.peak_levels[ch] {
                self.peak_levels[ch] = abs_sample;
            }

            // Accumulate for RMS
            sum_squares[ch] += sample * sample;
            sample_counts[ch] += 1;
        }

        // Calculate RMS
        for ch in 0..self.channels {
            if sample_counts[ch] > 0 {
                self.rms_levels[ch] = (sum_squares[ch] / sample_counts[ch] as f32).sqrt();
            }
        }
    }

    /// Get peak level for a specific channel (0.0 to 1.0+)
    pub fn peak_level(&self, channel: usize) -> f32 {
        self.peak_levels.get(channel).copied().unwrap_or(0.0)
    }

    /// Get RMS level for a specific channel (0.0 to 1.0)
    pub fn rms_level(&self, channel: usize) -> f32 {
        self.rms_levels.get(channel).copied().unwrap_or(0.0)
    }

    /// Get recording duration
    pub fn duration(&self) -> Duration {
        let total_frames = self.total_samples / self.channels;
        let secs = total_frames as f64 / self.sample_rate as f64;
        Duration::from_secs_f64(secs)
    }

    /// Get current buffer position (number of samples written)
    pub fn position(&self) -> usize {
        self.total_samples
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.samples.fill(0.0);
        self.write_pos = 0;
        self.total_samples = 0;
        self.peak_levels.fill(0.0);
        self.rms_levels.fill(0.0);
    }

    /// Get all recorded samples (copies data)
    pub fn get_samples(&self) -> Vec<f32> {
        if self.total_samples <= self.samples.len() {
            // Haven't wrapped yet, return linear data
            self.samples[0..self.total_samples].to_vec()
        } else {
            // Buffer wrapped, need to reconstruct proper order
            let mut result = Vec::with_capacity(self.samples.len());
            result.extend_from_slice(&self.samples[self.write_pos..]);
            result.extend_from_slice(&self.samples[..self.write_pos]);
            result
        }
    }
}

/// Audio recorder with state management and level metering
pub struct AudioRecorder {
    /// Recording configuration
    config: RecordingConfig,
    /// Audio buffer
    buffer: Arc<Mutex<RecordingBuffer>>,
    /// Current recording state
    state: Arc<Mutex<RecordingState>>,
    /// Recording start time
    start_time: Option<Instant>,
    /// Pause duration accumulator
    pause_duration: Duration,
    /// Last pause time
    pause_time: Option<Instant>,
    /// Monitoring mode
    monitoring_mode: MonitoringMode,
    /// Monitoring gain (0.0 to 1.0)
    monitoring_gain: f32,
}

impl AudioRecorder {
    /// Create a new audio recorder with the given configuration
    pub fn new(config: RecordingConfig) -> Self {
        let buffer = Arc::new(Mutex::new(RecordingBuffer::new(
            config.buffer_size,
            config.channels as usize,
            config.sample_rate,
        )));

        Self {
            config,
            buffer,
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            start_time: None,
            pause_duration: Duration::ZERO,
            pause_time: None,
            monitoring_mode: MonitoringMode::Off,
            monitoring_gain: 1.0,
        }
    }

    /// Start recording
    pub fn start(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        
        match *state {
            RecordingState::Idle => {
                *state = RecordingState::Recording;
                self.start_time = Some(Instant::now());
                self.pause_duration = Duration::ZERO;
                Ok(())
            }
            RecordingState::Stopped => {
                // Clear buffer and start fresh
                self.buffer.lock().unwrap().clear();
                *state = RecordingState::Recording;
                self.start_time = Some(Instant::now());
                self.pause_duration = Duration::ZERO;
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Cannot start recording from {:?} state", *state)),
        }
    }

    /// Stop recording
    pub fn stop(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        match *state {
            RecordingState::Recording | RecordingState::Paused => {
                *state = RecordingState::Stopped;
                
                // If paused, account for pause time
                if let Some(pause_time) = self.pause_time {
                    self.pause_duration += pause_time.elapsed();
                    self.pause_time = None;
                }
                
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Cannot stop recording from {:?} state", *state)),
        }
    }

    /// Pause recording
    pub fn pause(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if *state == RecordingState::Recording {
            *state = RecordingState::Paused;
            self.pause_time = Some(Instant::now());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Cannot pause recording from {:?} state", *state))
        }
    }

    /// Resume recording from paused state
    pub fn resume(&mut self) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if *state == RecordingState::Paused {
            *state = RecordingState::Recording;
            
            if let Some(pause_time) = self.pause_time.take() {
                self.pause_duration += pause_time.elapsed();
            }
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Cannot resume recording from {:?} state", *state))
        }
    }

    /// Get current recording state
    pub fn state(&self) -> RecordingState {
        *self.state.lock().unwrap()
    }

    /// Get recording duration (excluding pauses)
    pub fn duration(&self) -> Duration {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            let pause_time = if let Some(pause) = self.pause_time {
                self.pause_duration + pause.elapsed()
            } else {
                self.pause_duration
            };
            elapsed.saturating_sub(pause_time)
        } else {
            Duration::ZERO
        }
    }

    /// Get reference to the recording buffer
    pub fn buffer(&self) -> Arc<Mutex<RecordingBuffer>> {
        self.buffer.clone()
    }

    /// Get monitoring mode
    pub fn monitoring_mode(&self) -> MonitoringMode {
        self.monitoring_mode
    }

    /// Set monitoring mode
    pub fn set_monitoring_mode(&mut self, mode: MonitoringMode) {
        self.monitoring_mode = mode;
    }

    /// Get monitoring gain
    pub fn monitoring_gain(&self) -> f32 {
        self.monitoring_gain
    }

    /// Set monitoring gain (0.0 to 1.0)
    pub fn set_monitoring_gain(&mut self, gain: f32) {
        self.monitoring_gain = gain.clamp(0.0, 1.0);
    }

    /// Get recording configuration
    pub fn config(&self) -> &RecordingConfig {
        &self.config
    }

    /// Write audio samples to the buffer (called from audio thread)
    pub fn write_samples(&mut self, samples: &[f32]) {
        let state = self.state.lock().unwrap();
        
        // Only write if actively recording
        if *state == RecordingState::Recording {
            self.buffer.lock().unwrap().write(samples);
        }
    }

    /// Export recording to WAV file
    pub fn save_to_wav(&self, path: &std::path::Path) -> Result<()> {
        let buffer = self.buffer.lock().unwrap();
        let samples = buffer.get_samples();
        
        let spec = hound::WavSpec {
            channels: self.config.channels,
            sample_rate: self.config.sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        
        let mut writer = hound::WavWriter::create(path, spec)
            .context("Failed to create WAV file")?;
        
        for &sample in &samples {
            writer.write_sample(sample)
                .context("Failed to write sample to WAV file")?;
        }
        
        writer.finalize()
            .context("Failed to finalize WAV file")?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_state_transitions() {
        let mut recorder = AudioRecorder::new(RecordingConfig::default());
        
        assert_eq!(recorder.state(), RecordingState::Idle);
        
        // Idle -> Recording
        recorder.start().unwrap();
        assert_eq!(recorder.state(), RecordingState::Recording);
        
        // Recording -> Paused
        recorder.pause().unwrap();
        assert_eq!(recorder.state(), RecordingState::Paused);
        
        // Paused -> Recording
        recorder.resume().unwrap();
        assert_eq!(recorder.state(), RecordingState::Recording);
        
        // Recording -> Stopped
        recorder.stop().unwrap();
        assert_eq!(recorder.state(), RecordingState::Stopped);
    }

    #[test]
    fn test_buffer_level_metering() {
        let mut buffer = RecordingBuffer::new(1024, 2, 48000);
        
        // Write test signal: left channel at 0.5, right at 0.8
        let test_samples = vec![0.5, 0.8, 0.5, 0.8, 0.5, 0.8];
        buffer.write(&test_samples);
        
        // Check peak levels
        assert!((buffer.peak_level(0) - 0.5).abs() < 0.01);
        assert!((buffer.peak_level(1) - 0.8).abs() < 0.01);
        
        // Check RMS levels (should be same as peak for constant signal)
        assert!((buffer.rms_level(0) - 0.5).abs() < 0.01);
        assert!((buffer.rms_level(1) - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_buffer_duration() {
        let mut buffer = RecordingBuffer::new(1024, 2, 48000);
        
        // Write 1 second of audio (48000 samples per channel * 2 channels)
        let samples = vec![0.0; 96000];
        buffer.write(&samples);
        
        let duration = buffer.duration();
        assert!((duration.as_secs_f64() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_monitoring_mode() {
        let mut recorder = AudioRecorder::new(RecordingConfig::default());
        
        assert_eq!(recorder.monitoring_mode(), MonitoringMode::Off);
        
        recorder.set_monitoring_mode(MonitoringMode::Direct);
        assert_eq!(recorder.monitoring_mode(), MonitoringMode::Direct);
        
        recorder.set_monitoring_gain(0.5);
        assert_eq!(recorder.monitoring_gain(), 0.5);
    }
}
