use super::{
    theme::ThemeColors,
    utils::{AnimationState, ColorUtils},
};
use egui::{Color32, Pos2, Rect, Response, RichText, Sense, Stroke, Ui, Vec2};
use std::time::Duration;

/// Album art display component with loading states and animations
pub struct AlbumArtDisplay {
    pub texture: Option<egui::TextureHandle>,
    pub loading: bool,
    pub size: Vec2,
    pub rounded_corners: bool,
    pub show_placeholder: bool,
    fade_animation: AnimationState,
}

impl AlbumArtDisplay {
    pub fn new(size: Vec2) -> Self {
        Self {
            texture: None,
            loading: false,
            size,
            rounded_corners: true,
            show_placeholder: true,
            fade_animation: AnimationState::new(0.0, 5.0),
        }
    }

    pub fn set_texture(&mut self, texture: Option<egui::TextureHandle>) {
        let has_texture = texture.is_some();
        self.texture = texture;
        self.loading = false;
        self.fade_animation
            .set_target(if has_texture { 1.0 } else { 0.0 });
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn show(&mut self, ui: &mut Ui, colors: &ThemeColors) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());

        // Update animation
        let dt = ui.ctx().input(|i| i.stable_dt);
        self.fade_animation.update(dt);

        self.draw(ui, rect, colors);

        response
    }

    fn draw(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let fade_factor = self.fade_animation.value();

        // Draw background
        let bg_color = if self.loading {
            ColorUtils::lerp_color32(colors.surface, colors.primary, 0.1)
        } else {
            colors.surface
        };

        if self.rounded_corners {
            painter.rect_filled(rect, 8.0, bg_color);
        } else {
            painter.rect_filled(rect, 0.0, bg_color);
        }

        if let Some(texture) = &self.texture {
            // Draw the actual image
            let image_color = ColorUtils::with_alpha(Color32::WHITE, fade_factor);
            let image = egui::Image::from_texture(texture)
                .fit_to_exact_size(self.size)
                .tint(image_color);

            image.paint_at(ui, rect);
        } else if self.loading {
            // Draw loading spinner
            self.draw_loading_spinner(ui, rect, colors);
        } else if self.show_placeholder {
            // Draw placeholder
            self.draw_placeholder(ui, rect, colors);
        }

        // Draw border
        let border_color = ColorUtils::with_alpha(colors.text_secondary, 0.3);
        if self.rounded_corners {
            painter.rect_stroke(
                rect,
                8.0,
                Stroke::new(1.0, border_color),
                egui::epaint::StrokeKind::Outside,
            );
        } else {
            painter.rect_stroke(
                rect,
                0.0,
                Stroke::new(1.0, border_color),
                egui::epaint::StrokeKind::Outside,
            );
        }
    }

    fn draw_loading_spinner(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let center = rect.center();
        let radius = rect.width().min(rect.height()) * 0.15;
        let time = ui.ctx().input(|i| i.time) as f32;

        // Draw spinning circle segments
        for i in 0..8 {
            let angle = (i as f32 / 8.0) * 2.0 * std::f32::consts::PI + time * 2.0;
            let start_pos = center + Vec2::angled(angle) * radius * 0.8;
            let end_pos = center + Vec2::angled(angle) * radius;

            let alpha = ((i as f32 / 8.0) + time).sin().abs();
            let color = ColorUtils::with_alpha(colors.primary, alpha);

            ui.painter()
                .line_segment([start_pos, end_pos], Stroke::new(3.0, color));
        }
    }

    fn draw_placeholder(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let center = rect.center();
        let size = rect.width().min(rect.height()) * 0.3;

        // Draw music note icon (simplified)
        let icon_color = ColorUtils::with_alpha(colors.text_secondary, 0.5);

        // Draw note stem
        let stem_start = center + Vec2::new(size * 0.2, -size * 0.4);
        let stem_end = center + Vec2::new(size * 0.2, size * 0.3);
        ui.painter()
            .line_segment([stem_start, stem_end], Stroke::new(3.0, icon_color));

        // Draw note head
        ui.painter()
            .circle_filled(center + Vec2::new(0.0, size * 0.3), size * 0.15, icon_color);

        // Draw beam
        let beam_points = vec![
            stem_start,
            stem_start + Vec2::new(size * 0.3, -size * 0.1),
            stem_start + Vec2::new(size * 0.3, size * 0.05),
            stem_start + Vec2::new(0.0, size * 0.15),
        ];
        ui.painter().add(egui::Shape::convex_polygon(
            beam_points,
            icon_color,
            Stroke::NONE,
        ));
    }
}

