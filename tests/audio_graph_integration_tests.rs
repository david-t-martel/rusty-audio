// tests/audio_graph_integration_tests.rs
//! Integration tests for audio routing: Source → EQ → Analyser → Output

use rusty_audio::audio::backend::{AudioBackend, BackendType};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ============================================================================
// AUDIO GRAPH ROUTING TESTS
// ============================================================================

#[test]
fn test_audio_graph_complete_chain() {
    // Test the complete audio graph: Source → EQ → Analyser → Gain → Output
    let mut backend = create_test_backend();

    // 1. Load audio file (Source)
    backend.load_file("test_assets/sine_1khz.wav").expect("Failed to load file");

    // 2. Configure EQ
    backend.set_eq_band(4, 6.0).expect("Failed to set EQ");

    // 3. Start playback
    backend.play().expect("Failed to start playback");

    // 4. Wait for audio to flow through graph
    std::thread::sleep(Duration::from_millis(100));

    // 5. Verify spectrum data is flowing (Analyser node is connected)
    let spectrum = backend.get_spectrum_data();
    assert_eq!(spectrum.len(), 512, "Spectrum should have 512 bins");

    // Spectrum should show energy at 1kHz (not all zeros)
    let max_value = spectrum.iter().copied().fold(0.0f32, f32::max);
    assert!(max_value > 0.1, "Spectrum should show signal energy");

    // 6. Verify EQ is affecting signal
    let eq_gain = backend.get_eq_band(4);
    assert_eq!(eq_gain, 6.0, "EQ gain should be preserved");
}

#[test]
fn test_source_to_eq_connection() {
    let mut backend = create_test_backend();

    // Load a test tone
    backend.load_file("test_assets/sine_1khz.wav").unwrap();

    // Set different gains on different EQ bands
    for band in 0..8 {
        let gain = (band as f32) - 4.0; // Range: -4dB to +3dB
        backend.set_eq_band(band, gain).unwrap();
    }

    // Start playback
    backend.play().unwrap();
    std::thread::sleep(Duration::from_millis(100));

    // Verify all EQ settings are active
    for band in 0..8 {
        let expected_gain = (band as f32) - 4.0;
        let actual_gain = backend.get_eq_band(band);
        assert_eq!(
            actual_gain, expected_gain,
            "EQ band {} gain mismatch",
            band
        );
    }
}

#[test]
fn test_eq_to_analyser_connection() {
    let mut backend = create_test_backend();

    backend.load_file("test_assets/pink_noise.wav").unwrap();

    // Boost high frequencies
    backend.set_eq_band(6, 12.0).unwrap(); // 6kHz
    backend.set_eq_band(7, 12.0).unwrap(); // 12kHz

    backend.play().unwrap();
    std::thread::sleep(Duration::from_millis(100));

    let spectrum = backend.get_spectrum_data();

    // High frequency bins should show more energy than low frequency bins
    let low_freq_energy: f32 = spectrum[0..64].iter().sum();
    let high_freq_energy: f32 = spectrum[384..448].iter().sum();

    assert!(
        high_freq_energy > low_freq_energy,
        "High freq boost should increase high freq energy"
    );
}

#[test]
fn test_analyser_to_output_connection() {
    let mut backend = create_test_backend();

    backend.load_file("test_assets/sine_1khz.wav").unwrap();
    backend.play().unwrap();

    // Analyser should not affect audio output
    // Verify playback continues while getting spectrum data
    for _ in 0..10 {
        let spectrum = backend.get_spectrum_data();
        assert_eq!(spectrum.len(), 512);
        std::thread::sleep(Duration::from_millis(10));
    }

    // Playback should still be active
    assert!(backend.is_playing(), "Playback should continue during analysis");
}

// ============================================================================
// GAIN NODE TESTS
// ============================================================================

#[test]
fn test_gain_node_volume_control() {
    let mut backend = create_test_backend();

    backend.load_file("test_assets/sine_1khz.wav").unwrap();
    backend.play().unwrap();
    std::thread::sleep(Duration::from_millis(50));

    // Get baseline spectrum
    let spectrum_full = backend.get_spectrum_data();
    let energy_full: f32 = spectrum_full.iter().sum();

    // Reduce volume
    backend.set_volume(0.5).unwrap();
    std::thread::sleep(Duration::from_millis(50));

    let spectrum_half = backend.get_spectrum_data();
    let energy_half: f32 = spectrum_half.iter().sum();

    // Half volume should reduce energy
    assert!(
        energy_half < energy_full,
        "Reduced volume should decrease spectrum energy"
    );
}

#[test]
fn test_gain_node_mute() {
    let mut backend = create_test_backend();

    backend.load_file("test_assets/sine_1khz.wav").unwrap();
    backend.play().unwrap();

    // Mute (volume = 0.0)
    backend.set_volume(0.0).unwrap();
    std::thread::sleep(Duration::from_millis(50));

    let spectrum = backend.get_spectrum_data();
    let total_energy: f32 = spectrum.iter().sum();

    // Muted audio should have minimal energy
    assert!(
        total_energy < 1.0,
        "Muted audio should have near-zero spectrum energy"
    );
}

