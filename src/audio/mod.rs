//! Audio subsystem
//!
//! This module provides the complete audio infrastructure including:
//! - Backend abstraction for different audio systems
//! - Device enumeration and management
//! - Audio streaming for playback and recording
//! - MIDI support (Phase 3.3)
//! - Advanced format support (Phase 3.4)

pub mod backend;
pub mod device;
pub mod hybrid;
pub mod manager;
pub mod recorder;
pub mod web_bridge;

// Re-export commonly used types
pub use backend::{
    AudioBackend, AudioBackendError, AudioBuffer, AudioConfig, AudioStream, DeviceInfo, Result,
    SampleFormat, StreamDirection, StreamStatus,
};
pub use device::CpalBackend;
pub use hybrid::{BackendHealth, FallbackPolicy, HybridAudioBackend, HybridMode, HybridRingBuffer};
pub use manager::AudioDeviceManager;
pub use recorder::{
    AudioRecorder, MonitoringMode, RecordingConfig, RecordingFormat, RecordingState,
};
pub use web_bridge::{WebAudioBridge, WebAudioBridgeConfig};
