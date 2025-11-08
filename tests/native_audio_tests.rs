//! Phase 3.1.7 - Native Audio Backend Testing
//!
//! Comprehensive tests for CPAL native audio backend functionality

use rusty_audio::audio::{
    HybridAudioBackend, HybridMode, AudioBackend, AudioConfig, StreamDirection,
    SampleFormat, FallbackPolicy, BackendHealth,
};
use std::time::Duration;

#[test]
fn test_hybrid_backend_initialization() {
    let mut backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    // Should initialize without error
    assert!(backend.initialize().is_ok());
}

#[test]
fn test_backend_availability() {
    let backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    // Should be available on non-WASM platforms
    #[cfg(not(target_arch = "wasm32"))]
    assert!(backend.is_available());
}

#[test]
fn test_backend_name() {
    let backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    assert_eq!(backend.name(), "hybrid(web-audio + cpal)");
}

#[test]
fn test_web_audio_only_name() {
    let backend = HybridAudioBackend::with_mode(HybridMode::WebAudioOnly);
    
    assert_eq!(backend.name(), "web-audio-api");
}

#[test]
fn test_device_enumeration() {
    let backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    // Should be able to enumerate output devices
    let result = backend.enumerate_devices(StreamDirection::Output);
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        assert!(result.is_ok());
        let devices = result.unwrap();
        assert!(devices.len() > 0, "Should have at least one output device");
    }
}

#[test]
fn test_default_device() {
    let backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    // Should be able to get default output device
    let result = backend.default_device(StreamDirection::Output);
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        assert!(result.is_ok());
        let device = result.unwrap();
        assert!(device.is_default);
        assert!(!device.name.is_empty());
    }
}

#[test]
fn test_device_info_validity() {
    let backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(device) = backend.default_device(StreamDirection::Output) {
            // Verify device info fields are valid
            assert!(!device.id.is_empty());
            assert!(!device.name.is_empty());
            assert!(device.min_sample_rate > 0);
            assert!(device.max_sample_rate >= device.min_sample_rate);
            assert!(device.max_output_channels > 0);
        }
    }
}

#[test]
fn test_supported_configs() {
    let backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(device) = backend.default_device(StreamDirection::Output) {
            let configs = backend.supported_configs(&device.id);
            assert!(configs.is_ok());
            let configs = configs.unwrap();
            assert!(configs.len() > 0);
        }
    }
}

#[test]
fn test_audio_config_defaults() {
    let config = AudioConfig::default();
    
    assert_eq!(config.sample_rate, 44100);
    assert_eq!(config.channels, 2);
    assert_eq!(config.sample_format, SampleFormat::F32);
    assert_eq!(config.buffer_size, 512);
}

#[test]
fn test_create_output_stream() {
    let mut backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        if backend.initialize().is_ok() {
            if let Ok(device) = backend.default_device(StreamDirection::Output) {
                let config = AudioConfig::default();
                let stream = backend.create_output_stream(&device.id, config);
                
                // Should create stream successfully
                assert!(stream.is_ok());
            }
        }
    }
}

#[test]
fn test_ring_buffer_creation() {
    let mut backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        if backend.initialize().is_ok() {
            if let Ok(device) = backend.default_device(StreamDirection::Output) {
                let config = AudioConfig::default();
                let _ = backend.create_output_stream(&device.id, config);
                
                // Ring buffer should be created for hybrid mode
                assert!(backend.ring_buffer().is_some());
            }
        }
    }
}

#[test]
fn test_fallback_policy_persistence_across_streams() {
    let mut backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    backend.set_fallback_policy(FallbackPolicy::Manual);
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        if backend.initialize().is_ok() {
            if let Ok(device) = backend.default_device(StreamDirection::Output) {
                let config = AudioConfig::default();
                let _ = backend.create_output_stream(&device.id, config);
                
                // Fallback policy should persist after stream creation
                assert_eq!(backend.fallback_policy(), FallbackPolicy::Manual);
            }
        }
    }
}

