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
pub mod manager;
pub mod hybrid;
pub mod web_bridge;
pub mod recorder;

// Re-export commonly used types
pub use backend::{
    AudioBackend, AudioBackendError, AudioBuffer, AudioConfig, AudioStream, DeviceInfo, Result,
    SampleFormat, StreamDirection, StreamStatus,
};
pub use device::CpalBackend;
pub use manager::AudioDeviceManager;
pub use hybrid::{HybridAudioBackend, HybridMode, HybridRingBuffer, FallbackPolicy, BackendHealth};
pub use web_bridge::{WebAudioBridge, WebAudioBridgeConfig};
pub use recorder::{
    AudioRecorder, RecordingConfig, RecordingFormat, RecordingState,
    MonitoringMode,
};
