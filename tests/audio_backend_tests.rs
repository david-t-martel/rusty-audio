//! Audio Backend Tests (Phase 3.1.6)
//!
//! Tests for hybrid audio backend fallback mechanism

use rusty_audio::audio::{
    HybridAudioBackend, HybridMode, FallbackPolicy, BackendHealth,
};

#[test]
fn test_fallback_policy_default() {
    let backend = HybridAudioBackend::new();
    assert_eq!(backend.fallback_policy(), FallbackPolicy::AutoOnError);
}

#[test]
fn test_fallback_policy_manual() {
    let mut backend = HybridAudioBackend::new();
    backend.set_fallback_policy(FallbackPolicy::Manual);
    assert_eq!(backend.fallback_policy(), FallbackPolicy::Manual);
}

#[test]
fn test_fallback_policy_auto_with_preference() {
    let mut backend = HybridAudioBackend::new();
    let mode = HybridMode::HybridNative;
    backend.set_fallback_policy(FallbackPolicy::AutoWithPreference(mode));
    
    match backend.fallback_policy() {
        FallbackPolicy::AutoWithPreference(m) => assert_eq!(m, mode),
        _ => panic!("Expected AutoWithPreference policy"),
    }
}

#[test]
fn test_backend_health_initial() {
    let backend = HybridAudioBackend::new();
    assert_eq!(backend.health(), BackendHealth::Healthy);
}

#[test]
fn test_underrun_degrades_health() {
    let mut backend = HybridAudioBackend::new();
    
    // Report 3 underruns - should degrade health
    backend.report_underrun();
    backend.report_underrun();
    assert_eq!(backend.health(), BackendHealth::Healthy); // Still healthy at 2
    
    backend.report_underrun();
    assert_eq!(backend.health(), BackendHealth::Degraded); // Degraded at 3
}

#[test]
fn test_reset_underrun_restores_health() {
    let mut backend = HybridAudioBackend::new();
    
    // Degrade health
    for _ in 0..5 {
        backend.report_underrun();
    }
    assert_eq!(backend.health(), BackendHealth::Degraded);
    
    // Reset should restore to healthy
    backend.reset_underrun_count();
    assert_eq!(backend.health(), BackendHealth::Healthy);
}

#[test]
fn test_mode_switching_preserves_policy() {
    let mut backend = HybridAudioBackend::new();
    backend.set_fallback_policy(FallbackPolicy::Manual);
    
    // Switch mode
    let _ = backend.set_mode(HybridMode::WebAudioOnly);
    
    // Policy should be preserved
    assert_eq!(backend.fallback_policy(), FallbackPolicy::Manual);
}

#[test]
fn test_excessive_underruns_trigger_automatic_fallback() {
    let mut backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    backend.set_fallback_policy(FallbackPolicy::AutoOnError);
    
    // Report 10 consecutive underruns - should trigger automatic fallback
    for i in 0..10 {
        backend.report_underrun();
        if i < 9 {
            // Should not have switched yet
            assert_eq!(backend.mode(), HybridMode::HybridNative);
        }
    }
    
    // After 10 underruns, should have automatically fallen back
    assert_eq!(backend.mode(), HybridMode::WebAudioOnly);
    assert_eq!(backend.health(), BackendHealth::Healthy); // Health restored
}

#[test]
fn test_hybrid_mode_switching() {
    let mut backend = HybridAudioBackend::new();
    
    // Switch to web audio only
    assert!(backend.set_mode(HybridMode::WebAudioOnly).is_ok());
    assert_eq!(backend.mode(), HybridMode::WebAudioOnly);
    
    // Switch to hybrid native
    assert!(backend.set_mode(HybridMode::HybridNative).is_ok());
    assert_eq!(backend.mode(), HybridMode::HybridNative);
}

#[test]
fn test_ring_buffer_available_in_hybrid_mode() {
    let backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    // Ring buffer might be None until stream is created, which is OK
    // Just verify the method doesn't panic
    let _ = backend.ring_buffer();
}

#[test]
fn test_fallback_policy_variants() {
    // Verify all policy variants are distinct
    assert_ne!(
        FallbackPolicy::Manual,
        FallbackPolicy::AutoOnError
    );
    
    let pref1 = FallbackPolicy::AutoWithPreference(HybridMode::HybridNative);
    let pref2 = FallbackPolicy::AutoWithPreference(HybridMode::WebAudioOnly);
    assert_ne!(pref1, pref2);
}

#[test]
fn test_backend_health_variants() {
    // Verify all health states are distinct
    assert_ne!(BackendHealth::Healthy, BackendHealth::Degraded);
    assert_ne!(BackendHealth::Healthy, BackendHealth::Failed);
    assert_ne!(BackendHealth::Degraded, BackendHealth::Failed);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_hybrid_mode_name() {
    use rusty_audio::audio::AudioBackend;
    
    let backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    assert_eq!(backend.name(), "hybrid(web-audio + cpal)");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_web_audio_only_name() {
    use rusty_audio::audio::AudioBackend;
    
    let backend = HybridAudioBackend::with_mode(HybridMode::WebAudioOnly);
    assert_eq!(backend.name(), "web-audio-api");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_cpal_only_name() {
    use rusty_audio::audio::AudioBackend;
    
    let backend = HybridAudioBackend::with_mode(HybridMode::CpalOnly);
    assert_eq!(backend.name(), "cpal");
}
