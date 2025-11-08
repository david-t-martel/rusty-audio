//! Audio Backend Tests (Phase 3.1.6)
//!
//! Tests for hybrid audio backend fallback mechanism

use rusty_audio::audio::{
    HybridAudioBackend, HybridMode, FallbackPolicy, BackendHealth, FallbackTrigger,
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
fn test_manual_fallback_policy_prevents_auto_switch() {
    let mut backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    backend.set_fallback_policy(FallbackPolicy::Manual);
    
    let initial_mode = backend.mode();
    
    // Trigger fallback with manual policy - should not switch
    let trigger = FallbackTrigger::StreamUnderrun { consecutive_count: 10 };
    let result = backend.trigger_fallback(trigger);
    
    // Should fail with manual policy
    assert!(result.is_err());
    assert_eq!(backend.mode(), initial_mode); // Mode unchanged
}

#[test]
fn test_auto_fallback_policy_allows_switch() {
    let mut backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    backend.set_fallback_policy(FallbackPolicy::AutoOnError);
    
    // Trigger fallback - should switch to WebAudioOnly
    let trigger = FallbackTrigger::DeviceDisconnected;
    let result = backend.trigger_fallback(trigger);
    
    // Should succeed
    assert!(result.is_ok());
    assert_eq!(backend.mode(), HybridMode::WebAudioOnly);
    assert_eq!(backend.health(), BackendHealth::Healthy); // Health restored after fallback
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
fn test_web_audio_only_cannot_fallback() {
    let mut backend = HybridAudioBackend::with_mode(HybridMode::WebAudioOnly);
    backend.set_fallback_policy(FallbackPolicy::AutoOnError);
    
    // Cannot fallback from WebAudioOnly
    let trigger = FallbackTrigger::UnknownError("Test error".to_string());
    let result = backend.trigger_fallback(trigger);
    
    assert!(result.is_err());
    assert_eq!(backend.mode(), HybridMode::WebAudioOnly); // Mode unchanged
}

#[test]
fn test_excessive_underruns_trigger_failed_state() {
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

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_cpal_backend_available_non_wasm() {
    let backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    // Should have cpal backend on non-WASM platforms
    assert!(backend.is_available());
}

#[test]
fn test_web_audio_always_available() {
    let backend = HybridAudioBackend::with_mode(HybridMode::WebAudioOnly);
    assert!(backend.is_available());
}
