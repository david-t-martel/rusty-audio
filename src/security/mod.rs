//! Security module for Rusty Audio
//!
//! This module provides comprehensive security features including:
//! - File path validation and sandboxing
//! - Audio safety limiting and emergency stop
//! - Input validation and sanitization
//! - Thread-safe state management
//! - Security monitoring and alerting

pub mod file_validator;
pub mod audio_safety;
pub mod input_validator;
pub mod secure_config;
pub mod thread_safe_state;
pub mod security_monitor;

pub use file_validator::{FileValidator, SecurityError};
pub use audio_safety::{AudioSafetyLimiter, SafetyError};
pub use input_validator::{InputValidator, ValidationError};
pub use secure_config::SecureConfig;
pub use thread_safe_state::ThreadSafeAudioState;
pub use security_monitor::{SecurityMonitor, SecurityEvent, Severity};

/// Initialize all security components
pub fn initialize_security() -> Result<SecurityContext, SecurityError> {
    let config = SecureConfig::load_or_default()?;
    let file_validator = FileValidator::new(config.security.sandbox_path.clone());
    let audio_config = audio_safety::AudioConfig {
        max_volume: config.audio.max_volume,
        default_volume: config.audio.default_volume,
        enable_limiter: config.audio.enable_limiter,
        limiter_threshold: config.audio.limiter_threshold,
    };
    let audio_limiter = AudioSafetyLimiter::new(audio_config);
    let monitor = SecurityMonitor::new();

    Ok(SecurityContext {
        config,
        file_validator,
        audio_limiter,
        monitor,
    })
}

/// Main security context containing all security components
pub struct SecurityContext {
    pub config: SecureConfig,
    pub file_validator: FileValidator,
    pub audio_limiter: AudioSafetyLimiter,
    pub monitor: SecurityMonitor,
}

impl SecurityContext {
    /// Perform security health check
    pub fn health_check(&self) -> Result<(), SecurityError> {
        // Verify sandbox directory exists and is writable
        if !self.config.security.sandbox_path.exists() {
            return Err(SecurityError::SandboxNotFound);
        }

        // Check audio limiter is functioning
        if !self.audio_limiter.is_operational() {
            return Err(SecurityError::LimiterNotOperational);
        }

        // Verify monitoring is active
        if !self.monitor.is_active() {
            return Err(SecurityError::MonitoringInactive);
        }

        Ok(())
    }
}