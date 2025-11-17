// Rusty Audio Core Library
//
// This library provides comprehensive audio processing, UI components, effects,
// and utilities shared between desktop and WASM applications.
//
// Architecture:
// - Platform-agnostic core functionality
// - Feature-gated platform-specific code
// - Modular design with clear separation of concerns

#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![cfg_attr(not(test), warn(clippy::unwrap_used, clippy::expect_used))]

//! # Rusty Audio Core
//!
//! Core library for the Rusty Audio application, providing:
//! - Audio processing and DSP
//! - UI components and themes
//! - Effects and equalizer
//! - AI-enhanced features
//! - Security and validation
//! - Testing utilities

// ============================================================================
// Mathematical Testing Framework
// ============================================================================

/// Mathematical testing framework for audio signal verification
pub mod testing;

/// Calculate RMS (Root Mean Square) of a signal
///
/// # Arguments
/// * `samples` - Audio samples to analyze
///
/// # Returns
/// RMS value of the signal
pub fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

/// Calculate peak amplitude of a signal
///
/// # Arguments
/// * `samples` - Audio samples to analyze
///
/// # Returns
/// Peak amplitude value
pub fn calculate_peak(samples: &[f32]) -> f32 {
    samples.iter().map(|&x| x.abs()).fold(0.0, f32::max)
}

/// Test result structure for verification
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Name of the test
    pub test_name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Expected value
    pub expected: f32,
    /// Actual value
    pub actual: f32,
    /// Tolerance for comparison
    pub tolerance: f32,
    /// Error magnitude
    pub error_magnitude: f32,
}

impl TestResult {
    /// Create a new test result
    ///
    /// # Arguments
    /// * `name` - Test name
    /// * `expected` - Expected value
    /// * `actual` - Actual value
    /// * `tolerance` - Acceptable tolerance
    pub fn new(name: &str, expected: f32, actual: f32, tolerance: f32) -> Self {
        let error_magnitude = (expected - actual).abs();
        let passed = error_magnitude <= tolerance;

        Self {
            test_name: name.to_string(),
            passed,
            expected,
            actual,
            tolerance,
            error_magnitude,
        }
    }
}

// ============================================================================
// Core Modules (Platform-Agnostic)
// ============================================================================

/// UI components and themes
pub mod ui;

/// Performance optimization modules
pub mod audio_optimizations;
/// Audio performance monitoring
pub mod audio_performance;
/// Performance monitor utilities
pub mod performance_monitor;

/// Security modules for safe audio processing
pub mod security;

/// Error handling and reporting
pub mod error;

/// Audio metadata extraction and management
pub mod metadata;

/// AI-enhanced audio processing modules
pub mod ai;

/// Audio backend abstraction (supports both native and WASM)
pub mod audio;

/// Integrated audio manager (supports both native and WASM)
pub mod integrated_audio_manager;

/// Optimized audio performance modules
pub mod audio_performance_optimized;

/// Platform abstraction layer
pub mod platform;

// ============================================================================
// Native-Only Modules
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
/// Audio engine (native only - uses web_audio_api)
pub mod audio_engine;

#[cfg(not(target_arch = "wasm32"))]
/// Async audio loading (native only)
pub mod async_audio_loader;

#[cfg(not(target_arch = "wasm32"))]
/// Audio pipeline integration (native only - uses web_audio_api)
pub mod audio_pipeline_integration;

#[cfg(not(target_arch = "wasm32"))]
/// Performance integration layer (native only - uses web_audio_api::AnalyserNode)
pub mod audio_performance_integration;

// ============================================================================
// WASM-Only Modules
// ============================================================================

#[cfg(target_arch = "wasm32")]
/// WASM web integration
pub mod web;

#[cfg(target_arch = "wasm32")]
/// Refactored WASM web module
pub mod web_refactored;

#[cfg(target_arch = "wasm32")]
/// WASM panic handler
pub mod wasm_panic_handler;

// ============================================================================
// Re-exports for Convenience
// ============================================================================

// Re-export commonly used types
pub use error::{AudioError, Result};
pub use metadata::TrackMetadata;

// Re-export audio types
pub use audio::{AudioBackend, AudioBackendError};

// Re-export UI types
pub use ui::theme::{Theme as RustyAudioTheme, ThemeColors as ThemeColor};

// Re-export security types
pub use security::{AudioSafetyLimiter, FileValidator, InputValidator};

// Re-export AI types (when ai-features is enabled)
#[cfg(feature = "ai-features")]
pub use ai::{AudioAnalyzer, EqOptimizer, NoiseReduction};

// ============================================================================
// Prelude Module
// ============================================================================

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::audio::{AudioBackend, AudioBackendError};
    pub use crate::error::{AudioError, Result};
    pub use crate::metadata::TrackMetadata;
    pub use crate::security::{AudioSafetyLimiter, FileValidator, InputValidator};
    pub use crate::ui::theme::{Theme as RustyAudioTheme, ThemeColors as ThemeColor};

    #[cfg(feature = "ai-features")]
    pub use crate::ai::{AudioAnalyzer, EqOptimizer, NoiseReduction};
}

// ============================================================================
// Library Information
// ============================================================================

/// Get library version
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get library name
pub fn name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// Get library description
pub fn description() -> &'static str {
    env!("CARGO_PKG_DESCRIPTION")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_rms_empty() {
        let samples: Vec<f32> = vec![];
        assert_eq!(calculate_rms(&samples), 0.0);
    }

    #[test]
    fn test_calculate_rms_single() {
        let samples = vec![1.0];
        assert_eq!(calculate_rms(&samples), 1.0);
    }

    #[test]
    fn test_calculate_peak_empty() {
        let samples: Vec<f32> = vec![];
        assert_eq!(calculate_peak(&samples), 0.0);
    }

    #[test]
    fn test_calculate_peak_single() {
        let samples = vec![0.5];
        assert_eq!(calculate_peak(&samples), 0.5);
    }

    #[test]
    fn test_version_info() {
        assert!(!version().is_empty());
        assert!(!name().is_empty());
        assert!(!description().is_empty());
    }
}
