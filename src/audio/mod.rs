//! Audio subsystem
//!
//! This module provides the complete audio infrastructure including:
//! - Backend abstraction for different audio systems
//! - Device enumeration and management
//! - Audio streaming for playback and recording
//! - MIDI support (Phase 3.3)
//! - Advanced format support (Phase 3.4)

pub mod backend;
pub mod backend_selector;
pub mod destinations;
pub mod device;
pub mod hybrid;
pub mod manager;
pub mod recorder;
pub mod router;
pub mod sources;
pub mod web_bridge;

// Windows ASIO backend (Phase 1.1)
#[cfg(target_os = "windows")]
pub mod asio_backend;

// Windows MMCSS integration (Phase 1.2)
#[cfg(target_os = "windows")]
pub mod mmcss;

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

#[cfg(target_os = "windows")]
pub use asio_backend::{AsioBackend, WindowsBackendType};
pub use backend_selector::{BackendInfo, BackendSelector};

#[cfg(target_os = "windows")]
pub use mmcss::{MmcssHandle, MmcssTaskCategory};

pub use router::{AudioDestination, AudioRouter, AudioSource, DestId, Route, RouteId, SourceId};

// Audio sources and destinations
pub use destinations::{
    LevelMeterDestination, NullDestination, RingBufferDestination, RingBufferReader,
    SplitterDestination,
};
pub use sources::{RingBufferSource, RingBufferWriter, SignalGeneratorSource, SilenceSource};
