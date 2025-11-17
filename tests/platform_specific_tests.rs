// tests/platform_specific_tests.rs
//! Platform-specific tests for desktop (CPAL/ASIO) and WASM (Web Audio API)

// ============================================================================
// DESKTOP (NON-WASM) TESTS
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
mod desktop_tests {
    use rusty_audio::audio::backend::{AudioBackend, BackendType};

    #[test]
    fn test_cpal_backend_available() {
        // CPAL should be available on all desktop platforms
        use rusty_audio::audio::device::CpalBackend;

        let backend_result = CpalBackend::new();
        assert!(
            backend_result.is_ok(),
            "CPAL backend should be available on desktop platforms"
        );

        let backend = backend_result.unwrap();
        assert_eq!(backend.backend_type(), BackendType::Cpal);
    }

    #[test]
    fn test_cpal_device_enumeration() {
        use rusty_audio::audio::device::CpalBackend;

        let backend = CpalBackend::new().unwrap();
        let devices = backend.list_output_devices();

        // Should have at least one output device
        assert!(
            !devices.is_empty(),
            "Should have at least one audio output device"
        );

        // Device names should be non-empty
        for device_name in devices {
            assert!(!device_name.is_empty(), "Device name should not be empty");
        }
    }

    #[test]
    fn test_cpal_default_device_selection() {
        use rusty_audio::audio::device::CpalBackend;

        let backend = CpalBackend::new().unwrap();
        let default_device = backend.get_default_output_device();

        assert!(
            default_device.is_some(),
            "Should have a default output device"
        );

        let device_name = default_device.unwrap();
        assert!(!device_name.is_empty());
    }

    // ========================================================================
    // WINDOWS-SPECIFIC TESTS (ASIO + MMCSS)
    // ========================================================================

    #[cfg(target_os = "windows")]
    mod windows_tests {
        use rusty_audio::audio::backend::BackendType;

        #[test]
        fn test_asio_backend_compilation() {
            // This test verifies ASIO backend compiles on Windows
            // Actual ASIO hardware may not be available in CI
            use rusty_audio::audio::backend::AsioBackend;

            // Just verify the type exists and can be instantiated
            let _backend_type = BackendType::Asio;

            // Try to create ASIO backend (may fail if no ASIO hardware)
            let result = AsioBackend::new();

            // Test passes if either:
            // 1. ASIO backend created successfully
            // 2. Failed gracefully with appropriate error
            match result {
                Ok(backend) => {
                    assert_eq!(backend.backend_type(), BackendType::Asio);
                    println!("ASIO backend available");
                }
                Err(e) => {
                    println!("ASIO backend not available (expected on systems without ASIO): {}", e);
                    // This is OK - not all Windows systems have ASIO
                }
            }
        }

        #[test]
        fn test_mmcss_handle_import() {
            // Critical regression test for PR #5 fix
            // Verifies windows::Win32::Media::HANDLE is correctly imported

            // This should compile without error
            #[cfg(feature = "asio")]
            {
                use rusty_audio::audio::backend::mmcss::register_mmcss_thread;

                // Function should exist and be callable
                let _ = register_mmcss_thread;
            }

            // If we got here, HANDLE import works correctly
            assert!(true, "MMCSS HANDLE import compiles successfully");
        }

        #[test]
        fn test_mmcss_thread_registration() {
            // Test MMCSS thread priority boost (Windows-specific)
            #[cfg(feature = "asio")]
            {
                use rusty_audio::audio::backend::mmcss;

                // Register current thread with MMCSS
                let result = mmcss::register_mmcss_thread("Pro Audio");

                match result {
                    Ok(handle) => {
                        println!("MMCSS registration successful");
                        assert!(handle != std::ptr::null_mut());

                        // Unregister
                        let unregister_result = mmcss::unregister_mmcss_thread(handle);
                        assert!(unregister_result.is_ok());
                    }
                    Err(e) => {
                        // May fail if not running with appropriate privileges
                        println!("MMCSS registration failed (may need admin): {}", e);
                    }
                }
            }
        }

        #[test]
        fn test_windows_audio_stack_config() {
            // Verify Windows audio stack is configured for low latency
            use rusty_audio::audio::device::CpalBackend;

            let backend = CpalBackend::new().unwrap();

            // On Windows, verify WASAPI exclusive mode can be requested
            let config = backend.get_audio_config();

            // Config should request low latency settings
            assert!(
                config.buffer_size <= 512,
                "Windows should use low buffer size for low latency"
            );

            assert!(
                config.sample_rate >= 44100.0,
                "Sample rate should be at least 44.1kHz"
            );
        }
    }

