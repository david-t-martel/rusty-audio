//! Audio safety module for hearing and hardware protection
//!
//! Implements volume limiting, peak detection, and emergency stop mechanisms
//! to protect users' hearing and audio equipment.

use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use thiserror::Error;

/// Audio safety limiter with peak detection and emergency stop
pub struct AudioSafetyLimiter {
    config: AudioConfig,
    emergency_cutoff: Arc<RwLock<bool>>,
    volume_history: VecDeque<f32>,
    peak_detector: PeakDetector,
    rms_calculator: RmsCalculator,
    violation_count: usize,
    last_violation_time: Option<std::time::Instant>,
}

impl AudioSafetyLimiter {
    /// Create a new audio safety limiter
    pub fn new(config: AudioConfig) -> Self {
        Self {
            config,
            emergency_cutoff: Arc::new(RwLock::new(false)),
            volume_history: VecDeque::with_capacity(100),
            peak_detector: PeakDetector::new(),
            rms_calculator: RmsCalculator::new(2048),
            violation_count: 0,
            last_violation_time: None,
        }
    }

    /// Process audio samples with safety limiting
    pub fn process_audio(&mut self, samples: &mut [f32], volume: f32) -> Result<(), SafetyError> {
        // Check emergency cutoff first
        if *self.emergency_cutoff.read() {
            samples.fill(0.0);
            return Ok(());
        }

        // Validate volume parameter
        if !(0.0..=1.0).contains(&volume) {
            return Err(SafetyError::InvalidVolume { value: volume });
        }

        // Apply safe volume limit
        let safe_volume = volume.min(self.config.max_volume);

        // Track volume for history
        self.volume_history.push_back(safe_volume);
        if self.volume_history.len() > 100 {
            self.volume_history.pop_front();
        }

        // Check for sustained high volume (hearing protection)
        if self.is_sustained_high_volume() {
            tracing::warn!("Sustained high volume detected - applying reduction");
            self.apply_hearing_protection(samples, safe_volume);
        } else {
            self.apply_limiting(samples, safe_volume)?;
        }

        // Update RMS for monitoring
        self.rms_calculator.update(samples);

        Ok(())
    }

    /// Apply audio limiting with soft knee compression
    fn apply_limiting(&mut self, samples: &mut [f32], volume: f32) -> Result<(), SafetyError> {
        let threshold = self.config.limiter_threshold;
        let knee_start = threshold * 0.9; // Soft knee starts at 90% of threshold

        for sample in samples.iter_mut() {
            // Apply volume
            *sample *= volume;

            // Detect and log peaks
            if self.peak_detector.detect(*sample) {
                self.handle_peak_violation(*sample);
            }

            let abs_sample = sample.abs();

            if abs_sample > threshold {
                // Hard limiting
                *sample = threshold.copysign(*sample);
            } else if abs_sample > knee_start {
                // Soft knee compression (3:1 ratio)
                let excess = abs_sample - knee_start;
                let knee_range = threshold - knee_start;
                let compressed_excess = excess * (1.0 / 3.0);
                let limited = knee_start + compressed_excess.min(knee_range);
                *sample = limited.copysign(*sample);
            }

            // Final safety clamp
            *sample = sample.clamp(-1.0, 1.0);
        }

        Ok(())
    }

    /// Apply hearing protection for sustained high volumes
    fn apply_hearing_protection(&mut self, samples: &mut [f32], base_volume: f32) {
        // Reduce volume by 30% for hearing protection
        let protected_volume = base_volume * 0.7;

        for sample in samples.iter_mut() {
            *sample *= protected_volume;
            *sample = sample.clamp(
                -self.config.limiter_threshold,
                self.config.limiter_threshold,
            );
        }
    }

    /// Check if volume has been high for too long
    fn is_sustained_high_volume(&self) -> bool {
        if self.volume_history.len() < 50 {
            return false;
        }

        let high_volume_threshold = 0.8;
        let high_count = self
            .volume_history
            .iter()
            .filter(|&&v| v > high_volume_threshold)
            .count();

        // If more than 80% of recent samples are high volume
        high_count > (self.volume_history.len() * 4 / 5)
    }

    /// Handle peak violation
    fn handle_peak_violation(&mut self, sample_value: f32) {
        self.violation_count += 1;
        self.last_violation_time = Some(std::time::Instant::now());

        tracing::warn!(
            "Audio peak violation #{}: sample value = {:.3}",
            self.violation_count,
            sample_value
        );

        // Trigger emergency stop if too many violations
        if self.violation_count > 10 {
            tracing::error!("Too many peak violations - triggering emergency stop");
            self.emergency_stop();
        }
    }

    /// Trigger emergency audio cutoff
    pub fn emergency_stop(&self) {
        *self.emergency_cutoff.write() = true;
        tracing::error!("EMERGENCY STOP activated - all audio muted");
    }

    /// Reset emergency stop
    pub fn reset_emergency_stop(&self) {
        *self.emergency_cutoff.write() = false;
        tracing::info!("Emergency stop reset - audio resumed");
    }

    /// Check if limiter is operational
    pub fn is_operational(&self) -> bool {
        !*self.emergency_cutoff.read()
    }

    /// Get current RMS level
    pub fn get_rms_level(&self) -> f32 {
        self.rms_calculator.get_rms()
    }

    /// Get peak level
    pub fn get_peak_level(&self) -> f32 {
        self.peak_detector.get_current_peak()
    }

    /// Get violation count
    pub fn get_violation_count(&self) -> usize {
        self.violation_count
    }

    /// Reset violation counter
    pub fn reset_violations(&mut self) {
        self.violation_count = 0;
        self.last_violation_time = None;
    }
}

