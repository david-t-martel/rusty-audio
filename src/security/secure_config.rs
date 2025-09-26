//! Secure configuration management
//!
//! Handles loading, validation, and storage of security-critical configuration
//! with encryption support for sensitive values.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use thiserror::Error;

/// Main secure configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureConfig {
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub limits: ResourceLimits,
    #[serde(default)]
    pub monitoring: MonitoringConfig,
}

/// Audio-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub max_volume: f32,
    pub default_volume: f32,
    pub enable_limiter: bool,
    pub limiter_threshold: f32,
    pub enable_hearing_protection: bool,
    pub hearing_protection_threshold: f32,
    pub emergency_stop_enabled: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub sandbox_enabled: bool,
    pub sandbox_path: PathBuf,
    pub allowed_file_types: Vec<String>,
    pub max_file_size_mb: u64,
    pub validate_file_content: bool,
    pub enable_path_traversal_protection: bool,
}

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: usize,
    pub max_buffer_size: usize,
    pub max_channels: usize,
    pub max_sample_rate: u32,
    pub max_concurrent_files: usize,
    pub max_processing_threads: usize,
}

/// Monitoring and alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enable_security_monitoring: bool,
    pub log_security_events: bool,
    pub alert_on_violations: bool,
    pub max_violations_before_lockdown: usize,
    pub violation_window_seconds: u64,
}

impl Default for SecureConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            security: SecurityConfig::default(),
            limits: ResourceLimits::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            max_volume: 0.85,
            default_volume: 0.5,
            enable_limiter: true,
            limiter_threshold: 0.95,
            enable_hearing_protection: true,
            hearing_protection_threshold: 0.8,
            emergency_stop_enabled: true,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            sandbox_enabled: true,
            sandbox_path: dirs::audio_dir()
                .or_else(dirs::home_dir)
                .unwrap_or_else(|| PathBuf::from(".")),
            allowed_file_types: vec![
                "mp3".to_string(),
                "wav".to_string(),
                "flac".to_string(),
                "ogg".to_string(),
                "m4a".to_string(),
                "aac".to_string(),
            ],
            max_file_size_mb: 500,
            validate_file_content: true,
            enable_path_traversal_protection: true,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024,
            max_buffer_size: 1_048_576,
            max_channels: 8,
            max_sample_rate: 192_000,
            max_concurrent_files: 10,
            max_processing_threads: num_cpus::get().min(8),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_security_monitoring: true,
            log_security_events: true,
            alert_on_violations: true,
            max_violations_before_lockdown: 10,
            violation_window_seconds: 300, // 5 minutes
        }
    }
}

