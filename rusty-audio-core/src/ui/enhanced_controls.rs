use super::accessibility::{AccessibilityManager, FocusableControlType, TooltipInfo};
use super::theme::ThemeColors;
use super::utils::{AnimationState, ColorUtils, DrawUtils};
use egui::{
    emath::remap_clamp, epaint::CircleShape, Color32, CursorIcon, Id, Key, Pos2, Rect, Response,
    RichText, Sense, Stroke, Ui, Vec2,
};
use std::f32::consts::PI;

/// Enhanced slider with comprehensive accessibility support
pub struct AccessibleSlider {
    id: Id,
    value: f32,
    range: std::ops::RangeInclusive<f32>,
    animation: AnimationState,
    drag_animation: AnimationState,
    hover_animation: AnimationState,
    focus_animation: AnimationState,
    orientation: SliderOrientation,
    style: AccessibleSliderStyle,
    label: String,
    description: String,
    shortcuts: Vec<String>,
    safety_info: Option<String>,
    step_size: Option<f32>,
    is_focused: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SliderOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct AccessibleSliderStyle {
    pub track_width: f32,
    pub handle_radius: f32,
    pub track_rounding: f32,
    pub handle_rounding: f32,
    pub show_value: bool,
    pub show_ticks: bool,
    pub tick_count: usize,
    pub gradient: bool,
    pub glow: bool,
    pub touch_target_size: f32, // Minimum 44px for accessibility
}

impl Default for AccessibleSliderStyle {
    fn default() -> Self {
        Self {
            track_width: 8.0,
            handle_radius: 16.0, // Larger for easier interaction
            track_rounding: 4.0,
            handle_rounding: 16.0,
            show_value: true,
            show_ticks: true,
            tick_count: 5,
            gradient: true,
            glow: true,
            touch_target_size: 44.0, // WCAG AA requirement
        }
    }
}

impl AccessibleSlider {
    pub fn new(
        id: Id,
        value: f32,
        range: std::ops::RangeInclusive<f32>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id,
            value,
            range,
            animation: AnimationState::new(value, 10.0),
            drag_animation: AnimationState::new(0.0, 15.0),
            hover_animation: AnimationState::new(0.0, 12.0),
            focus_animation: AnimationState::new(0.0, 10.0),
            orientation: SliderOrientation::Horizontal,
            style: AccessibleSliderStyle::default(),
            label: label.into(),
            description: String::new(),
            shortcuts: vec![
                "←/→ or ↑/↓: Adjust value".to_string(),
                "Home/End: Min/Max value".to_string(),
                "Page Up/Down: Large steps".to_string(),
            ],
            safety_info: None,
            step_size: None,
            is_focused: false,
        }
    }

    pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn style(mut self, style: AccessibleSliderStyle) -> Self {
        self.style = style;
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn safety_info(mut self, info: impl Into<String>) -> Self {
        self.safety_info = Some(info.into());
        self
    }

    pub fn step_size(mut self, step: f32) -> Self {
        self.step_size = Some(step);
        self
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        colors: &ThemeColors,
        accessibility: &mut AccessibilityManager,
    ) -> Response {
        let desired_size = match self.orientation {
            SliderOrientation::Horizontal => Vec2::new(
                200.0_f32.max(self.style.touch_target_size * 3.0),
                self.style.touch_target_size,
            ),
            SliderOrientation::Vertical => Vec2::new(
                self.style.touch_target_size,
                200.0_f32.max(self.style.touch_target_size * 3.0),
            ),
        };

        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        // Register with accessibility system
        accessibility.register_focusable(self.id, FocusableControlType::Slider, rect);

        // Handle focus and keyboard input
        self.is_focused = accessibility.draw_focus_indicator(ui, self.id, rect, colors);

        if self.is_focused {
            self.handle_keyboard_input(ui, &mut response);
        }

        // Handle mouse/touch input
        if response.clicked() || response.dragged() {
            let new_value = self.calculate_value_from_pos(
                response.interact_pointer_pos().unwrap_or_default(),
                rect,
            );
            if (new_value - self.value).abs() > f32::EPSILON {
                self.set_value(new_value);
                response.mark_changed();

                // Announce value change for screen readers
                accessibility.announce(
                    format!("{}: {:.2}", self.label, self.value),
                    super::accessibility::AnnouncementPriority::Medium,
                );
            }
        }

        // Update animations
        let dt = ui.ctx().input(|i| i.stable_dt);
        self.animation.update(dt);

        self.hover_animation
            .set_target(if response.hovered() { 1.0 } else { 0.0 });
        self.hover_animation.update(dt);

        self.drag_animation
            .set_target(if response.dragged() { 1.0 } else { 0.0 });
        self.drag_animation.update(dt);

        self.focus_animation
            .set_target(if self.is_focused { 1.0 } else { 0.0 });
        self.focus_animation.update(dt);

        // Draw the slider
        self.draw(ui, rect, colors);

        // Show tooltip with accessibility information
        if response.hovered() || self.is_focused {
            let tooltip_info = TooltipInfo {
                title: self.label.clone(),
                description: self.description.clone(),
                shortcuts: self.shortcuts.clone(),
                value_range: Some((*self.range.start(), *self.range.end())),
                current_value: Some(self.value),
                safety_info: self.safety_info.clone(),
            };
            accessibility.show_tooltip(ui, &response, tooltip_info);
        }

        // Set appropriate cursor
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        response
    }

    fn handle_keyboard_input(&mut self, ui: &Ui, response: &mut Response) {
        ui.input(|i| {
            let step = self
                .step_size
                .unwrap_or((self.range.end() - self.range.start()) * 0.01);
            let large_step = step * 10.0;

            if i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::ArrowDown) {
                self.adjust_value(-step);
                response.mark_changed();
            } else if i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::ArrowUp) {
                self.adjust_value(step);
                response.mark_changed();
            } else if i.key_pressed(Key::Home) {
                self.set_value(*self.range.start());
                response.mark_changed();
            } else if i.key_pressed(Key::End) {
                self.set_value(*self.range.end());
                response.mark_changed();
            } else if i.key_pressed(Key::PageUp) {
                self.adjust_value(large_step);
                response.mark_changed();
            } else if i.key_pressed(Key::PageDown) {
                self.adjust_value(-large_step);
                response.mark_changed();
            }
        });
    }

    fn adjust_value(&mut self, delta: f32) {
        let new_value = (self.value + delta).clamp(*self.range.start(), *self.range.end());
        self.set_value(new_value);
    }

    fn calculate_value_from_pos(&self, pos: Pos2, rect: Rect) -> f32 {
        let t = match self.orientation {
            SliderOrientation::Horizontal => (pos.x - rect.min.x) / rect.width(),
            SliderOrientation::Vertical => 1.0 - (pos.y - rect.min.y) / rect.height(),
        };

        remap_clamp(t.clamp(0.0, 1.0), 0.0..=1.0, self.range.clone())
    }

    fn draw(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let animated_value = self.animation.value();
        let hover_factor = self.hover_animation.value();
        let drag_factor = self.drag_animation.value();
        let focus_factor = self.focus_animation.value();

        // Calculate positions and sizes
        let track_rect = self.calculate_track_rect(rect);
        let handle_pos = self.calculate_handle_position(rect, animated_value);
        let handle_radius =
            self.style.handle_radius * (1.0 + drag_factor * 0.1 + focus_factor * 0.05);

        // Draw track background
        let track_bg_color = ColorUtils::lerp_color32(
            colors.surface,
            ColorUtils::lighten(colors.surface, 0.2),
            hover_factor + focus_factor * 0.5,
        );
        painter.rect_filled(track_rect, self.style.track_rounding, track_bg_color);

        // Draw track border for better contrast
        painter.rect_stroke(
            track_rect,
            self.style.track_rounding,
            Stroke::new(1.0, ColorUtils::darken(track_bg_color, 0.2)),
            egui::epaint::StrokeKind::Outside,
        );

        // Draw filled track (progress)
        let fill_rect = self.calculate_fill_rect(track_rect, animated_value);
        let fill_color = if self.style.gradient {
            ColorUtils::lerp_color32(colors.primary, colors.accent, animated_value)
        } else {
            colors.primary
        };
        painter.rect_filled(fill_rect, self.style.track_rounding, fill_color);

        // Draw ticks if enabled
        if self.style.show_ticks {
            self.draw_ticks(ui, rect, colors);
        }

        // Draw glow effect if enabled
        if self.style.glow && (hover_factor > 0.0 || drag_factor > 0.0 || focus_factor > 0.0) {
            let glow_intensity =
                (hover_factor * 0.3 + drag_factor * 0.5 + focus_factor * 0.4).max(0.0);
            DrawUtils::draw_glow_effect(
                ui,
                handle_pos,
                handle_radius * 1.5,
                fill_color,
                glow_intensity,
            );
        }

        // Draw handle shadow
        let shadow_offset = Vec2::new(2.0, 2.0) * (1.0 + drag_factor * 0.5);
        painter.add(CircleShape {
            center: handle_pos + shadow_offset,
            radius: handle_radius,
            fill: ColorUtils::with_alpha(Color32::BLACK, 0.3),
            stroke: Stroke::NONE,
        });

        // Draw handle
        let handle_color = ColorUtils::lerp_color32(
            colors.text,
            colors.accent,
            hover_factor * 0.3 + drag_factor * 0.5 + focus_factor * 0.4,
        );

        painter.add(CircleShape {
            center: handle_pos,
            radius: handle_radius,
            fill: handle_color,
            stroke: Stroke::new(2.0, ColorUtils::darken(handle_color, 0.3)),
        });

        // Draw inner handle highlight
        let inner_radius = handle_radius * 0.6;
        let inner_color = ColorUtils::with_alpha(colors.text, 0.9);
        painter.circle_filled(handle_pos, inner_radius, inner_color);

        // Draw focus ring
        if focus_factor > 0.0 {
            let focus_radius = handle_radius + 4.0;
            painter.circle_stroke(
                handle_pos,
                focus_radius,
                Stroke::new(2.0 * focus_factor, colors.accent),
            );
        }

        // Draw value if enabled
        if self.style.show_value {
            let value_text = format!("{:.2}", self.value);
            let text_pos = match self.orientation {
                SliderOrientation::Horizontal => Pos2::new(rect.center().x, rect.min.y - 20.0),
                SliderOrientation::Vertical => Pos2::new(rect.max.x + 25.0, rect.center().y),
            };

            // Draw value background for better readability
            let text_size = ui
                .painter()
                .layout_no_wrap(value_text.clone(), egui::FontId::default(), Color32::WHITE)
                .size();

            let text_rect = Rect::from_center_size(text_pos, text_size + Vec2::splat(8.0));
            painter.rect_filled(text_rect, 4.0, ColorUtils::with_alpha(colors.surface, 0.9));
            painter.rect_stroke(
                text_rect,
                4.0,
                Stroke::new(1.0, colors.text_secondary),
                egui::epaint::StrokeKind::Outside,
            );

            painter.text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                value_text,
                egui::FontId::default(),
                colors.text,
            );
        }
    }

    fn calculate_track_rect(&self, rect: Rect) -> Rect {
        match self.orientation {
            SliderOrientation::Horizontal => {
                let center_y = rect.center().y;
                Rect::from_center_size(
                    Pos2::new(rect.center().x, center_y),
                    Vec2::new(
                        rect.width() - self.style.handle_radius * 2.0,
                        self.style.track_width,
                    ),
                )
            }
            SliderOrientation::Vertical => {
                let center_x = rect.center().x;
                Rect::from_center_size(
                    Pos2::new(center_x, rect.center().y),
                    Vec2::new(
                        self.style.track_width,
                        rect.height() - self.style.handle_radius * 2.0,
                    ),
                )
            }
        }
    }

    fn calculate_handle_position(&self, rect: Rect, value: f32) -> Pos2 {
        let t = remap_clamp(value, self.range.clone(), 0.0..=1.0);

        match self.orientation {
            SliderOrientation::Horizontal => {
                let x = rect.min.x
                    + self.style.handle_radius
                    + t * (rect.width() - self.style.handle_radius * 2.0);
                Pos2::new(x, rect.center().y)
            }
            SliderOrientation::Vertical => {
                let y = rect.max.y
                    - self.style.handle_radius
                    - t * (rect.height() - self.style.handle_radius * 2.0);
                Pos2::new(rect.center().x, y)
            }
        }
    }

    fn calculate_fill_rect(&self, track_rect: Rect, value: f32) -> Rect {
        let t = remap_clamp(value, self.range.clone(), 0.0..=1.0);

        match self.orientation {
            SliderOrientation::Horizontal => {
                let width = track_rect.width() * t;
                Rect::from_min_size(track_rect.min, Vec2::new(width, track_rect.height()))
            }
            SliderOrientation::Vertical => {
                let height = track_rect.height() * t;
                let start_y = track_rect.max.y - height;
                Rect::from_min_size(
                    Pos2::new(track_rect.min.x, start_y),
                    Vec2::new(track_rect.width(), height),
                )
            }
        }
    }

    fn draw_ticks(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();

        for i in 0..=self.style.tick_count {
            let t = i as f32 / self.style.tick_count as f32;
            let value = remap_clamp(t, 0.0..=1.0, self.range.clone());

            let (tick_start, tick_end) = match self.orientation {
                SliderOrientation::Horizontal => {
                    let x = rect.min.x
                        + self.style.handle_radius
                        + t * (rect.width() - self.style.handle_radius * 2.0);
                    (
                        Pos2::new(x, rect.center().y + self.style.track_width * 0.5 + 2.0),
                        Pos2::new(x, rect.center().y + self.style.track_width * 0.5 + 8.0),
                    )
                }
                SliderOrientation::Vertical => {
                    let y = rect.max.y
                        - self.style.handle_radius
                        - t * (rect.height() - self.style.handle_radius * 2.0);
                    (
                        Pos2::new(rect.center().x + self.style.track_width * 0.5 + 2.0, y),
                        Pos2::new(rect.center().x + self.style.track_width * 0.5 + 8.0, y),
                    )
                }
            };

            painter.line_segment([tick_start, tick_end], (1.0, colors.text_secondary));

            // Draw tick value labels for major ticks
            if i % (self.style.tick_count / 4).max(1) == 0 {
                let label_text = format!("{:.1}", value);
                let label_pos = match self.orientation {
                    SliderOrientation::Horizontal => Pos2::new(tick_end.x, tick_end.y + 15.0),
                    SliderOrientation::Vertical => Pos2::new(tick_end.x + 15.0, tick_end.y),
                };

                painter.text(
                    label_pos,
                    egui::Align2::CENTER_CENTER,
                    label_text,
                    egui::FontId::new(9.0, egui::FontFamily::Monospace),
                    colors.text_secondary,
                );
            }
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        let clamped = value.clamp(*self.range.start(), *self.range.end());
        self.value = clamped;
        self.animation.set_target(clamped);
    }
}

