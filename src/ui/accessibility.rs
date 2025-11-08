use egui::{Ui, Vec2, Rect, Pos2, Color32, Response, RichText, Key, Modifiers, CursorIcon, Id};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use super::theme::ThemeColors;
use super::utils::{AnimationState, ColorUtils, DrawUtils};

/// Comprehensive accessibility system for the audio application
#[derive(Debug, Clone)]
pub struct AccessibilityManager {
    /// Tooltip system for contextual help
    tooltips: TooltipManager,
    /// Keyboard navigation system
    keyboard_nav: KeyboardNavigationManager,
    /// Screen reader support
    screen_reader: ScreenReaderManager,
    /// Focus management
    focus_manager: FocusManager,
    /// Safety system for audio controls
    safety_manager: SafetyManager,
    /// High contrast mode support
    high_contrast_mode: bool,
    /// Help system
    help_system: HelpSystem,
}

impl Default for AccessibilityManager {
    fn default() -> Self {
        Self {
            tooltips: TooltipManager::new(),
            keyboard_nav: KeyboardNavigationManager::new(),
            screen_reader: ScreenReaderManager::new(),
            focus_manager: FocusManager::new(),
            safety_manager: SafetyManager::new(),
            high_contrast_mode: false,
            help_system: HelpSystem::new(),
        }
    }
}

impl AccessibilityManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update accessibility systems each frame
    pub fn update(&mut self, ui: &Ui, dt: f32) {
        self.tooltips.update(dt);
        self.keyboard_nav.update(ui);
        self.focus_manager.update(ui, dt);
        self.safety_manager.update(dt);
        self.help_system.update(dt);
    }

    /// Handle keyboard input for accessibility features
    pub fn handle_keyboard_input(&mut self, ui: &Ui) -> AccessibilityAction {
        let mut action = AccessibilityAction::None;

        ui.input(|i| {
            // Emergency volume reduction (Escape key)
            if i.key_pressed(Key::Escape) {
                action = AccessibilityAction::EmergencyVolumeReduction;
            }

            // Help system toggle (F1)
            if i.key_pressed(Key::F1) {
                self.help_system.toggle_overlay();
                action = AccessibilityAction::ToggleHelp;
            }

            // High contrast mode toggle (Ctrl+Shift+H)
            if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(Key::H) {
                self.high_contrast_mode = !self.high_contrast_mode;
                action = AccessibilityAction::ToggleHighContrast;
            }

            // Tab navigation
            if i.key_pressed(Key::Tab) {
                if i.modifiers.shift {
                    self.focus_manager.focus_previous();
                } else {
                    self.focus_manager.focus_next();
                }
                action = AccessibilityAction::NavigateFocus;
            }

            // Arrow key navigation for focused controls
            if self.focus_manager.has_focus() {
                if i.key_pressed(Key::ArrowLeft) {
                    action = AccessibilityAction::AdjustFocusedControl(-0.05);
                } else if i.key_pressed(Key::ArrowRight) {
                    action = AccessibilityAction::AdjustFocusedControl(0.05);
                } else if i.key_pressed(Key::ArrowUp) {
                    action = AccessibilityAction::AdjustFocusedControl(0.05);
                } else if i.key_pressed(Key::ArrowDown) {
                    action = AccessibilityAction::AdjustFocusedControl(-0.05);
                }
            }
        });

        action
    }

    /// Show tooltip for a control with accessibility information
    pub fn show_tooltip(&mut self, ui: &Ui, response: &Response, tooltip_info: TooltipInfo) {
        if response.hovered() {
            self.tooltips.show_tooltip(ui, response.rect, tooltip_info);
        }
    }

    /// Register a focusable control
    pub fn register_focusable(&mut self, id: Id, control_type: FocusableControlType, rect: Rect) {
        self.focus_manager.register_control(id, control_type, rect);
    }

    /// Check if a control has focus and draw focus indicator
    pub fn draw_focus_indicator(&mut self, ui: &Ui, id: Id, rect: Rect, colors: &ThemeColors) -> bool {
        let has_focus = self.focus_manager.is_focused(id);
        if has_focus {
            self.draw_focus_outline(ui, rect, colors);
        }
        has_focus
    }

    /// Draw focus outline around a control
    fn draw_focus_outline(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let focus_color = if self.high_contrast_mode {
            Color32::YELLOW
        } else {
            colors.accent
        };

        // Animated focus outline
        let outline_rect = rect.expand(3.0);
        painter.rect_stroke(
            outline_rect,
            4.0,
            egui::Stroke::new(2.0, focus_color),
            egui::epaint::StrokeKind::Outside,
        );

        // Additional inner outline for high contrast
        if self.high_contrast_mode {
            painter.rect_stroke(
                rect.expand(1.0),
                2.0,
                egui::Stroke::new(1.0, Color32::BLACK),
                egui::epaint::StrokeKind::Outside,
            );
        }
    }

    /// Get current volume safety status
    pub fn get_volume_safety_status(&self) -> VolumeSafetyStatus {
        self.safety_manager.get_status()
    }

    /// Check if volume level is safe
    pub fn is_volume_safe(&self, volume: f32) -> bool {
        self.safety_manager.is_volume_safe(volume)
    }

    /// Get accessibility-enhanced colors for high contrast mode
    pub fn get_accessible_colors(&self, base_colors: &ThemeColors) -> ThemeColors {
        if self.high_contrast_mode {
            ThemeColors {
                primary: Color32::YELLOW,
                secondary: Color32::from_rgb(0, 255, 255), // CYAN
                accent: Color32::WHITE,
                background: Color32::BLACK,
                surface: Color32::from_gray(32),
                text: Color32::WHITE,
                text_secondary: Color32::LIGHT_GRAY,
                success: Color32::GREEN,
                warning: Color32::YELLOW,
                error: Color32::RED,
                spectrum_colors: vec![
                    Color32::RED,
                    Color32::YELLOW,
                    Color32::GREEN,
                    Color32::from_rgb(0, 255, 255), // CYAN
                    Color32::BLUE,
                    Color32::from_rgb(255, 0, 255), // MAGENTA
                    Color32::WHITE,
                    Color32::LIGHT_GRAY,
                ],
            }
        } else {
            base_colors.clone()
        }
    }

    /// Show contextual help overlay
    pub fn show_help_overlay(&mut self, ui: &Ui, colors: &ThemeColors) {
        if self.help_system.is_overlay_visible() {
            self.help_system.draw_overlay(ui, colors);
        }
    }

    /// Add screen reader announcement
    pub fn announce(&mut self, message: String, priority: AnnouncementPriority) {
        self.screen_reader.announce(message, priority);
    }

    /// Check if high contrast mode is enabled
    pub fn is_high_contrast_mode(&self) -> bool {
        self.high_contrast_mode
    }

    /// Toggle high contrast mode
    pub fn toggle_high_contrast_mode(&mut self) {
        self.high_contrast_mode = !self.high_contrast_mode;
    }
}

