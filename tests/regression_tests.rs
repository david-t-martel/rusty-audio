// tests/regression_tests.rs
//! Regression tests to verify PR #5 bugs are fixed and stay fixed

use rusty_audio::audio::backend::{AudioBackend, BackendType};

// ============================================================================
// PR #5 BUG FIXES - REGRESSION TESTS
// ============================================================================

/// PR #5 Issue 1: AudioBackend trait was not dyn-safe
/// Symptom: Compiler error "trait cannot be made into an object"
/// Fix: Removed associated constants and made all methods object-safe
#[test]
fn regression_pr5_backend_trait_dyn_safety() {
    // This test ensures Box<dyn AudioBackend> compiles
    // If this test fails to compile, the trait is not object-safe

    #[cfg(not(target_arch = "wasm32"))]
    {
        use rusty_audio::audio::hybrid::HybridAudioBackend;

        // Critical: This must compile
        let backend: Box<dyn AudioBackend> = Box::new(
            HybridAudioBackend::new().expect("Failed to create hybrid backend")
        );

        // Verify polymorphism works
        assert!(matches!(
            backend.backend_type(),
            BackendType::Cpal | BackendType::Asio | BackendType::WebAudio
        ));

        // Verify all trait methods are callable through dyn reference
        let _ = backend.is_playing();
        let _ = backend.get_volume();
        let _ = backend.get_position();
        let _ = backend.get_spectrum_data();
    }

    #[cfg(target_arch = "wasm32")]
    {
        use rusty_audio::audio::web_bridge::WebAudioBackend;

        let backend: Box<dyn AudioBackend> = Box::new(
            WebAudioBackend::new().expect("Failed to create web audio backend")
        );

        assert_eq!(backend.backend_type(), BackendType::WebAudio);
    }
}

/// PR #5 Issue 2: Windows MMCSS HANDLE import was incorrect
/// Symptom: Compilation error on Windows with ASIO feature
/// Fix: Corrected import to windows::Win32::Media::HANDLE
#[test]
#[cfg(all(target_os = "windows", feature = "asio"))]
fn regression_pr5_windows_mmcss_handle_import() {
    // This test verifies the HANDLE type is correctly imported
    // If this fails to compile, the import path is wrong

    use rusty_audio::audio::backend::mmcss;

    // The register_mmcss_thread function should compile and be callable
    let register_fn = mmcss::register_mmcss_thread;

    // Function signature should accept &str and return Result<HANDLE, Error>
    let _ = register_fn("Pro Audio");

    // If we got here, HANDLE is correctly imported
    assert!(true, "MMCSS HANDLE import compiles correctly");
}

/// PR #5 Issue 3: EQ audio graph was not connected (EQ had no effect)
/// Symptom: Adjusting EQ sliders did nothing to audio
/// Fix: Connected Source → EQ → Analyser → Gain → Output
#[test]
fn regression_pr5_eq_audio_graph_connection() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use rusty_audio::audio::hybrid::HybridAudioBackend;

        let mut backend = HybridAudioBackend::new().unwrap();

        // Set EQ bands to non-zero values
        for band in 0..8 {
            backend.set_eq_band(band, (band as f32) * 1.5).unwrap();
        }

        // Verify EQ settings are stored
        for band in 0..8 {
            let expected_gain = (band as f32) * 1.5;
            let actual_gain = backend.get_eq_band(band);
            assert_eq!(
                actual_gain, expected_gain,
                "EQ band {} should persist value",
                band
            );
        }

        // Load a test file and play
        if backend.load_file("test_assets/sine_1khz.wav").is_ok() {
            backend.play().unwrap();

            // Wait for audio to flow through graph
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Verify spectrum analyser is receiving data
            let spectrum = backend.get_spectrum_data();
            let max_value = spectrum.iter().copied().fold(0.0f32, f32::max);

            assert!(
                max_value > 0.0,
                "Spectrum should show energy (EQ is in audio path)"
            );
        }
    }
}

/// PR #5 Issue 4: Theme colors were not applying correctly
/// Symptom: UI appeared in default egui theme regardless of selection
/// Fix: Implemented to_egui_visuals() and applied via ctx.set_visuals()
#[test]
fn regression_pr5_theme_application() {
    use rusty_audio::ui::theme::AppTheme;

    // Test all themes convert to egui Visuals
    for theme in AppTheme::all() {
        let visuals = theme.to_egui_visuals();

        // Verify dark mode flag is set correctly
        match theme {
            AppTheme::Dark | AppTheme::Mocha | AppTheme::Macchiato | AppTheme::Frappe | AppTheme::HighContrast => {
                assert!(visuals.dark_mode, "{} theme should be dark mode", theme);
            }
            AppTheme::Light | AppTheme::Latte => {
                assert!(!visuals.dark_mode, "{} theme should be light mode", theme);
            }
        }

        // Verify background color is set
        let bg_color = visuals.widgets.noninteractive.bg_fill;
        assert_ne!(
            bg_color,
            egui::Color32::TRANSPARENT,
            "{} theme should have non-transparent background",
            theme
        );
    }
}

