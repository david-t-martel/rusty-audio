use super::utils::{AnimationState, LayoutUtils, ScreenSize};
use egui::{Id, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PanelType {
    Playback,
    Effects,
    Equalizer,
    Playlist,
    Settings,
    Spectrum,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DockSide {
    Left,
    Right,
    Top,
    Bottom,
    Center,
    Floating,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResizeDirection {
    Horizontal,
    Vertical,
    Both,
    None,
}

#[derive(Debug, Clone)]
pub struct PanelConfig {
    pub panel_type: PanelType,
    pub title: String,
    pub dock_side: DockSide,
    pub size: Vec2,
    pub min_size: Vec2,
    pub max_size: Vec2,
    pub resizable: ResizeDirection,
    pub closable: bool,
    pub collapsible: bool,
    pub collapsed: bool,
    pub visible: bool,
    pub weight: f32, // For proportional sizing
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            panel_type: PanelType::Custom("Default".to_string()),
            title: "Panel".to_string(),
            dock_side: DockSide::Center,
            size: Vec2::new(300.0, 200.0),
            min_size: Vec2::new(100.0, 100.0),
            max_size: Vec2::new(800.0, 600.0),
            resizable: ResizeDirection::Both,
            closable: true,
            collapsible: true,
            collapsed: false,
            visible: true,
            weight: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutState {
    pub panels: HashMap<String, PanelConfig>,
    pub dock_sizes: HashMap<DockSide, f32>,
    pub floating_panels: Vec<String>,
    pub resize_states: HashMap<String, Vec2>,
    pub last_screen_size: Vec2,
    pub layout_name: String,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            panels: HashMap::new(),
            dock_sizes: HashMap::new(),
            floating_panels: Vec::new(),
            resize_states: HashMap::new(),
            last_screen_size: Vec2::ZERO,
            layout_name: "Default".to_string(),
        }
    }
}

pub struct LayoutManager {
    state: LayoutState,
    saved_layouts: HashMap<String, LayoutState>,
    resize_handles: HashMap<String, bool>, // Track active resize handles
    animations: HashMap<String, AnimationState>,
    drag_state: Option<DragState>,
}

#[derive(Debug, Clone)]
struct DragState {
    panel_id: String,
    start_pos: Pos2,
    current_pos: Pos2,
    original_dock: DockSide,
}

impl Default for LayoutManager {
    fn default() -> Self {
        let mut manager = Self {
            state: LayoutState::default(),
            saved_layouts: HashMap::new(),
            resize_handles: HashMap::new(),
            animations: HashMap::new(),
            drag_state: None,
        };

        // Initialize with default panels
        manager.create_default_layout();
        manager
    }
}

impl LayoutManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_panel(&mut self, id: String, config: PanelConfig) {
        self.state.panels.insert(id.clone(), config);
        self.animations.insert(id, AnimationState::new(1.0, 8.0));
    }

    pub fn remove_panel(&mut self, id: &str) -> Option<PanelConfig> {
        self.animations.remove(id);
        self.resize_handles.remove(id);
        self.state.panels.remove(id)
    }

    pub fn get_panel(&self, id: &str) -> Option<&PanelConfig> {
        self.state.panels.get(id)
    }

    pub fn get_panel_mut(&mut self, id: &str) -> Option<&mut PanelConfig> {
        self.state.panels.get_mut(id)
    }

    pub fn set_panel_visible(&mut self, id: &str, visible: bool) {
        if let Some(panel) = self.state.panels.get_mut(id) {
            panel.visible = visible;
        }
    }

    pub fn toggle_panel_collapsed(&mut self, id: &str) {
        if let Some(panel) = self.state.panels.get_mut(id) {
            if panel.collapsible {
                panel.collapsed = !panel.collapsed;
                if let Some(animation) = self.animations.get_mut(id) {
                    animation.set_target(if panel.collapsed { 0.0 } else { 1.0 });
                }
            }
        }
    }

    pub fn dock_panel(&mut self, id: &str, dock_side: DockSide) {
        if let Some(panel) = self.state.panels.get_mut(id) {
            // Remove from floating list if it was floating
            if panel.dock_side == DockSide::Floating {
                self.state.floating_panels.retain(|p| p != id);
            }

            panel.dock_side = dock_side.clone();

            // Add to floating list if docking to floating
            if dock_side == DockSide::Floating {
                self.state.floating_panels.push(id.to_string());
            }
        }
    }

    pub fn save_layout(&mut self, name: String) {
        self.state.layout_name = name.clone();
        self.saved_layouts.insert(name, self.state.clone());
    }

    pub fn load_layout(&mut self, name: &str) -> bool {
        if let Some(layout) = self.saved_layouts.get(name).cloned() {
            self.state = layout;

            // Rebuild animations for all panels
            self.animations.clear();
            for id in self.state.panels.keys() {
                self.animations
                    .insert(id.clone(), AnimationState::new(1.0, 8.0));
            }

            true
        } else {
            false
        }
    }

    pub fn get_saved_layouts(&self) -> Vec<&String> {
        self.saved_layouts.keys().collect()
    }

    pub fn update_responsive_layout(&mut self, screen_size: Vec2) {
        if (screen_size - self.state.last_screen_size).length() < 10.0 {
            return; // No significant change
        }

        self.state.last_screen_size = screen_size;
        let screen_category = ScreenSize::from_width(screen_size.x);

        // Adjust panel sizes based on screen size
        for (id, panel) in self.state.panels.iter_mut() {
            let scale_factor = screen_category.scale_factor();

            // Don't resize floating panels automatically
            if panel.dock_side != DockSide::Floating {
                let new_size = panel.size * scale_factor;
                panel.size = Vec2::new(
                    new_size.x.clamp(panel.min_size.x, panel.max_size.x),
                    new_size.y.clamp(panel.min_size.y, panel.max_size.y),
                );
            }
        }

        // Auto-collapse panels on mobile if needed
        if screen_category == ScreenSize::Mobile {
            for panel in self.state.panels.values_mut() {
                if panel.collapsible && panel.size.x > screen_size.x * 0.8 {
                    panel.collapsed = true;
                }
            }
        }
    }

    pub fn update_animations(&mut self, dt: f32) {
        for animation in self.animations.values_mut() {
            animation.update(dt);
        }
    }

    pub fn draw_layout(&mut self, ui: &mut Ui) -> HashMap<String, Rect> {
        let mut panel_rects = HashMap::new();
        let available_rect = ui.available_rect_before_wrap();

        // Calculate dock areas
        let dock_areas = self.calculate_dock_areas(available_rect);

        // Draw docked panels
        for dock_side in [
            DockSide::Left,
            DockSide::Top,
            DockSide::Right,
            DockSide::Bottom,
            DockSide::Center,
        ] {
            if let Some(area) = dock_areas.get(&dock_side) {
                let dock_rects = self.draw_docked_panels(ui, *area, dock_side);
                panel_rects.extend(dock_rects);
            }
        }

        // Draw floating panels
        for panel_id in self.state.floating_panels.clone() {
            if let Some(rect) = self.draw_floating_panel(ui, &panel_id) {
                panel_rects.insert(panel_id, rect);
            }
        }

        // Handle drag and drop
        self.handle_drag_and_drop(ui);

        panel_rects
    }

    fn calculate_dock_areas(&self, available_rect: Rect) -> HashMap<DockSide, Rect> {
        let mut areas = HashMap::new();
        let mut remaining = available_rect;

        // Calculate sizes for each dock side
        let left_size = self.get_dock_size(DockSide::Left);
        let right_size = self.get_dock_size(DockSide::Right);
        let top_size = self.get_dock_size(DockSide::Top);
        let bottom_size = self.get_dock_size(DockSide::Bottom);

        // Left dock
        if left_size > 0.0 {
            let left_area =
                Rect::from_min_size(remaining.min, Vec2::new(left_size, remaining.height()));
            areas.insert(DockSide::Left, left_area);
            remaining.min.x += left_size;
        }

        // Right dock
        if right_size > 0.0 {
            let right_area = Rect::from_min_size(
                Pos2::new(remaining.max.x - right_size, remaining.min.y),
                Vec2::new(right_size, remaining.height()),
            );
            areas.insert(DockSide::Right, right_area);
            remaining.max.x -= right_size;
        }

        // Top dock
        if top_size > 0.0 {
            let top_area =
                Rect::from_min_size(remaining.min, Vec2::new(remaining.width(), top_size));
            areas.insert(DockSide::Top, top_area);
            remaining.min.y += top_size;
        }

        // Bottom dock
        if bottom_size > 0.0 {
            let bottom_area = Rect::from_min_size(
                Pos2::new(remaining.min.x, remaining.max.y - bottom_size),
                Vec2::new(remaining.width(), bottom_size),
            );
            areas.insert(DockSide::Bottom, bottom_area);
            remaining.max.y -= bottom_size;
        }

        // Center area gets the rest
        if remaining.width() > 0.0 && remaining.height() > 0.0 {
            areas.insert(DockSide::Center, remaining);
        }

        areas
    }

    fn draw_docked_panels(
        &mut self,
        ui: &mut Ui,
        area: Rect,
        dock_side: DockSide,
    ) -> HashMap<String, Rect> {
        let mut panel_rects = HashMap::new();
        let panels: Vec<_> = self
            .state
            .panels
            .iter()
            .filter(|(_, panel)| panel.dock_side == dock_side && panel.visible)
            .map(|(id, _)| id.clone())
            .collect();

        if panels.is_empty() {
            return panel_rects;
        }

        // Calculate layout for panels in this dock area
        let is_horizontal = matches!(dock_side, DockSide::Top | DockSide::Bottom);
        let weights: Vec<f32> = panels
            .iter()
            .filter_map(|id| self.state.panels.get(id).map(|p| p.weight))
            .collect();

        let sizes = if is_horizontal {
            LayoutUtils::distribute_space(area.width(), &weights, 4.0)
        } else {
            LayoutUtils::distribute_space(area.height(), &weights, 4.0)
        };

        let mut current_pos = area.min;

        // Clone panel data to avoid borrow checker issues
        let panel_data: Vec<_> = panels
            .iter()
            .enumerate()
            .filter_map(|(i, panel_id)| {
                self.state
                    .panels
                    .get(panel_id)
                    .map(|panel| (i, panel_id.clone(), panel.clone()))
            })
            .collect();

        for (i, panel_id, panel) in panel_data {
            let size = if i < sizes.len() {
                if is_horizontal {
                    Vec2::new(sizes[i], area.height())
                } else {
                    Vec2::new(area.width(), sizes[i])
                }
            } else {
                continue;
            };

            let panel_rect = Rect::from_min_size(current_pos, size);

            // Draw the panel
            let actual_rect = self.draw_panel(ui, panel_rect, &panel_id, &panel);
            panel_rects.insert(panel_id.clone(), actual_rect);

            // Update position for next panel
            if is_horizontal {
                current_pos.x += size.x + 4.0;
            } else {
                current_pos.y += size.y + 4.0;
            }
        }

        panel_rects
    }

    fn draw_floating_panel(&mut self, ui: &mut Ui, panel_id: &str) -> Option<Rect> {
        if let Some(panel) = self.state.panels.get(panel_id).cloned() {
            if !panel.visible {
                return None;
            }

            // Floating panels are drawn as windows
            let mut open = panel.visible;
            let window_response = egui::Window::new(&panel.title)
                .id(Id::new(format!("floating_{}", panel_id)))
                .open(&mut open)
                .resizable(panel.resizable != ResizeDirection::None)
                .collapsible(panel.collapsible)
                .default_size(panel.size)
                .min_size(panel.min_size)
                .max_size(panel.max_size)
                .show(ui.ctx(), |ui| {
                    // Panel content will be drawn by the caller
                });

            // Update panel visibility if window was closed
            if !open {
                if let Some(panel_mut) = self.state.panels.get_mut(panel_id) {
                    panel_mut.visible = false;
                }
            }

            window_response.map(|r| r.response.rect)
        } else {
            None
        }
    }

    fn draw_panel(&mut self, ui: &mut Ui, rect: Rect, panel_id: &str, panel: &PanelConfig) -> Rect {
        let animation_factor = self
            .animations
            .get(panel_id)
            .map(|a| a.value())
            .unwrap_or(1.0);

        let actual_height = if panel.collapsed {
            30.0 // Just enough for the title bar
        } else {
            rect.height() * animation_factor
        };

        let actual_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), actual_height));

        ui.allocate_ui_at_rect(actual_rect, |ui| {
            egui::Frame::none()
                .fill(ui.visuals().panel_fill)
                .stroke(Stroke::new(
                    1.0,
                    ui.visuals().widgets.noninteractive.bg_stroke.color,
                ))
                .rounding(ui.visuals().widgets.noninteractive.rounding())
                .inner_margin(egui::Margin::same(4))
                .show(ui, |ui| {
                    // Panel header
                    ui.horizontal(|ui| {
                        ui.label(&panel.title);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if panel.closable {
                                if ui.small_button("✕").clicked() {
                                    // This would need to be handled by the caller
                                }
                            }

                            if panel.collapsible {
                                let collapse_button_text =
                                    if panel.collapsed { "▼" } else { "▲" };
                                if ui.small_button(collapse_button_text).clicked() {
                                    // This would need to be handled by the caller
                                }
                            }
                        });
                    });

                    if !panel.collapsed {
                        ui.separator();
                        // Panel content area - this is where the actual content would be drawn
                        let content_rect = ui.available_rect_before_wrap();
                        ui.allocate_rect(content_rect, Sense::hover());
                    }
                });
        });

        // Draw resize handle if resizable
        if panel.resizable != ResizeDirection::None {
            self.draw_resize_handle(ui, actual_rect, panel_id, panel.resizable.clone());
        }

        actual_rect
    }

    fn draw_resize_handle(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        panel_id: &str,
        resize_dir: ResizeDirection,
    ) {
        let handle_size = 8.0;
        let handle_rect = match resize_dir {
            ResizeDirection::Horizontal => Rect::from_min_size(
                Pos2::new(rect.max.x - handle_size * 0.5, rect.min.y),
                Vec2::new(handle_size, rect.height()),
            ),
            ResizeDirection::Vertical => Rect::from_min_size(
                Pos2::new(rect.min.x, rect.max.y - handle_size * 0.5),
                Vec2::new(rect.width(), handle_size),
            ),
            ResizeDirection::Both => Rect::from_min_size(
                Pos2::new(rect.max.x - handle_size, rect.max.y - handle_size),
                Vec2::new(handle_size, handle_size),
            ),
            ResizeDirection::None => return,
        };

        let response = ui.allocate_rect(handle_rect, Sense::drag());

        if response.hovered() || response.dragged() {
            let cursor = match resize_dir {
                ResizeDirection::Horizontal => egui::CursorIcon::ResizeHorizontal,
                ResizeDirection::Vertical => egui::CursorIcon::ResizeVertical,
                ResizeDirection::Both => egui::CursorIcon::ResizeNwSe,
                ResizeDirection::None => egui::CursorIcon::Default,
            };
            ui.ctx().set_cursor_icon(cursor);
        }

        if response.dragged() {
            self.resize_handles.insert(panel_id.to_string(), true);
            // Handle resizing logic would go here
        } else {
            self.resize_handles.remove(panel_id);
        }

        // Draw visual handle
        let handle_color = if response.hovered() {
            ui.visuals().widgets.hovered.bg_fill
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };

        ui.painter().rect_filled(handle_rect, 2.0, handle_color);
    }

    fn handle_drag_and_drop(&mut self, ui: &mut Ui) {
        // This would implement drag and drop logic for rearranging panels
        // For now, this is a placeholder
        if let Some(_drag_state) = &self.drag_state {
            // Handle ongoing drag
        }
    }

    fn get_dock_size(&self, dock_side: DockSide) -> f32 {
        let panels_in_dock: Vec<_> = self
            .state
            .panels
            .values()
            .filter(|panel| panel.dock_side == dock_side && panel.visible && !panel.collapsed)
            .collect();

        if panels_in_dock.is_empty() {
            return 0.0;
        }

        match dock_side {
            DockSide::Left | DockSide::Right => {
                panels_in_dock.iter().map(|p| p.size.x).fold(0.0, f32::max)
            }
            DockSide::Top | DockSide::Bottom => {
                panels_in_dock.iter().map(|p| p.size.y).fold(0.0, f32::max)
            }
            _ => 0.0,
        }
    }

    fn create_default_layout(&mut self) {
        let playback_panel = PanelConfig {
            panel_type: PanelType::Playback,
            title: "Playback".to_string(),
            dock_side: DockSide::Center,
            size: Vec2::new(400.0, 300.0),
            weight: 2.0,
            ..Default::default()
        };

        let effects_panel = PanelConfig {
            panel_type: PanelType::Effects,
            title: "Effects".to_string(),
            dock_side: DockSide::Right,
            size: Vec2::new(200.0, 200.0),
            weight: 1.0,
            ..Default::default()
        };

        let eq_panel = PanelConfig {
            panel_type: PanelType::Equalizer,
            title: "Equalizer".to_string(),
            dock_side: DockSide::Bottom,
            size: Vec2::new(300.0, 150.0),
            weight: 1.0,
            ..Default::default()
        };

        self.add_panel("playback".to_string(), playback_panel);
        self.add_panel("effects".to_string(), effects_panel);
        self.add_panel("equalizer".to_string(), eq_panel);
    }
}

impl PanelType {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Playback => "Playback",
            Self::Effects => "Effects",
            Self::Equalizer => "Equalizer",
            Self::Playlist => "Playlist",
            Self::Settings => "Settings",
            Self::Spectrum => "Spectrum",
            Self::Custom(name) => name,
        }
    }
}

impl DockSide {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Left,
            Self::Right,
            Self::Top,
            Self::Bottom,
            Self::Center,
            Self::Floating,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
            Self::Center => "Center",
            Self::Floating => "Floating",
        }
    }
}
