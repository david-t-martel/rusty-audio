//! Platform-specific abstractions for cross-platform compatibility
//!
//! This module provides platform-specific types and functions that work
//! on both native (desktop) and WASM (web) targets.

use std::sync::Arc;

// ============================================================================
// File Handle Abstraction
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub type PlatformFileHandle = Arc<std::path::PathBuf>;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub struct WasmFileHandle {
    pub name: String,
    pub data: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
pub type PlatformFileHandle = Arc<WasmFileHandle>;

// ============================================================================
// Audio Context Abstraction
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub use web_audio_api::context::AudioContext as PlatformAudioContext;

#[cfg(target_arch = "wasm32")]
/// Stub audio context for WASM (will be replaced with web-sys AudioContext)
pub struct PlatformAudioContext;

#[cfg(target_arch = "wasm32")]
impl PlatformAudioContext {
    pub fn default() -> Self {
        Self
    }
}

// ============================================================================
// File Picker Functions
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub fn pick_audio_file() -> Option<PlatformFileHandle> {
    rfd::FileDialog::new()
        .add_filter("audio", &["mp3", "wav", "flac", "ogg", "m4a"])
        .pick_file()
        .map(Arc::new)
}

#[cfg(target_arch = "wasm32")]
pub fn pick_audio_file() -> Option<PlatformFileHandle> {
    // TODO: Implement file picker using web-sys File API
    log::warn!("File picker not yet implemented for WASM");
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_file_bytes(handle: &PlatformFileHandle) -> std::io::Result<Vec<u8>> {
    std::fs::read(handle.as_ref())
}

#[cfg(target_arch = "wasm32")]
pub fn read_file_bytes(handle: &PlatformFileHandle) -> std::io::Result<Vec<u8>> {
    Ok(handle.data.clone())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_file_name(handle: &PlatformFileHandle) -> String {
    handle
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

#[cfg(target_arch = "wasm32")]
pub fn get_file_name(handle: &PlatformFileHandle) -> String {
    handle.name.clone()
}
