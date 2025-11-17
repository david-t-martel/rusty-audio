//! Backend selection and management
//!
//! This module provides utilities for enumerating available audio backends
//! and creating instances based on user preference.

use super::backend::{AudioBackend, AudioBackendError, Result};

#[cfg(not(target_arch = "wasm32"))]
use super::device::CpalBackend;

#[cfg(target_os = "windows")]
use super::asio_backend::{AsioBackend, WindowsBackendType};

#[cfg(target_arch = "wasm32")]
use super::web_audio_backend::WebAudioBackend;

/// Information about an available audio backend
#[derive(Debug, Clone)]
pub struct BackendInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_available: bool,
    pub is_low_latency: bool,
    pub platform: String,
}

/// Backend selector for choosing between available audio backends
pub struct BackendSelector {
    available_backends: Vec<BackendInfo>,
}

impl BackendSelector {
    /// Create a new backend selector
    pub fn new() -> Self {
        let available_backends = Self::enumerate_backends();
        Self { available_backends }
    }

    /// Enumerate all available backends on the current platform
    pub fn enumerate_backends() -> Vec<BackendInfo> {
        let mut backends = Vec::new();

        #[cfg(target_arch = "wasm32")]
        {
            // Web Audio API is the only backend for WASM
            backends.push(BackendInfo {
                id: "web_audio".to_string(),
                name: "Web Audio API".to_string(),
                description: "Browser-based Web Audio API".to_string(),
                is_available: true,
                is_low_latency: false,
                platform: "WebAssembly".to_string(),
            });
            return backends;
        }

        #[cfg(target_os = "windows")]
        {
            // Check ASIO availability
            if AsioBackend::asio_available() {
                backends.push(BackendInfo {
                    id: "asio".to_string(),
                    name: "ASIO".to_string(),
                    description: "Professional low-latency audio (requires ASIO drivers)"
                        .to_string(),
                    is_available: true,
                    is_low_latency: true,
                    platform: "Windows".to_string(),
                });
            }

            // WASAPI is always available on Windows Vista and later
            backends.push(BackendInfo {
                id: "wasapi".to_string(),
                name: "WASAPI".to_string(),
                description: "Windows Audio Session API (standard)".to_string(),
                is_available: true,
                is_low_latency: false,
                platform: "Windows".to_string(),
            });

            // DirectSound for compatibility
            backends.push(BackendInfo {
                id: "directsound".to_string(),
                name: "DirectSound".to_string(),
                description: "Legacy DirectSound (compatibility mode)".to_string(),
                is_available: true,
                is_low_latency: false,
                platform: "Windows".to_string(),
            });
        }

        #[cfg(target_os = "macos")]
        {
            backends.push(BackendInfo {
                id: "coreaudio".to_string(),
                name: "Core Audio".to_string(),
                description: "macOS Core Audio framework".to_string(),
                is_available: true,
                is_low_latency: true,
                platform: "macOS".to_string(),
            });
        }

        #[cfg(target_os = "linux")]
        {
            backends.push(BackendInfo {
                id: "alsa".to_string(),
                name: "ALSA".to_string(),
                description: "Advanced Linux Sound Architecture".to_string(),
                is_available: true,
                is_low_latency: true,
                platform: "Linux".to_string(),
            });

            // Note: JACK and PulseAudio would need additional implementation
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Always add CPAL default as fallback for non-WASM platforms
            backends.push(BackendInfo {
                id: "cpal_default".to_string(),
                name: "Default".to_string(),
                description: "Platform default audio backend".to_string(),
                is_available: true,
                is_low_latency: false,
                platform: std::env::consts::OS.to_string(),
            });
        }

        backends
    }

    /// Get available backends
    pub fn available_backends(&self) -> &[BackendInfo] {
        &self.available_backends
    }

    /// Get backend info by ID
    pub fn get_backend_info(&self, backend_id: &str) -> Option<&BackendInfo> {
        self.available_backends.iter().find(|b| b.id == backend_id)
    }