/// Actions that can be triggered by accessibility system
#[derive(Debug, Clone, PartialEq)]
pub enum AccessibilityAction {
    None,
    EmergencyVolumeReduction,
    ToggleHelp,
    ToggleHighContrast,
    NavigateFocus,
    AdjustFocusedControl(f32),
}

/// Tooltip management system
#[derive(Debug, Clone)]
pub struct TooltipManager {
    current_tooltip: Option<(TooltipInfo, Instant)>,
    show_delay: Duration,
    fade_animation: AnimationState,
}

#[derive(Debug, Clone)]
pub struct TooltipInfo {
    pub title: String,
    pub description: String,
    pub shortcuts: Vec<String>,
    pub value_range: Option<(f32, f32)>,
    pub current_value: Option<f32>,
    pub safety_info: Option<String>,
}

impl TooltipManager {
    pub fn new() -> Self {
        Self {
            current_tooltip: None,
            show_delay: Duration::from_millis(800),
            fade_animation: AnimationState::new(0.0, 8.0),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.fade_animation.update(dt);

        // Clear old tooltips
        if let Some((_, start_time)) = &self.current_tooltip {
            if start_time.elapsed() > Duration::from_secs(10) {
                self.current_tooltip = None;
                self.fade_animation.set_target(0.0);
            }
        }
    }

    pub fn show_tooltip(&mut self, ui: &Ui, anchor_rect: Rect, info: TooltipInfo) {
        let now = Instant::now();

        // Update or set current tooltip
        match &self.current_tooltip {
            Some((current_info, start_time)) if current_info.title == info.title => {
                // Same tooltip, keep showing
                if start_time.elapsed() > self.show_delay {
                    self.fade_animation.set_target(1.0);
                    self.draw_tooltip(ui, anchor_rect, &info);
                }
            }
            _ => {
                // New tooltip
                self.current_tooltip = Some((info.clone(), now));
                self.fade_animation.set_target(0.0);
            }
        }
    }

    fn draw_tooltip(&self, ui: &Ui, anchor_rect: Rect, info: &TooltipInfo) {
        let alpha = self.fade_animation.value();
        if alpha < 0.01 {
            return;
        }

        let tooltip_pos = Pos2::new(
            anchor_rect.max.x + 10.0,
            anchor_rect.min.y,
        );

        egui::Area::new("accessibility_tooltip".into())
            .fixed_pos(tooltip_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .shadow(egui::epaint::Shadow::default())
                    .show(ui, |ui| {
                        ui.set_max_width(300.0);

                        // Title
                        ui.label(RichText::new(&info.title).strong().size(14.0));

                        if !info.description.is_empty() {
                            ui.add_space(4.0);
                            ui.label(&info.description);
                        }

                        // Current value
                        if let Some(value) = info.current_value {
                            ui.add_space(4.0);
                            let value_text = if let Some((min, max)) = info.value_range {
                                format!("Value: {:.2} (Range: {:.2} - {:.2})", value, min, max)
                            } else {
                                format!("Value: {:.2}", value)
                            };
                            ui.label(RichText::new(value_text).color(Color32::LIGHT_BLUE));
                        }

                        // Keyboard shortcuts
                        if !info.shortcuts.is_empty() {
                            ui.add_space(6.0);
                            ui.label(RichText::new("Shortcuts:").strong());
                            for shortcut in &info.shortcuts {
                                ui.label(RichText::new(shortcut).family(egui::FontFamily::Monospace));
                            }
                        }

                        // Safety information
                        if let Some(safety) = &info.safety_info {
                            ui.add_space(6.0);
                            ui.label(RichText::new("⚠️ Safety:").strong().color(Color32::YELLOW));
                            ui.label(RichText::new(safety).color(Color32::YELLOW));
                        }
                    });
            });
    }
}

