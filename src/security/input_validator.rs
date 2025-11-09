//! Input validation module for secure parameter handling
//!
//! Validates and sanitizes all user-controllable inputs to prevent
//! injection attacks and ensure safe operating parameters.

use std::ops::RangeInclusive;
use thiserror::Error;

/// Input validator for all user-controllable parameters
pub struct InputValidator;

impl InputValidator {
    /// Validate volume parameter (0.0 to 1.0)
    pub fn validate_volume(value: f32) -> Result<f32, ValidationError> {
        const SAFE_RANGE: RangeInclusive<f32> = 0.0..=1.0;

        // Check for NaN or infinity
        if !value.is_finite() {
            return Err(ValidationError::NonFiniteValue {
                parameter: "volume".to_string(),
                value: format!("{:?}", value),
            });
        }

        // Check range
        if !SAFE_RANGE.contains(&value) {
            return Err(ValidationError::OutOfRange {
                parameter: "volume".to_string(),
                value: value.to_string(),
                expected: "0.0 to 1.0".to_string(),
            });
        }

        Ok(value)
    }

    /// Validate audio frequency (20Hz to 20kHz)
    pub fn validate_frequency(freq: f32) -> Result<f32, ValidationError> {
        const AUDIBLE_RANGE: RangeInclusive<f32> = 20.0..=20000.0;

        // Check for NaN or infinity
        if !freq.is_finite() {
            return Err(ValidationError::NonFiniteValue {
                parameter: "frequency".to_string(),
                value: format!("{:?}", freq),
            });
        }

        // Check range
        if !AUDIBLE_RANGE.contains(&freq) {
            return Err(ValidationError::OutOfRange {
                parameter: "frequency".to_string(),
                value: freq.to_string(),
                expected: "20Hz to 20kHz".to_string(),
            });
        }

        Ok(freq)
    }

    /// Validate equalizer gain (-40dB to +20dB)
    pub fn validate_eq_gain(gain: f32) -> Result<f32, ValidationError> {
        const SAFE_GAIN_RANGE: RangeInclusive<f32> = -40.0..=20.0;

        // Check for NaN or infinity
        if !gain.is_finite() {
            return Err(ValidationError::NonFiniteValue {
                parameter: "EQ gain".to_string(),
                value: format!("{:?}", gain),
            });
        }

        // Check range
        if !SAFE_GAIN_RANGE.contains(&gain) {
            return Err(ValidationError::OutOfRange {
                parameter: "EQ gain".to_string(),
                value: gain.to_string(),
                expected: "-40dB to +20dB".to_string(),
            });
        }

        Ok(gain)
    }

    /// Validate sample rate
    pub fn validate_sample_rate(rate: u32) -> Result<u32, ValidationError> {
        const VALID_RATES: &[u32] = &[
            8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000, 176400, 192000,
        ];

        if !VALID_RATES.contains(&rate) {
            return Err(ValidationError::InvalidSampleRate {
                rate,
                valid_rates: VALID_RATES.to_vec(),
            });
        }

        Ok(rate)
    }

    /// Validate buffer size (must be power of 2)
    pub fn validate_buffer_size(size: usize) -> Result<usize, ValidationError> {
        const MIN_SIZE: usize = 64;
        const MAX_SIZE: usize = 16384;

        // Check if power of 2
        if !size.is_power_of_two() {
            return Err(ValidationError::InvalidBufferSize {
                size,
                reason: "Must be a power of 2".to_string(),
            });
        }

        // Check range
        if size < MIN_SIZE || size > MAX_SIZE {
            return Err(ValidationError::InvalidBufferSize {
                size,
                reason: format!("Must be between {} and {}", MIN_SIZE, MAX_SIZE),
            });
        }

        Ok(size)
    }

    /// Validate channel count
    pub fn validate_channel_count(channels: usize) -> Result<usize, ValidationError> {
        const MAX_CHANNELS: usize = 32;

        if channels == 0 {
            return Err(ValidationError::InvalidChannelCount {
                count: channels,
                reason: "Must have at least 1 channel".to_string(),
            });
        }

        if channels > MAX_CHANNELS {
            return Err(ValidationError::InvalidChannelCount {
                count: channels,
                reason: format!("Maximum {} channels supported", MAX_CHANNELS),
            });
        }

        Ok(channels)
    }