/// Enhanced progress bar with multiple display modes
pub struct ProgressBar {
    pub progress: f32,
    pub total: f32,
    pub buffered: f32,
    pub style: ProgressBarStyle,
    hover_animation: AnimationState,
    drag_animation: AnimationState,
}

#[derive(Debug, Clone)]
pub struct ProgressBarStyle {
    pub height: f32,
    pub rounding: f32,
    pub show_time: bool,
    pub show_buffer: bool,
    pub gradient: bool,
    pub animated: bool,
}

impl Default for ProgressBarStyle {
    fn default() -> Self {
        Self {
            height: 6.0,
            rounding: 3.0,
            show_time: true,
            show_buffer: true,
            gradient: true,
            animated: true,
        }
    }
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            total: 1.0,
            buffered: 0.0,
            style: ProgressBarStyle::default(),
            hover_animation: AnimationState::new(0.0, 10.0),
            drag_animation: AnimationState::new(0.0, 15.0),
        }
    }

    pub fn set_progress(&mut self, progress: f32, total: f32) {
        self.progress = progress.clamp(0.0, total);
        self.total = total.max(0.0);
    }

    pub fn set_buffered(&mut self, buffered: f32) {
        self.buffered = buffered.clamp(0.0, self.total);
    }

    pub fn style(mut self, style: ProgressBarStyle) -> Self {
        self.style = style;
        self
    }

    pub fn show(&mut self, ui: &mut Ui, colors: &ThemeColors) -> Response {
        let desired_height = if self.style.show_time {
            self.style.height + 40.0
        } else {
            self.style.height + 16.0
        };

        let (rect, mut response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), desired_height),
            Sense::click_and_drag(),
        );

        // Handle seeking
        if response.clicked() || response.dragged() {
            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let progress_rect = self.get_progress_rect(rect);
                let seek_t =
                    ((pointer_pos.x - progress_rect.min.x) / progress_rect.width()).clamp(0.0, 1.0);
                let new_progress = seek_t * self.total;

                if (new_progress - self.progress).abs() > 0.1 {
                    self.progress = new_progress;
                    response.mark_changed();
                }
            }
        }

        // Update animations
        let dt = ui.ctx().input(|i| i.stable_dt);
        self.hover_animation
            .set_target(if response.hovered() { 1.0 } else { 0.0 });
        self.hover_animation.update(dt);

        self.drag_animation
            .set_target(if response.dragged() { 1.0 } else { 0.0 });
        self.drag_animation.update(dt);

        self.draw(ui, rect, colors);

        response
    }

    fn get_progress_rect(&self, rect: Rect) -> Rect {
        let y_offset = if self.style.show_time { 20.0 } else { 8.0 };
        Rect::from_min_size(
            rect.min + Vec2::new(0.0, y_offset),
            Vec2::new(rect.width(), self.style.height),
        )
    }

    fn draw(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let progress_rect = self.get_progress_rect(rect);

        let hover_factor = self.hover_animation.value();
        let drag_factor = self.drag_animation.value();

        // Enhanced height when interacting
        let enhanced_height = self.style.height * (1.0 + (hover_factor * 0.5 + drag_factor * 0.3));
        let enhanced_rect = Rect::from_center_size(
            progress_rect.center(),
            Vec2::new(progress_rect.width(), enhanced_height),
        );

        // Draw background track
        let bg_color = ColorUtils::lerp_color32(
            colors.surface,
            ColorUtils::lighten(colors.surface, 0.1),
            hover_factor,
        );
        painter.rect_filled(enhanced_rect, self.style.rounding, bg_color);

        // Draw buffered progress if enabled
        if self.style.show_buffer && self.buffered > 0.0 {
            let buffer_width = (self.buffered / self.total) * enhanced_rect.width();
            let buffer_rect = Rect::from_min_size(
                enhanced_rect.min,
                Vec2::new(buffer_width, enhanced_rect.height()),
            );

            let buffer_color = ColorUtils::with_alpha(colors.text_secondary, 0.3);
            painter.rect_filled(buffer_rect, self.style.rounding, buffer_color);
        }

        // Draw progress
        if self.progress > 0.0 {
            let progress_width = (self.progress / self.total) * enhanced_rect.width();
            let fill_rect = Rect::from_min_size(
                enhanced_rect.min,
                Vec2::new(progress_width, enhanced_rect.height()),
            );

            let progress_color = if self.style.gradient {
                // Gradient based on position
                let t = self.progress / self.total;
                ColorUtils::lerp_color32(colors.primary, colors.accent, t)
            } else {
                colors.primary
            };

            painter.rect_filled(fill_rect, self.style.rounding, progress_color);

            // Draw glow effect when dragging
            if drag_factor > 0.0 {
                let glow_rect = fill_rect.expand2(Vec2::splat(2.0 * drag_factor));
                let glow_color = ColorUtils::with_alpha(progress_color, 0.3 * drag_factor);
                painter.rect_filled(glow_rect, self.style.rounding + 2.0, glow_color);
            }
        }

        // Draw playhead
        if self.progress > 0.0 {
            let playhead_x =
                enhanced_rect.min.x + (self.progress / self.total) * enhanced_rect.width();
            let playhead_rect = Rect::from_center_size(
                Pos2::new(playhead_x, enhanced_rect.center().y),
                Vec2::splat(enhanced_height + 4.0),
            );

            let playhead_color = ColorUtils::lerp_color32(
                colors.text,
                colors.accent,
                hover_factor * 0.5 + drag_factor * 0.5,
            );

            painter.circle_filled(
                playhead_rect.center(),
                playhead_rect.width() * 0.5,
                playhead_color,
            );
        }

        // Draw time labels if enabled
        if self.style.show_time {
            let current_time = format_duration(Duration::from_secs_f32(self.progress));
            let total_time = format_duration(Duration::from_secs_f32(self.total));

            // Current time (left)
            painter.text(
                rect.min + Vec2::new(0.0, 0.0),
                egui::Align2::LEFT_TOP,
                current_time,
                egui::FontId::proportional(12.0),
                colors.text_secondary,
            );

            // Total time (right)
            painter.text(
                rect.min + Vec2::new(rect.width(), 0.0),
                egui::Align2::RIGHT_TOP,
                total_time,
                egui::FontId::proportional(12.0),
                colors.text_secondary,
            );
        }
    }
}