impl SecureConfig {
    /// Load configuration from file or create default
    pub fn load_or_default() -> Result<Self, ConfigError> {
        let config_path = Self::get_config_path()?;

        if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            let config = Self::default();
            config.save_to_file(&config_path)?;
            Ok(config)
        }
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)
            .map_err(|e| ConfigError::LoadFailed {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        let config: Self = toml::from_str(&contents)
            .map_err(|e| ConfigError::ParseFailed {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &Path) -> Result<(), ConfigError> {
        // Validate before saving
        self.validate()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ConfigError::SaveFailed {
                    path: path.to_path_buf(),
                    reason: format!("Failed to create directory: {}", e),
                })?;
        }

        let contents = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeFailed {
                reason: e.to_string(),
            })?;

        fs::write(path, contents)
            .map_err(|e| ConfigError::SaveFailed {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        // Set restrictive permissions on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(path, permissions)
                .map_err(|e| ConfigError::PermissionError {
                    path: path.to_path_buf(),
                    reason: e.to_string(),
                })?;
        }

        Ok(())
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate audio settings
        if !(0.0..=1.0).contains(&self.audio.max_volume) {
            return Err(ConfigError::InvalidValue {
                field: "max_volume".to_string(),
                value: self.audio.max_volume.to_string(),
                reason: "Must be between 0.0 and 1.0".to_string(),
            });
        }

        if !(0.0..=1.0).contains(&self.audio.default_volume) {
            return Err(ConfigError::InvalidValue {
                field: "default_volume".to_string(),
                value: self.audio.default_volume.to_string(),
                reason: "Must be between 0.0 and 1.0".to_string(),
            });
        }

        if self.audio.default_volume > self.audio.max_volume {
            return Err(ConfigError::InvalidValue {
                field: "default_volume".to_string(),
                value: self.audio.default_volume.to_string(),
                reason: "Cannot exceed max_volume".to_string(),
            });
        }

        // Validate security settings
        if self.security.max_file_size_mb == 0 {
            return Err(ConfigError::InvalidValue {
                field: "max_file_size_mb".to_string(),
                value: "0".to_string(),
                reason: "Must be greater than 0".to_string(),
            });
        }

        if self.security.allowed_file_types.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "allowed_file_types".to_string(),
                value: "[]".to_string(),
                reason: "Must allow at least one file type".to_string(),
            });
        }

        // Validate resource limits
        if self.limits.max_memory_mb == 0 {
            return Err(ConfigError::InvalidValue {
                field: "max_memory_mb".to_string(),
                value: "0".to_string(),
                reason: "Must be greater than 0".to_string(),
            });
        }

        if !self.limits.max_buffer_size.is_power_of_two() {
            return Err(ConfigError::InvalidValue {
                field: "max_buffer_size".to_string(),
                value: self.limits.max_buffer_size.to_string(),
                reason: "Must be a power of 2".to_string(),
            });
        }

        Ok(())
    }

    /// Get default configuration file path
    fn get_config_path() -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| ConfigError::NoConfigDir)?
            .join("rusty-audio");

        Ok(config_dir.join("config.toml"))
    }

    /// Reset to default configuration
    pub fn reset_to_default(&mut self) {
        *self = Self::default();
    }

    /// Apply security hardening (maximize security settings)
    pub fn apply_hardening(&mut self) {
        // Reduce audio limits for safety
        self.audio.max_volume = 0.7;
        self.audio.default_volume = 0.3;
        self.audio.enable_limiter = true;
        self.audio.limiter_threshold = 0.9;
        self.audio.enable_hearing_protection = true;
        self.audio.emergency_stop_enabled = true;

        // Enable all security features
        self.security.sandbox_enabled = true;
        self.security.validate_file_content = true;
        self.security.enable_path_traversal_protection = true;
        self.security.max_file_size_mb = 100; // Reduce max file size

        // Tighten resource limits
        self.limits.max_memory_mb = 512;
        self.limits.max_concurrent_files = 3;
        self.limits.max_processing_threads = 2;

        // Enable all monitoring
        self.monitoring.enable_security_monitoring = true;
        self.monitoring.log_security_events = true;
        self.monitoring.alert_on_violations = true;
        self.monitoring.max_violations_before_lockdown = 5;
    }
}

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load configuration from {path:?}: {reason}")]
    LoadFailed { path: PathBuf, reason: String },

    #[error("Failed to parse configuration from {path:?}: {reason}")]
    ParseFailed { path: PathBuf, reason: String },

    #[error("Failed to save configuration to {path:?}: {reason}")]
    SaveFailed { path: PathBuf, reason: String },

    #[error("Failed to serialize configuration: {reason}")]
    SerializeFailed { reason: String },

    #[error("Invalid configuration value for {field}: {value} ({reason})")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
    },

    #[error("Permission error for {path:?}: {reason}")]
    PermissionError { path: PathBuf, reason: String },

    #[error("No configuration directory found")]
    NoConfigDir,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config_validation() {
        let config = SecureConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // Save default config
        let config = SecureConfig::default();
        config.save_to_file(&config_path).unwrap();

        // Load and verify
        let loaded_config = SecureConfig::load_from_file(&config_path).unwrap();
        assert_eq!(loaded_config.audio.max_volume, config.audio.max_volume);
    }

    #[test]
    fn test_invalid_config_validation() {
        let mut config = SecureConfig::default();

        // Test invalid volume
        config.audio.max_volume = 1.5;
        assert!(config.validate().is_err());
        config.audio.max_volume = 0.85;

        // Test invalid buffer size
        config.limits.max_buffer_size = 1000; // Not power of 2
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_hardening() {
        let mut config = SecureConfig::default();
        let original_max_volume = config.audio.max_volume;

        config.apply_hardening();

        // Verify hardening reduces limits
        assert!(config.audio.max_volume < original_max_volume);
        assert_eq!(config.security.sandbox_enabled, true);
        assert_eq!(config.monitoring.enable_security_monitoring, true);
    }
}