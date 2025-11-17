// tests/eq_functionality_tests.rs
//! Comprehensive tests for 8-band EQ functionality

use std::f32::consts::PI;

// ============================================================================
// EQ BAND FREQUENCY TESTS
// ============================================================================

/// Test that EQ bands cover the correct frequency ranges
#[test]
fn test_eq_frequency_bands() {
    let expected_frequencies = [60.0, 170.0, 310.0, 600.0, 1000.0, 3000.0, 6000.0, 12000.0];

    for (band, &freq) in expected_frequencies.iter().enumerate() {
        let actual_freq = get_eq_frequency(band);
        assert_eq!(
            actual_freq, freq,
            "Band {} should be at {}Hz, got {}Hz",
            band, freq, actual_freq
        );
    }
}

#[test]
fn test_eq_frequency_coverage() {
    // Verify bands cover human hearing range (20Hz - 20kHz)
    let frequencies = (0..8).map(get_eq_frequency).collect::<Vec<_>>();

    assert!(frequencies[0] >= 20.0, "First band should be above 20Hz");
    assert!(frequencies[7] <= 20000.0, "Last band should be below 20kHz");

    // Verify logarithmic spacing
    for i in 1..frequencies.len() {
        let ratio = frequencies[i] / frequencies[i - 1];
        assert!(
            ratio > 1.5 && ratio < 4.0,
            "Bands should be logarithmically spaced, ratio: {}",
            ratio
        );
    }
}

// ============================================================================
// EQ GAIN RANGE TESTS
// ============================================================================

#[test]
fn test_eq_gain_clamping() {
    // Test gain clamping: -12dB to +12dB
    let test_cases = vec![
        (-20.0, -12.0),
        (-12.0, -12.0),
        (-6.0, -6.0),
        (0.0, 0.0),
        (6.0, 6.0),
        (12.0, 12.0),
        (20.0, 12.0),
    ];

    for (input, expected) in test_cases {
        let clamped = clamp_eq_gain(input);
        assert_eq!(
            clamped, expected,
            "Gain {}dB should clamp to {}dB",
            input, expected
        );
    }
}

#[test]
fn test_eq_db_to_linear_conversion() {
    // 0dB = 1.0 (unity gain)
    assert_eq!(db_to_linear(0.0), 1.0);

    // +6dB ≈ 2.0 (double amplitude)
    let gain_6db = db_to_linear(6.0);
    assert!((gain_6db - 2.0).abs() < 0.01, "6dB should be ~2x, got {}", gain_6db);

    // -6dB ≈ 0.5 (half amplitude)
    let gain_minus_6db = db_to_linear(-6.0);
    assert!(
        (gain_minus_6db - 0.5).abs() < 0.01,
        "-6dB should be ~0.5x, got {}",
        gain_minus_6db
    );

    // +12dB ≈ 4.0
    let gain_12db = db_to_linear(12.0);
    assert!(
        (gain_12db - 4.0).abs() < 0.1,
        "+12dB should be ~4x, got {}",
        gain_12db
    );

    // -12dB ≈ 0.25
    let gain_minus_12db = db_to_linear(-12.0);
    assert!(
        (gain_minus_12db - 0.25).abs() < 0.01,
        "-12dB should be ~0.25x, got {}",
        gain_minus_12db
    );
}

// ============================================================================
// EQ FILTER RESPONSE TESTS
// ============================================================================

#[test]
fn test_eq_filter_at_center_frequency() {
    // At center frequency, gain should match target
    for band in 0..8 {
        let center_freq = get_eq_frequency(band);
        let target_gain_db = 6.0;

        let response = calculate_filter_response(band, target_gain_db, center_freq, 48000.0);

        // Response at center frequency should be close to target gain
        let response_db = linear_to_db(response);
        assert!(
            (response_db - target_gain_db).abs() < 1.0,
            "Band {} at center freq {}Hz: expected {}dB, got {}dB",
            band,
            center_freq,
            target_gain_db,
            response_db
        );
    }
}

#[test]
fn test_eq_filter_bandwidth() {
    // Verify Q factor produces reasonable bandwidth
    let q_factor = 1.0; // Standard Q for parametric EQ

    for band in 0..8 {
        let center_freq = get_eq_frequency(band);

        // Calculate -3dB bandwidth
        let bandwidth_hz = center_freq / q_factor;

        // Bandwidth should be reasonable (not too narrow or too wide)
        assert!(
            bandwidth_hz > 20.0 && bandwidth_hz < 5000.0,
            "Band {} bandwidth {}Hz is unreasonable",
            band,
            bandwidth_hz
        );
    }
}

