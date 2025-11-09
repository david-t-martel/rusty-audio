use super::theme::ThemeColors;
use super::utils::{AnimationState, ColorUtils, DrawUtils};
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum SpectrumMode {
    Bars,
    Line,
    Filled,
    Circular,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrequencyScale {
    Linear,
    Logarithmic,
    Mel,
}

#[derive(Debug, Clone)]
pub struct SpectrumVisualizerConfig {
    pub mode: SpectrumMode,
    pub frequency_scale: FrequencyScale,
    pub smoothing: f32,
    pub peak_hold_time: f32,
    pub peak_decay_rate: f32,
    pub bar_width: f32,
    pub bar_spacing: f32,
    pub gradient_enabled: bool,
    pub glow_enabled: bool,
    pub mirror_enabled: bool,
    pub num_bars: usize,
    pub db_range: (f32, f32), // (min_db, max_db)
    pub update_rate: f32,     // Hz
    pub show_labels: bool,
    pub show_db_scale: bool,
    pub show_frequency_labels: bool,
}

impl Default for SpectrumVisualizerConfig {
    fn default() -> Self {
        Self {
            mode: SpectrumMode::Bars,
            frequency_scale: FrequencyScale::Logarithmic,
            smoothing: 0.8,
            peak_hold_time: 1.0,
            peak_decay_rate: 20.0,
            bar_width: 0.8,
            bar_spacing: 0.2,
            gradient_enabled: true,
            glow_enabled: false,
            mirror_enabled: false,
            num_bars: 64,
            db_range: (-100.0, 0.0),
            update_rate: 60.0,
            show_labels: true,
            show_db_scale: true,
            show_frequency_labels: true,
        }
    }
}

#[derive(Debug, Clone)]
struct PeakData {
    value: f32,
    hold_time: f32,
}

impl Default for PeakData {
    fn default() -> Self {
        Self {
            value: 0.0,
            hold_time: 0.0,
        }
    }
}

pub struct SpectrumVisualizer {
    config: SpectrumVisualizerConfig,
    smoothed_data: Vec<f32>,
    peak_data: Vec<PeakData>,
    last_update: Instant,
    frame_time: f32,
    bars_animation: Vec<AnimationState>,
    frequency_bins: Vec<f32>,
}

impl Default for SpectrumVisualizer {
    fn default() -> Self {
        Self::new(SpectrumVisualizerConfig::default())
    }
}

impl SpectrumVisualizer {
    pub fn new(config: SpectrumVisualizerConfig) -> Self {
        let num_bars = config.num_bars;
        Self {
            config,
            smoothed_data: vec![0.0; num_bars],
            peak_data: vec![PeakData::default(); num_bars],
            last_update: Instant::now(),
            frame_time: 0.0,
            bars_animation: (0..num_bars)
                .map(|_| AnimationState::new(0.0, 15.0))
                .collect(),
            frequency_bins: Self::calculate_frequency_bins(num_bars, FrequencyScale::Logarithmic),
        }
    }

    pub fn set_config(&mut self, config: SpectrumVisualizerConfig) {
        if config.num_bars != self.config.num_bars
            || config.frequency_scale != self.config.frequency_scale
        {
            let num_bars = config.num_bars;
            self.smoothed_data.resize(num_bars, 0.0);
            self.peak_data.resize(num_bars, PeakData::default());
            self.bars_animation = (0..num_bars)
                .map(|_| AnimationState::new(0.0, 15.0))
                .collect();
            self.frequency_bins =
                Self::calculate_frequency_bins(num_bars, config.frequency_scale.clone());
        }
        self.config = config;
    }

    pub fn config(&self) -> &SpectrumVisualizerConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut SpectrumVisualizerConfig {
        &mut self.config
    }

    pub fn update(&mut self, spectrum_data: &[f32]) {
        let now = Instant::now();
        self.frame_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // Skip update if update rate is too low
        if self.frame_time < (1.0 / self.config.update_rate) {
            return;
        }

        let processed_data = self.process_spectrum_data(spectrum_data);
        self.update_smoothed_data(&processed_data);
        self.update_peak_data();
        self.update_animations();
    }

    pub fn draw(&mut self, ui: &mut Ui, rect: Rect, colors: &ThemeColors) -> Response {
        let response = ui.allocate_rect(rect, Sense::hover());

        // Reserve space for labels if enabled
        let (spectrum_rect, label_rect) = if self.config.show_labels {
            let spectrum_height = rect.height() - 25.0;
            let spectrum_rect =
                Rect::from_min_size(rect.min, Vec2::new(rect.width(), spectrum_height));
            let label_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, rect.min.y + spectrum_height),
                Vec2::new(rect.width(), 25.0),
            );
            (spectrum_rect, Some(label_rect))
        } else {
            (rect, None)
        };

        match self.config.mode {
            SpectrumMode::Bars => self.draw_bars(ui, spectrum_rect, colors),
            SpectrumMode::Line => self.draw_line(ui, spectrum_rect, colors),
            SpectrumMode::Filled => self.draw_filled(ui, spectrum_rect, colors),
            SpectrumMode::Circular => self.draw_circular(ui, spectrum_rect, colors),
        }

        // Draw labels and scales
        if let Some(label_rect) = label_rect {
            if self.config.show_frequency_labels {
                self.draw_frequency_labels(ui, label_rect, colors);
            }
        }

        if self.config.show_db_scale {
            self.draw_db_scale(ui, spectrum_rect, colors);
        }

        response
    }

    fn process_spectrum_data(&self, spectrum_data: &[f32]) -> Vec<f32> {
        if spectrum_data.is_empty() {
            return vec![0.0; self.config.num_bars];
        }

        let mut processed = Vec::with_capacity(self.config.num_bars);

        for i in 0..self.config.num_bars {
            let bin_start = (i * spectrum_data.len()) / self.config.num_bars;
            let bin_end = ((i + 1) * spectrum_data.len()) / self.config.num_bars;

            // Average the frequency bins for this bar
            let mut sum = 0.0;
            let mut count = 0;

            for j in bin_start..bin_end.min(spectrum_data.len()) {
                sum += spectrum_data[j];
                count += 1;
            }

            let average = if count > 0 { sum / count as f32 } else { 0.0 };

            // Convert from linear to dB scale and normalize
            let db_value = if average > 0.0 {
                20.0 * average.log10()
            } else {
                self.config.db_range.0
            };
            let normalized = ((db_value - self.config.db_range.0)
                / (self.config.db_range.1 - self.config.db_range.0))
                .clamp(0.0, 1.0);

            processed.push(normalized);
        }

        processed
    }

    fn update_smoothed_data(&mut self, new_data: &[f32]) {
        for (i, &new_value) in new_data.iter().enumerate() {
            if i < self.smoothed_data.len() {
                let current = self.smoothed_data[i];
                self.smoothed_data[i] =
                    current * self.config.smoothing + new_value * (1.0 - self.config.smoothing);

                // Set animation target
                self.bars_animation[i].set_target(self.smoothed_data[i]);
            }
        }
    }

    fn update_peak_data(&mut self) {
        for i in 0..self.peak_data.len() {
            if i < self.smoothed_data.len() {
                let current_value = self.smoothed_data[i];

                if current_value > self.peak_data[i].value {
                    // New peak
                    self.peak_data[i].value = current_value;
                    self.peak_data[i].hold_time = self.config.peak_hold_time;
                } else {
                    // Decay existing peak
                    self.peak_data[i].hold_time -= self.frame_time;
                    if self.peak_data[i].hold_time <= 0.0 {
                        let decay = self.config.peak_decay_rate * self.frame_time;
                        self.peak_data[i].value =
                            (self.peak_data[i].value - decay).max(current_value);
                    }
                }
            }
        }
    }

    fn update_animations(&mut self) {
        for animation in &mut self.bars_animation {
            animation.update(self.frame_time);
        }
    }

    fn draw_bars(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let num_bars = self.config.num_bars;

        if num_bars == 0 {
            return;
        }

        let total_bar_width = rect.width() / num_bars as f32;
        let bar_width = total_bar_width * self.config.bar_width;
        let bar_spacing = total_bar_width * self.config.bar_spacing;
        let effective_bar_width = bar_width - bar_spacing;

        for i in 0..num_bars {
            if i >= self.bars_animation.len() {
                break;
            }

            let x = rect.min.x + i as f32 * total_bar_width + bar_spacing * 0.5;
            let height = self.bars_animation[i].value() * rect.height();

            let bar_rect = Rect::from_min_size(
                Pos2::new(x, rect.max.y - height),
                Vec2::new(effective_bar_width, height),
            );

            // Get color for this bar
            let color_index = (i * colors.spectrum_colors.len()) / num_bars;
            let bar_color = colors
                .spectrum_colors
                .get(color_index)
                .copied()
                .unwrap_or(colors.primary);

            // Draw main bar
            if self.config.gradient_enabled {
                self.draw_gradient_bar(ui, bar_rect, bar_color);
            } else {
                painter.rect_filled(bar_rect, 2.0, bar_color);
            }

            // Draw glow effect if enabled
            if self.config.glow_enabled && height > 0.1 {
                let glow_intensity = (height / rect.height()) * 0.8;
                DrawUtils::draw_glow_effect(
                    ui,
                    bar_rect.center(),
                    effective_bar_width * 0.8,
                    bar_color,
                    glow_intensity,
                );
            }

            // Draw peak indicator
            if self.peak_data[i].value > self.smoothed_data[i] {
                let peak_height = self.peak_data[i].value * rect.height();
                let peak_y = rect.max.y - peak_height;

                let peak_rect = Rect::from_min_size(
                    Pos2::new(x, peak_y - 2.0),
                    Vec2::new(effective_bar_width, 2.0),
                );

                painter.rect_filled(peak_rect, 1.0, ColorUtils::lighten(bar_color, 0.3));
            }

            // Mirror effect
            if self.config.mirror_enabled {
                let mirror_rect = Rect::from_min_size(
                    Pos2::new(x, rect.max.y),
                    Vec2::new(effective_bar_width, height * 0.3),
                );

                let mirror_color = ColorUtils::with_alpha(bar_color, 0.3);
                painter.rect_filled(mirror_rect, 2.0, mirror_color);
            }
        }
    }

    fn draw_line(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let num_bars = self.config.num_bars;

        if num_bars < 2 {
            return;
        }

        let mut points = Vec::with_capacity(num_bars);
        let step = rect.width() / (num_bars - 1) as f32;

        for i in 0..num_bars {
            if i >= self.bars_animation.len() {
                break;
            }

            let x = rect.min.x + i as f32 * step;
            let height = self.bars_animation[i].value() * rect.height();
            let y = rect.max.y - height;

            points.push(Pos2::new(x, y));
        }

        if points.len() > 1 {
            let stroke = Stroke::new(2.0, colors.primary);
            painter.add(egui::Shape::line(points, stroke));
        }
    }

    fn draw_filled(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let num_bars = self.config.num_bars;

        if num_bars < 2 {
            return;
        }

        let mut points = Vec::with_capacity(num_bars + 2);
        let step = rect.width() / (num_bars - 1) as f32;

        // Start from bottom left
        points.push(Pos2::new(rect.min.x, rect.max.y));

        for i in 0..num_bars {
            if i >= self.bars_animation.len() {
                break;
            }

            let x = rect.min.x + i as f32 * step;
            let height = self.bars_animation[i].value() * rect.height();
            let y = rect.max.y - height;

            points.push(Pos2::new(x, y));
        }

        // End at bottom right
        points.push(Pos2::new(rect.max.x, rect.max.y));

        if points.len() > 2 {
            let fill_color = ColorUtils::with_alpha(colors.primary, 0.6);
            let stroke = Stroke::new(1.0, colors.primary);

            painter.add(egui::Shape::convex_polygon(points, fill_color, stroke));
        }
    }

    fn draw_circular(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let center = rect.center();
        let radius = rect.width().min(rect.height()) * 0.4;
        let num_bars = self.config.num_bars;

        for i in 0..num_bars {
            if i >= self.bars_animation.len() {
                break;
            }

            let angle = (i as f32 / num_bars as f32) * 2.0 * std::f32::consts::PI;
            let height = self.bars_animation[i].value() * radius * 0.5;

            let inner_radius = radius * 0.3;
            let outer_radius = inner_radius + height;

            let inner_pos =
                center + Vec2::new(inner_radius * angle.cos(), inner_radius * angle.sin());

            let outer_pos =
                center + Vec2::new(outer_radius * angle.cos(), outer_radius * angle.sin());

            // Get color for this bar
            let color_index = (i * colors.spectrum_colors.len()) / num_bars;
            let bar_color = colors
                .spectrum_colors
                .get(color_index)
                .copied()
                .unwrap_or(colors.primary);

            let stroke = Stroke::new(3.0, bar_color);
            painter.line_segment([inner_pos, outer_pos], stroke);
        }
    }

    fn draw_gradient_bar(&self, ui: &Ui, rect: Rect, color: Color32) {
        let painter = ui.painter();

        // Simple gradient effect using multiple rectangles
        let steps = 10;
        let step_height = rect.height() / steps as f32;

        for i in 0..steps {
            let alpha = 1.0 - (i as f32 / steps as f32) * 0.7;
            let step_color = ColorUtils::with_alpha(color, alpha);

            let step_rect = Rect::from_min_size(
                Pos2::new(rect.min.x, rect.min.y + i as f32 * step_height),
                Vec2::new(rect.width(), step_height),
            );

            painter.rect_filled(step_rect, 1.0, step_color);
        }
    }

    fn calculate_frequency_bins(num_bars: usize, scale: FrequencyScale) -> Vec<f32> {
        let mut bins = Vec::with_capacity(num_bars);

        match scale {
            FrequencyScale::Linear => {
                for i in 0..num_bars {
                    bins.push(i as f32 / num_bars as f32);
                }
            }
            FrequencyScale::Logarithmic => {
                let log_start = 1.0_f32.ln();
                let log_end = (num_bars as f32).ln();
                let log_range = log_end - log_start;

                for i in 0..num_bars {
                    let log_pos = log_start + (i as f32 / (num_bars - 1) as f32) * log_range;
                    bins.push((log_pos.exp() - 1.0) / (num_bars - 1) as f32);
                }
            }
            FrequencyScale::Mel => {
                // Simplified mel scale approximation
                for i in 0..num_bars {
                    let linear = i as f32 / num_bars as f32;
                    let mel = 2595.0 * (1.0 + linear * 7000.0).log10();
                    bins.push(mel / 4000.0); // Normalize
                }
            }
        }

        bins
    }

    fn draw_frequency_labels(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let font_id = egui::FontId::proportional(9.0);

        // Draw frequency labels at key positions (20Hz, 100Hz, 1kHz, 10kHz, 20kHz)
        let frequencies = [20.0, 100.0, 1000.0, 10000.0, 20000.0];
        let sample_rate = 44100.0; // Assume standard sample rate

        for &freq in &frequencies {
            // Map frequency to position (logarithmic)
            let nyquist = sample_rate / 2.0_f32;
            let freq_ratio = freq / nyquist;
            let normalized = if freq_ratio > 0.0 {
                freq_ratio.ln() / nyquist.ln()
            } else {
                0.0
            };
            let x = rect.min.x + (normalized + 1.0) * rect.width() * 0.5; // Adjust for log scale

            if x >= rect.min.x && x <= rect.max.x {
                // Draw tick mark
                let tick_start = Pos2::new(x, rect.min.y);
                let tick_end = Pos2::new(x, rect.min.y + 4.0);
                painter.line_segment(
                    [tick_start, tick_end],
                    Stroke::new(1.0, colors.text_secondary),
                );

                // Draw label
                let label = if freq >= 1000.0 {
                    format!("{:.0}k", freq / 1000.0)
                } else {
                    format!("{:.0}", freq)
                };

                let text_pos = Pos2::new(x, rect.min.y + 6.0);
                painter.text(
                    text_pos,
                    egui::Align2::CENTER_TOP,
                    label,
                    font_id.clone(),
                    colors.text_secondary,
                );
            }
        }
    }

    fn draw_db_scale(&self, ui: &Ui, rect: Rect, colors: &ThemeColors) {
        let painter = ui.painter();
        let font_id = egui::FontId::proportional(9.0);

        // Draw dB scale on the left side
        let db_marks = [-60.0, -40.0, -20.0, -10.0, -3.0, 0.0];

        for &db in &db_marks {
            // Map dB to y position
            let normalized = ((db - self.config.db_range.0)
                / (self.config.db_range.1 - self.config.db_range.0))
                .clamp(0.0, 1.0);
            let y = rect.max.y - normalized * rect.height();

            // Draw horizontal grid line
            let line_start = Pos2::new(rect.min.x, y);
            let line_end = Pos2::new(rect.max.x, y);
            let alpha = if db == 0.0 { 0.3 } else { 0.1 };
            painter.line_segment(
                [line_start, line_end],
                Stroke::new(1.0, ColorUtils::with_alpha(colors.text_secondary, alpha)),
            );

            // Draw dB label on the left
            let label = format!("{:.0}", db);
            let text_pos = Pos2::new(rect.min.x + 3.0, y);
            painter.text(
                text_pos,
                egui::Align2::LEFT_CENTER,
                label,
                font_id.clone(),
                colors.text_secondary,
            );
        }
    }
}

impl SpectrumMode {
    pub fn all() -> Vec<Self> {
        vec![Self::Bars, Self::Line, Self::Filled, Self::Circular]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Bars => "Bars",
            Self::Line => "Line",
            Self::Filled => "Filled",
            Self::Circular => "Circular",
        }
    }
}

impl FrequencyScale {
    pub fn all() -> Vec<Self> {
        vec![Self::Linear, Self::Logarithmic, Self::Mel]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Linear => "Linear",
            Self::Logarithmic => "Logarithmic",
            Self::Mel => "Mel Scale",
        }
    }
}
