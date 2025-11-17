// Modern Docking Layout System for rusty-audio
//
// This module implements a professional DAW-like docking interface using egui_dock.
// Provides flexible panel management with drag-and-drop docking, workspace presets,
// and persistent layout configuration.

use egui::{Context, Ui};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Panel identifiers for the docking system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelId {
    /// File browser and project explorer
    FileBrowser,
    /// Main waveform display
    Waveform,
    /// Spectrum analyzer
    Spectrum,
    /// Signal generator controls
    Generator,
    /// Audio effects rack
    Effects,
    /// Parametric equalizer
    Equalizer,
    /// Property inspector/details panel
    Inspector,
    /// Mixer and levels
    Mixer,
    /// Timeline and transport controls (bottom bar)
    Transport,
    /// Settings and preferences
    Settings,
}

impl PanelId {
    /// Get the display title for this panel
    pub fn title(&self) -> &'static str {
        match self {
            PanelId::FileBrowser => "📁 Files",
            PanelId::Waveform => "🎵 Waveform",
            PanelId::Spectrum => "📊 Spectrum",
            PanelId::Generator => "🔊 Generator",
            PanelId::Effects => "🎛️ Effects",
            PanelId::Equalizer => "⚖️ Equalizer",
            PanelId::Inspector => "🔍 Inspector",
            PanelId::Mixer => "🎚️ Mixer",
            PanelId::Transport => "⏯️ Transport",
            PanelId::Settings => "⚙️ Settings",
        }
    }

    /// Get a unique string identifier for this panel
    pub fn id(&self) -> &'static str {
        match self {
            PanelId::FileBrowser => "file_browser",
            PanelId::Waveform => "waveform",
            PanelId::Spectrum => "spectrum",
            PanelId::Generator => "generator",
            PanelId::Effects => "effects",
            PanelId::Equalizer => "equalizer",
            PanelId::Inspector => "inspector",
            PanelId::Mixer => "mixer",
            PanelId::Transport => "transport",
            PanelId::Settings => "settings",
        }
    }
}

/// Workspace preset configurations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspacePreset {
    /// Default balanced layout
    Default,
    /// Optimized for audio analysis
    Analyzer,
    /// Optimized for signal generation
    Generator,
    /// Optimized for mixing and effects
    Mixing,
    /// Minimal playback interface
    Playback,
    /// Custom user-defined layout
    Custom(String),
}

impl WorkspacePreset {
    pub fn name(&self) -> &str {
        match self {
            WorkspacePreset::Default => "Default",
            WorkspacePreset::Analyzer => "Analyzer",
            WorkspacePreset::Generator => "Generator",
            WorkspacePreset::Mixing => "Mixing",
            WorkspacePreset::Playback => "Playback",
            WorkspacePreset::Custom(name) => name,
        }
    }
}

/// Application state for panel content
pub trait PanelContent {
    /// Render the file browser panel
    fn show_file_browser(&mut self, ui: &mut Ui);

    /// Render the waveform display panel
    fn show_waveform(&mut self, ui: &mut Ui);

    /// Render the spectrum analyzer panel
    fn show_spectrum(&mut self, ui: &mut Ui);

    /// Render the signal generator panel
    fn show_generator(&mut self, ui: &mut Ui);

    /// Render the effects rack panel
    fn show_effects(&mut self, ui: &mut Ui);

    /// Render the equalizer panel
    fn show_equalizer(&mut self, ui: &mut Ui);

    /// Render the inspector/properties panel
    fn show_inspector(&mut self, ui: &mut Ui);

    /// Render the mixer panel
    fn show_mixer(&mut self, ui: &mut Ui);

    /// Render the transport controls panel
    fn show_transport(&mut self, ui: &mut Ui);

    /// Render the settings panel
    fn show_settings(&mut self, ui: &mut Ui);
}

/// Docking layout manager with workspace support
pub struct DockLayoutManager {
    /// Current dock state
    dock_state: DockState<PanelId>,

    /// Saved workspaces
    saved_workspaces: HashMap<String, DockState<PanelId>>,