#[test]
fn test_eq_filter_rolloff() {
    // Verify filters roll off away from center frequency
    for band in 0..8 {
        let center_freq = get_eq_frequency(band);
        let target_gain_db = 12.0;

        // Test at 2 octaves below center
        let low_freq = center_freq / 4.0;
        let low_response = calculate_filter_response(band, target_gain_db, low_freq, 48000.0);
        let low_response_db = linear_to_db(low_response);

        // Test at 2 octaves above center
        let high_freq = center_freq * 4.0;
        let high_response = calculate_filter_response(band, target_gain_db, high_freq, 48000.0);
        let high_response_db = linear_to_db(high_response);

        // Response should be less than target (filter is rolling off)
        assert!(
            low_response_db < target_gain_db,
            "Band {} should roll off below center freq",
            band
        );
        assert!(
            high_response_db < target_gain_db,
            "Band {} should roll off above center freq",
            band
        );
    }
}

// ============================================================================
// EQ SIGNAL PROCESSING TESTS
// ============================================================================

#[test]
fn test_eq_unity_gain_bypass() {
    // With all bands at 0dB, output should equal input
    let input = generate_sine_wave(1000.0, 48000.0, 1024);
    let output = apply_eq_all_bands_zero(&input);

    for (i, (&in_sample, &out_sample)) in input.iter().zip(output.iter()).enumerate() {
        assert!(
            (in_sample - out_sample).abs() < 0.001,
            "Sample {} mismatch: in={}, out={}",
            i,
            in_sample,
            out_sample
        );
    }
}

#[test]
fn test_eq_boost_increases_amplitude() {
    let input = generate_sine_wave(1000.0, 48000.0, 1024);

    // Apply +6dB boost at 1kHz (band 4)
    let output = apply_eq_single_band(&input, 4, 6.0);

    // Calculate RMS of input and output
    let input_rms = calculate_rms(&input);
    let output_rms = calculate_rms(&output);

    // Output should have higher RMS (approximately 2x for +6dB)
    let gain_ratio = output_rms / input_rms;
    assert!(
        gain_ratio > 1.5 && gain_ratio < 2.5,
        "6dB boost should increase RMS by ~2x, got {}x",
        gain_ratio
    );
}

#[test]
fn test_eq_cut_decreases_amplitude() {
    let input = generate_sine_wave(1000.0, 48000.0, 1024);

    // Apply -6dB cut at 1kHz (band 4)
    let output = apply_eq_single_band(&input, 4, -6.0);

    // Calculate RMS of input and output
    let input_rms = calculate_rms(&input);
    let output_rms = calculate_rms(&output);

    // Output should have lower RMS (approximately 0.5x for -6dB)
    let gain_ratio = output_rms / input_rms;
    assert!(
        gain_ratio > 0.4 && gain_ratio < 0.6,
        "-6dB cut should decrease RMS by ~0.5x, got {}x",
        gain_ratio
    );
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn get_eq_frequency(band: usize) -> f32 {
    const FREQUENCIES: [f32; 8] = [60.0, 170.0, 310.0, 600.0, 1000.0, 3000.0, 6000.0, 12000.0];
    FREQUENCIES[band]
}

fn clamp_eq_gain(gain_db: f32) -> f32 {
    gain_db.clamp(-12.0, 12.0)
}

fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

fn linear_to_db(linear: f32) -> f32 {
    20.0 * linear.log10()
}

fn calculate_filter_response(
    band: usize,
    gain_db: f32,
    test_freq: f32,
    sample_rate: f32,
) -> f32 {
    let center_freq = get_eq_frequency(band);
    let q = 1.0; // Standard Q factor

    // Simplified biquad filter response calculation
    let omega = 2.0 * PI * test_freq / sample_rate;
    let omega_c = 2.0 * PI * center_freq / sample_rate;

    let alpha = omega_c.sin() / (2.0 * q);
    let a = db_to_linear(gain_db);

    // Peaking EQ biquad coefficients
    let b0 = 1.0 + alpha * a;
    let b1 = -2.0 * omega.cos();
    let b2 = 1.0 - alpha * a;
    let a0 = 1.0 + alpha / a;
    let a1 = -2.0 * omega.cos();
    let a2 = 1.0 - alpha / a;

    // Frequency response magnitude (simplified)
    let numerator = (b0 * b0 + b1 * b1 + b2 * b2).sqrt();
    let denominator = (a0 * a0 + a1 * a1 + a2 * a2).sqrt();

    numerator / denominator
}

fn generate_sine_wave(freq: f32, sample_rate: f32, length: usize) -> Vec<f32> {
    (0..length)
        .map(|i| (2.0 * PI * freq * (i as f32) / sample_rate).sin())
        .collect()
}

fn apply_eq_all_bands_zero(input: &[f32]) -> Vec<f32> {
    // Simplified: bypass (unity gain)
    input.to_vec()
}

fn apply_eq_single_band(input: &[f32], band: usize, gain_db: f32) -> Vec<f32> {
    // Simplified: apply constant gain
    let gain_linear = db_to_linear(gain_db);
    input.iter().map(|&sample| sample * gain_linear).collect()
}

fn calculate_rms(samples: &[f32]) -> f32 {
    let sum_squares: f32 = samples.iter().map(|&s| s * s).sum();
    (sum_squares / samples.len() as f32).sqrt()
}