    /// Create a backend instance by ID
    pub fn create_backend(&self, backend_id: &str) -> Result<Box<dyn AudioBackend>> {
        #[cfg(target_arch = "wasm32")]
        {
            match backend_id {
                "web_audio" => Ok(Box::new(WebAudioBackend::new())),
                _ => Err(AudioBackendError::BackendNotAvailable(format!(
                    "Backend not available on WASM platform: {}",
                    backend_id
                ))),
            }
        }

        #[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
        {
            match backend_id {
                "asio" => {
                    let backend = AsioBackend::with_backend_type(WindowsBackendType::Asio);
                    if !backend.is_available() {
                        return Err(AudioBackendError::BackendNotAvailable(
                            "ASIO drivers not installed".to_string(),
                        ));
                    }
                    Ok(Box::new(backend))
                }
                "wasapi" => {
                    let backend = AsioBackend::with_backend_type(WindowsBackendType::Wasapi);
                    Ok(Box::new(backend))
                }
                "directsound" => {
                    let backend = AsioBackend::with_backend_type(WindowsBackendType::DirectSound);
                    Ok(Box::new(backend))
                }
                "cpal_default" => Ok(Box::new(CpalBackend::new())),
                _ => Err(AudioBackendError::BackendNotAvailable(format!(
                    "Unknown backend: {}",
                    backend_id
                ))),
            }
        }

        #[cfg(all(not(target_os = "windows"), not(target_arch = "wasm32")))]
        {
            match backend_id {
                "cpal_default" | "coreaudio" | "alsa" => Ok(Box::new(CpalBackend::new())),
                _ => Err(AudioBackendError::BackendNotAvailable(format!(
                    "Backend not available on this platform: {}",
                    backend_id
                ))),
            }
        }
    }

    /// Get the recommended backend for the current platform
    pub fn recommended_backend_id(&self) -> &str {
        #[cfg(target_arch = "wasm32")]
        {
            "web_audio"
        }

        #[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
        {
            // Prefer ASIO if available, fallback to WASAPI
            if self
                .available_backends
                .iter()
                .any(|b| b.id == "asio" && b.is_available)
            {
                "asio"
            } else {
                "wasapi"
            }
        }

        #[cfg(all(target_os = "macos", not(target_arch = "wasm32")))]
        {
            "coreaudio"
        }

        #[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
        {
            "alsa"
        }

        #[cfg(all(
            not(any(target_os = "windows", target_os = "macos", target_os = "linux")),
            not(target_arch = "wasm32")
        ))]
        {
            "cpal_default"
        }
    }

    /// Create the recommended backend
    pub fn create_recommended_backend(&self) -> Result<Box<dyn AudioBackend>> {
        let backend_id = self.recommended_backend_id();
        self.create_backend(backend_id)
    }

    /// Refresh the list of available backends
    pub fn refresh(&mut self) {
        self.available_backends = Self::enumerate_backends();
    }
}

impl Default for BackendSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_enumeration() {
        let selector = BackendSelector::new();
        let backends = selector.available_backends();

        // Should always have at least the default backend
        assert!(!backends.is_empty());

        // All backends should have non-empty names
        for backend in backends {
            assert!(!backend.name.is_empty());
            assert!(!backend.id.is_empty());
        }
    }

    #[test]
    fn test_get_backend_info() {
        let selector = BackendSelector::new();

        // Should always have default backend
        let default_info = selector.get_backend_info("cpal_default");
        assert!(default_info.is_some());
    }

    #[test]
    fn test_recommended_backend() {
        let selector = BackendSelector::new();
        let backend_id = selector.recommended_backend_id();

        // Should be able to get info for recommended backend
        let info = selector.get_backend_info(backend_id);
        assert!(info.is_some());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_backends() {
        let selector = BackendSelector::new();
        let backends = selector.available_backends();

        // Windows should have at least WASAPI and default
        assert!(backends.iter().any(|b| b.id == "wasapi"));
        assert!(backends.iter().any(|b| b.id == "cpal_default"));
    }

    #[test]
    fn test_create_default_backend() {
        let selector = BackendSelector::new();
        let result = selector.create_backend("cpal_default");
        assert!(result.is_ok());
    }
}