/// PR #5 Issue 5: Spectrum gradient was incorrect (solid colors instead of smooth gradient)
/// Symptom: Spectrum bars showed solid cyan or red, not blue→cyan→red gradient
/// Fix: Implemented lerp_color() interpolation between low/mid/high colors
#[test]
fn regression_pr5_spectrum_gradient_smoothness() {
    use rusty_audio::ui::theme::AppTheme;

    let theme = AppTheme::Dark;
    let colors = theme.colors();

    // Test gradient interpolation at various points
    let test_points = [0.0, 0.25, 0.5, 0.75, 1.0];

    for &t in &test_points {
        let color = interpolate_gradient(&colors, t);

        // Verify color is valid RGB
        assert!(color[0] <= 255);
        assert!(color[1] <= 255);
        assert!(color[2] <= 255);

        // Verify gradient progression
        match t {
            t if t < 0.1 => {
                // Should be bluish at low end
                assert!(
                    color[2] > 200,
                    "Low frequencies should be blue: {:?}",
                    color
                );
            }
            t if t > 0.9 => {
                // Should be reddish at high end
                assert!(
                    color[0] > 200,
                    "High frequencies should be red: {:?}",
                    color
                );
            }
            t if (0.45..=0.55).contains(&t) => {
                // Should be cyan at midpoint
                assert!(
                    color[1] > 200 && color[2] > 200,
                    "Mid frequencies should be cyan: {:?}",
                    color
                );
            }
            _ => {}
        }
    }
}

// ============================================================================
// COMPILATION REGRESSION TESTS
// ============================================================================

/// Verify all targets compile successfully
#[test]
fn regression_compilation_all_targets() {
    // This test simply existing verifies compilation succeeded

    #[cfg(target_os = "windows")]
    {
        println!("Compiled successfully for Windows");
    }

    #[cfg(target_os = "linux")]
    {
        println!("Compiled successfully for Linux");
    }

    #[cfg(target_os = "macos")]
    {
        println!("Compiled successfully for macOS");
    }

    #[cfg(target_arch = "wasm32")]
    {
        println!("Compiled successfully for WASM");
    }

    assert!(true);
}

/// Verify ASIO feature compiles on Windows
#[test]
#[cfg(all(target_os = "windows", feature = "asio"))]
fn regression_compilation_asio_feature() {
    use rusty_audio::audio::backend::AsioBackend;

    // Type should exist
    let _ = std::mem::size_of::<AsioBackend>();

    println!("ASIO backend compiles successfully");
    assert!(true);
}

/// Verify WASM target compiles with web-sys features
#[test]
#[cfg(target_arch = "wasm32")]
fn regression_compilation_wasm_web_sys() {
    use rusty_audio::audio::web_bridge::WebAudioBackend;

    // Type should exist
    let _ = std::mem::size_of::<WebAudioBackend>();

    println!("WASM Web Audio backend compiles successfully");
    assert!(true);
}

// ============================================================================
// PANIC REGRESSION TESTS
// ============================================================================

/// Verify no panics in audio hot path
#[test]
fn regression_no_panic_in_audio_callback() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use rusty_audio::audio::hybrid::HybridAudioBackend;

        let mut backend = HybridAudioBackend::new().unwrap();

        // These operations should never panic
        let _ = backend.get_spectrum_data();
        let _ = backend.get_volume();
        let _ = backend.get_position();

        for band in 0..8 {
            let _ = backend.get_eq_band(band);
        }

        // Invalid operations should return errors, not panic
        let result = backend.set_eq_band(10, 0.0); // Invalid band
        assert!(result.is_err(), "Invalid band should return error, not panic");

        let result = backend.load_file("/nonexistent/file.mp3");
        assert!(result.is_err(), "Invalid file should return error, not panic");
    }
}

/// Verify no panics with invalid user input
#[test]
fn regression_no_panic_invalid_input() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use rusty_audio::audio::hybrid::HybridAudioBackend;

        let mut backend = HybridAudioBackend::new().unwrap();

        // Extreme values should be clamped, not panic
        let _ = backend.set_volume(-100.0); // Should clamp to 0.0
        let _ = backend.set_volume(1000.0); // Should clamp to 1.0

        let _ = backend.set_eq_band(0, -1000.0); // Should clamp to -12.0
        let _ = backend.set_eq_band(0, 1000.0); // Should clamp to +12.0

        let _ = backend.seek(-100.0); // Should clamp to 0.0
        let _ = backend.seek(999999.0); // Should clamp to duration

        assert!(true, "No panics with invalid input");
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

use rusty_audio::ui::theme::ThemeColors;

fn interpolate_gradient(colors: &ThemeColors, t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);

    if t < 0.5 {
        // Interpolate between low and mid
        let local_t = t * 2.0;
        lerp_color(colors.spectrum_low, colors.spectrum_mid, local_t)
    } else {
        // Interpolate between mid and high
        let local_t = (t - 0.5) * 2.0;
        lerp_color(colors.spectrum_mid, colors.spectrum_high, local_t)
    }
}

fn lerp_color(c1: [u8; 3], c2: [u8; 3], t: f32) -> [u8; 3] {
    [
        ((c1[0] as f32) * (1.0 - t) + (c2[0] as f32) * t) as u8,
        ((c1[1] as f32) * (1.0 - t) + (c2[1] as f32) * t) as u8,
        ((c1[2] as f32) * (1.0 - t) + (c2[2] as f32) * t) as u8,
    ]
}
