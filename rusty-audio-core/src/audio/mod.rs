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

// Native-only modules (use CPAL, hound, etc.)
#[cfg(not(target_arch = "wasm32"))]
pub mod device;
#[cfg(not(target_arch = "wasm32"))]
pub mod device_destination;
#[cfg(not(target_arch = "wasm32"))]
pub mod device_source;
#[cfg(not(target_arch = "wasm32"))]
pub mod file_recorder;
#[cfg(not(target_arch = "wasm32"))]
pub mod hybrid;
#[cfg(not(target_arch = "wasm32"))]
pub mod manager;
#[cfg(not(target_arch = "wasm32"))]
pub mod recorder;

pub mod router;
pub mod sources;

// Web bridge is native-only (bridges web-audio-api to CPAL hardware)
#[cfg(not(target_arch = "wasm32"))]
pub mod web_bridge;

// WASM Web Audio API backend (Phase 3.5)
#[cfg(target_arch = "wasm32")]
pub mod wasm_processing;
#[cfg(target_arch = "wasm32")]
pub mod web_audio_backend;
#[cfg(target_arch = "wasm32")]
pub mod web_audio_destination;

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

// Native-only re-exports
#[cfg(not(target_arch = "wasm32"))]
pub use device::CpalBackend;
#[cfg(not(target_arch = "wasm32"))]
pub use hybrid::{BackendHealth, FallbackPolicy, HybridAudioBackend, HybridMode, HybridRingBuffer};
#[cfg(not(target_arch = "wasm32"))]
pub use manager::AudioDeviceManager;
#[cfg(not(target_arch = "wasm32"))]
pub use recorder::{
    AudioRecorder, MonitoringMode, RecordingConfig, RecordingFormat, RecordingState,
};

// Web bridge is native-only
#[cfg(not(target_arch = "wasm32"))]
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

// Device-based sources and destinations (Native only)
#[cfg(not(target_arch = "wasm32"))]
pub use device_destination::OutputDeviceDestination;
#[cfg(not(target_arch = "wasm32"))]
pub use device_source::InputDeviceSource;
#[cfg(not(target_arch = "wasm32"))]
pub use file_recorder::FileRecorderDestination;

// Web Audio API backend and destination (WASM only)
#[cfg(target_arch = "wasm32")]
pub use wasm_processing::{
    AtomicAudioBuffer, AudioProcessingTask, EffectType, WorkerAudioProcessor,
};
#[cfg(target_arch = "wasm32")]
pub use web_audio_backend::WebAudioBackend;
#[cfg(target_arch = "wasm32")]
pub use web_audio_destination::WebAudioDestination;