    /// Validate duration in seconds
    pub fn validate_duration(seconds: f32) -> Result<f32, ValidationError> {
        const MAX_DURATION: f32 = 3600.0; // 1 hour maximum

        if !seconds.is_finite() || seconds < 0.0 {
            return Err(ValidationError::InvalidDuration {
                value: seconds,
                reason: "Must be a positive finite number".to_string(),
            });
        }

        if seconds > MAX_DURATION {
            return Err(ValidationError::InvalidDuration {
                value: seconds,
                reason: format!("Maximum duration is {} seconds", MAX_DURATION),
            });
        }

        Ok(seconds)
    }

    /// Sanitize string input (remove dangerous characters)
    pub fn sanitize_string(input: &str, max_length: usize) -> String {
        input
            .chars()
            .filter(|c| {
                c.is_alphanumeric()
                    || c.is_whitespace()
                    || *c == '-'
                    || *c == '_'
                    || *c == '.'
                    || *c == ','
                    || *c == '!'
                    || *c == '?'
            })
            .take(max_length)
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Validate and sanitize metadata string
    pub fn validate_metadata(key: &str, value: &str) -> Result<(String, String), ValidationError> {
        const MAX_KEY_LENGTH: usize = 50;
        const MAX_VALUE_LENGTH: usize = 200;

        // Sanitize key
        let clean_key = Self::sanitize_string(key, MAX_KEY_LENGTH);
        if clean_key.is_empty() {
            return Err(ValidationError::InvalidMetadata {
                key: key.to_string(),
                reason: "Key cannot be empty after sanitization".to_string(),
            });
        }

        // Sanitize value
        let clean_value = Self::sanitize_string(value, MAX_VALUE_LENGTH);

        Ok((clean_key, clean_value))
    }

    /// Validate playback speed/rate
    pub fn validate_playback_rate(rate: f32) -> Result<f32, ValidationError> {
        const MIN_RATE: f32 = 0.25; // 1/4 speed
        const MAX_RATE: f32 = 4.0; // 4x speed

        if !rate.is_finite() || rate <= 0.0 {
            return Err(ValidationError::InvalidPlaybackRate {
                rate,
                reason: "Must be a positive finite number".to_string(),
            });
        }

        if rate < MIN_RATE || rate > MAX_RATE {
            return Err(ValidationError::InvalidPlaybackRate {
                rate,
                reason: format!("Must be between {}x and {}x", MIN_RATE, MAX_RATE),
            });
        }

        Ok(rate)
    }

    /// Validate seek position
    pub fn validate_seek_position(position: f32, duration: f32) -> Result<f32, ValidationError> {
        if !position.is_finite() || position < 0.0 {
            return Err(ValidationError::InvalidSeekPosition {
                position,
                duration,
                reason: "Position must be non-negative".to_string(),
            });
        }

        if position > duration {
            return Err(ValidationError::InvalidSeekPosition {
                position,
                duration,
                reason: "Position exceeds duration".to_string(),
            });
        }

        Ok(position)
    }

    /// Batch validate multiple parameters
    pub fn validate_audio_parameters(
        volume: f32,
        sample_rate: u32,
        channels: usize,
        buffer_size: usize,
    ) -> Result<AudioParameters, ValidationError> {
        Ok(AudioParameters {
            volume: Self::validate_volume(volume)?,
            sample_rate: Self::validate_sample_rate(sample_rate)?,
            channels: Self::validate_channel_count(channels)?,
            buffer_size: Self::validate_buffer_size(buffer_size)?,
        })
    }
}

/// Validated audio parameters
#[derive(Debug, Clone)]
pub struct AudioParameters {
    pub volume: f32,
    pub sample_rate: u32,
    pub channels: usize,
    pub buffer_size: usize,
}

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Value out of range for {parameter}: {value}, expected: {expected}")]
    OutOfRange {
        parameter: String,
        value: String,
        expected: String,
    },

    #[error("Non-finite value for {parameter}: {value}")]
    NonFiniteValue { parameter: String, value: String },

    #[error("Invalid sample rate: {rate}Hz, valid rates: {valid_rates:?}")]
    InvalidSampleRate { rate: u32, valid_rates: Vec<u32> },

    #[error("Invalid buffer size: {size} ({reason})")]
    InvalidBufferSize { size: usize, reason: String },

    #[error("Invalid channel count: {count} ({reason})")]
    InvalidChannelCount { count: usize, reason: String },