/// Keyboard navigation management
#[derive(Debug, Clone)]
pub struct KeyboardNavigationManager {
    focusable_controls: Vec<FocusableControl>,
    tab_order: Vec<Id>,
    current_focus_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FocusableControl {
    pub id: Id,
    pub control_type: FocusableControlType,
    pub rect: Rect,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusableControlType {
    Slider,
    Knob,
    Button,
    Tab,
    FileDialog,
}

impl KeyboardNavigationManager {
    pub fn new() -> Self {
        Self {
            focusable_controls: Vec::new(),
            tab_order: Vec::new(),
            current_focus_index: None,
        }
    }

    pub fn update(&mut self, ui: &Ui) {
        // Update positions of registered controls
        // This would be called each frame to update positions
    }

    pub fn register_control(&mut self, id: Id, control_type: FocusableControlType, rect: Rect) {
        // Add or update control
        if let Some(existing) = self.focusable_controls.iter_mut().find(|c| c.id == id) {
            existing.rect = rect;
            existing.enabled = true;
        } else {
            self.focusable_controls.push(FocusableControl {
                id,
                control_type,
                rect,
                enabled: true,
            });
            self.tab_order.push(id);
        }
    }

    pub fn focus_next(&mut self) {
        if self.focusable_controls.is_empty() {
            return;
        }

        let current = self.current_focus_index.unwrap_or(0);
        let next = (current + 1) % self.focusable_controls.len();
        self.current_focus_index = Some(next);
    }

    pub fn focus_previous(&mut self) {
        if self.focusable_controls.is_empty() {
            return;
        }

        let current = self.current_focus_index.unwrap_or(0);
        let prev = if current == 0 {
            self.focusable_controls.len() - 1
        } else {
            current - 1
        };
        self.current_focus_index = Some(prev);
    }

    pub fn get_focused_control(&self) -> Option<&FocusableControl> {
        self.current_focus_index
            .and_then(|index| self.focusable_controls.get(index))
    }

    pub fn clear_focus(&mut self) {
        self.current_focus_index = None;
    }
}

/// Focus management system
#[derive(Debug, Clone)]
pub struct FocusManager {
    focused_id: Option<Id>,
    focus_animations: HashMap<Id, AnimationState>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            focused_id: None,
            focus_animations: HashMap::new(),
        }
    }

    pub fn update(&mut self, ui: &Ui, dt: f32) {
        // Update focus animations
        for animation in self.focus_animations.values_mut() {
            animation.update(dt);
        }
    }

    pub fn register_control(&mut self, id: Id, control_type: FocusableControlType, rect: Rect) {
        if !self.focus_animations.contains_key(&id) {
            self.focus_animations.insert(id, AnimationState::new(0.0, 10.0));
        }
    }

