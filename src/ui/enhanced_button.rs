use egui::{
    Ui, Vec2, Rect, Color32, Response, Sense, CursorIcon, Id, Key, RichText, Pos2,
};
use super::utils::{AnimationState, ColorUtils, DrawUtils};
use super::theme::ThemeColors;
use super::accessibility::{AccessibilityManager, TooltipInfo, FocusableControlType, AnnouncementPriority};

/// Enhanced button with comprehensive accessibility support
pub struct AccessibleButton {
    id: Id,
    text: String,
    hover_animation: AnimationState,
    press_animation: AnimationState,
    focus_animation: AnimationState,
    icon: Option<String>,
    style: AccessibleButtonStyle,
    description: String,
    shortcuts: Vec<String>,
    confirmation_required: bool,
    confirmation_message: Option<String>,
    is_focused: bool,
    is_disabled: bool,
    loading: bool,
    loading_animation: AnimationState,
}

#[derive(Debug, Clone)]
pub struct AccessibleButtonStyle {
    pub rounding: f32,
    pub padding: Vec2,
    pub min_size: Vec2,
    pub gradient: bool,
    pub shadow: bool,
    pub glow: bool,
    pub touch_target_size: f32, // Minimum 44px for accessibility
    pub high_contrast: bool,
}

impl Default for AccessibleButtonStyle {
    fn default() -> Self {
        Self {
            rounding: 8.0,
            padding: Vec2::new(16.0, 12.0), // Larger padding for better touch targets
            min_size: Vec2::new(44.0, 44.0), // WCAG AA minimum touch target
            gradient: false,
            shadow: true,
            glow: false,
            touch_target_size: 44.0,
            high_contrast: false,
        }
    }
}

