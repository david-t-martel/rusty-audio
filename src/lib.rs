// Rusty Audio Library
//
// This library provides comprehensive mathematical testing framework
// and audio processing utilities for verification and benchmarking.

// Mathematical testing framework
pub mod testing;

// Basic mathematical functions for standalone testing
/// Calculate RMS (Root Mean Square) of a signal
pub fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

/// Calculate peak amplitude of a signal
pub fn calculate_peak(samples: &[f32]) -> f32 {
    samples.iter().map(|&x| x.abs()).fold(0.0, f32::max)
}

// Basic test result structure
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub expected: f32,
    pub actual: f32,
    pub tolerance: f32,
    pub error_magnitude: f32,
}

impl TestResult {
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

// UI modules for enhanced interface components
pub mod ui;
// Performance optimization modules
pub mod audio_performance;
pub mod audio_optimizations;
pub mod performance_monitor;
// Security modules for safe audio processing
pub mod security;
// Audio engine and metadata modules
pub mod audio_engine;
pub mod metadata;
pub mod error;
// AI-enhanced audio processing modules
pub mod ai;
// Native audio backend (Phase 3)
pub mod audio;
// Async audio loading (Phase 1.4)
pub mod async_audio_loader;
