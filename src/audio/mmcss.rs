//! Windows Multimedia Class Scheduler Service (MMCSS) integration
//!
//! MMCSS provides guaranteed CPU scheduling for multimedia threads, ensuring
//! that audio processing threads get consistent CPU time without being
//! preempted by lower-priority threads.
//!
//! This module provides safe Rust wrappers around Windows MMCSS APIs for:
//! - Thread priority boosting
//! - Guaranteed CPU scheduling
//! - Reduced audio glitches and dropouts
//!
//! # Platform Support
//! Windows Vista and later only. On other platforms, these functions are no-ops.
//!
//! # Usage
//! ```rust,no_run
//! use rusty_audio::audio::mmcss::MmcssHandle;
//!
//! // Register current thread for audio processing
//! let handle = MmcssHandle::register_thread("Pro Audio")?;
//! // Thread now has guaranteed scheduling
//! // ... do audio processing ...
//! // handle automatically unregisters on drop
//! ```

use thiserror::Error;

/// Errors that can occur during MMCSS operations
#[derive(Error, Debug)]
pub enum MmcssError {
    #[error("MMCSS not available on this platform")]
    NotAvailable,

    #[error("Failed to register thread: {0}")]
    RegistrationFailed(String),

    #[error("Failed to set priority: {0}")]
    SetPriorityFailed(String),

    #[error("Invalid task name: {0}")]
    InvalidTaskName(String),
}

pub type Result<T> = std::result::Result<T, MmcssError>;

/// MMCSS task categories
///
/// These are predefined Windows task names that specify different
/// multimedia workload characteristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmcssTaskCategory {
    /// Professional audio workloads (lowest latency, highest priority)
    ProAudio,
    /// Standard audio playback
    Audio,
    /// Audio capture/recording
    Capture,
    /// Game audio
    Games,
    /// Playback with buffering
    Playback,
    /// Window Manager rendering
    WindowManager,
}

impl MmcssTaskCategory {
    /// Get the Windows task name for this category
    pub fn task_name(&self) -> &'static str {
        match self {
            MmcssTaskCategory::ProAudio => "Pro Audio",
            MmcssTaskCategory::Audio => "Audio",
            MmcssTaskCategory::Capture => "Capture",
            MmcssTaskCategory::Games => "Games",
            MmcssTaskCategory::Playback => "Playback",
            MmcssTaskCategory::WindowManager => "Window Manager",
        }
    }
}

/// Handle to an MMCSS-registered thread
///
/// Automatically unregisters the thread when dropped.
#[cfg(target_os = "windows")]
pub struct MmcssHandle {
    task_handle: windows::Win32::System::Threading::HANDLE,
}

#[cfg(target_os = "windows")]
impl MmcssHandle {
    /// Register the current thread with MMCSS
    ///
    /// # Arguments
    /// * `category` - The MMCSS task category for this thread
    ///
    /// # Returns
    /// A handle that automatically unregisters the thread on drop
    ///
    /// # Example
    /// ```rust,no_run
    /// let handle = MmcssHandle::register_thread(MmcssTaskCategory::ProAudio)?;
    /// // Audio processing with guaranteed scheduling
    /// ```
    pub fn register_thread(category: MmcssTaskCategory) -> Result<Self> {
        Self::register_thread_with_name(category.task_name())
    }

    /// Register the current thread with MMCSS using a custom task name
    ///
    /// # Arguments
    /// * `task_name` - Windows MMCSS task name (e.g., "Pro Audio", "Audio")
    pub fn register_thread_with_name(task_name: &str) -> Result<Self> {
        use windows::core::PCWSTR;
        use windows::Win32::System::Threading::AvSetMmThreadCharacteristicsW;

        // Convert task name to wide string
        let task_name_wide: Vec<u16> = task_name.encode_utf16().chain(std::iter::once(0)).collect();

        let mut task_index: u32 = 0;

        // SAFETY: We're calling a Windows API with a valid null-terminated wide string
        let task_handle = unsafe {
            AvSetMmThreadCharacteristicsW(
                PCWSTR(task_name_wide.as_ptr()),
                &mut task_index as *mut u32,
            )
        }
        .map_err(|e| MmcssError::RegistrationFailed(format!("Windows API error: {}", e)))?;

        if task_handle.is_invalid() {
            return Err(MmcssError::RegistrationFailed(
                "Invalid handle returned".to_string(),
            ));
        }

        Ok(Self { task_handle })
    }

    /// Set the relative thread priority within the MMCSS task
    ///
    /// # Arguments
    /// * `priority` - Priority value (AVRT_PRIORITY_NORMAL, HIGH, etc.)
    ///                Valid range: AVRT_PRIORITY_LOW (-1) to AVRT_PRIORITY_HIGH (1)
    ///
    /// # Note
    /// This adjusts priority relative to other threads in the same MMCSS task,
    /// not system-wide priority.
    pub fn set_priority(&self, priority: i32) -> Result<()> {
        use windows::Win32::System::Threading::AvSetMmThreadPriority;
        use windows::Win32::System::Threading::AVRT_PRIORITY;

        // Clamp priority to valid range
        let priority = priority.clamp(-1, 1);

        // SAFETY: We're calling a Windows API with a valid task handle
        unsafe { AvSetMmThreadPriority(self.task_handle, AVRT_PRIORITY(priority)) }
            .map_err(|e| MmcssError::SetPriorityFailed(format!("Windows API error: {}", e)))
    }

    /// Get the task handle (for advanced use cases)
    pub fn handle(&self) -> windows::Win32::System::Threading::HANDLE {
        self.task_handle
    }
}

#[cfg(target_os = "windows")]
impl Drop for MmcssHandle {
    fn drop(&mut self) {
        use windows::Win32::System::Threading::AvRevertMmThreadCharacteristics;

        // SAFETY: We're reverting the MMCSS characteristics we set
        let _ = unsafe { AvRevertMmThreadCharacteristics(self.task_handle) };
    }
}

// Stub implementation for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub struct MmcssHandle;

#[cfg(not(target_os = "windows"))]
impl MmcssHandle {
    pub fn register_thread(_category: MmcssTaskCategory) -> Result<Self> {
        Err(MmcssError::NotAvailable)
    }

    pub fn register_thread_with_name(_task_name: &str) -> Result<Self> {
        Err(MmcssError::NotAvailable)
    }

    pub fn set_priority(&self, _priority: i32) -> Result<()> {
        Err(MmcssError::NotAvailable)
    }
}

/// Check if MMCSS is available on the current platform
pub fn is_available() -> bool {
    cfg!(target_os = "windows")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_category_names() {
        assert_eq!(MmcssTaskCategory::ProAudio.task_name(), "Pro Audio");
        assert_eq!(MmcssTaskCategory::Audio.task_name(), "Audio");
        assert_eq!(MmcssTaskCategory::Capture.task_name(), "Capture");
    }

    #[test]
    fn test_availability() {
        #[cfg(target_os = "windows")]
        {
            assert!(is_available());
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert!(!is_available());
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_thread_registration() {
        // This test should work on Windows with MMCSS enabled
        let result = MmcssHandle::register_thread(MmcssTaskCategory::Audio);
        // May fail if not running with sufficient privileges
        // Just verify the API doesn't panic
        let _ = result;
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_not_available_on_other_platforms() {
        let result = MmcssHandle::register_thread(MmcssTaskCategory::Audio);
        assert!(matches!(result, Err(MmcssError::NotAvailable)));
    }
}
