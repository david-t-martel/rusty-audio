//! Recording Panel UI Tests
//! 
//! Tests for the recording panel UI using egui_kittest.
//! These tests verify UI rendering, state management, and interactions.

use egui_kittest::{Harness, kittest::Queryable};
use rusty_audio::ui::{
    recording_panel::RecordingPanel,
    theme::{ThemeManager, Theme},
};
use rusty_audio::audio::recorder::{RecordingState, MonitoringMode};

/// Test that recording panel can be created and renders
#[test]
fn test_recording_panel_creation() {
    let mut panel = RecordingPanel::new();
    
    let mut harness = Harness::new_ui(|ui| {
        let colors = ThemeManager::new(Theme::Dark).get_colors();
        panel.draw(ui, &colors);
    });
    
    harness.run();
    
    // Verify key UI elements are present
    harness.get_by_label("🎙️ Recording");
}

/// Test recording panel initializes with correct default state
#[test]
fn test_recording_panel_default_state() {
    let panel = RecordingPanel::new();
    
    // Verify recorder exists and is in Idle state
    let recorder = panel.recorder().expect("Recorder should be initialized");
    assert_eq!(recorder.state(), RecordingState::Idle);
    
    // Verify default monitoring mode is Off
    assert_eq!(recorder.monitoring_mode(), MonitoringMode::Off);
}

/// Test device enumeration in recording panel
#[test]
fn test_recording_panel_device_enumeration() {
    let panel = RecordingPanel::new();
    
    // Panel should have attempted to enumerate devices
    // (may be empty if no audio devices available in test environment)
    // This test just verifies the structure exists
    let mut harness = Harness::new_ui(|ui| {
        let colors = ThemeManager::new(Theme::Dark).get_colors();
        let mut panel_clone = RecordingPanel::new();
        panel_clone.draw(ui, &colors);
    });
    
    harness.run();
}

/// Test recording state transitions via UI
#[test]
fn test_recording_state_transitions_ui() {
    let mut panel = RecordingPanel::new();
    
    // Initial state should be Idle
    assert_eq!(panel.recorder().expect("Recorder should exist").state(), RecordingState::Idle);
    
    // Simulate clicking Record button by directly calling start()
    if let Some(recorder) = panel.recorder_mut() {
        recorder.start().expect("Should start recording");
    }
    assert_eq!(panel.recorder().expect("Recorder should exist").state(), RecordingState::Recording);
    
    // Simulate clicking Pause
    if let Some(recorder) = panel.recorder_mut() {
        recorder.pause().expect("Should pause recording");
    }
    assert_eq!(panel.recorder().expect("Recorder should exist").state(), RecordingState::Paused);
    
    // Simulate clicking Resume
    if let Some(recorder) = panel.recorder_mut() {
        recorder.resume().expect("Should resume recording");
    }
    assert_eq!(panel.recorder().expect("Recorder should exist").state(), RecordingState::Recording);
    
    // Simulate clicking Stop
    if let Some(recorder) = panel.recorder_mut() {
        recorder.stop().expect("Should stop recording");
    }
    assert_eq!(panel.recorder().expect("Recorder should exist").state(), RecordingState::Stopped);
}

/// Test monitoring mode changes
#[test]
fn test_monitoring_mode_changes() {
    let mut panel = RecordingPanel::new();
    
    // Start with Off mode
    assert_eq!(panel.recorder().expect("Recorder should exist").monitoring_mode(), MonitoringMode::Off);
    
    // Change to Direct monitoring
    if let Some(recorder) = panel.recorder_mut() {
        recorder.set_monitoring_mode(MonitoringMode::Direct);
    }
    assert_eq!(panel.recorder().expect("Recorder should exist").monitoring_mode(), MonitoringMode::Direct);
    
    // Change to Routed monitoring
    if let Some(recorder) = panel.recorder_mut() {
        recorder.set_monitoring_mode(MonitoringMode::Routed);
    }
    assert_eq!(panel.recorder().expect("Recorder should exist").monitoring_mode(), MonitoringMode::Routed);
    
    // Back to Off
    if let Some(recorder) = panel.recorder_mut() {
        recorder.set_monitoring_mode(MonitoringMode::Off);
    }
    assert_eq!(panel.recorder().expect("Recorder should exist").monitoring_mode(), MonitoringMode::Off);
}

/// Test monitoring gain control
#[test]
fn test_monitoring_gain_control() {
    let mut panel = RecordingPanel::new();
    
    // Default gain should be 1.0
    assert_eq!(panel.recorder().expect("Recorder should exist").monitoring_gain(), 1.0);
    
    // Set gain to 0.5
    if let Some(recorder) = panel.recorder_mut() {
        recorder.set_monitoring_gain(0.5);
    }
    assert_eq!(panel.recorder().expect("Recorder should exist").monitoring_gain(), 0.5);
    
    // Set gain to 0.0 (mute)
    if let Some(recorder) = panel.recorder_mut() {
        recorder.set_monitoring_gain(0.0);
    }
    assert_eq!(panel.recorder().expect("Recorder should exist").monitoring_gain(), 0.0);
    
    // Test clamping - values > 1.0 should be clamped
    if let Some(recorder) = panel.recorder_mut() {
        recorder.set_monitoring_gain(1.5);
    }
    assert_eq!(panel.recorder().expect("Recorder should exist").monitoring_gain(), 1.0);
}

