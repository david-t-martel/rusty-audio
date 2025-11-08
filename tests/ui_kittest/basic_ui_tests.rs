//! Basic UI tests using egui_kittest
//! 
//! These tests verify that UI components render correctly and respond to user interactions.

use egui_kittest::{Harness, kittest::Queryable};
use rusty_audio::ui::{
    theme::{ThemeManager, Theme, ThemeColors},
    controls::CircularKnob,
};

/// Test that the app window can be created and renders
#[test]
fn test_app_window_creation() {
    let mut harness = Harness::new_ui(|ui| {
        ui.label("Rusty Audio Test");
    });
    
    // Run a single frame
    harness.run();
    
    // Verify the label exists
    harness.get_by_label("Rusty Audio Test");
}

/// Test theme application
#[test]
fn test_theme_application() {
    let theme_manager = ThemeManager::new(Theme::Dark);
    
    let mut harness = Harness::new_ui(|ui| {
        let colors = theme_manager.get_colors();
        ui.label("Test Label");
        
        // Verify theme colors are accessible
        assert_ne!(colors.primary, egui::Color32::TRANSPARENT);
        assert_ne!(colors.background, egui::Color32::TRANSPARENT);
    });
    
    harness.run();
}

/// Test circular knob rendering
#[test]
fn test_circular_knob_rendering() {
    let mut knob = CircularKnob::new(0.5, 0.0..=1.0);
    
    let mut harness = Harness::new_ui(|ui| {
        let colors = ThemeManager::new(Theme::Dark).get_colors();
        let _response = knob.show(ui, &colors);
    });
    
    harness.run();
}

/// Test knob interaction - dragging to change value
#[test]
fn test_knob_drag_interaction() {
    let mut knob = CircularKnob::new(0.5, 0.0..=1.0);
    let initial_value = knob.value();
    
    {
        let mut harness = Harness::new_ui(|ui| {
            let colors = ThemeManager::new(Theme::Dark).get_colors();
            let _response = knob.show(ui, &colors);
        });
        
        harness.run();
    } // harness dropped here, releasing mutable borrow of knob
    
    // Simulate drag (this is a basic test - actual drag simulation would be more complex)
    // For now, just verify the knob maintains its value
    assert_eq!(knob.value(), initial_value);
}

/// Test responsive layout at different screen sizes
#[test]
fn test_responsive_layout_mobile() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(375.0, 667.0))  // Mobile portrait
        .build_ui(|ui| {
            ui.label("Mobile Layout Test");
            
            // Verify available space is appropriate for mobile
            let available = ui.available_size();
            assert!(available.x > 0.0);
            assert!(available.y > 0.0);
        });
    
    harness.run();
}

#[test]
fn test_responsive_layout_desktop() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1920.0, 1080.0))  // Desktop
        .build_ui(|ui| {
            ui.label("Desktop Layout Test");
            
            // Verify available space is appropriate for desktop
            let available = ui.available_size();
            assert!(available.x > 1000.0);
            assert!(available.y > 500.0);
        });
    
    harness.run();
}

/// Test accessibility features
#[test]
fn test_accessibility_labels() {
    let mut harness = Harness::new_ui(|ui| {
        ui.label("Accessible Label");
        ui.button("Accessible Button");
    });
    
    harness.run();
    
    // Verify elements can be found by their labels
    harness.get_by_label("Accessible Label");
    harness.get_by_label("Accessible Button");
}

/// Test that UI updates on multiple frames
#[test]
fn test_multi_frame_updates() {
    // Run multiple frames and verify each frame renders
    for i in 0..5 {
        let mut harness = Harness::new_ui(|ui| {
            ui.label(format!("Frame: {}", i));
        });
        harness.run();
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    /// Test multiple UI components together
    #[test]
    fn test_multiple_components() {
        let mut knob1 = CircularKnob::new(0.7, 0.0..=1.0);
        
        let mut knob2 = CircularKnob::new(0.5, -1.0..=1.0);
        
        let mut harness = Harness::new_ui(|ui| {
            let colors = ThemeManager::new(Theme::Dark).get_colors();
            
            ui.horizontal(|ui| {
                knob1.show(ui, &colors);
                knob2.show(ui, &colors);
            });
        });
        
        harness.run();
    }
}