    /// Currently active workspace
    active_workspace: WorkspacePreset,

    /// Dock area style
    style: Style,
}

impl Default for DockLayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DockLayoutManager {
    /// Create a new dock layout manager with default layout
    pub fn new() -> Self {
        let mut manager = Self {
            dock_state: DockState::new(vec![PanelId::Waveform]),
            saved_workspaces: HashMap::new(),
            active_workspace: WorkspacePreset::Default,
            style: Style::default(),
        };

        manager.apply_preset(WorkspacePreset::Default);
        manager
    }

    /// Apply a workspace preset
    pub fn apply_preset(&mut self, preset: WorkspacePreset) {
        self.dock_state = match preset {
            WorkspacePreset::Default => self.create_default_layout(),
            WorkspacePreset::Analyzer => self.create_analyzer_layout(),
            WorkspacePreset::Generator => self.create_generator_layout(),
            WorkspacePreset::Mixing => self.create_mixing_layout(),
            WorkspacePreset::Playback => self.create_playback_layout(),
            WorkspacePreset::Custom(ref name) => {
                if let Some(saved) = self.saved_workspaces.get(name) {
                    saved.clone()
                } else {
                    self.create_default_layout()
                }
            }
        };

        self.active_workspace = preset;
    }

    /// Create the default balanced layout
    fn create_default_layout(&self) -> DockState<PanelId> {
        let mut state = DockState::new(vec![PanelId::Waveform]);

        // Left sidebar: File browser
        let [_center, left] =
            state
                .main_surface_mut()
                .split_left(NodeIndex::root(), 0.2, vec![PanelId::FileBrowser]);

        // Right sidebar: Inspector
        let [center, _right] =
            state
                .main_surface_mut()
                .split_right(NodeIndex::root(), 0.2, vec![PanelId::Inspector]);

        // Center area: Waveform and Spectrum tabs
        state
            .main_surface_mut()
            .push_to_focused_leaf(PanelId::Spectrum);

        // Bottom area: Transport and Mixer
        let [_main, _bottom] =
            state
                .main_surface_mut()
                .split_below(center, 0.15, vec![PanelId::Transport]);

        state
    }

    /// Create analyzer-focused layout
    fn create_analyzer_layout(&self) -> DockState<PanelId> {
        let mut state = DockState::new(vec![PanelId::Spectrum]);

        // Large spectrum display with waveform tab
        state
            .main_surface_mut()
            .push_to_focused_leaf(PanelId::Waveform);

        // Right panel: Generator and Inspector
        let [_center, _right] = state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.25,
            vec![PanelId::Generator, PanelId::Inspector],
        );

        // Bottom: Transport
        let [_main, _bottom] =
            state
                .main_surface_mut()
                .split_below(NodeIndex::root(), 0.12, vec![PanelId::Transport]);