// ============================================================================
// DYNAMIC ROUTING TESTS
// ============================================================================

#[test]
fn test_eq_changes_during_playback() {
    let mut backend = create_test_backend();

    backend.load_file("test_assets/pink_noise.wav").unwrap();
    backend.play().unwrap();

    // Dynamically change EQ during playback
    for i in 0..5 {
        let gain = (i as f32) * 3.0;
        backend.set_eq_band(4, gain).unwrap(); // 1kHz band

        std::thread::sleep(Duration::from_millis(50));

        let current_gain = backend.get_eq_band(4);
        assert_eq!(current_gain, gain, "EQ should update during playback");
    }

    assert!(backend.is_playing(), "Playback should continue during EQ changes");
}

#[test]
fn test_eq_reset_during_playback() {
    let mut backend = create_test_backend();

    backend.load_file("test_assets/sine_1khz.wav").unwrap();

    // Set all bands to max boost
    for band in 0..8 {
        backend.set_eq_band(band, 12.0).unwrap();
    }

    backend.play().unwrap();
    std::thread::sleep(Duration::from_millis(50));

    // Reset EQ during playback
    backend.reset_eq().unwrap();

    // Verify all bands are reset
    for band in 0..8 {
        assert_eq!(
            backend.get_eq_band(band),
            0.0,
            "Band {} should be reset to 0dB",
            band
        );
    }

    assert!(backend.is_playing(), "Playback should continue after EQ reset");
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_graph_survives_invalid_file() {
    let mut backend = create_test_backend();

    // Load valid file first
    backend.load_file("test_assets/sine_1khz.wav").unwrap();
    backend.set_eq_band(0, 6.0).unwrap();

    // Try to load invalid file
    let result = backend.load_file("nonexistent.wav");
    assert!(result.is_err(), "Loading invalid file should fail");

    // Graph should still be functional
    assert_eq!(
        backend.get_eq_band(0),
        6.0,
        "EQ state should be preserved after error"
    );
}

#[test]
fn test_graph_survives_playback_error() {
    let mut backend = create_test_backend();

    // Set EQ before loading file
    backend.set_eq_band(2, 3.0).unwrap();

    // Try to play without loading file
    let result = backend.play();
    // May or may not error depending on implementation

    // Graph should still be functional
    assert_eq!(
        backend.get_eq_band(2),
        3.0,
        "EQ state should survive playback error"
    );

    // Should be able to load file afterward
    let load_result = backend.load_file("test_assets/sine_1khz.wav");
    assert!(load_result.is_ok(), "Should be able to load file after error");
}

// ============================================================================
// MULTI-THREADING TESTS
// ============================================================================

#[test]
fn test_concurrent_eq_access() {
    let backend = Arc::new(Mutex::new(create_test_backend()));

    // Spawn multiple threads modifying EQ
    let handles: Vec<_> = (0..8)
        .map(|band| {
            let backend_clone = Arc::clone(&backend);
            std::thread::spawn(move || {
                for i in 0..100 {
                    let gain = (i % 25) as f32 - 12.0; // -12 to +12 dB
                    backend_clone
                        .lock()
                        .unwrap()
                        .set_eq_band(band, gain)
                        .unwrap();
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify backend is still functional
    let mut backend_guard = backend.lock().unwrap();
    for band in 0..8 {
        let gain = backend_guard.get_eq_band(band);
        assert!(
            gain >= -12.0 && gain <= 12.0,
            "EQ band {} should be in valid range",
            band
        );
    }
}

#[test]
fn test_concurrent_spectrum_reads() {
    let backend = Arc::new(Mutex::new(create_test_backend()));

    {
        let mut backend_guard = backend.lock().unwrap();
        backend_guard.load_file("test_assets/pink_noise.wav").unwrap();
        backend_guard.play().unwrap();
    }

    // Spawn multiple threads reading spectrum
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let backend_clone = Arc::clone(&backend);
            std::thread::spawn(move || {
                for _ in 0..100 {
                    let spectrum = backend_clone.lock().unwrap().get_spectrum_data();
                    assert_eq!(spectrum.len(), 512);
                    std::thread::sleep(Duration::from_millis(5));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_test_backend() -> Box<dyn AudioBackend> {
    // Return a real backend implementation (CPAL, ASIO, or Web Audio)
    // For testing, use the hybrid backend which supports all platforms
    #[cfg(not(target_arch = "wasm32"))]
    {
        use rusty_audio::audio::hybrid::HybridAudioBackend;
        Box::new(HybridAudioBackend::new().expect("Failed to create hybrid backend"))
    }

    #[cfg(target_arch = "wasm32")]
    {
        use rusty_audio::audio::web_bridge::WebAudioBackend;
        Box::new(WebAudioBackend::new().expect("Failed to create web audio backend"))
    }
}
