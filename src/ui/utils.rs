use egui::{Ui, Vec2, Rect, Pos2, Color32, Stroke};

/// Responsive sizing utilities
pub struct ResponsiveSize {
    pub base: Vec2,
    pub min: Vec2,
    pub max: Vec2,
}

impl ResponsiveSize {
    pub fn new(base: Vec2) -> Self {
        Self {
            base,
            min: base * 0.5,
            max: base * 2.0,
        }
    }

    pub fn with_constraints(base: Vec2, min: Vec2, max: Vec2) -> Self {
        Self { base, min, max }
    }

    pub fn calculate(&self, available: Vec2, scale_factor: f32) -> Vec2 {
        let scaled = self.base * scale_factor;
        Vec2::new(
            scaled.x.clamp(self.min.x, self.max.x.min(available.x)),
            scaled.y.clamp(self.min.y, self.max.y.min(available.y)),
        )
    }
}

/// Screen size categories for responsive design
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenSize {
    Mobile,    // < 600px width
    Tablet,    // 600-900px width
    Desktop,   // 900-1200px width
    Large,     // > 1200px width
}

impl ScreenSize {
    pub fn from_width(width: f32) -> Self {
        match width {
            w if w < 600.0 => Self::Mobile,
            w if w < 900.0 => Self::Tablet,
            w if w < 1200.0 => Self::Desktop,
            _ => Self::Large,
        }
    }

    pub fn scale_factor(&self) -> f32 {
        match self {
            Self::Mobile => 0.8,
            Self::Tablet => 0.9,
            Self::Desktop => 1.0,
            Self::Large => 1.1,
        }
    }

    pub fn font_size(&self) -> f32 {
        match self {
            Self::Mobile => 12.0,
            Self::Tablet => 13.0,
            Self::Desktop => 14.0,
            Self::Large => 15.0,
        }
    }

    pub fn spacing(&self) -> f32 {
        match self {
            Self::Mobile => 4.0,
            Self::Tablet => 6.0,
            Self::Desktop => 8.0,
            Self::Large => 10.0,
        }
    }

    pub fn padding(&self) -> f32 {
        match self {
            Self::Mobile => 8.0,
            Self::Tablet => 10.0,
            Self::Desktop => 12.0,
            Self::Large => 16.0,
        }
    }
}

/// Animation helper for smooth transitions
#[derive(Debug, Clone)]
pub struct AnimationState {
    pub current: f32,
    pub target: f32,
    pub speed: f32,
    pub threshold: f32,
}

impl AnimationState {
    pub fn new(initial: f32, speed: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            speed,
            threshold: 0.001,
        }
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn update(&mut self, dt: f32) -> bool {
        let diff = self.target - self.current;
        if diff.abs() < self.threshold {
            self.current = self.target;
            false
        } else {
            self.current += diff * self.speed * dt;
            true
        }
    }

    pub fn value(&self) -> f32 {
        self.current
    }

    pub fn is_animating(&self) -> bool {
        (self.target - self.current).abs() > self.threshold
    }
}

/// Color utilities for UI theming
pub struct ColorUtils;

impl ColorUtils {
    pub fn lerp_color32(a: Color32, b: Color32, t: f32) -> Color32 {
        let t = t.clamp(0.0, 1.0);
        Color32::from_rgba_unmultiplied(
            (a.r() as f32 * (1.0 - t) + b.r() as f32 * t) as u8,
            (a.g() as f32 * (1.0 - t) + b.g() as f32 * t) as u8,
            (a.b() as f32 * (1.0 - t) + b.b() as f32 * t) as u8,
            (a.a() as f32 * (1.0 - t) + b.a() as f32 * t) as u8,
        )
    }

    pub fn with_alpha(color: Color32, alpha: f32) -> Color32 {
        let alpha = (alpha.clamp(0.0, 1.0) * 255.0) as u8;
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
    }

    pub fn lighten(color: Color32, amount: f32) -> Color32 {
        Self::lerp_color32(color, Color32::WHITE, amount)
    }

    pub fn darken(color: Color32, amount: f32) -> Color32 {
        Self::lerp_color32(color, Color32::BLACK, amount)
    }
}