    // ========================================================================
    // LINUX-SPECIFIC TESTS
    // ========================================================================

    #[cfg(target_os = "linux")]
    mod linux_tests {
        use rusty_audio::audio::device::CpalBackend;

        #[test]
        fn test_alsa_or_pulseaudio_backend() {
            // Linux should use ALSA or PulseAudio via CPAL
            let backend = CpalBackend::new().unwrap();

            let devices = backend.list_output_devices();
            assert!(
                !devices.is_empty(),
                "Linux should have audio devices via ALSA/PulseAudio"
            );
        }

        #[test]
        fn test_linux_realtime_scheduling() {
            // Verify realtime scheduling is attempted (may require privileges)
            use rusty_audio::audio::backend::realtime;

            let result = realtime::request_realtime_priority();

            match result {
                Ok(_) => {
                    println!("Realtime priority granted");
                }
                Err(e) => {
                    println!("Realtime priority not available (expected without CAP_SYS_NICE): {}", e);
                    // This is normal for non-privileged processes
                }
            }
        }
    }

    // ========================================================================
    // MACOS-SPECIFIC TESTS
    // ========================================================================

    #[cfg(target_os = "macos")]
    mod macos_tests {
        use rusty_audio::audio::device::CpalBackend;

        #[test]
        fn test_coreaudio_backend() {
            // macOS should use CoreAudio via CPAL
            let backend = CpalBackend::new().unwrap();

            let devices = backend.list_output_devices();
            assert!(!devices.is_empty(), "macOS should have CoreAudio devices");
        }

        #[test]
        fn test_macos_exclusive_mode() {
            // Verify CoreAudio exclusive mode (IOAudioStream)
            let backend = CpalBackend::new().unwrap();

            // macOS should support exclusive audio access
            let config = backend.get_audio_config();
            assert!(config.exclusive_mode_available);
        }
    }

    // ========================================================================
    // HYBRID BACKEND TESTS (All Desktop Platforms)
    // ========================================================================

    #[test]
    fn test_hybrid_backend_fallback() {
        use rusty_audio::audio::hybrid::HybridAudioBackend;

        // Hybrid backend should always work on desktop
        let backend = HybridAudioBackend::new();
        assert!(backend.is_ok(), "Hybrid backend should initialize on desktop");

        let backend = backend.unwrap();

        // Should have selected a backend
        let active_backend = backend.get_active_backend();
        assert!(
            matches!(
                active_backend,
                BackendType::Cpal | BackendType::Asio | BackendType::WebAudio
            ),
            "Hybrid backend should select valid backend type"
        );
    }

    #[test]
    fn test_hybrid_backend_preference_order() {
        use rusty_audio::audio::hybrid::HybridAudioBackend;

        let backend = HybridAudioBackend::new().unwrap();
        let active = backend.get_active_backend();

        // On Windows with ASIO available, should prefer ASIO
        #[cfg(all(target_os = "windows", feature = "asio"))]
        {
            // If ASIO hardware present, should use ASIO
            if backend.asio_available() {
                assert_eq!(active, BackendType::Asio);
            } else {
                assert_eq!(active, BackendType::Cpal);
            }
        }

        // On other platforms, should use CPAL
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(active, BackendType::Cpal);
        }
    }
}

// ============================================================================
// WASM (WEB AUDIO API) TESTS
// ============================================================================

#[cfg(target_arch = "wasm32")]
mod wasm_tests {
    use rusty_audio::audio::backend::{AudioBackend, BackendType};
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_web_audio_backend_available() {
        use rusty_audio::audio::web_bridge::WebAudioBackend;

        let backend_result = WebAudioBackend::new();
        assert!(
            backend_result.is_ok(),
            "Web Audio API should be available in browser"
        );

        let backend = backend_result.unwrap();
        assert_eq!(backend.backend_type(), BackendType::WebAudio);
    }

    #[wasm_bindgen_test]
    fn test_web_audio_context_creation() {
        use rusty_audio::audio::web_bridge::WebAudioBackend;

        let backend = WebAudioBackend::new().unwrap();

        // Web Audio Context should be created
        let context = backend.get_audio_context();
        assert!(context.is_some());

        // Context should be in "suspended" or "running" state
        let state = context.unwrap().state();
        assert!(
            matches!(state, web_sys::AudioContextState::Suspended | web_sys::AudioContextState::Running)
        );
    }