/// Metadata display component for track information
pub struct MetadataDisplay {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub layout: MetadataLayout,
    scroll_animation: AnimationState,
    title_scroll_offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataLayout {
    Vertical,
    Horizontal,
    Compact,
}

impl MetadataDisplay {
    pub fn new() -> Self {
        Self {
            title: "No Title".to_string(),
            artist: "Unknown Artist".to_string(),
            album: "Unknown Album".to_string(),
            year: "----".to_string(),
            layout: MetadataLayout::Vertical,
            scroll_animation: AnimationState::new(0.0, 8.0),
            title_scroll_offset: 0.0,
        }
    }

    pub fn set_metadata(&mut self, title: String, artist: String, album: String, year: String) {
        self.title = title;
        self.artist = artist;
        self.album = album;
        self.year = year;
        self.title_scroll_offset = 0.0;
        self.scroll_animation.set_target(0.0);
    }

    pub fn layout(mut self, layout: MetadataLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn show(&mut self, ui: &mut Ui, colors: &ThemeColors) -> Response {
        let available_rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(available_rect, Sense::hover());

        match self.layout {
            MetadataLayout::Vertical => self.draw_vertical(ui, available_rect, colors),
            MetadataLayout::Horizontal => self.draw_horizontal(ui, available_rect, colors),
            MetadataLayout::Compact => self.draw_compact(ui, available_rect, colors),
        }

        response
    }

    fn draw_vertical(&mut self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let line_height = 18.0;
        let mut y = rect.min.y;

        // Title (larger, with scrolling if needed)
        let title_font = egui::FontId::proportional(16.0);
        let title_text = RichText::new(&self.title)
            .font(title_font.clone())
            .color(colors.text);

        let title_size = painter
            .layout_no_wrap(self.title.clone(), title_font, colors.text)
            .size();

        if title_size.x > rect.width() {
            // Scroll long titles
            self.update_title_scroll(ui, rect.width(), title_size.x);
            let scroll_x = rect.min.x - self.title_scroll_offset;
            painter.text(
                Pos2::new(scroll_x, y),
                egui::Align2::LEFT_TOP,
                title_text.text(),
                egui::FontId::proportional(16.0),
                colors.text,
            );
        } else {
            painter.text(
                Pos2::new(rect.center().x, y),
                egui::Align2::CENTER_TOP,
                title_text.text(),
                egui::FontId::proportional(16.0),
                colors.text,
            );
        }
        y += line_height + 4.0;

        // Artist
        painter.text(
            Pos2::new(rect.center().x, y),
            egui::Align2::CENTER_TOP,
            &self.artist,
            egui::FontId::proportional(14.0),
            colors.text_secondary,
        );
        y += line_height;

        // Album and year
        let album_year = format!("{} ({})", self.album, self.year);
        painter.text(
            Pos2::new(rect.center().x, y),
            egui::Align2::CENTER_TOP,
            album_year,
            egui::FontId::proportional(12.0),
            colors.text_secondary,
        );
    }

    fn draw_horizontal(&self, ui: &mut Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();

        ui.allocate_ui_at_rect(rect, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(&self.title).size(14.0).color(colors.text));
                    ui.label(
                        RichText::new(&self.artist)
                            .size(12.0)
                            .color(colors.text_secondary),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(&self.year)
                                .size(10.0)
                                .color(colors.text_secondary),
                        );
                        ui.label(
                            RichText::new(&self.album)
                                .size(11.0)
                                .color(colors.text_secondary),
                        );
                    });
                });
            });
        });
    }

    fn draw_compact(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();

        let compact_text = if self.artist != "Unknown Artist" && self.title != "No Title" {
            format!("{} - {}", self.artist, self.title)
        } else {
            self.title.clone()
        };

        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            compact_text,
            egui::FontId::proportional(13.0),
            colors.text,
        );
    }

    fn update_title_scroll(&mut self, ui: &Ui, available_width: f32, text_width: f32) {
        let dt = ui.ctx().input(|i| i.stable_dt);
        let overflow = text_width - available_width;

        if overflow > 0.0 {
            // Scroll speed: pixels per second
            let scroll_speed = 30.0;
            let max_offset = overflow + 50.0; // Add some padding

            // Oscillate back and forth
            let time = ui.ctx().input(|i| i.time) as f32;
            let cycle_time = max_offset / scroll_speed * 2.0 + 2.0; // 2 seconds pause at each end
            let t = (time % cycle_time) / cycle_time;

            self.title_scroll_offset = if t < 0.5 {
                // Forward scroll
                (t * 2.0) * max_offset
            } else {
                // Backward scroll
                ((1.0 - t) * 2.0) * max_offset
            };
        } else {
            self.title_scroll_offset = 0.0;
        }
    }
}

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

impl Default for MetadataDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}