impl AccessibleButton {
    pub fn new(id: Id, text: impl Into<String>) -> Self {
        Self {
            id,
            text: text.into(),
            hover_animation: AnimationState::new(0.0, 12.0),
            press_animation: AnimationState::new(0.0, 20.0),
            focus_animation: AnimationState::new(0.0, 10.0),
            icon: None,
            style: AccessibleButtonStyle::default(),
            description: String::new(),
            shortcuts: vec!["Enter/Space: Activate".to_string()],
            confirmation_required: false,
            confirmation_message: None,
            is_focused: false,
            is_disabled: false,
            loading: false,
            loading_animation: AnimationState::new(0.0, 5.0),
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn style(mut self, style: AccessibleButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn confirmation_required(mut self, message: impl Into<String>) -> Self {
        self.confirmation_required = true;
        self.confirmation_message = Some(message.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.is_disabled = disabled;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        colors: &ThemeColors,
        accessibility: &mut AccessibilityManager,
    ) -> Response {
        // Calculate button size
        let text_size = ui.painter().layout_no_wrap(
            if self.loading { "Loading..." } else { &self.text },
            egui::FontId::default(),
            Color32::WHITE,
        ).size();

        let icon_size = if self.icon.is_some() { Vec2::new(16.0, 16.0) } else { Vec2::ZERO };
        let spacing = if self.icon.is_some() { 8.0 } else { 0.0 };

        let content_size = Vec2::new(
            text_size.x + icon_size.x + spacing,
            text_size.y.max(icon_size.y),
        );

        let button_size = Vec2::new(
            (content_size.x + self.style.padding.x * 2.0).max(self.style.min_size.x),
            (content_size.y + self.style.padding.y * 2.0).max(self.style.min_size.y),
        );

        let (rect, mut response) = ui.allocate_exact_size(button_size, Sense::click());

        // Disable interaction if button is disabled or loading
        if self.is_disabled || self.loading {
            response = response.on_disabled_hover_text(&self.description);
        }

        // Register with accessibility system
        accessibility.register_focusable(self.id, FocusableControlType::Button, rect);

        // Handle focus and keyboard input
        self.is_focused = accessibility.draw_focus_indicator(ui, self.id, rect, colors);

        if self.is_focused && !self.is_disabled && !self.loading {
            self.handle_keyboard_input(ui, &mut response);
        }

        // Handle clicks with confirmation if required
        if response.clicked() && !self.is_disabled && !self.loading {
            if self.confirmation_required {
                self.show_confirmation_dialog(ui, colors);
            } else {
                // Announce button activation for screen readers
                accessibility.announce(
                    format!("Button activated: {}", self.text),
                    AnnouncementPriority::Medium,
                );
            }
        }

        // Update animations
        let dt = ui.ctx().input(|i| i.stable_dt);

        self.hover_animation.set_target(if response.hovered() && !self.is_disabled { 1.0 } else { 0.0 });
        self.hover_animation.update(dt);

        self.press_animation.set_target(if response.is_pointer_button_down_on() && !self.is_disabled { 1.0 } else { 0.0 });
        self.press_animation.update(dt);

        self.focus_animation.set_target(if self.is_focused { 1.0 } else { 0.0 });
        self.focus_animation.update(dt);

        if self.loading {
            self.loading_animation.set_target(1.0);
        } else {
            self.loading_animation.set_target(0.0);
        }
        self.loading_animation.update(dt);

        // Draw the button
        self.draw(ui, rect, colors);

        // Show tooltip with accessibility information
        if (response.hovered() || self.is_focused) && !self.description.is_empty() {
            let tooltip_info = TooltipInfo {
                title: self.text.clone(),
                description: self.description.clone(),
                shortcuts: self.shortcuts.clone(),
                value_range: None,
                current_value: None,
                safety_info: if self.confirmation_required {
                    self.confirmation_message.clone()
                } else {
                    None
                },
            };
            accessibility.show_tooltip(ui, &response, tooltip_info);
        }

        // Set appropriate cursor
        if !self.is_disabled && !self.loading {
            if response.hovered() {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            }
        }

        response
    }

    fn handle_keyboard_input(&mut self, ui: &Ui, response: &mut Response) {
        ui.input(|i| {
            if i.key_pressed(Key::Enter) || i.key_pressed(Key::Space) {
                response.mark_changed();
                // The click will be handled by the main logic
            }
        });
    }

    fn show_confirmation_dialog(&self, ui: &Ui, colors: &ThemeColors) {
        if let Some(message) = &self.confirmation_message {
            egui::Window::new("⚠️ Confirmation Required")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(message);
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("✓ Confirm").clicked() {
                            // Handle confirmation
                        }
                        if ui.button("✗ Cancel").clicked() {
                            // Handle cancellation
                        }
                    });
                });
        }
    }