/// Enhanced circular knob with accessibility support
pub struct AccessibleKnob {
    id: Id,
    value: f32,
    range: std::ops::RangeInclusive<f32>,
    animation: AnimationState,
    hover_animation: AnimationState,
    drag_animation: AnimationState,
    focus_animation: AnimationState,
    radius: f32,
    sensitivity: f32,
    show_value: bool,
    show_arc: bool,
    label: String,
    description: String,
    shortcuts: Vec<String>,
    step_size: Option<f32>,
    is_focused: bool,
}

impl AccessibleKnob {
    pub fn new(
        id: Id,
        value: f32,
        range: std::ops::RangeInclusive<f32>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id,
            value,
            range,
            animation: AnimationState::new(value, 10.0),
            hover_animation: AnimationState::new(0.0, 12.0),
            drag_animation: AnimationState::new(0.0, 15.0),
            focus_animation: AnimationState::new(0.0, 10.0),
            radius: 30.0, // Larger for better accessibility
            sensitivity: 0.005,
            show_value: true,
            show_arc: true,
            label: label.into(),
            description: String::new(),
            shortcuts: vec![
                "↑/↓: Adjust value".to_string(),
                "Home/End: Min/Max value".to_string(),
                "Page Up/Down: Large steps".to_string(),
            ],
            step_size: None,
            is_focused: false,
        }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius.max(22.0); // Minimum size for accessibility
        self
    }

    pub fn sensitivity(mut self, sensitivity: f32) -> Self {
        self.sensitivity = sensitivity;
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn step_size(mut self, step: f32) -> Self {
        self.step_size = Some(step);
        self
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        colors: &ThemeColors,
        accessibility: &mut AccessibilityManager,
    ) -> Response {
        let desired_size = Vec2::splat((self.radius * 2.8).max(44.0)); // Ensure minimum touch target
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
        let center = rect.center();

        // Register with accessibility system
        accessibility.register_focusable(self.id, FocusableControlType::Knob, rect);

        // Handle focus and keyboard input
        self.is_focused = accessibility.draw_focus_indicator(ui, self.id, rect, colors);

        if self.is_focused {
            self.handle_keyboard_input(ui, &mut response);
        }

        // Handle mouse/touch input
        if response.dragged() {
            if let Some(_pointer_pos) = response.interact_pointer_pos() {
                if let Some(_last_pos) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                    let drag_delta = ui.ctx().input(|i| i.pointer.delta());
                    let value_delta =
                        -drag_delta.y * self.sensitivity * (self.range.end() - self.range.start());
                    let new_value =
                        (self.value + value_delta).clamp(*self.range.start(), *self.range.end());

                    if (new_value - self.value).abs() > f32::EPSILON {
                        self.set_value(new_value);
                        response.mark_changed();

                        // Announce value change for screen readers
                        accessibility.announce(
                            format!("{}: {:.2}", self.label, self.value),
                            super::accessibility::AnnouncementPriority::Medium,
                        );
                    }
                }
            }
        }

        // Update animations
        let dt = ui.ctx().input(|i| i.stable_dt);
        self.animation.update(dt);

        self.hover_animation
            .set_target(if response.hovered() { 1.0 } else { 0.0 });
        self.hover_animation.update(dt);

        self.drag_animation
            .set_target(if response.dragged() { 1.0 } else { 0.0 });
        self.drag_animation.update(dt);

        self.focus_animation
            .set_target(if self.is_focused { 1.0 } else { 0.0 });
        self.focus_animation.update(dt);

        // Draw the knob
        self.draw(ui, rect, center, colors);

        // Show tooltip with accessibility information
        if response.hovered() || self.is_focused {
            let tooltip_info = TooltipInfo {
                title: self.label.clone(),
                description: self.description.clone(),
                shortcuts: self.shortcuts.clone(),
                value_range: Some((*self.range.start(), *self.range.end())),
                current_value: Some(self.value),
                safety_info: None,
            };
            accessibility.show_tooltip(ui, &response, tooltip_info);
        }

        // Set appropriate cursor
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }
        if response.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }

        response
    }

    fn handle_keyboard_input(&mut self, ui: &Ui, response: &mut Response) {
        ui.input(|i| {
            let step = self
                .step_size
                .unwrap_or((self.range.end() - self.range.start()) * 0.01);
            let large_step = step * 10.0;

            if i.key_pressed(Key::ArrowUp) || i.key_pressed(Key::ArrowRight) {
                self.adjust_value(step);
                response.mark_changed();
            } else if i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::ArrowLeft) {
                self.adjust_value(-step);
                response.mark_changed();
            } else if i.key_pressed(Key::Home) {
                self.set_value(*self.range.start());
                response.mark_changed();
            } else if i.key_pressed(Key::End) {
                self.set_value(*self.range.end());
                response.mark_changed();
            } else if i.key_pressed(Key::PageUp) {
                self.adjust_value(large_step);
                response.mark_changed();
            } else if i.key_pressed(Key::PageDown) {
                self.adjust_value(-large_step);
                response.mark_changed();
            }
        });
    }

    fn adjust_value(&mut self, delta: f32) {
        let new_value = (self.value + delta).clamp(*self.range.start(), *self.range.end());
        self.set_value(new_value);
    }

    fn draw(&self, ui: &Ui, rect: Rect, center: Pos2, colors: &ThemeColors) {
        let painter = ui.painter();
        let animated_value = self.animation.value();
        let hover_factor = self.hover_animation.value();
        let drag_factor = self.drag_animation.value();
        let focus_factor = self.focus_animation.value();

        // Calculate angle for current value
        let value_t = remap_clamp(animated_value, self.range.clone(), 0.0..=1.0);
        let start_angle = -PI * 0.75; // Start at 7:30
        let end_angle = PI * 0.75; // End at 4:30
        let current_angle = start_angle + value_t * (end_angle - start_angle);

        let knob_radius = self.radius * (1.0 + drag_factor * 0.1 + focus_factor * 0.05);

        // Draw background arc if enabled
        if self.show_arc {
            self.draw_arc(
                ui,
                center,
                self.radius * 1.3,
                start_angle,
                end_angle,
                6.0,
                colors.surface,
            );
            // Draw value arc
            self.draw_arc(
                ui,
                center,
                self.radius * 1.3,
                start_angle,
                current_angle,
                6.0,
                colors.primary,
            );
        }

        // Draw knob shadow
        let shadow_offset = Vec2::new(3.0, 3.0) * (1.0 + drag_factor * 0.5);
        painter.circle_filled(
            center + shadow_offset,
            knob_radius,
            ColorUtils::with_alpha(Color32::BLACK, 0.3),
        );

        // Draw knob body with gradient effect
        let knob_color = ColorUtils::lerp_color32(
            colors.surface,
            ColorUtils::lighten(colors.surface, 0.4),
            hover_factor * 0.5 + drag_factor * 0.3 + focus_factor * 0.2,
        );

        // Draw main knob
        painter.circle_filled(center, knob_radius, knob_color);

        // Draw outer ring
        painter.circle_stroke(
            center,
            knob_radius,
            Stroke::new(
                3.0,
                ColorUtils::lerp_color32(
                    colors.text_secondary,
                    colors.primary,
                    hover_factor + focus_factor,
                ),
            ),
        );

        // Draw focus ring
        if focus_factor > 0.0 {
            painter.circle_stroke(
                center,
                knob_radius + 6.0,
                Stroke::new(2.0 * focus_factor, colors.accent),
            );
        }

        // Draw indicator line
        let indicator_start = center + Vec2::angled(current_angle) * (knob_radius * 0.3);
        let indicator_end = center + Vec2::angled(current_angle) * (knob_radius * 0.8);
        painter.line_segment(
            [indicator_start, indicator_end],
            Stroke::new(4.0, colors.accent),
        );

        // Draw center dot
        painter.circle_filled(center, 4.0, colors.accent);

        // Draw value text if enabled
        if self.show_value {
            let value_text = format!("{:.2}", animated_value);
            let text_pos = center + Vec2::new(0.0, self.radius * 1.8);

            // Draw value background for better readability
            let text_size = ui
                .painter()
                .layout_no_wrap(value_text.clone(), egui::FontId::default(), Color32::WHITE)
                .size();

            let text_rect = Rect::from_center_size(text_pos, text_size + Vec2::splat(6.0));
            painter.rect_filled(text_rect, 3.0, ColorUtils::with_alpha(colors.surface, 0.9));
            painter.rect_stroke(
                text_rect,
                3.0,
                Stroke::new(1.0, colors.text_secondary),
                egui::epaint::StrokeKind::Outside,
            );

            painter.text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                value_text,
                egui::FontId::default(),
                colors.text,
            );
        }

        // Draw label
        let label_pos = center + Vec2::new(0.0, -self.radius * 1.8);
        painter.text(
            label_pos,
            egui::Align2::CENTER_CENTER,
            &self.label,
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
            colors.text,
        );
    }

    fn draw_arc(
        &self,
        ui: &Ui,
        center: Pos2,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        thickness: f32,
        color: Color32,
    ) {
        let painter = ui.painter();
        let segments = ((end_angle - start_angle).abs() * 25.0) as usize;

        if segments < 2 {
            return;
        }

        let mut points = Vec::with_capacity(segments + 1);
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let angle = start_angle + t * (end_angle - start_angle);
            let point = center + Vec2::angled(angle) * radius;
            points.push(point);
        }

        for i in 0..points.len() - 1 {
            painter.line_segment([points[i], points[i + 1]], Stroke::new(thickness, color));
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        let clamped = value.clamp(*self.range.start(), *self.range.end());
        self.value = clamped;
        self.animation.set_target(clamped);
    }
}