    pub fn focus_next(&mut self) {
        // Implementation would cycle through registered controls
    }

    pub fn focus_previous(&mut self) {
        // Implementation would cycle through registered controls in reverse
    }

    pub fn is_focused(&self, id: Id) -> bool {
        self.focused_id == Some(id)
    }

    pub fn has_focus(&self) -> bool {
        self.focused_id.is_some()
    }

    pub fn set_focus(&mut self, id: Id) {
        self.focused_id = Some(id);
        if let Some(animation) = self.focus_animations.get_mut(&id) {
            animation.set_target(1.0);
        }
    }

    pub fn clear_focus(&mut self) {
        if let Some(id) = self.focused_id {
            if let Some(animation) = self.focus_animations.get_mut(&id) {
                animation.set_target(0.0);
            }
        }
        self.focused_id = None;
    }
}

/// Screen reader support system
#[derive(Debug, Clone)]
pub struct ScreenReaderManager {
    announcements: Vec<Announcement>,
    last_announcement: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct Announcement {
    pub message: String,
    pub priority: AnnouncementPriority,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnnouncementPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl ScreenReaderManager {
    pub fn new() -> Self {
        Self {
            announcements: Vec::new(),
            last_announcement: None,
        }
    }

    pub fn announce(&mut self, message: String, priority: AnnouncementPriority) {
        let announcement = Announcement {
            message,
            priority,
            timestamp: Instant::now(),
        };

        self.announcements.push(announcement);
        self.last_announcement = Some(Instant::now());

        // Keep only recent announcements
        self.announcements.retain(|a| a.timestamp.elapsed() < Duration::from_secs(30));
    }
}

/// Safety management for audio controls
#[derive(Debug, Clone)]
pub struct SafetyManager {
    volume_limit: f32,
    last_volume_warning: Option<Instant>,
    volume_warning_cooldown: Duration,
    emergency_volume_level: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VolumeSafetyStatus {
    Safe,
    Warning,
    Dangerous,
    Critical,
}

impl SafetyManager {
    pub fn new() -> Self {
        Self {
            volume_limit: 0.8, // 80% maximum recommended
            last_volume_warning: None,
            volume_warning_cooldown: Duration::from_secs(5),
            emergency_volume_level: 0.2, // 20% for emergency reduction
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Update safety systems
    }

    pub fn is_volume_safe(&self, volume: f32) -> bool {
        volume <= self.volume_limit
    }

    pub fn get_status(&self) -> VolumeSafetyStatus {
        // This would check current audio levels and return appropriate status
        VolumeSafetyStatus::Safe
    }

    pub fn get_emergency_volume(&self) -> f32 {
        self.emergency_volume_level
    }

    pub fn should_show_volume_warning(&mut self, volume: f32) -> bool {
        if volume > self.volume_limit {
            if let Some(last_warning) = self.last_volume_warning {
                if last_warning.elapsed() > self.volume_warning_cooldown {
                    self.last_volume_warning = Some(Instant::now());
                    true
                } else {
                    false
                }
            } else {
                self.last_volume_warning = Some(Instant::now());
                true
            }
        } else {
            false
        }
    }
}

/// Contextual help system
#[derive(Debug, Clone)]
pub struct HelpSystem {
    overlay_visible: bool,
    help_topics: HashMap<String, HelpTopic>,
    fade_animation: AnimationState,
}

#[derive(Debug, Clone)]
pub struct HelpTopic {
    pub title: String,
    pub description: String,
    pub controls: Vec<ControlHelp>,
    pub shortcuts: Vec<ShortcutHelp>,
}

#[derive(Debug, Clone)]
pub struct ControlHelp {
    pub name: String,
    pub description: String,
    pub usage: String,
}

#[derive(Debug, Clone)]
pub struct ShortcutHelp {
    pub keys: String,
    pub description: String,
}

impl HelpSystem {
    pub fn new() -> Self {
        let mut help_topics = HashMap::new();

        // Add default help topics
        help_topics.insert("playback".to_string(), HelpTopic {
            title: "Playback Controls".to_string(),
            description: "Control audio playback and file operations".to_string(),
            controls: vec![
                ControlHelp {
                    name: "Play/Pause Button".to_string(),
                    description: "Start or pause audio playback".to_string(),
                    usage: "Click to toggle, or press Space".to_string(),
                },
                ControlHelp {
                    name: "Stop Button".to_string(),
                    description: "Stop playback and return to beginning".to_string(),
                    usage: "Click to stop, or press S".to_string(),
                },
                ControlHelp {
                    name: "Volume Slider".to_string(),
                    description: "Adjust playback volume".to_string(),
                    usage: "Drag slider or use arrow keys when focused".to_string(),
                },
            ],
            shortcuts: vec![
                ShortcutHelp {
                    keys: "Space".to_string(),
                    description: "Play/Pause".to_string(),
                },
                ShortcutHelp {
                    keys: "S".to_string(),
                    description: "Stop".to_string(),
                },
                ShortcutHelp {
                    keys: "↑/↓".to_string(),
                    description: "Volume Up/Down".to_string(),
                },
                ShortcutHelp {
                    keys: "←/→".to_string(),
                    description: "Seek -5s/+5s".to_string(),
                },
            ],
        });

        help_topics.insert("eq".to_string(), HelpTopic {
            title: "Equalizer".to_string(),
            description: "Adjust frequency response and audio characteristics".to_string(),
            controls: vec![
                ControlHelp {
                    name: "EQ Knobs".to_string(),
                    description: "Adjust gain for specific frequency bands".to_string(),
                    usage: "Drag to adjust, or use arrow keys when focused".to_string(),
                },
                ControlHelp {
                    name: "Reset Button".to_string(),
                    description: "Reset all EQ bands to flat response".to_string(),
                    usage: "Click to reset all bands to 0dB".to_string(),
                },
            ],
            shortcuts: vec![
                ShortcutHelp {
                    keys: "Tab".to_string(),
                    description: "Navigate between EQ bands".to_string(),
                },
                ShortcutHelp {
                    keys: "←/→".to_string(),
                    description: "Fine adjust focused band".to_string(),
                },
            ],
        });

        Self {
            overlay_visible: false,
            help_topics,
            fade_animation: AnimationState::new(0.0, 8.0),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.fade_animation.update(dt);
    }

    pub fn toggle_overlay(&mut self) {
        self.overlay_visible = !self.overlay_visible;
        self.fade_animation.set_target(if self.overlay_visible { 1.0 } else { 0.0 });
    }

    pub fn is_overlay_visible(&self) -> bool {
        self.overlay_visible && self.fade_animation.value() > 0.01
    }

    pub fn draw_overlay(&self, ui: &Ui, colors: &ThemeColors) {
        let alpha = self.fade_animation.value();
        if alpha < 0.01 {
            return;
        }

        egui::Window::new("🆘 Help & Keyboard Shortcuts")
            .collapsible(true)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(600.0);
                ui.set_max_width(800.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("📖 Quick Help").heading());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("❌ Close").clicked() {
                            // Help will be closed by the caller
                        }
                    });
                });

                ui.separator();

                ui.vertical(|ui| {
                    for (_, topic) in &self.help_topics {
                        ui.collapsing(&topic.title, |ui| {
                            ui.label(&topic.description);
                            ui.add_space(8.0);

                            if !topic.controls.is_empty() {
                                ui.label(RichText::new("Controls:").strong());
                                for control in &topic.controls {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&control.name).strong());
                                        ui.label("-");
                                        ui.label(&control.description);
                                    });
                                    ui.label(RichText::new(&control.usage).italics().color(colors.text_secondary));
                                    ui.add_space(4.0);
                                }
                            }

                            if !topic.shortcuts.is_empty() {
                                ui.add_space(8.0);
                                ui.label(RichText::new("Keyboard Shortcuts:").strong());
                                for shortcut in &topic.shortcuts {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&shortcut.keys)
                                            .family(egui::FontFamily::Monospace)
                                            .background_color(colors.surface));
                                        ui.label("-");
                                        ui.label(&shortcut.description);
                                    });
                                }
                            }
                        });
                    }

                    ui.add_space(16.0);

                    ui.collapsing("🔧 Accessibility Features", |ui| {
                        ui.label("• F1 - Toggle this help system");
                        ui.label("• Tab/Shift+Tab - Navigate between controls");
                        ui.label("• Arrow keys - Adjust focused control values");
                        ui.label("• Escape - Emergency volume reduction");
                        ui.label("• Ctrl+Shift+H - Toggle high contrast mode");
                        ui.label("• Hover controls for detailed tooltips");

                        ui.add_space(8.0);
                        ui.label(RichText::new("⚠️ Safety Features:").strong().color(colors.warning));
                        ui.label("• Volume limiting prevents hearing damage");
                        ui.label("• Visual warnings for high volume levels");
                        ui.label("• Escape key immediately reduces volume");
                        ui.label("• Gradual volume changes prevent sudden spikes");
                    });
                });
            });
    }
}