/// Peak detector for audio signals
struct PeakDetector {
    threshold: f32,
    attack_time: f32,
    release_time: f32,
    envelope: f32,
    current_peak: f32,
    hold_time: usize,
    hold_counter: usize,
}

impl PeakDetector {
    fn new() -> Self {
        Self {
            threshold: 0.95,
            attack_time: 0.001,  // 1ms attack
            release_time: 0.100, // 100ms release
            envelope: 0.0,
            current_peak: 0.0,
            hold_time: 100, // Hold peak for 100 samples
            hold_counter: 0,
        }
    }

    fn detect(&mut self, sample: f32) -> bool {
        let abs_sample = sample.abs();

        // Update envelope follower
        if abs_sample > self.envelope {
            self.envelope =
                abs_sample * self.attack_time + self.envelope * (1.0 - self.attack_time);
        } else {
            self.envelope *= 1.0 - self.release_time;
        }

        // Update peak with hold
        if abs_sample > self.current_peak {
            self.current_peak = abs_sample;
            self.hold_counter = self.hold_time;
        } else if self.hold_counter > 0 {
            self.hold_counter -= 1;
        } else {
            self.current_peak *= 0.99; // Slow decay
        }

        self.envelope > self.threshold
    }

    fn get_current_peak(&self) -> f32 {
        self.current_peak
    }
}

/// RMS (Root Mean Square) calculator for audio level monitoring
struct RmsCalculator {
    buffer: VecDeque<f32>,
    buffer_size: usize,
    sum_of_squares: f32,
}

impl RmsCalculator {
    fn new(buffer_size: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(buffer_size),
            buffer_size,
            sum_of_squares: 0.0,
        }
    }

    fn update(&mut self, samples: &[f32]) {
        for &sample in samples {
            let squared = sample * sample;

            if self.buffer.len() >= self.buffer_size {
                if let Some(old) = self.buffer.pop_front() {
                    self.sum_of_squares -= old * old;
                }
            }

            self.buffer.push_back(sample);
            self.sum_of_squares += squared;
        }
    }

    fn get_rms(&self) -> f32 {
        if self.buffer.is_empty() {
            return 0.0;
        }

        (self.sum_of_squares / self.buffer.len() as f32).sqrt()
    }
}

/// Audio configuration for safety limiter
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub max_volume: f32,
    pub default_volume: f32,
    pub enable_limiter: bool,
    pub limiter_threshold: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            max_volume: 0.85,    // -1.4 dB headroom
            default_volume: 0.5, // 50% default
            enable_limiter: true,
            limiter_threshold: 0.95,
        }
    }
}

/// Safety errors that can occur during audio processing
#[derive(Debug, Error)]
pub enum SafetyError {
    #[error("Volume exceeds safe limits: {value}")]
    UnsafeVolume { value: f32 },

    #[error("Invalid volume value: {value} (must be 0.0-1.0)")]
    InvalidVolume { value: f32 },

    #[error("Audio peak detected above threshold")]
    PeakDetected,

    #[error("Sustained high volume detected")]
    SustainedHighVolume,

    #[error("Emergency stop activated")]
    EmergencyStopActive,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_limiting() {
        let config = AudioConfig::default();
        let mut limiter = AudioSafetyLimiter::new(config);

        // Test with excessive volume samples
        let mut samples = vec![2.0, -1.5, 1.8, -2.2]; // Out of range values
        assert!(limiter.process_audio(&mut samples, 1.0).is_ok());

        // All samples should be within -1.0 to 1.0
        for sample in &samples {
            assert!(sample.abs() <= 1.0, "Sample {} exceeds safe range", sample);
        }
    }

    #[test]
    fn test_emergency_stop() {
        let config = AudioConfig::default();
        let mut limiter = AudioSafetyLimiter::new(config);

        // Trigger emergency stop
        limiter.emergency_stop();

        // Process audio - should be muted
        let mut samples = vec![0.5, -0.5, 0.3, -0.3];
        assert!(limiter.process_audio(&mut samples, 0.5).is_ok());

        // All samples should be zero
        assert!(
            samples.iter().all(|&s| s == 0.0),
            "Audio not muted during emergency stop"
        );

        // Reset and test again
        limiter.reset_emergency_stop();
        let mut samples = vec![0.5, -0.5, 0.3, -0.3];
        assert!(limiter.process_audio(&mut samples, 0.5).is_ok());

        // Samples should not be zero after reset
        assert!(
            !samples.iter().all(|&s| s == 0.0),
            "Audio still muted after reset"
        );
    }

    #[test]
    fn test_peak_detection() {
        let mut detector = PeakDetector::new();

        // Test peak detection
        assert!(!detector.detect(0.5)); // Below threshold
        assert!(!detector.detect(0.9)); // Still below
        assert!(detector.detect(0.98)); // Above threshold
        assert!(detector.detect(1.0)); // Peak

        // Check peak value is retained
        assert!(detector.get_current_peak() >= 0.98);
    }

    #[test]
    fn test_rms_calculation() {
        let mut calculator = RmsCalculator::new(4);

        // Test with known values
        let samples = vec![0.5, 0.5, 0.5, 0.5];
        calculator.update(&samples);

        let rms = calculator.get_rms();
        assert!(
            (rms - 0.5).abs() < 0.01,
            "RMS calculation incorrect: {}",
            rms
        );
    }

    #[test]
    fn test_invalid_volume() {
        let config = AudioConfig::default();
        let mut limiter = AudioSafetyLimiter::new(config);

        let mut samples = vec![0.5; 4];

        // Test with invalid volume values
        assert!(limiter.process_audio(&mut samples, -0.1).is_err());
        assert!(limiter.process_audio(&mut samples, 1.5).is_err());
        assert!(limiter.process_audio(&mut samples, f32::NAN).is_err());
    }
}