    fn draw(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let hover_factor = self.hover_animation.value();
        let press_factor = self.press_animation.value();
        let focus_factor = self.focus_animation.value();
        let loading_factor = self.loading_animation.value();

        // Calculate button transform
        let scale = if self.is_disabled {
            0.95
        } else {
            1.0 - press_factor * 0.05
        };
        let scaled_rect = Rect::from_center_size(rect.center(), rect.size() * scale);

        // Draw shadow if enabled and not disabled
        if self.style.shadow && !self.is_disabled {
            let shadow_offset = Vec2::new(2.0, 2.0) * (1.0 - press_factor * 0.5);
            let shadow_rect = Rect::from_center_size(
                scaled_rect.center() + shadow_offset,
                scaled_rect.size(),
            );
            painter.rect_filled(
                shadow_rect,
                self.style.rounding,
                ColorUtils::with_alpha(Color32::BLACK, 0.2 * (1.0 - press_factor * 0.5)),
            );
        }

        // Calculate button color based on state
        let base_color = if self.is_disabled {
            ColorUtils::with_alpha(colors.primary, 0.5)
        } else {
            colors.primary
        };

        let button_color = if self.style.gradient && !self.is_disabled {
            ColorUtils::lerp_color32(
                base_color,
                ColorUtils::lighten(base_color, 0.2),
                hover_factor,
            )
        } else if self.style.high_contrast {
            if hover_factor > 0.0 || focus_factor > 0.0 {
                Color32::YELLOW
            } else {
                Color32::WHITE
            }
        } else {
            ColorUtils::lerp_color32(
                base_color,
                ColorUtils::lighten(base_color, 0.1),
                hover_factor,
            )
        };

        // Draw glow effect if enabled
        if self.style.glow && (hover_factor > 0.0 || focus_factor > 0.0) && !self.is_disabled {
            let glow_intensity = (hover_factor * 0.6 + focus_factor * 0.4).max(0.0);
            DrawUtils::draw_glow_effect(
                ui,
                scaled_rect.center(),
                scaled_rect.width() * 0.6,
                button_color,
                glow_intensity,
            );
        }

        // Draw button background
        painter.rect_filled(scaled_rect, self.style.rounding, button_color);

        // Draw button border
        let border_width = if focus_factor > 0.0 { 3.0 } else { 1.0 };
        let border_color = if self.style.high_contrast {
            Color32::BLACK
        } else if focus_factor > 0.0 {
            colors.accent
        } else {
            ColorUtils::lerp_color32(
                ColorUtils::darken(button_color, 0.2),
                colors.accent,
                hover_factor,
            )
        };
        painter.rect_stroke(scaled_rect, self.style.rounding, egui::Stroke::new(border_width, border_color));

        // Draw focus ring
        if focus_factor > 0.0 {
            let focus_rect = scaled_rect.expand(4.0);
            painter.rect_stroke(
                focus_rect,
                self.style.rounding + 2.0,
                egui::Stroke::new(2.0 * focus_factor, colors.accent),
            );
        }

        // Draw content
        let content_rect = scaled_rect.shrink2(self.style.padding);
        let mut content_pos = content_rect.center();

        // Draw loading spinner if loading
        if self.loading {
            let spinner_radius = 8.0;
            let time = ui.ctx().input(|i| i.time) as f32;
            let angle = time * 2.0 * std::f32::consts::PI;

            for i in 0..8 {
                let a = angle + (i as f32 * std::f32::consts::PI * 2.0 / 8.0);
                let alpha = (1.0 + (a * 2.0).sin()) * 0.5;
                let pos = content_pos + Vec2::angled(a) * spinner_radius;
                painter.circle_filled(
                    pos,
                    2.0,
                    ColorUtils::with_alpha(colors.text, alpha),
                );
            }
        } else {
            // Draw icon if present
            if let Some(icon) = &self.icon {
                let icon_pos = Pos2::new(
                    content_pos.x - 8.0,
                    content_pos.y,
                );
                painter.text(
                    icon_pos,
                    egui::Align2::CENTER_CENTER,
                    icon,
                    egui::FontId::default(),
                    if self.style.high_contrast { Color32::BLACK } else { colors.text },
                );
                content_pos.x += 12.0;
            }

            // Draw text
            let text_color = if self.is_disabled {
                ColorUtils::with_alpha(colors.text, 0.6)
            } else if self.style.high_contrast {
                Color32::BLACK
            } else {
                colors.text
            };

            painter.text(
                content_pos,
                egui::Align2::CENTER_CENTER,
                &self.text,
                egui::FontId::default(),
                text_color,
            );
        }
    }
}

/// Progress indicator for operations that take time
pub struct ProgressIndicator {
    progress: f32,
    indeterminate: bool,
    animation: AnimationState,
    label: String,
    show_percentage: bool,
}

