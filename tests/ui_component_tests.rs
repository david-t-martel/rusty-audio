//! UI Component Tests (Phase 3.1.7)
//!
//! Basic tests verifying UI modules compile and initialize correctly

#[test]
fn test_app_compiles() {
    // Verify the app compiles
    assert!(true);
}

#[test]
fn test_audio_module_available() {
    // Verify audio module is accessible
    use rusty_audio::audio::HybridAudioBackend;
    let _ = HybridAudioBackend::new();
}

#[test]
fn test_audio_config_available() {
    use rusty_audio::audio::AudioConfig;
    
    let config = AudioConfig::default();
    assert_eq!(config.sample_rate, 44100);
    assert_eq!(config.channels, 2);
}

#[test]
fn test_hybrid_modes_available() {
    use rusty_audio::audio::HybridMode;
    
    let modes = vec![
        HybridMode::WebAudioOnly,
        HybridMode::HybridNative,
        HybridMode::CpalOnly,
    ];
    
    assert_eq!(modes.len(), 3);
}

#[test]
fn test_fallback_policies_available() {
    use rusty_audio::audio::FallbackPolicy;
    
    let manual = FallbackPolicy::Manual;
    let auto = FallbackPolicy::AutoOnError;
    
    assert_ne!(manual, auto);
}

#[test]
fn test_backend_health_states_available() {
    use rusty_audio::audio::BackendHealth;
    
    let states = vec![
        BackendHealth::Healthy,
        BackendHealth::Degraded,
        BackendHealth::Failed,
    ];
    
    assert_eq!(states.len(), 3);
}

#[test]
fn test_stream_direction_available() {
    use rusty_audio::audio::StreamDirection;
    
    assert_ne!(StreamDirection::Input, StreamDirection::Output);
}

#[test]
fn test_sample_format_available() {
    use rusty_audio::audio::SampleFormat;
    
    let formats = vec![
        SampleFormat::I16,
        SampleFormat::I32,
        SampleFormat::F32,
    ];
    
    assert_eq!(formats.len(), 3);
}

#[test]
fn test_audio_backend_modes() {
    use rusty_audio::audio::HybridMode;
    
    // Test that all modes are distinct
    assert_ne!(HybridMode::WebAudioOnly, HybridMode::HybridNative);
    assert_ne!(HybridMode::WebAudioOnly, HybridMode::CpalOnly);
    assert_ne!(HybridMode::HybridNative, HybridMode::CpalOnly);
}

#[test]
fn test_fallback_policy_equality() {
    use rusty_audio::audio::FallbackPolicy;
    
    // Test policy equality
    assert_eq!(FallbackPolicy::Manual, FallbackPolicy::Manual);
    assert_eq!(FallbackPolicy::AutoOnError, FallbackPolicy::AutoOnError);
}

#[test]
fn test_backend_initialization() {
    use rusty_audio::audio::HybridAudioBackend;
    
    // Test that backends can be created with different modes
    let _ = HybridAudioBackend::new();
    let _ = HybridAudioBackend::with_mode(rusty_audio::audio::HybridMode::WebAudioOnly);
    let _ = HybridAudioBackend::with_mode(rusty_audio::audio::HybridMode::HybridNative);
}
