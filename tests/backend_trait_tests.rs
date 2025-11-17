// tests/backend_trait_tests.rs
//! Unit tests for AudioBackend trait dyn-safety and polymorphism

use rusty_audio::audio::backend::{AudioBackend, BackendType, PlaybackState};
use std::sync::{Arc, Mutex};

// ============================================================================
// DYN-SAFETY TESTS
// ============================================================================

#[test]
fn test_backend_trait_is_dyn_safe() {
    // Critical: This must compile for trait to be dyn-safe
    // If this fails, trait has non-object-safe methods
    let _boxed: Box<dyn AudioBackend> = create_mock_backend();
}

#[test]
fn test_backend_polymorphism_via_box() {
    let backend: Box<dyn AudioBackend> = create_mock_backend();
    assert_eq!(backend.backend_type(), BackendType::Cpal);
    assert!(!backend.is_playing());
}

#[test]
fn test_backend_polymorphism_via_arc() {
    let backend: Arc<Mutex<dyn AudioBackend>> = Arc::new(Mutex::new(MockBackend::new()));
    let backend_guard = backend.lock().unwrap();
    assert_eq!(backend_guard.backend_type(), BackendType::Cpal);
}

// ============================================================================
// TRAIT METHOD TESTS
// ============================================================================

#[test]
fn test_backend_lifecycle() {
    let mut backend = MockBackend::new();

    // Initial state
    assert_eq!(backend.playback_state(), PlaybackState::Stopped);

    // Play
    backend.play().expect("Play should succeed");
    assert_eq!(backend.playback_state(), PlaybackState::Playing);

    // Pause
    backend.pause().expect("Pause should succeed");
    assert_eq!(backend.playback_state(), PlaybackState::Paused);

    // Stop
    backend.stop().expect("Stop should succeed");
    assert_eq!(backend.playback_state(), PlaybackState::Stopped);
}

#[test]
fn test_backend_volume_clamping() {
    let mut backend = MockBackend::new();

    // Valid range: 0.0 to 1.0
    backend.set_volume(0.5).expect("Set volume should succeed");
    assert_eq!(backend.get_volume(), 0.5);

    // Clamping tests
    backend.set_volume(-0.5).expect("Negative volume should clamp");
    assert_eq!(backend.get_volume(), 0.0);

    backend.set_volume(2.0).expect("Excessive volume should clamp");
    assert_eq!(backend.get_volume(), 1.0);
}

#[test]
fn test_backend_eq_band_range() {
    let mut backend = MockBackend::new();

    // Valid EQ range: -12.0 to +12.0 dB
    for band in 0..8 {
        backend.set_eq_band(band, 6.0).expect("Set EQ band should succeed");
        assert_eq!(backend.get_eq_band(band), 6.0);

        // Clamping
        backend.set_eq_band(band, -20.0).expect("Negative EQ should clamp");
        assert_eq!(backend.get_eq_band(band), -12.0);

        backend.set_eq_band(band, 20.0).expect("Excessive EQ should clamp");
        assert_eq!(backend.get_eq_band(band), 12.0);
    }
}

#[test]
fn test_backend_eq_reset() {
    let mut backend = MockBackend::new();

    // Set all bands to non-zero
    for band in 0..8 {
        backend.set_eq_band(band, (band as f32) + 1.0).unwrap();
    }

    // Reset
    backend.reset_eq().expect("Reset EQ should succeed");

    // Verify all bands are 0.0
    for band in 0..8 {
        assert_eq!(backend.get_eq_band(band), 0.0);
    }
}

#[test]
fn test_backend_position_tracking() {
    let mut backend = MockBackend::new();
    backend.play().unwrap();

    // Mock backend should track position
    std::thread::sleep(std::time::Duration::from_millis(100));
    let position = backend.get_position();
    assert!(position > 0.0, "Position should advance during playback");
}

#[test]
fn test_backend_spectrum_data_size() {
    let backend = MockBackend::new();
    let spectrum = backend.get_spectrum_data();

    // Should return 512 frequency bins
    assert_eq!(spectrum.len(), 512, "Spectrum should have 512 bins");

    // All values should be normalized (0.0 to 1.0)
    for &value in &spectrum {
        assert!(value >= 0.0 && value <= 1.0, "Spectrum value out of range: {}", value);
    }
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_backend_error_on_invalid_band() {
    let mut backend = MockBackend::new();

    // Band index out of range
    let result = backend.set_eq_band(8, 0.0);
    assert!(result.is_err(), "Setting invalid band should fail");

    let result = backend.get_eq_band(10);
    assert_eq!(result, 0.0, "Invalid band should return 0.0");
}

#[test]
fn test_backend_error_on_load_invalid_file() {
    let mut backend = MockBackend::new();

    let result = backend.load_file("/nonexistent/file.mp3");
    assert!(result.is_err(), "Loading invalid file should fail");
}

// ============================================================================
// MOCK BACKEND IMPLEMENTATION
// ============================================================================

struct MockBackend {
    state: PlaybackState,
    volume: f32,
    eq_bands: [f32; 8],
    position: Arc<Mutex<f32>>,
}

impl MockBackend {
    fn new() -> Self {
        Self {
            state: PlaybackState::Stopped,
            volume: 1.0,
            eq_bands: [0.0; 8],
            position: Arc::new(Mutex::new(0.0)),
        }
    }
}

impl AudioBackend for MockBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Cpal
    }

    fn load_file(&mut self, _path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if _path.contains("nonexistent") {
            Err("File not found".into())
        } else {
            Ok(())
        }
    }

    fn play(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state = PlaybackState::Playing;
        Ok(())
    }

    fn pause(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state = PlaybackState::Paused;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state = PlaybackState::Stopped;
        *self.position.lock().unwrap() = 0.0;
        Ok(())
    }

    fn is_playing(&self) -> bool {
        matches!(self.state, PlaybackState::Playing)
    }

    fn playback_state(&self) -> PlaybackState {
        self.state
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
        self.volume = volume.clamp(0.0, 1.0);
        Ok(())
    }

    fn get_volume(&self) -> f32 {
        self.volume
    }

    fn set_eq_band(&mut self, band: usize, gain_db: f32) -> Result<(), Box<dyn std::error::Error>> {
        if band >= 8 {
            return Err("Invalid band index".into());
        }
        self.eq_bands[band] = gain_db.clamp(-12.0, 12.0);
        Ok(())
    }

    fn get_eq_band(&self, band: usize) -> f32 {
        self.eq_bands.get(band).copied().unwrap_or(0.0)
    }

    fn reset_eq(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.eq_bands = [0.0; 8];
        Ok(())
    }

    fn get_position(&self) -> f32 {
        *self.position.lock().unwrap()
    }

    fn seek(&mut self, position: f32) -> Result<(), Box<dyn std::error::Error>> {
        *self.position.lock().unwrap() = position;
        Ok(())
    }

    fn get_duration(&self) -> f32 {
        100.0 // Mock duration
    }

    fn get_spectrum_data(&self) -> Vec<f32> {
        vec![0.5; 512] // Mock spectrum data
    }
}

fn create_mock_backend() -> Box<dyn AudioBackend> {
    Box::new(MockBackend::new())
}