        state
    }

    /// Create generator-focused layout
    fn create_generator_layout(&self) -> DockState<PanelId> {
        let mut state = DockState::new(vec![PanelId::Generator]);

        // Center: Generator and Waveform
        state
            .main_surface_mut()
            .push_to_focused_leaf(PanelId::Waveform);

        // Right: Spectrum analyzer
        let [_center, _right] =
            state
                .main_surface_mut()
                .split_right(NodeIndex::root(), 0.35, vec![PanelId::Spectrum]);

        // Bottom: Transport
        let [_main, _bottom] =
            state
                .main_surface_mut()
                .split_below(NodeIndex::root(), 0.12, vec![PanelId::Transport]);

        state
    }

    /// Create mixing-focused layout
    fn create_mixing_layout(&self) -> DockState<PanelId> {
        let mut state = DockState::new(vec![PanelId::Equalizer]);

        // Center: EQ and Effects
        state
            .main_surface_mut()
            .push_to_focused_leaf(PanelId::Effects);
        state
            .main_surface_mut()
            .push_to_focused_leaf(PanelId::Waveform);

        // Right: Mixer and Inspector
        let [_center, _right] = state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.25,
            vec![PanelId::Mixer, PanelId::Inspector],
        );

        // Left: Files
        let [_center, _left] =
            state
                .main_surface_mut()
                .split_left(NodeIndex::root(), 0.2, vec![PanelId::FileBrowser]);

        // Bottom: Transport
        let [_main, _bottom] =
            state
                .main_surface_mut()
                .split_below(NodeIndex::root(), 0.12, vec![PanelId::Transport]);

        state
    }

    /// Create minimal playback layout
    fn create_playback_layout(&self) -> DockState<PanelId> {
        let mut state = DockState::new(vec![PanelId::Waveform]);

        // Simple: Waveform and transport
        let [_main, _bottom] =
            state
                .main_surface_mut()
                .split_below(NodeIndex::root(), 0.15, vec![PanelId::Transport]);

        state
    }

    /// Save the current layout as a custom workspace
    pub fn save_workspace(&mut self, name: String) {
        self.saved_workspaces
            .insert(name.clone(), self.dock_state.clone());
        self.active_workspace = WorkspacePreset::Custom(name);
    }

    /// Get a list of saved workspace names
    pub fn list_workspaces(&self) -> Vec<String> {
        self.saved_workspaces.keys().cloned().collect()
    }

    /// Delete a saved workspace
    pub fn delete_workspace(&mut self, name: &str) -> bool {
        self.saved_workspaces.remove(name).is_some()
    }

    /// Get the active workspace name
    pub fn active_workspace(&self) -> &WorkspacePreset {
        &self.active_workspace
    }

    /// Switch to a different workspace preset (alias for apply_preset)
    pub fn switch_workspace(&mut self, preset: WorkspacePreset) {
        self.apply_preset(preset);
    }

    /// Show the dock area with all panels
    pub fn show<T: PanelContent>(&mut self, ctx: &Context, app_state: &mut T) {
        let mut tab_viewer = AppTabViewer { app_state };

        DockArea::new(&mut self.dock_state)
            .style(self.style.clone())
            .show(ctx, &mut tab_viewer);
    }

    /// Update the style based on current egui visuals
    #[allow(unused_variables)]
    pub fn update_style(&mut self, visuals: &egui::Visuals) {
        // For now, keep using default style
        // TODO: Create custom style based on visuals when needed
        self.style = Style::default();
    }
}

/// Tab viewer implementation for panel rendering
struct AppTabViewer<'a, T: PanelContent> {
    app_state: &'a mut T,
}

impl<'a, T: PanelContent> TabViewer for AppTabViewer<'a, T> {
    type Tab = PanelId;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            PanelId::FileBrowser => self.app_state.show_file_browser(ui),
            PanelId::Waveform => self.app_state.show_waveform(ui),
            PanelId::Spectrum => self.app_state.show_spectrum(ui),
            PanelId::Generator => self.app_state.show_generator(ui),
            PanelId::Effects => self.app_state.show_effects(ui),
            PanelId::Equalizer => self.app_state.show_equalizer(ui),
            PanelId::Inspector => self.app_state.show_inspector(ui),
            PanelId::Mixer => self.app_state.show_mixer(ui),
            PanelId::Transport => self.app_state.show_transport(ui),
            PanelId::Settings => self.app_state.show_settings(ui),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_id_uniqueness() {
        let panels = vec![
            PanelId::FileBrowser,
            PanelId::Waveform,
            PanelId::Spectrum,
            PanelId::Generator,
            PanelId::Effects,
            PanelId::Equalizer,
            PanelId::Inspector,
            PanelId::Mixer,
            PanelId::Transport,
            PanelId::Settings,
        ];

        let ids: Vec<&str> = panels.iter().map(|p| p.id()).collect();
        let unique_ids: std::collections::HashSet<&str> = ids.iter().copied().collect();

        assert_eq!(ids.len(), unique_ids.len(), "Panel IDs must be unique");
    }

    #[test]
    fn test_workspace_presets() {
        let manager = DockLayoutManager::new();
        assert_eq!(manager.active_workspace(), &WorkspacePreset::Default);
    }
}
