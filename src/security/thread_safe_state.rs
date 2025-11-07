//! Thread-safe audio state management
//!
//! Provides atomic and lock-based synchronization for audio state
//! to prevent race conditions and data corruption.

use parking_lot::{RwLock, Mutex};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

/// Thread-safe audio state container
#[derive(Clone)]
pub struct ThreadSafeAudioState {
    playback_state: Arc<RwLock<PlaybackState>>,
    position_us: Arc<AtomicU64>,
    duration_us: Arc<AtomicU64>,
    is_seeking: Arc<AtomicBool>,
    is_looping: Arc<AtomicBool>,
    volume: Arc<RwLock<f32>>,
    buffer_underrun_count: Arc<AtomicUsize>,
    last_error: Arc<RwLock<Option<String>>>,
}

/// Playback state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Buffering,
    Error,
}

impl ThreadSafeAudioState {
    /// Create a new thread-safe audio state
    pub fn new() -> Self {
        Self {
            playback_state: Arc::new(RwLock::new(PlaybackState::Stopped)),
            position_us: Arc::new(AtomicU64::new(0)),
            duration_us: Arc::new(AtomicU64::new(0)),
            is_seeking: Arc::new(AtomicBool::new(false)),
            is_looping: Arc::new(AtomicBool::new(false)),
            volume: Arc::new(RwLock::new(0.5)),
            buffer_underrun_count: Arc::new(AtomicUsize::new(0)),
            last_error: Arc::new(RwLock::new(None)),
        }
    }

    // === Playback State Methods ===

    /// Get current playback state
    pub fn get_state(&self) -> PlaybackState {
        *self.playback_state.read()
    }

    /// Set playback state
    pub fn set_state(&self, state: PlaybackState) {
        *self.playback_state.write() = state;
    }

    /// Atomically transition state if current matches expected
    pub fn compare_and_set_state(&self, expected: PlaybackState, new: PlaybackState) -> bool {
        let mut state = self.playback_state.write();
        if *state == expected {
            *state = new;
            true
        } else {
            false
        }
    }

    // === Position Methods ===

    /// Update playback position (in microseconds for precision)
    pub fn update_position(&self, position_us: u64) {
        if !self.is_seeking.load(Ordering::Acquire) {
            self.position_us.store(position_us, Ordering::Release);
        }
    }

    /// Get current position as Duration
    pub fn get_position(&self) -> Duration {
        Duration::from_micros(self.position_us.load(Ordering::Acquire))
    }

    /// Set position directly (used during seeking)
    pub fn set_position(&self, position: Duration) {
        self.position_us.store(position.as_micros() as u64, Ordering::Release);
    }

    // === Duration Methods ===

    /// Set total duration
    pub fn set_duration(&self, duration: Duration) {
        self.duration_us.store(duration.as_micros() as u64, Ordering::Release);
    }

    /// Get total duration
    pub fn get_duration(&self) -> Duration {
        Duration::from_micros(self.duration_us.load(Ordering::Acquire))
    }

    // === Seeking Methods ===

    /// Set seeking state
    pub fn set_seeking(&self, seeking: bool) {
        self.is_seeking.store(seeking, Ordering::SeqCst);
    }

    /// Check if currently seeking
    pub fn is_seeking(&self) -> bool {
        self.is_seeking.load(Ordering::Acquire)
    }

    /// Perform atomic seek operation
    pub fn perform_seek(&self, position: Duration) -> Result<(), String> {
        // Set seeking flag
        self.set_seeking(true);

        // Validate position
        let duration = self.get_duration();
        if position > duration {
            self.set_seeking(false);
            return Err(format!("Seek position {:?} exceeds duration {:?}", position, duration));
        }

        // Update position
        self.set_position(position);

        // Clear seeking flag
        self.set_seeking(false);
        Ok(())
    }

    // === Volume Methods ===

    /// Get current volume (thread-safe)
    pub fn get_volume(&self) -> f32 {
        *self.volume.read()
    }

    /// Set volume (thread-safe)
    pub fn set_volume(&self, volume: f32) -> Result<(), String> {
        if !(0.0..=1.0).contains(&volume) {
            return Err(format!("Invalid volume: {}", volume));
        }
        *self.volume.write() = volume;
        Ok(())
    }

    // === Loop Methods ===

    /// Set loop state
    pub fn set_looping(&self, looping: bool) {
        self.is_looping.store(looping, Ordering::Release);
    }

    /// Check if looping
    pub fn is_looping(&self) -> bool {
        self.is_looping.load(Ordering::Acquire)
    }

    // === Error Handling ===

    /// Set last error
    pub fn set_error(&self, error: Option<String>) {
        let has_error = error.is_some();
        *self.last_error.write() = error;
        if has_error {
            self.set_state(PlaybackState::Error);
        }
    }

    /// Get last error
    pub fn get_error(&self) -> Option<String> {
        self.last_error.read().clone()
    }