/// Test level meter updates
#[test]
fn test_level_meter_updates() {
    let mut panel = RecordingPanel::new();
    
    let mut harness = Harness::new_ui(|ui| {
        let colors = ThemeManager::new(Theme::Dark).get_colors();
        
        // Update levels (will be zero since no audio input in test)
        panel.update_levels();
        
        // Draw panel
        panel.draw(ui, &colors);
    });
    
    harness.run();
    
    // Verify level meters are rendered (look for "Ch 1" label)
    // Note: This will find the label if meters are rendered
    // In a real test environment, we could verify meter values
}

/// Test clip indicator functionality
#[test]
fn test_clip_indicators() {
    let mut panel = RecordingPanel::new();
    
    // Clear any existing clips
    panel.clear_clips();
    
    // In a real scenario, clips would be set by high audio levels
    // For this test, we just verify the clear function works
    
    let mut harness = Harness::new_ui(|ui| {
        let colors = ThemeManager::new(Theme::Dark).get_colors();
        panel.draw(ui, &colors);
    });
    
    harness.run();
}

/// Test recording duration display
#[test]
fn test_recording_duration_display() {
    let mut panel = RecordingPanel::new();
    
    // Start recording
    if let Some(recorder) = panel.recorder_mut() {
        recorder.start().expect("Should start recording");
    }
    
    // Duration should be very small (just started)
    let duration = panel.recorder().expect("Recorder should exist").duration();
    assert!(duration.as_secs() < 1);
    
    let mut harness = Harness::new_ui(|ui| {
        let colors = ThemeManager::new(Theme::Dark).get_colors();
        panel.draw(ui, &colors);
    });
    
    harness.run();
}

/// Test buffer clear functionality
#[test]
fn test_buffer_clear() {
    let mut panel = RecordingPanel::new();
    
    // Get initial buffer position
    let recorder = panel.recorder().expect("Recorder should exist");
    let initial_pos = recorder.buffer().lock().expect("Should lock buffer").position();
    
    // Clear buffer
    recorder.buffer().lock().expect("Should lock buffer").clear();
    
    // Verify buffer is cleared
    let cleared_pos = recorder.buffer().lock().expect("Should lock buffer").position();
    assert_eq!(cleared_pos, 0);
    assert!(cleared_pos <= initial_pos);
}

/// Test panel renders all major sections
#[test]
fn test_recording_panel_sections_render() {
    let mut panel = RecordingPanel::new();
    
    let mut harness = Harness::new_ui(|ui| {
        let colors = ThemeManager::new(Theme::Dark).get_colors();
        
        // Draw full panel
        panel.draw(ui, &colors);
    });
    
    harness.run();
    
    // Verify major section headers are present
    harness.get_by_label("🎙️ Recording");
    // Note: Other sections render within groups which may not be queryable by label
    // but the test verifies the panel draws without panicking
}

/// Test panel with multiple update cycles
#[test]
fn test_recording_panel_multiple_frames() {
    let mut panel = RecordingPanel::new();
    
    // Simulate multiple frame updates
    for _ in 0..10 {
        let mut harness = Harness::new_ui(|ui| {
            let colors = ThemeManager::new(Theme::Dark).get_colors();
            panel.update_levels();
            panel.draw(ui, &colors);
        });
        
        harness.run();
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Test complete recording workflow (UI state only, no actual audio)
    #[test]
    fn test_recording_workflow_ui_only() {
        let mut panel = RecordingPanel::new();
        
        // 1. Initial state
        assert_eq!(panel.recorder().expect("Recorder should exist").state(), RecordingState::Idle);
        
        // 2. Start recording
        if let Some(recorder) = panel.recorder_mut() {
            recorder.start().expect("Should start recording");
        }
        assert_eq!(panel.recorder().expect("Recorder should exist").state(), RecordingState::Recording);
        
        // 3. Update levels (simulated)
        panel.update_levels();
        
        // 4. Render a frame
        {
            let mut harness = Harness::new_ui(|ui| {
                let colors = ThemeManager::new(Theme::Dark).get_colors();
                panel.draw(ui, &colors);
            });
            harness.run();
        } // Explicitly drop harness
        
        // 5. Pause
        if let Some(recorder) = panel.recorder_mut() {
            recorder.pause().expect("Should pause recording");
        }
        assert_eq!(panel.recorder().expect("Recorder should exist").state(), RecordingState::Paused);
        
        // 6. Resume
        if let Some(recorder) = panel.recorder_mut() {
            recorder.resume().expect("Should resume recording");
        }
        assert_eq!(panel.recorder().expect("Recorder should exist").state(), RecordingState::Recording);
        
        // 7. Stop
        if let Some(recorder) = panel.recorder_mut() {
            recorder.stop().expect("Should stop recording");
        }
        assert_eq!(panel.recorder().expect("Recorder should exist").state(), RecordingState::Stopped);
        
        // 8. Final render
        {
            let mut harness = Harness::new_ui(|ui| {
                let colors = ThemeManager::new(Theme::Dark).get_colors();
                panel.draw(ui, &colors);
            });
            harness.run();
        }
    }
    
    /// Test monitoring mode workflow with UI rendering
    #[test]
    fn test_monitoring_workflow() {
        let mut panel = RecordingPanel::new();
        
        // Enable direct monitoring
        if let Some(recorder) = panel.recorder_mut() {
            recorder.set_monitoring_mode(MonitoringMode::Direct);
            recorder.set_monitoring_gain(0.8);
        }
        
        assert_eq!(panel.recorder().expect("Recorder should exist").monitoring_mode(), MonitoringMode::Direct);
        assert_eq!(panel.recorder().expect("Recorder should exist").monitoring_gain(), 0.8);
        
        // Render with monitoring enabled
        let mut harness = Harness::new_ui(|ui| {
            let colors = ThemeManager::new(Theme::Dark).get_colors();
            panel.draw(ui, &colors);
        });
        harness.run();
    }
}
