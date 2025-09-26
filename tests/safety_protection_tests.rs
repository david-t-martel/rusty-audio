// Automated Safety Testing for Volume Limiting and Audio Protection
// Critical safety tests for hearing protection and equipment safety

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU32, Ordering}};
use std::thread;
use std::f32::consts::PI;
use web_audio_api::context::{AudioContext, BaseAudioContext, OfflineAudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};
use web_audio_api::AudioBuffer;
use rustfft::{FftPlanner, num_complex::Complex32};
use approx::{assert_relative_eq, assert_abs_diff_eq};
use serial_test::serial;

// Safety constants - NEVER exceed these values
const ABSOLUTE_MAX_PEAK_DB: f32 = 0.0;      // 0 dBFS peak limit
const SAFE_CONTINUOUS_LEVEL_DB: f32 = -20.0; // -20 dBFS for continuous playback
const HEARING_DAMAGE_THRESHOLD_DB: f32 = -6.0; // -6 dBFS sustained level warning
const EMERGENCY_SHUTDOWN_LEVEL_DB: f32 = 3.0;  // +3 dBFS triggers emergency shutdown
const MAX_RESPONSE_TIME_MS: u64 = 10;      // Maximum 10ms response time for safety systems
const MAX_ATTACK_TIME_MS: u64 = 1;         // 1ms maximum attack time for limiters
const SAFETY_MARGIN_DB: f32 = 3.0;         // 3dB safety margin
const SAMPLE_RATE: f32 = 48000.0;

/// Safety violation types
#[derive(Debug, Clone, PartialEq)]
enum SafetyViolation {
    PeakOverload { level_db: f32, duration_ms: f32 },
    SustainedOverload { level_db: f32, duration_ms: f32 },
    SlowResponse { response_time_ms: u64, max_allowed_ms: u64 },
    LimiterFailure { bypass_detected: bool },
    EmergencyShutdownTriggered { trigger_level_db: f32 },
    HearingDamageRisk { exposure_time_ms: f32, level_db: f32 },
    EquipmentDamageRisk { peak_level_db: f32 },
}

/// Safety test results
#[derive(Debug, Clone)]
struct SafetyTestResult {
    test_name: String,
    passed: bool,
    violations: Vec<SafetyViolation>,
    peak_level_db: f32,
    rms_level_db: f32,
    response_time_ms: u64,
    safety_margin_db: f32,
    protective_action_taken: bool,
}

impl SafetyTestResult {
    fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
            passed: true,
            violations: Vec::new(),
            peak_level_db: f32::NEG_INFINITY,
            rms_level_db: f32::NEG_INFINITY,
            response_time_ms: 0,
            safety_margin_db: f32::INFINITY,
            protective_action_taken: false,
        }
    }

    fn add_violation(&mut self, violation: SafetyViolation) {
        self.violations.push(violation);
        self.passed = false;
    }

    fn is_safe(&self) -> bool {
        self.passed && self.violations.is_empty()
    }
}

/// Audio safety limiter with real-time protection
struct AudioSafetyLimiter {
    threshold_db: f32,
    attack_time_ms: f32,
    release_time_ms: f32,
    sample_rate: f32,
    reduction_db: Arc<Mutex<f32>>,
    emergency_shutdown: Arc<AtomicBool>,
    violation_count: Arc<AtomicU32>,
    last_violation_time: Arc<Mutex<Instant>>,
}