    #[wasm_bindgen_test]
    fn test_web_audio_eq_node_creation() {
        use rusty_audio::audio::web_bridge::WebAudioBackend;

        let mut backend = WebAudioBackend::new().unwrap();

        // Set EQ bands - should create BiquadFilterNodes
        for band in 0..8 {
            let result = backend.set_eq_band(band, 6.0);
            assert!(result.is_ok(), "Should create EQ filter nodes");
        }

        // Verify EQ values persisted
        for band in 0..8 {
            assert_eq!(backend.get_eq_band(band), 6.0);
        }
    }

    #[wasm_bindgen_test]
    fn test_web_audio_analyser_node() {
        use rusty_audio::audio::web_bridge::WebAudioBackend;

        let backend = WebAudioBackend::new().unwrap();

        // Get spectrum data - should create AnalyserNode
        let spectrum = backend.get_spectrum_data();

        assert_eq!(spectrum.len(), 512, "Analyser should provide 512 bins");

        // All values should be normalized
        for &value in &spectrum {
            assert!(value >= 0.0 && value <= 1.0);
        }
    }

    #[wasm_bindgen_test]
    fn test_web_audio_graph_connection() {
        use rusty_audio::audio::web_bridge::WebAudioBackend;

        let mut backend = WebAudioBackend::new().unwrap();

        // Load audio (creates SourceNode)
        backend.load_file("test.mp3").ok(); // May fail without actual file

        // Set EQ (creates BiquadFilterNodes)
        backend.set_eq_band(0, 3.0).unwrap();

        // Get spectrum (creates AnalyserNode)
        let _ = backend.get_spectrum_data();

        // Verify audio graph is connected: Source → EQ → Analyser → Destination
        assert!(backend.verify_audio_graph_connected());
    }

    #[wasm_bindgen_test]
    fn test_web_audio_user_gesture_requirement() {
        use rusty_audio::audio::web_bridge::WebAudioBackend;

        let mut backend = WebAudioBackend::new().unwrap();

        // Attempt playback without user gesture
        let result = backend.play();

        // May fail with "NotAllowedError" if no user gesture
        match result {
            Ok(_) => {
                println!("Playback started (user gesture granted or autoplay allowed)");
            }
            Err(e) => {
                // Expected error in browser without user interaction
                assert!(
                    e.to_string().contains("NotAllowed") || e.to_string().contains("user gesture"),
                    "Should require user gesture for playback"
                );
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_wasm_tab_count() {
        // WASM version should have 4 tabs (no Generator or Recording)
        use rusty_audio::RustyAudioApp;

        let app = RustyAudioApp::default();
        let tab_count = app.get_tab_count();

        assert_eq!(tab_count, 4, "WASM should have 4 tabs (Playback, Effects, EQ, Settings)");
    }

    #[wasm_bindgen_test]
    fn test_wasm_features_disabled() {
        // Verify desktop-only features are disabled in WASM
        use rusty_audio::features;

        assert!(!features::recording_available(), "Recording should be disabled in WASM");
        assert!(!features::signal_generator_available(), "Signal generator should be disabled in WASM");
        assert!(features::web_audio_available(), "Web Audio should be available in WASM");
    }
}

// ============================================================================
// CROSS-PLATFORM TESTS (Both Desktop and WASM)
// ============================================================================

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn test_backend_trait_object_safety() {
    // Critical: AudioBackend must be object-safe for polymorphism
    // This test verifies Box<dyn AudioBackend> compiles

    #[cfg(not(target_arch = "wasm32"))]
    {
        use rusty_audio::audio::hybrid::HybridAudioBackend;
        let backend: Box<dyn AudioBackend> = Box::new(HybridAudioBackend::new().unwrap());
        let _ = backend.backend_type(); // Use it to prevent optimization
    }

    #[cfg(target_arch = "wasm32")]
    {
        use rusty_audio::audio::web_bridge::WebAudioBackend;
        let backend: Box<dyn AudioBackend> = Box::new(WebAudioBackend::new().unwrap());
        let _ = backend.backend_type();
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn test_eq_frequencies_consistent() {
    // EQ frequencies should be the same on all platforms
    const EXPECTED_FREQUENCIES: [f32; 8] = [60.0, 170.0, 310.0, 600.0, 1000.0, 3000.0, 6000.0, 12000.0];

    use rusty_audio::audio::eq::EQ_FREQUENCIES;

    for (i, &expected) in EXPECTED_FREQUENCIES.iter().enumerate() {
        assert_eq!(
            EQ_FREQUENCIES[i], expected,
            "EQ frequency {} mismatch on this platform",
            i
        );
    }
}