/// Drawing utilities for custom widgets
pub struct DrawUtils;

impl DrawUtils {
    pub fn draw_rounded_rect_outline(
        ui: &Ui,
        rect: Rect,
        rounding: f32,
        stroke: Stroke,
    ) {
        ui.painter().rect_stroke(rect, rounding, stroke, egui::epaint::StrokeKind::Outside);
    }

    pub fn draw_rounded_rect_filled(
        ui: &Ui,
        rect: Rect,
        rounding: f32,
        fill: Color32,
    ) {
        ui.painter().rect_filled(rect, rounding, fill);
    }

    pub fn draw_gradient_rect(
        ui: &Ui,
        rect: Rect,
        top_color: Color32,
        bottom_color: Color32,
        rounding: f32,
    ) {
        let painter = ui.painter();
        let mesh = egui::Mesh::with_texture(egui::TextureId::default());

        // This is a simplified gradient - for full gradients, we'd need to create a proper mesh
        let mid_color = ColorUtils::lerp_color32(top_color, bottom_color, 0.5);
        painter.rect_filled(rect, rounding, mid_color);
    }

    pub fn draw_shadow(
        ui: &Ui,
        rect: Rect,
        rounding: f32,
        blur: f32,
        color: Color32,
    ) {
        let shadow_rect = Rect::from_min_size(
            rect.min + Vec2::splat(blur * 0.5),
            rect.size(),
        );
        ui.painter().rect_filled(shadow_rect, rounding, ColorUtils::with_alpha(color, 0.3));
    }

    pub fn draw_glow_effect(
        ui: &Ui,
        center: Pos2,
        radius: f32,
        color: Color32,
        intensity: f32,
    ) {
        let painter = ui.painter();
        let glow_color = ColorUtils::with_alpha(color, intensity * 0.5);

        for i in 0..5 {
            let r = radius * (1.0 + i as f32 * 0.2);
            let alpha = intensity * (1.0 - i as f32 * 0.2);
            let circle_color = ColorUtils::with_alpha(color, alpha);
            painter.circle_filled(center, r, circle_color);
        }
    }
}

/// Layout calculation utilities
pub struct LayoutUtils;

impl LayoutUtils {
    pub fn calculate_grid_layout(
        available: Vec2,
        item_count: usize,
        min_item_size: Vec2,
        spacing: f32,
    ) -> (usize, usize, Vec2) {
        if item_count == 0 {
            return (0, 0, Vec2::ZERO);
        }

        let cols = ((available.x + spacing) / (min_item_size.x + spacing)).floor().max(1.0) as usize;
        let rows = (item_count + cols - 1) / cols;

        let item_width = (available.x - spacing * (cols - 1) as f32) / cols as f32;
        let item_height = (available.y - spacing * (rows - 1) as f32) / rows as f32;

        let item_size = Vec2::new(
            item_width.max(min_item_size.x),
            item_height.max(min_item_size.y),
        );

        (cols, rows, item_size)
    }

    pub fn distribute_space(
        available: f32,
        weights: &[f32],
        spacing: f32,
    ) -> Vec<f32> {
        if weights.is_empty() {
            return vec![];
        }

        let total_weight: f32 = weights.iter().sum();
        let total_spacing = spacing * (weights.len() - 1) as f32;
        let available_for_content = available - total_spacing;

        weights
            .iter()
            .map(|&weight| (weight / total_weight) * available_for_content)
            .collect()
    }

    pub fn center_rect_in_rect(inner: Vec2, outer: Rect) -> Rect {
        let center = outer.center();
        Rect::from_center_size(center, inner)
    }

    pub fn fit_rect_in_rect(inner: Vec2, outer: Rect, maintain_aspect: bool) -> Rect {
        if maintain_aspect {
            let scale_x = outer.width() / inner.x;
            let scale_y = outer.height() / inner.y;
            let scale = scale_x.min(scale_y);
            let fitted_size = inner * scale;
            Self::center_rect_in_rect(fitted_size, outer)
        } else {
            outer
        }
    }
}