    /// Clear error state
    pub fn clear_error(&self) {
        *self.last_error.write() = None;
    }

    // === Buffer Underrun Tracking ===

    /// Increment buffer underrun count
    pub fn increment_underrun_count(&self) {
        self.buffer_underrun_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get buffer underrun count
    pub fn get_underrun_count(&self) -> usize {
        self.buffer_underrun_count.load(Ordering::Relaxed)
    }

    /// Reset buffer underrun count
    pub fn reset_underrun_count(&self) {
        self.buffer_underrun_count.store(0, Ordering::Relaxed);
    }

    // === State Queries ===

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        matches!(self.get_state(), PlaybackState::Playing)
    }

    /// Check if stopped
    pub fn is_stopped(&self) -> bool {
        matches!(self.get_state(), PlaybackState::Stopped)
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        matches!(self.get_state(), PlaybackState::Paused)
    }

    /// Get playback progress as percentage (0.0 to 1.0)
    pub fn get_progress(&self) -> f32 {
        let position = self.position_us.load(Ordering::Acquire) as f64;
        let duration = self.duration_us.load(Ordering::Acquire) as f64;

        if duration > 0.0 {
            (position / duration).min(1.0) as f32
        } else {
            0.0
        }
    }

    /// Reset all state to initial values
    pub fn reset(&self) {
        self.set_state(PlaybackState::Stopped);
        self.position_us.store(0, Ordering::Release);
        self.duration_us.store(0, Ordering::Release);
        self.is_seeking.store(false, Ordering::Release);
        self.is_looping.store(false, Ordering::Release);
        *self.volume.write() = 0.5;
        self.reset_underrun_count();
        self.clear_error();
    }
}

impl Default for ThreadSafeAudioState {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe audio statistics
pub struct AudioStatistics {
    samples_processed: Arc<AtomicU64>,
    peak_level: Arc<RwLock<f32>>,
    rms_level: Arc<RwLock<f32>>,
    clipping_count: Arc<AtomicUsize>,
}

impl AudioStatistics {
    pub fn new() -> Self {
        Self {
            samples_processed: Arc::new(AtomicU64::new(0)),
            peak_level: Arc::new(RwLock::new(0.0)),
            rms_level: Arc::new(RwLock::new(0.0)),
            clipping_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn update_samples_processed(&self, count: u64) {
        self.samples_processed.fetch_add(count, Ordering::Relaxed);
    }

    pub fn update_levels(&self, peak: f32, rms: f32) {
        *self.peak_level.write() = peak;
        *self.rms_level.write() = rms;
    }

    pub fn increment_clipping(&self) {
        self.clipping_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_statistics(&self) -> (u64, f32, f32, usize) {
        (
            self.samples_processed.load(Ordering::Relaxed),
            *self.peak_level.read(),
            *self.rms_level.read(),
            self.clipping_count.load(Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn test_thread_safety() {
        let state = Arc::new(ThreadSafeAudioState::new());
        let mut handles = vec![];

        // Spawn multiple threads that modify state
        for i in 0..10 {
            let state_clone = state.clone();
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let position = (i * 100 + j) as u64;
                    state_clone.update_position(position);
                    state_clone.set_volume((position as f32 % 100.0) / 100.0).ok();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify state is still valid
        assert!(state.get_volume() >= 0.0 && state.get_volume() <= 1.0);
    }

    #[test]
    fn test_state_transitions() {
        let state = ThreadSafeAudioState::new();

        // Test state transitions
        assert_eq!(state.get_state(), PlaybackState::Stopped);

        state.set_state(PlaybackState::Playing);
        assert_eq!(state.get_state(), PlaybackState::Playing);

        // Test compare-and-set
        assert!(state.compare_and_set_state(PlaybackState::Playing, PlaybackState::Paused));
        assert_eq!(state.get_state(), PlaybackState::Paused);

        // Failed compare-and-set
        assert!(!state.compare_and_set_state(PlaybackState::Playing, PlaybackState::Stopped));
        assert_eq!(state.get_state(), PlaybackState::Paused);
    }

    #[test]
    fn test_seeking() {
        let state = ThreadSafeAudioState::new();
        state.set_duration(Duration::from_secs(100));

        // Valid seek
        assert!(state.perform_seek(Duration::from_secs(50)).is_ok());
        assert_eq!(state.get_position(), Duration::from_secs(50));

        // Invalid seek (beyond duration)
        assert!(state.perform_seek(Duration::from_secs(150)).is_err());
    }

    #[test]
    fn test_progress_calculation() {
        let state = ThreadSafeAudioState::new();

        state.set_duration(Duration::from_secs(100));
        state.set_position(Duration::from_secs(25));

        assert!((state.get_progress() - 0.25).abs() < 0.001);

        state.set_position(Duration::from_secs(100));
        assert!((state.get_progress() - 1.0).abs() < 0.001);
    }
}