impl ProgressIndicator {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            progress: 0.0,
            indeterminate: false,
            animation: AnimationState::new(0.0, 5.0),
            label: label.into(),
            show_percentage: true,
        }
    }

    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self.indeterminate = false;
        self
    }

    pub fn indeterminate(mut self) -> Self {
        self.indeterminate = true;
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn show(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        let desired_size = Vec2::new(ui.available_width().min(300.0), 24.0);
        let (rect, _response) = ui.allocate_exact_size(desired_size, Sense::hover());

        // Update animation
        let dt = ui.ctx().input(|i| i.stable_dt);
        if self.indeterminate {
            let time = ui.ctx().input(|i| i.time) as f32;
            self.animation.set_target((time * 0.5).sin() * 0.5 + 0.5);
        } else {
            self.animation.set_target(self.progress);
        }
        self.animation.update(dt);

        self.draw(ui, rect, colors);
    }

    fn draw(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();

        // Draw background
        painter.rect_filled(rect, 4.0, colors.surface);
        painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, colors.text_secondary));

        // Draw progress bar
        let progress_value = if self.indeterminate {
            self.animation.value()
        } else {
            self.animation.value()
        };

        let progress_width = if self.indeterminate {
            rect.width() * 0.3 // Fixed width for indeterminate
        } else {
            rect.width() * progress_value
        };

        let progress_x = if self.indeterminate {
            rect.min.x + (rect.width() - progress_width) * progress_value
        } else {
            rect.min.x
        };

        let progress_rect = Rect::from_min_size(
            Pos2::new(progress_x, rect.min.y),
            Vec2::new(progress_width, rect.height()),
        );

        painter.rect_filled(progress_rect, 4.0, colors.primary);

        // Draw label and percentage
        let text = if self.show_percentage && !self.indeterminate {
            format!("{} ({:.0}%)", self.label, self.progress * 100.0)
        } else {
            self.label.clone()
        };

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::default(),
            colors.text,
        );
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        self.indeterminate = false;
    }
}

/// Volume safety warning indicator
pub struct VolumeSafetyIndicator {
    current_volume: f32,
    warning_threshold: f32,
    danger_threshold: f32,
    pulse_animation: AnimationState,
}

impl VolumeSafetyIndicator {
    pub fn new() -> Self {
        Self {
            current_volume: 0.0,
            warning_threshold: 0.7,
            danger_threshold: 0.9,
            pulse_animation: AnimationState::new(0.0, 8.0),
        }
    }

    pub fn update_volume(&mut self, volume: f32) {
        self.current_volume = volume;
    }

    pub fn show(&mut self, ui: &mut Ui, colors: &ThemeColors) {
        let dt = ui.ctx().input(|i| i.stable_dt);

        let status = if self.current_volume >= self.danger_threshold {
            VolumeSafetyStatus::Danger
        } else if self.current_volume >= self.warning_threshold {
            VolumeSafetyStatus::Warning
        } else {
            VolumeSafetyStatus::Safe
        };

        // Update pulse animation for warnings
        if matches!(status, VolumeSafetyStatus::Warning | VolumeSafetyStatus::Danger) {
            let time = ui.ctx().input(|i| i.time) as f32;
            self.pulse_animation.set_target((time * 4.0).sin() * 0.5 + 0.5);
        } else {
            self.pulse_animation.set_target(0.0);
        }
        self.pulse_animation.update(dt);

        if !matches!(status, VolumeSafetyStatus::Safe) {
            let (icon, color, message) = match status {
                VolumeSafetyStatus::Warning => ("⚠️", colors.warning, "Volume Warning"),
                VolumeSafetyStatus::Danger => ("🔊", colors.error, "Volume Too High!"),
                _ => return,
            };

            let pulse_factor = self.pulse_animation.value();
            let alpha = 0.8 + pulse_factor * 0.2;

            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(16.0));
                ui.label(
                    RichText::new(message)
                        .color(ColorUtils::with_alpha(color, alpha))
                        .strong()
                );
            });

            if matches!(status, VolumeSafetyStatus::Danger) {
                ui.label(
                    RichText::new("Press ESC to reduce volume immediately")
                        .size(12.0)
                        .color(colors.error)
                );
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum VolumeSafetyStatus {
    Safe,
    Warning,
    Danger,
}