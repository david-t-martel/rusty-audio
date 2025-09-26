// Integration test for Rusty Audio Mathematical Testing Framework

use rusty_audio::testing::{
    signal_generators::{SineGenerator, SignalGenerator},
    calculate_rms, calculate_peak,
    TestSuite, TestResult,
    SAMPLE_RATE, TOLERANCE
};

#[test]
fn test_sine_wave_generator() {
    let generator = SineGenerator::new(1000.0);
    let samples = generator.generate(1.0, SAMPLE_RATE);

    // Test sample count
    assert_eq!(samples.len(), SAMPLE_RATE as usize);

    // Test RMS value
    let rms = calculate_rms(&samples);
    let expected_rms = 1.0 / 2.0f32.sqrt(); // RMS of unit sine wave
    assert!((rms - expected_rms).abs() < TOLERANCE * 10.0);

    // Test peak value
    let peak = calculate_peak(&samples);
    assert!((peak - 1.0).abs() < TOLERANCE * 10.0);
}

#[test]
fn test_mathematical_functions() {
    let samples = vec![1.0, -1.0, 1.0, -1.0];

    let rms = calculate_rms(&samples);
    assert!((rms - 1.0).abs() < TOLERANCE);

    let peak = calculate_peak(&samples);
    assert_eq!(peak, 1.0);
}

#[test]
fn test_test_suite() {
    let mut suite = TestSuite::new();

    let result1 = TestResult::new("Test 1", 1.0, 1.0, 0.1);
    let result2 = TestResult::new("Test 2", 2.0, 2.5, 0.1); // This will fail

    suite.add_result(result1);
    suite.add_result(result2);

    assert_eq!(suite.passed_count(), 1);
    assert_eq!(suite.failed_count(), 1);
    assert_eq!(suite.success_rate(), 0.5);
}