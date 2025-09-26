use egui::{
    Ui, Vec2, Rect, Color32, Stroke, Pos2, Response, Sense, CursorIcon,
    emath::remap_clamp, epaint::CircleShape,
};
use super::utils::{AnimationState, ColorUtils, DrawUtils};
use super::theme::ThemeColors;
use std::f32::consts::PI;

/// Enhanced slider with custom styling and animations
pub struct EnhancedSlider {
    value: f32,
    range: std::ops::RangeInclusive<f32>,
    animation: AnimationState,
    drag_animation: AnimationState,
    hover_animation: AnimationState,
    orientation: SliderOrientation,
    style: SliderStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SliderOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct SliderStyle {
    pub track_width: f32,
    pub handle_radius: f32,
    pub track_rounding: f32,
    pub handle_rounding: f32,
    pub show_value: bool,
    pub show_ticks: bool,
    pub tick_count: usize,
    pub gradient: bool,
    pub glow: bool,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            track_width: 6.0,
            handle_radius: 12.0,
            track_rounding: 3.0,
            handle_rounding: 12.0,
            show_value: true,
            show_ticks: false,
            tick_count: 5,
            gradient: true,
            glow: false,
        }
    }
}

impl EnhancedSlider {
    pub fn new(value: f32, range: std::ops::RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            animation: AnimationState::new(value, 10.0),
            drag_animation: AnimationState::new(0.0, 15.0),
            hover_animation: AnimationState::new(0.0, 12.0),
            orientation: SliderOrientation::Horizontal,
            style: SliderStyle::default(),
        }
    }

    pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn style(mut self, style: SliderStyle) -> Self {
        self.style = style;
        self
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        colors: &ThemeColors,
    ) -> Response {
        let desired_size = match self.orientation {
            SliderOrientation::Horizontal => Vec2::new(120.0, 24.0),
            SliderOrientation::Vertical => Vec2::new(24.0, 120.0),
        };

        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        if response.clicked() || response.dragged() {
            let new_value = self.calculate_value_from_pos(response.interact_pointer_pos().unwrap_or_default(), rect);
            if (new_value - self.value).abs() > f32::EPSILON {
                self.value = new_value;
                self.animation.set_target(new_value);
                response.mark_changed();
            }
        }

        // Update animations
        let dt = ui.ctx().input(|i| i.stable_dt);
        self.animation.update(dt);

        self.hover_animation.set_target(if response.hovered() { 1.0 } else { 0.0 });
        self.hover_animation.update(dt);

        self.drag_animation.set_target(if response.dragged() { 1.0 } else { 0.0 });
        self.drag_animation.update(dt);

        // Draw the slider
        self.draw(ui, rect, colors);

        // Set cursor
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        response
    }

    fn calculate_value_from_pos(&self, pos: Pos2, rect: Rect) -> f32 {
        let t = match self.orientation {
            SliderOrientation::Horizontal => {
                (pos.x - rect.min.x) / rect.width()
            }
            SliderOrientation::Vertical => {
                1.0 - (pos.y - rect.min.y) / rect.height()
            }
        };

        remap_clamp(t.clamp(0.0, 1.0), 0.0..=1.0, self.range.clone())
    }

    fn draw(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let animated_value = self.animation.value();
        let hover_factor = self.hover_animation.value();
        let drag_factor = self.drag_animation.value();

        // Calculate positions and sizes
        let track_rect = self.calculate_track_rect(rect);
        let handle_pos = self.calculate_handle_position(rect, animated_value);
        let handle_radius = self.style.handle_radius * (1.0 + drag_factor * 0.2);

        // Draw track background
        let track_bg_color = ColorUtils::lerp_color32(
            colors.surface,
            ColorUtils::lighten(colors.surface, 0.1),
            hover_factor,
        );
        painter.rect_filled(track_rect, self.style.track_rounding, track_bg_color);

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
        if self.style.glow && (hover_factor > 0.0 || drag_factor > 0.0) {
            let glow_intensity = (hover_factor * 0.3 + drag_factor * 0.5).max(0.0);
            DrawUtils::draw_glow_effect(ui, handle_pos, handle_radius * 1.5, fill_color, glow_intensity);
        }

        // Draw handle
        let handle_color = ColorUtils::lerp_color32(
            colors.text,
            colors.accent,
            hover_factor * 0.5 + drag_factor * 0.5,
        );

        painter.add(CircleShape {
            center: handle_pos,
            radius: handle_radius,
            fill: handle_color,
            stroke: Stroke::new(2.0, ColorUtils::darken(handle_color, 0.3)),
        });

        // Draw inner handle highlight
        let inner_radius = handle_radius * 0.6;
        let inner_color = ColorUtils::with_alpha(colors.text, 0.8);
        painter.circle_filled(handle_pos, inner_radius, inner_color);

        // Draw value if enabled
        if self.style.show_value {
            let value_text = format!("{:.1}", self.value);
            let text_pos = match self.orientation {
                SliderOrientation::Horizontal => {
                    Pos2::new(rect.center().x, rect.min.y - 5.0)
                }
                SliderOrientation::Vertical => {
                    Pos2::new(rect.max.x + 10.0, rect.center().y)
                }
            };

            ui.painter().text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                value_text,
                egui::FontId::default(),
                colors.text_secondary,
            );
        }
    }

    fn calculate_track_rect(&self, rect: Rect) -> Rect {
        match self.orientation {
            SliderOrientation::Horizontal => {
                let center_y = rect.center().y;
                Rect::from_center_size(
                    Pos2::new(rect.center().x, center_y),
                    Vec2::new(rect.width() - self.style.handle_radius * 2.0, self.style.track_width),
                )
            }
            SliderOrientation::Vertical => {
                let center_x = rect.center().x;
                Rect::from_center_size(
                    Pos2::new(center_x, rect.center().y),
                    Vec2::new(self.style.track_width, rect.height() - self.style.handle_radius * 2.0),
                )
            }
        }
    }

    fn calculate_handle_position(&self, rect: Rect, value: f32) -> Pos2 {
        let t = remap_clamp(value, self.range.clone(), 0.0..=1.0);

        match self.orientation {
            SliderOrientation::Horizontal => {
                let x = rect.min.x + self.style.handle_radius + t * (rect.width() - self.style.handle_radius * 2.0);
                Pos2::new(x, rect.center().y)
            }
            SliderOrientation::Vertical => {
                let y = rect.max.y - self.style.handle_radius - t * (rect.height() - self.style.handle_radius * 2.0);
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

            let (tick_start, tick_end) = match self.orientation {
                SliderOrientation::Horizontal => {
                    let x = rect.min.x + self.style.handle_radius + t * (rect.width() - self.style.handle_radius * 2.0);
                    (
                        Pos2::new(x, rect.center().y + self.style.track_width * 0.5 + 2.0),
                        Pos2::new(x, rect.center().y + self.style.track_width * 0.5 + 8.0),
                    )
                }
                SliderOrientation::Vertical => {
                    let y = rect.max.y - self.style.handle_radius - t * (rect.height() - self.style.handle_radius * 2.0);
                    (
                        Pos2::new(rect.center().x + self.style.track_width * 0.5 + 2.0, y),
                        Pos2::new(rect.center().x + self.style.track_width * 0.5 + 8.0, y),
                    )
                }
            };

            painter.line_segment(
                [tick_start, tick_end],
                Stroke::new(1.0, colors.text_secondary),
            );
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

/// Circular knob control for more precise adjustments
pub struct CircularKnob {
    value: f32,
    range: std::ops::RangeInclusive<f32>,
    animation: AnimationState,
    hover_animation: AnimationState,
    drag_animation: AnimationState,
    radius: f32,
    sensitivity: f32,
    show_value: bool,
    show_arc: bool,
}

impl CircularKnob {
    pub fn new(value: f32, range: std::ops::RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            animation: AnimationState::new(value, 10.0),
            hover_animation: AnimationState::new(0.0, 12.0),
            drag_animation: AnimationState::new(0.0, 15.0),
            radius: 25.0,
            sensitivity: 0.005,
            show_value: true,
            show_arc: true,
        }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn sensitivity(mut self, sensitivity: f32) -> Self {
        self.sensitivity = sensitivity;
        self
    }

    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    pub fn show_arc(mut self, show: bool) -> Self {
        self.show_arc = show;
        self
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        colors: &ThemeColors,
    ) -> Response {
        let desired_size = Vec2::splat(self.radius * 2.5);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
        let center = rect.center();

        if response.dragged() {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let delta = pointer_pos - center;
                let angle = delta.y.atan2(delta.x);

                // Convert drag delta to value change
                if let Some(last_pos) = ui.ctx().input(|i| i.pointer.hover_pos()) {
                    let drag_delta = ui.ctx().input(|i| i.pointer.delta());
                    let value_delta = -drag_delta.y * self.sensitivity * (self.range.end() - self.range.start());
                    let new_value = (self.value + value_delta).clamp(*self.range.start(), *self.range.end());

                    if (new_value - self.value).abs() > f32::EPSILON {
                        self.value = new_value;
                        self.animation.set_target(new_value);
                        response.mark_changed();
                    }
                }
            }
        }

        // Update animations
        let dt = ui.ctx().input(|i| i.stable_dt);
        self.animation.update(dt);

        self.hover_animation.set_target(if response.hovered() { 1.0 } else { 0.0 });
        self.hover_animation.update(dt);

        self.drag_animation.set_target(if response.dragged() { 1.0 } else { 0.0 });
        self.drag_animation.update(dt);

        // Draw the knob
        self.draw(ui, rect, center, colors);

        // Set cursor
        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }
        if response.dragged() {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }

        response
    }

    fn draw(&self, ui: &Ui, rect: Rect, center: Pos2, colors: &ThemeColors) {
        let painter = ui.painter();
        let animated_value = self.animation.value();
        let hover_factor = self.hover_animation.value();
        let drag_factor = self.drag_animation.value();

        // Calculate angle for current value
        let value_t = remap_clamp(animated_value, self.range.clone(), 0.0..=1.0);
        let start_angle = -PI * 0.75; // Start at 7:30
        let end_angle = PI * 0.75;    // End at 4:30
        let current_angle = start_angle + value_t * (end_angle - start_angle);

        let knob_radius = self.radius * (1.0 + drag_factor * 0.1);

        // Draw background arc if enabled
        if self.show_arc {
            self.draw_arc(ui, center, self.radius * 1.2, start_angle, end_angle, 4.0, colors.surface);
            // Draw value arc
            self.draw_arc(ui, center, self.radius * 1.2, start_angle, current_angle, 4.0, colors.primary);
        }

        // Draw knob body with gradient effect
        let knob_color = ColorUtils::lerp_color32(
            colors.surface,
            ColorUtils::lighten(colors.surface, 0.3),
            hover_factor * 0.5 + drag_factor * 0.3,
        );

        // Draw shadow
        let shadow_offset = Vec2::new(2.0, 2.0) * (1.0 + drag_factor * 0.5);
        painter.circle_filled(
            center + shadow_offset,
            knob_radius,
            ColorUtils::with_alpha(Color32::BLACK, 0.2),
        );

        // Draw main knob
        painter.circle_filled(center, knob_radius, knob_color);

        // Draw outer ring
        painter.circle_stroke(
            center,
            knob_radius,
            Stroke::new(2.0, ColorUtils::lerp_color32(colors.text_secondary, colors.primary, hover_factor)),
        );

        // Draw indicator line
        let indicator_start = center + Vec2::angled(current_angle) * (knob_radius * 0.3);
        let indicator_end = center + Vec2::angled(current_angle) * (knob_radius * 0.8);
        painter.line_segment(
            [indicator_start, indicator_end],
            Stroke::new(3.0, colors.accent),
        );

        // Draw center dot
        painter.circle_filled(center, 3.0, colors.accent);

        // Draw value text if enabled
        if self.show_value {
            let value_text = format!("{:.1}", animated_value);
            painter.text(
                center + Vec2::new(0.0, self.radius * 1.5),
                egui::Align2::CENTER_CENTER,
                value_text,
                egui::FontId::default(),
                colors.text,
            );
        }
    }

    fn draw_arc(&self, ui: &Ui, center: Pos2, radius: f32, start_angle: f32, end_angle: f32, thickness: f32, color: Color32) {
        let painter = ui.painter();
        let segments = ((end_angle - start_angle).abs() * 20.0) as usize;

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
            painter.line_segment(
                [points[i], points[i + 1]],
                Stroke::new(thickness, color),
            );
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

/// Enhanced button with animations and custom styling
pub struct EnhancedButton {
    text: String,
    hover_animation: AnimationState,
    press_animation: AnimationState,
    icon: Option<String>,
    style: ButtonStyle,
}

#[derive(Debug, Clone)]
pub struct ButtonStyle {
    pub rounding: f32,
    pub padding: Vec2,
    pub min_size: Vec2,
    pub gradient: bool,
    pub shadow: bool,
    pub glow: bool,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            rounding: 8.0,
            padding: Vec2::new(16.0, 8.0),
            min_size: Vec2::new(80.0, 32.0),
            gradient: false,
            shadow: true,
            glow: false,
        }
    }
}

impl EnhancedButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            hover_animation: AnimationState::new(0.0, 12.0),
            press_animation: AnimationState::new(0.0, 20.0),
            icon: None,
            style: ButtonStyle::default(),
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        colors: &ThemeColors,
    ) -> Response {
        let text_size = ui.painter().layout_no_wrap(
            self.text.clone(),
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

        let (rect, response) = ui.allocate_exact_size(button_size, Sense::click());

        // Update animations
        let dt = ui.ctx().input(|i| i.stable_dt);

        self.hover_animation.set_target(if response.hovered() { 1.0 } else { 0.0 });
        self.hover_animation.update(dt);

        self.press_animation.set_target(if response.is_pointer_button_down_on() { 1.0 } else { 0.0 });
        self.press_animation.update(dt);

        // Draw the button
        self.draw(ui, rect, colors);

        response
    }

    fn draw(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let hover_factor = self.hover_animation.value();
        let press_factor = self.press_animation.value();

        // Calculate button transform
        let scale = 1.0 - press_factor * 0.05;
        let scaled_rect = Rect::from_center_size(rect.center(), rect.size() * scale);

        // Draw shadow if enabled
        if self.style.shadow {
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

        // Calculate button color
        let base_color = colors.primary;
        let button_color = if self.style.gradient {
            ColorUtils::lerp_color32(
                base_color,
                ColorUtils::lighten(base_color, 0.2),
                hover_factor,
            )
        } else {
            ColorUtils::lerp_color32(
                base_color,
                ColorUtils::lighten(base_color, 0.1),
                hover_factor,
            )
        };

        // Draw glow effect if enabled
        if self.style.glow && hover_factor > 0.0 {
            DrawUtils::draw_glow_effect(
                ui,
                scaled_rect.center(),
                scaled_rect.width() * 0.6,
                button_color,
                hover_factor * 0.6,
            );
        }

        // Draw button background
        painter.rect_filled(scaled_rect, self.style.rounding, button_color);

        // Draw button border
        let border_color = ColorUtils::lerp_color32(
            ColorUtils::darken(button_color, 0.2),
            colors.accent,
            hover_factor,
        );
        painter.rect_stroke(scaled_rect, self.style.rounding, Stroke::new(1.0, border_color));

        // Draw content
        let content_rect = scaled_rect.shrink2(self.style.padding);
        let mut content_pos = content_rect.center();

        if let Some(icon) = &self.icon {
            // Draw icon
            let icon_pos = Pos2::new(
                content_pos.x - 8.0,
                content_pos.y,
            );
            painter.text(
                icon_pos,
                egui::Align2::CENTER_CENTER,
                icon,
                egui::FontId::default(),
                colors.text,
            );
            content_pos.x += 12.0;
        }

        // Draw text
        painter.text(
            content_pos,
            egui::Align2::CENTER_CENTER,
            &self.text,
            egui::FontId::default(),
            colors.text,
        );
    }
}