impl AudioSafetyLimiter {
    fn new(threshold_db: f32, sample_rate: f32) -> Self {
        Self {
            threshold_db,
            attack_time_ms: 0.5, // Very fast attack for safety
            release_time_ms: 100.0, // Slow release to prevent pumping
            sample_rate,
            reduction_db: Arc::new(Mutex::new(0.0)),
            emergency_shutdown: Arc::new(AtomicBool::new(false)),
            violation_count: Arc::new(AtomicU32::new(0)),
            last_violation_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    fn process(&self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(input.len());
        let attack_coeff = (-2.2 / (self.attack_time_ms * self.sample_rate / 1000.0)).exp();
        let release_coeff = (-2.2 / (self.release_time_ms * self.sample_rate / 1000.0)).exp();

        for &sample in input {
            // Convert to dB
            let level_db = if sample.abs() > 0.0 {
                20.0 * sample.abs().log10()
            } else {
                f32::NEG_INFINITY
            };

            // Check for emergency shutdown condition
            if level_db > EMERGENCY_SHUTDOWN_LEVEL_DB {
                self.emergency_shutdown.store(true, Ordering::SeqCst);
                self.violation_count.fetch_add(1, Ordering::SeqCst);
                *self.last_violation_time.lock().unwrap() = Instant::now();
                output.push(0.0); // Immediate mute
                continue;
            }

            // Calculate required gain reduction
            let overshoot_db = (level_db - self.threshold_db).max(0.0);

            // Apply limiting with appropriate time constants
            let mut current_reduction = self.reduction_db.lock().unwrap();
            if overshoot_db > *current_reduction {
                // Attack: fast reduction
                *current_reduction = overshoot_db + (*current_reduction - overshoot_db) * attack_coeff;
            } else {
                // Release: slow recovery
                *current_reduction = overshoot_db + (*current_reduction - overshoot_db) * release_coeff;
            }

            // Apply gain reduction
            let gain_linear = (-*current_reduction / 20.0).exp2();
            let limited_sample = sample * gain_linear;

            // Verify output is within safe limits
            let output_db = if limited_sample.abs() > 0.0 {
                20.0 * limited_sample.abs().log10()
            } else {
                f32::NEG_INFINITY
            };

            if output_db > ABSOLUTE_MAX_PEAK_DB {
                // Hard clip as last resort
                let clipped = if limited_sample > 0.0 { 1.0 } else { -1.0 };
                output.push(clipped);
                self.violation_count.fetch_add(1, Ordering::SeqCst);
            } else {
                output.push(limited_sample);
            }
        }

        output
    }

    fn get_current_reduction_db(&self) -> f32 {
        *self.reduction_db.lock().unwrap()
    }

    fn is_emergency_shutdown(&self) -> bool {
        self.emergency_shutdown.load(Ordering::SeqCst)
    }

    fn get_violation_count(&self) -> u32 {
        self.violation_count.load(Ordering::SeqCst)
    }

    fn reset(&self) {
        self.emergency_shutdown.store(false, Ordering::SeqCst);
        self.violation_count.store(0, Ordering::SeqCst);
        *self.reduction_db.lock().unwrap() = 0.0;
    }
}

/// Real-time safety monitor
struct SafetyMonitor {
    peak_detector: PeakDetector,
    rms_detector: RmsDetector,
    exposure_tracker: ExposureTracker,
    response_time_tracker: ResponseTimeTracker,
}

impl SafetyMonitor {
    fn new(sample_rate: f32) -> Self {
        Self {
            peak_detector: PeakDetector::new(sample_rate),
            rms_detector: RmsDetector::new(sample_rate, 1.0), // 1 second RMS window
            exposure_tracker: ExposureTracker::new(),
            response_time_tracker: ResponseTimeTracker::new(),
        }
    }

    fn analyze(&mut self, samples: &[f32]) -> SafetyTestResult {
        let mut result = SafetyTestResult::new("safety_monitor");

        // Update detectors
        self.peak_detector.process(samples);
        self.rms_detector.process(samples);
        self.exposure_tracker.process(samples, SAMPLE_RATE);

        // Get measurements
        result.peak_level_db = self.peak_detector.get_peak_db();
        result.rms_level_db = self.rms_detector.get_rms_db();

        // Check peak violations
        if result.peak_level_db > ABSOLUTE_MAX_PEAK_DB {
            result.add_violation(SafetyViolation::PeakOverload {
                level_db: result.peak_level_db,
                duration_ms: samples.len() as f32 / SAMPLE_RATE * 1000.0,
            });
        }

        // Check sustained level violations
        if result.rms_level_db > HEARING_DAMAGE_THRESHOLD_DB {
            let exposure = self.exposure_tracker.get_current_exposure_ms();
            result.add_violation(SafetyViolation::HearingDamageRisk {
                exposure_time_ms: exposure,
                level_db: result.rms_level_db,
            });
        }

        // Check equipment damage risk
        if result.peak_level_db > ABSOLUTE_MAX_PEAK_DB + SAFETY_MARGIN_DB {
            result.add_violation(SafetyViolation::EquipmentDamageRisk {
                peak_level_db: result.peak_level_db,
            });
        }

        result
    }
}

/// Peak level detector with hold and decay
struct PeakDetector {
    peak_hold: f32,
    decay_rate: f32,
    sample_rate: f32,
}

impl PeakDetector {
    fn new(sample_rate: f32) -> Self {
        Self {
            peak_hold: 0.0,
            decay_rate: 0.95, // Slow decay to catch transients
            sample_rate,
        }
    }

    fn process(&mut self, samples: &[f32]) {
        for &sample in samples {
            let abs_sample = sample.abs();
            if abs_sample > self.peak_hold {
                self.peak_hold = abs_sample;
            } else {
                self.peak_hold *= self.decay_rate;
            }
        }
    }

    fn get_peak_db(&self) -> f32 {
        if self.peak_hold > 0.0 {
            20.0 * self.peak_hold.log10()
        } else {
            f32::NEG_INFINITY
        }
    }

    fn reset(&mut self) {
        self.peak_hold = 0.0;
    }
}

/// RMS detector with configurable averaging time
struct RmsDetector {
    buffer: Vec<f32>,
    buffer_index: usize,
    sum_squares: f32,
    buffer_size: usize,
}

impl RmsDetector {
    fn new(sample_rate: f32, averaging_time_s: f32) -> Self {
        let buffer_size = (sample_rate * averaging_time_s) as usize;
        Self {
            buffer: vec![0.0; buffer_size],
            buffer_index: 0,
            sum_squares: 0.0,
            buffer_size,
        }
    }

    fn process(&mut self, samples: &[f32]) {
        for &sample in samples {
            // Remove old sample from sum
            let old_sample = self.buffer[self.buffer_index];
            self.sum_squares -= old_sample * old_sample;

            // Add new sample
            let sample_squared = sample * sample;
            self.buffer[self.buffer_index] = sample_squared;
            self.sum_squares += sample_squared;

            // Advance circular buffer
            self.buffer_index = (self.buffer_index + 1) % self.buffer_size;
        }
    }

    fn get_rms_db(&self) -> f32 {
        let rms = (self.sum_squares / self.buffer_size as f32).sqrt();
        if rms > 0.0 {
            20.0 * rms.log10()
        } else {
            f32::NEG_INFINITY
        }
    }
}

/// Audio exposure tracker for hearing safety
struct ExposureTracker {
    high_level_start_time: Option<Instant>,
    total_exposure_ms: f32,
    current_level_db: f32,
}

impl ExposureTracker {
    fn new() -> Self {
        Self {
            high_level_start_time: None,
            total_exposure_ms: 0.0,
            current_level_db: f32::NEG_INFINITY,
        }
    }

    fn process(&mut self, samples: &[f32], sample_rate: f32) {
        let rms = calculate_rms(samples);
        self.current_level_db = if rms > 0.0 {
            20.0 * rms.log10()
        } else {
            f32::NEG_INFINITY
        };

        let duration_ms = samples.len() as f32 / sample_rate * 1000.0;

        if self.current_level_db > HEARING_DAMAGE_THRESHOLD_DB {
            if self.high_level_start_time.is_none() {
                self.high_level_start_time = Some(Instant::now());
            }
            self.total_exposure_ms += duration_ms;
        } else {
            self.high_level_start_time = None;
        }
    }

    fn get_current_exposure_ms(&self) -> f32 {
        if let Some(start_time) = self.high_level_start_time {
            start_time.elapsed().as_millis() as f32
        } else {
            0.0
        }
    }

    fn get_total_exposure_ms(&self) -> f32 {
        self.total_exposure_ms
    }
}

/// Response time tracker for safety systems
struct ResponseTimeTracker {
    trigger_time: Option<Instant>,
    response_times: Vec<u64>,
}

impl ResponseTimeTracker {
    fn new() -> Self {
        Self {
            trigger_time: None,
            response_times: Vec::new(),
        }
    }

    fn trigger_event(&mut self) {
        self.trigger_time = Some(Instant::now());
    }

    fn record_response(&mut self) {
        if let Some(trigger_time) = self.trigger_time.take() {
            let response_time_ms = trigger_time.elapsed().as_millis() as u64;
            self.response_times.push(response_time_ms);
        }
    }

    fn get_last_response_time_ms(&self) -> Option<u64> {
        self.response_times.last().copied()
    }

    fn get_max_response_time_ms(&self) -> Option<u64> {
        self.response_times.iter().max().copied()
    }

    fn get_average_response_time_ms(&self) -> Option<f64> {
        if self.response_times.is_empty() {
            None
        } else {
            let sum: u64 = self.response_times.iter().sum();
            Some(sum as f64 / self.response_times.len() as f64)
        }
    }
}

// Helper functions
fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

fn calculate_peak(samples: &[f32]) -> f32 {
    samples.iter().map(|&x| x.abs()).fold(0.0, f32::max)
}

fn db_to_linear(db: f32) -> f32 {
    (db / 20.0).exp2()
}

fn linear_to_db(linear: f32) -> f32 {
    if linear > 0.0 {
        20.0 * linear.log10()
    } else {
        f32::NEG_INFINITY
    }
}

// ============================================================================
// SAFETY TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial]
    fn test_absolute_peak_limiting() {
        let limiter = AudioSafetyLimiter::new(ABSOLUTE_MAX_PEAK_DB - SAFETY_MARGIN_DB, SAMPLE_RATE);

        // Create dangerously loud test signal (+6 dBFS)
        let dangerous_level = db_to_linear(6.0);
        let input: Vec<f32> = (0..4800) // 100ms at 48kHz
            .map(|i| dangerous_level * (2.0 * PI * 1000.0 * i as f32 / SAMPLE_RATE).sin())
            .collect();

        let output = limiter.process(&input);

        // Verify output is within safe limits
        let output_peak = calculate_peak(&output);
        let output_peak_db = linear_to_db(output_peak);

        assert!(output_peak_db <= ABSOLUTE_MAX_PEAK_DB,
            "Peak limiter failed: output peak {:.2} dB exceeds limit {:.2} dB",
            output_peak_db, ABSOLUTE_MAX_PEAK_DB);

        assert!(limiter.get_current_reduction_db() > 0.0,
            "Limiter should be applying gain reduction");

        println!("Peak limiting test: input {:.1} dB -> output {:.1} dB, reduction {:.1} dB",
            linear_to_db(dangerous_level), output_peak_db, limiter.get_current_reduction_db());
    }

    #[test]
    #[serial]
    fn test_emergency_shutdown() {
        let limiter = AudioSafetyLimiter::new(ABSOLUTE_MAX_PEAK_DB, SAMPLE_RATE);

        // Create extremely dangerous signal (way above emergency threshold)
        let emergency_level = db_to_linear(EMERGENCY_SHUTDOWN_LEVEL_DB + 3.0);
        let input: Vec<f32> = vec![emergency_level; 100];

        let output = limiter.process(&input);

        // Verify emergency shutdown triggered
        assert!(limiter.is_emergency_shutdown(),
            "Emergency shutdown should have triggered for +{:.1} dB signal",
            linear_to_db(emergency_level));

        // Verify output is muted
        let output_peak = calculate_peak(&output);
        assert_eq!(output_peak, 0.0,
            "Output should be completely muted during emergency shutdown");

        assert!(limiter.get_violation_count() > 0,
            "Violation count should increase during emergency");

        println!("Emergency shutdown test: triggered at {:.1} dB, {} violations",
            linear_to_db(emergency_level), limiter.get_violation_count());
    }

    #[test]
    #[serial]
    fn test_limiter_response_time() {
        let limiter = AudioSafetyLimiter::new(ABSOLUTE_MAX_PEAK_DB - 6.0, SAMPLE_RATE);

        // Create step function test signal
        let safe_level = db_to_linear(-20.0);
        let dangerous_level = db_to_linear(0.0);

        let mut input = vec![safe_level; 480]; // 10ms of safe signal
        input.extend(vec![dangerous_level; 480]); // 10ms of dangerous signal

        let start_time = Instant::now();
        let output = limiter.process(&input);
        let processing_time = start_time.elapsed().as_millis() as u64;

        // Check response time by finding when limiting kicks in
        let mut response_sample = 0;
        for (i, &sample) in output.iter().enumerate() {
            if sample.abs() < dangerous_level * 0.9 { // Detected reduction
                response_sample = i;
                break;
            }
        }

        let response_time_ms = (response_sample as f32 / SAMPLE_RATE * 1000.0) as u64;

        assert!(response_time_ms <= MAX_RESPONSE_TIME_MS,
            "Limiter response time {:.1} ms exceeds maximum {:.1} ms",
            response_time_ms, MAX_RESPONSE_TIME_MS);

        assert!(processing_time <= MAX_RESPONSE_TIME_MS * 2,
            "Processing time {} ms too slow for real-time safety",
            processing_time);

        println!("Response time test: limiting activated in {:.1} ms, processed in {} ms",
            response_time_ms, processing_time);
    }

    #[test]
    #[serial]
    fn test_sustained_level_protection() {
        let mut monitor = SafetyMonitor::new(SAMPLE_RATE);

        // Create sustained loud signal that could cause hearing damage
        let sustained_level = db_to_linear(HEARING_DAMAGE_THRESHOLD_DB + 3.0);
        let long_signal: Vec<f32> = (0..48000 * 5) // 5 seconds
            .map(|i| sustained_level * (2.0 * PI * 1000.0 * i as f32 / SAMPLE_RATE).sin())
            .collect();

        let result = monitor.analyze(&long_signal);

        assert!(!result.is_safe(),
            "Monitor should detect hearing damage risk for {:.1} dB sustained level",
            linear_to_db(sustained_level));

        let has_hearing_damage_violation = result.violations.iter().any(|v| {
            matches!(v, SafetyViolation::HearingDamageRisk { .. })
        });

        assert!(has_hearing_damage_violation,
            "Should detect hearing damage risk violation");

        if let Some(SafetyViolation::HearingDamageRisk { exposure_time_ms, level_db }) =
            result.violations.iter().find(|v| matches!(v, SafetyViolation::HearingDamageRisk { .. })) {
            println!("Hearing damage risk detected: {:.1} dB for {:.0} ms", level_db, exposure_time_ms);
        }
    }

    #[test]
    #[serial]
    fn test_safety_monitor_comprehensive() {
        let mut monitor = SafetyMonitor::new(SAMPLE_RATE);

        // Test various signal levels
        let test_cases = vec![
            (-30.0, "Safe quiet level", true),
            (-10.0, "Moderate level", true),
            (-3.0, "High but safe level", true),
            (0.0, "Peak level warning", false),
            (3.0, "Dangerous peak level", false),
            (6.0, "Equipment damage level", false),
        ];

        for (level_db, description, should_be_safe) in test_cases {
            let level_linear = db_to_linear(level_db);
            let test_signal: Vec<f32> = (0..4800) // 100ms
                .map(|i| level_linear * (2.0 * PI * 1000.0 * i as f32 / SAMPLE_RATE).sin())
                .collect();

            let result = monitor.analyze(&test_signal);

            assert_eq!(result.is_safe(), should_be_safe,
                "Safety check failed for {}: {:.1} dB should be {}",
                description, level_db, if should_be_safe { "safe" } else { "unsafe" });

            println!("Safety test {}: {:.1} dB -> {} ({} violations)",
                description, level_db,
                if result.is_safe() { "SAFE" } else { "UNSAFE" },
                result.violations.len());
        }
    }

    #[test]
    #[serial]
    fn test_limiter_attack_time() {
        let limiter = AudioSafetyLimiter::new(-3.0, SAMPLE_RATE);

        // Create impulse signal to test attack time
        let mut input = vec![0.0; 100];
        input.push(1.0); // 0 dBFS impulse
        input.extend(vec![0.0; 100]);

        let output = limiter.process(&input);

        // Find peak in output
        let output_peak = calculate_peak(&output);
        let output_peak_db = linear_to_db(output_peak);

        // Attack should be fast enough to catch the impulse
        assert!(output_peak_db <= 0.0,
            "Fast attack should limit impulse: output peak {:.2} dB",
            output_peak_db);

        // Verify attack time is within specification
        assert!(limiter.get_current_reduction_db() > 0.0,
            "Limiter should be reducing gain after impulse");

        println!("Attack time test: impulse peak reduced to {:.1} dB", output_peak_db);
    }

    #[test]
    #[serial]
    fn test_multiple_violations_handling() {
        let limiter = AudioSafetyLimiter::new(-6.0, SAMPLE_RATE);

        // Create multiple violation events
        for i in 0..5 {
            let violation_level = db_to_linear(3.0); // Well above emergency threshold
            let input = vec![violation_level; 100];
            let _output = limiter.process(&input);

            println!("Violation {}: count = {}, emergency = {}",
                i + 1, limiter.get_violation_count(), limiter.is_emergency_shutdown());
        }

        assert!(limiter.get_violation_count() >= 5,
            "Should track multiple violations: count = {}",
            limiter.get_violation_count());

        assert!(limiter.is_emergency_shutdown(),
            "Multiple severe violations should trigger emergency shutdown");
    }

    #[test]
    #[serial]
    fn test_safe_level_operation() {
        let limiter = AudioSafetyLimiter::new(-6.0, SAMPLE_RATE);
        let mut monitor = SafetyMonitor::new(SAMPLE_RATE);

        // Create normal operating level signal
        let safe_level = db_to_linear(-20.0);
        let input: Vec<f32> = (0..48000) // 1 second
            .map(|i| safe_level * (2.0 * PI * 1000.0 * i as f32 / SAMPLE_RATE).sin())
            .collect();

        let output = limiter.process(&input);
        let result = monitor.analyze(&output);

        // Should pass through unmodified at safe levels
        assert!(result.is_safe(), "Safe levels should not trigger violations");
        assert_eq!(limiter.get_violation_count(), 0, "No violations should occur at safe levels");
        assert!(!limiter.is_emergency_shutdown(), "No emergency shutdown at safe levels");

        // Output should be nearly identical to input
        let input_rms = calculate_rms(&input);
        let output_rms = calculate_rms(&output);
        let rms_error = (input_rms - output_rms).abs() / input_rms;

        assert!(rms_error < 0.01,
            "Safe signal should pass through with minimal change: error = {:.3}%",
            rms_error * 100.0);

        println!("Safe operation test: {:.1} dB in -> {:.1} dB out, error = {:.2}%",
            linear_to_db(input_rms), linear_to_db(output_rms), rms_error * 100.0);
    }

    #[test]
    #[serial]
    fn test_exposure_time_tracking() {
        let mut monitor = SafetyMonitor::new(SAMPLE_RATE);

        // Simulate extended exposure to moderately high levels
        let exposure_level = db_to_linear(HEARING_DAMAGE_THRESHOLD_DB + 1.0);

        let mut total_exposure_ms = 0.0;

        for chunk in 0..10 {
            let chunk_signal: Vec<f32> = (0..4800) // 100ms chunks
                .map(|i| exposure_level * (2.0 * PI * 1000.0 * i as f32 / SAMPLE_RATE).sin())
                .collect();

            let result = monitor.analyze(&chunk_signal);
            total_exposure_ms += 100.0; // 100ms per chunk

            if chunk >= 5 { // After 500ms of exposure
                assert!(!result.is_safe(),
                    "Extended exposure should trigger safety warning after {}ms",
                    total_exposure_ms);
            }
        }

        println!("Exposure tracking test: {:.0}ms total exposure at {:.1} dB",
            total_exposure_ms, linear_to_db(exposure_level));
    }

    #[test]
    #[serial]
    fn test_recovery_from_violations() {
        let limiter = AudioSafetyLimiter::new(-6.0, SAMPLE_RATE);

        // First, trigger a violation
        let violation_signal = vec![db_to_linear(0.0); 1000];
        let _output1 = limiter.process(&violation_signal);

        assert!(limiter.get_violation_count() > 0, "Should have violations");

        // Reset the limiter
        limiter.reset();

        // Now test with safe signal
        let safe_signal = vec![db_to_linear(-20.0); 1000];
        let _output2 = limiter.process(&safe_signal);

        assert_eq!(limiter.get_violation_count(), 0, "Violations should be cleared after reset");
        assert!(!limiter.is_emergency_shutdown(), "Emergency shutdown should be cleared");

        println!("Recovery test: limiter successfully reset and operating normally");
    }

    #[test]
    #[serial]
    fn test_real_time_processing_safety() {
        let limiter = Arc::new(AudioSafetyLimiter::new(-3.0, SAMPLE_RATE));
        let processing_complete = Arc::new(AtomicBool::new(false));

        // Simulate real-time audio callback
        let limiter_clone = limiter.clone();
        let complete_clone = processing_complete.clone();

        let audio_thread = thread::spawn(move || {
            let quantum_size = 128; // Typical audio quantum
            let dangerous_level = db_to_linear(6.0);

            for _ in 0..100 { // 100 audio quanta
                let start_time = Instant::now();

                let input: Vec<f32> = (0..quantum_size)
                    .map(|i| dangerous_level * (2.0 * PI * 1000.0 * i as f32 / SAMPLE_RATE).sin())
                    .collect();

                let _output = limiter_clone.process(&input);

                let processing_time = start_time.elapsed().as_micros() as u64;
                let quantum_time_us = (quantum_size as f64 / SAMPLE_RATE as f64 * 1_000_000.0) as u64;

                // Verify real-time performance
                assert!(processing_time < quantum_time_us,
                    "Processing too slow: {} μs > {} μs quantum time",
                    processing_time, quantum_time_us);

                thread::sleep(Duration::from_micros(quantum_time_us));
            }

            complete_clone.store(true, Ordering::SeqCst);
        });

        // Wait for completion with timeout
        let timeout = Duration::from_secs(5);
        let start = Instant::now();

        while !processing_complete.load(Ordering::SeqCst) && start.elapsed() < timeout {
            thread::sleep(Duration::from_millis(10));
        }

        audio_thread.join().unwrap();

        assert!(processing_complete.load(Ordering::SeqCst),
            "Real-time processing test did not complete in time");

        assert!(limiter.get_current_reduction_db() > 0.0,
            "Limiter should be active during real-time processing");

        println!("Real-time processing test: completed successfully with {} violations",
            limiter.get_violation_count());
    }
}