    #[error("Invalid duration: {value} ({reason})")]
    InvalidDuration { value: f32, reason: String },

    #[error("Invalid metadata for key '{key}': {reason}")]
    InvalidMetadata { key: String, reason: String },

    #[error("Invalid playback rate: {rate} ({reason})")]
    InvalidPlaybackRate { rate: f32, reason: String },

    #[error("Invalid seek position: {position}/{duration} ({reason})")]
    InvalidSeekPosition {
        position: f32,
        duration: f32,
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_validation() {
        // Valid values
        assert!(InputValidator::validate_volume(0.0).is_ok());
        assert!(InputValidator::validate_volume(0.5).is_ok());
        assert!(InputValidator::validate_volume(1.0).is_ok());

        // Invalid values
        assert!(InputValidator::validate_volume(-0.1).is_err());
        assert!(InputValidator::validate_volume(1.1).is_err());
        assert!(InputValidator::validate_volume(f32::NAN).is_err());
        assert!(InputValidator::validate_volume(f32::INFINITY).is_err());
    }

    #[test]
    fn test_frequency_validation() {
        // Valid values
        assert!(InputValidator::validate_frequency(20.0).is_ok());
        assert!(InputValidator::validate_frequency(1000.0).is_ok());
        assert!(InputValidator::validate_frequency(20000.0).is_ok());

        // Invalid values
        assert!(InputValidator::validate_frequency(19.9).is_err());
        assert!(InputValidator::validate_frequency(20000.1).is_err());
        assert!(InputValidator::validate_frequency(f32::NAN).is_err());
    }

    #[test]
    fn test_buffer_size_validation() {
        // Valid values (powers of 2)
        assert!(InputValidator::validate_buffer_size(64).is_ok());
        assert!(InputValidator::validate_buffer_size(128).is_ok());
        assert!(InputValidator::validate_buffer_size(256).is_ok());
        assert!(InputValidator::validate_buffer_size(512).is_ok());
        assert!(InputValidator::validate_buffer_size(1024).is_ok());

        // Invalid values
        assert!(InputValidator::validate_buffer_size(100).is_err()); // Not power of 2
        assert!(InputValidator::validate_buffer_size(32).is_err()); // Too small
        assert!(InputValidator::validate_buffer_size(32768).is_err()); // Too large
    }

    #[test]
    fn test_string_sanitization() {
        // Test dangerous input sanitization
        assert_eq!(
            InputValidator::sanitize_string("<script>alert('xss')</script>", 100),
            "scriptalertxssscript"
        );

        assert_eq!(
            InputValidator::sanitize_string("../../etc/passwd", 100),
            "..etcpasswd"
        );

        assert_eq!(
            InputValidator::sanitize_string("Robert'); DROP TABLE users;--", 100),
            "Robert DROP TABLE users"
        );

        // Test length limiting
        let long_string = "a".repeat(200);
        assert_eq!(InputValidator::sanitize_string(&long_string, 50).len(), 50);
    }

    #[test]
    fn test_metadata_validation() {
        // Valid metadata
        let result = InputValidator::validate_metadata("artist", "John Doe");
        assert!(result.is_ok());
        let (key, value) = result.unwrap();
        assert_eq!(key, "artist");
        assert_eq!(value, "John Doe");

        // Metadata with special characters
        let result = InputValidator::validate_metadata("title", "Song & Dance!");
        assert!(result.is_ok());
        let (_, value) = result.unwrap();
        assert_eq!(value, "Song  Dance!");

        // Empty key after sanitization
        let result = InputValidator::validate_metadata("<<<>>>", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_sample_rate_validation() {
        // Valid rates
        assert!(InputValidator::validate_sample_rate(44100).is_ok());
        assert!(InputValidator::validate_sample_rate(48000).is_ok());
        assert!(InputValidator::validate_sample_rate(96000).is_ok());

        // Invalid rate
        assert!(InputValidator::validate_sample_rate(44000).is_err());
    }

    #[test]
    fn test_batch_validation() {
        // Valid parameters
        let result = InputValidator::validate_audio_parameters(
            0.5,   // volume
            44100, // sample rate
            2,     // channels
            1024,  // buffer size
        );
        assert!(result.is_ok());

        // Invalid parameters (should fail on first invalid)
        let result = InputValidator::validate_audio_parameters(
            1.5, // invalid volume
            44100, 2, 1024,
        );
        assert!(result.is_err());
    }
}