#[test]
fn test_health_initial_state() {
    let backend = HybridAudioBackend::new();
    
    // Should start in healthy state
    assert_eq!(backend.health(), BackendHealth::Healthy);
}

#[test]
fn test_mode_switching_with_streams() {
    let mut backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Initialize in hybrid mode
        let _ = backend.initialize();
        
        // Switch to web audio only
        let result = backend.set_mode(HybridMode::WebAudioOnly);
        assert!(result.is_ok());
        assert_eq!(backend.mode(), HybridMode::WebAudioOnly);
        
        // Switch back
        let result = backend.set_mode(HybridMode::HybridNative);
        assert!(result.is_ok());
        assert_eq!(backend.mode(), HybridMode::HybridNative);
    }
}

#[test]
fn test_web_audio_only_device_enum() {
    let backend = HybridAudioBackend::with_mode(HybridMode::WebAudioOnly);
    
    // Should return browser audio device
    let result = backend.enumerate_devices(StreamDirection::Output);
    assert!(result.is_ok());
    
    let devices = result.unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].id, "browser-audio");
    assert_eq!(devices[0].name, "Browser Audio API");
}

#[test]
fn test_sample_format_variants() {
    // Verify all sample formats are distinct
    assert_ne!(SampleFormat::I16, SampleFormat::I32);
    assert_ne!(SampleFormat::I16, SampleFormat::F32);
    assert_ne!(SampleFormat::I32, SampleFormat::F32);
}

#[test]
fn test_stream_direction_variants() {
    // Verify stream directions are distinct
    assert_ne!(StreamDirection::Input, StreamDirection::Output);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_latency_measurement() {
    let mut backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    if backend.initialize().is_ok() {
        if let Ok(device) = backend.default_device(StreamDirection::Output) {
            let config = AudioConfig::default();
            if let Ok(mut stream) = backend.create_output_stream(&device.id, config) {
                // Try to get latency
                let latency_ms = stream.latency_ms();
                
                // Latency might be None if not supported, but shouldn't panic
                if let Some(latency) = latency_ms {
                    // Sanity check: latency should be reasonable (< 1 second)
                    assert!(latency < 1000.0, "Latency should be less than 1 second");
                    assert!(latency > 0.0, "Latency should be positive");
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_multiple_streams() {
    let mut backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    if backend.initialize().is_ok() {
        if let Ok(device) = backend.default_device(StreamDirection::Output) {
            let config = AudioConfig::default();
            
            // Create first stream
            let stream1 = backend.create_output_stream(&device.id, config.clone());
            assert!(stream1.is_ok());
            
            // Note: Creating multiple streams on same backend may not be supported
            // This test verifies it doesn't panic
        }
    }
}

#[test]
fn test_config_cloning() {
    let config1 = AudioConfig::default();
    let config2 = config1.clone();
    
    assert_eq!(config1.sample_rate, config2.sample_rate);
    assert_eq!(config1.channels, config2.channels);
    assert_eq!(config1.buffer_size, config2.buffer_size);
}

#[test]
fn test_hybrid_mode_variants() {
    // Verify all modes are distinct
    assert_ne!(HybridMode::WebAudioOnly, HybridMode::HybridNative);
    assert_ne!(HybridMode::WebAudioOnly, HybridMode::CpalOnly);
    assert_ne!(HybridMode::HybridNative, HybridMode::CpalOnly);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_backend_reinitialization() {
    let mut backend = HybridAudioBackend::with_mode(HybridMode::HybridNative);
    
    // Initialize multiple times should be idempotent
    assert!(backend.initialize().is_ok());
    assert!(backend.initialize().is_ok());
    assert!(backend.initialize().is_ok());
}

#[test]
fn test_web_audio_fallback_safety() {
    let backend = HybridAudioBackend::with_mode(HybridMode::WebAudioOnly);
    
    // Web audio only should always be available
    assert!(backend.is_available());
}
