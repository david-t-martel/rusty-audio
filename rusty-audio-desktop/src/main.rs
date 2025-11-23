use eframe::{egui, NativeOptions};
use egui::{load::SizedTexture, Color32, Layout, RichText, Vec2};
use image::GenericImageView;
use lofty::{file::TaggedFileExt, tag::Accessor};

// Platform-specific imports
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileHandle;

use std::sync::Arc;
use std::time::{Duration, Instant};

// Audio context - platform specific
#[cfg(not(target_arch = "wasm32"))]
use web_audio_api::context::{AudioContext, BaseAudioContext};
#[cfg(not(target_arch = "wasm32"))]
use web_audio_api::node::{
    AnalyserNode, AudioNode, AudioScheduledSourceNode, BiquadFilterNode, BiquadFilterType,
};

// Import hybrid audio backend (native only for now)
#[cfg(not(target_arch = "wasm32"))]
use rusty_audio_core::audio::{
    AudioConfig, AudioDeviceManager, BackendHealth, FallbackPolicy, HybridAudioBackend, HybridMode,
    StreamDirection, WebAudioBridge, WebAudioBridgeConfig,
};

// Use library modules instead of declaring them locally
use rusty_audio_core::{audio_performance, platform, testing, ui};

// Async components (native only)
#[cfg(not(target_arch = "wasm32"))]
use rusty_audio_core::async_audio_loader::{AsyncAudioLoader, AsyncLoadConfig};

mod panel_implementation;

use testing::signal_generators::*;
use ui::{
    accessibility::{AccessibilityAction, AccessibilityManager},
    components::{AlbumArtDisplay, MetadataDisplay, MetadataLayout, ProgressBar, ProgressBarStyle},
    controls::{ButtonStyle, CircularKnob, EnhancedButton},
    dock_layout::{DockLayoutManager, PanelContent, PanelId},
    enhanced_button::{AccessibleButton, ProgressIndicator, VolumeSafetyIndicator},
    enhanced_controls::{AccessibleKnob, AccessibleSlider},
    error_handling::{ErrorManager, RecoveryActionType},
    layout::{DockSide, LayoutManager, PanelConfig, PanelType},
    recording_panel::RecordingPanel,
    signal_generator::{GeneratorRoutingMode, GeneratorState, SignalGeneratorPanel},
    spectrum::{SpectrumMode, SpectrumVisualizer, SpectrumVisualizerConfig},
    theme::{Theme, ThemeColors, ThemeManager},
    utils::{ColorUtils, ScreenSize},
};

// ============================================================================
// Type Definitions
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum Tab {
    Playback,
    Effects,
    Eq,
    Generator,
    Recording,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone)]
struct TrackMetadata {
    title: String,
    artist: String,
    album: String,
    year: String,
}

const WAVEFORM_PREVIEW_SAMPLES: usize = 1024;

// ============================================================================
// Native Application (Desktop)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
struct AudioPlayerApp {
    // Audio Engine Abstraction (replaces 12 audio fields)
    audio_engine: Box<dyn rusty_audio_core::audio_engine::AudioEngineInterface>,

    // Playback state (kept in UI for responsiveness)
    playback_state: PlaybackState,
    current_file: Option<Arc<FileHandle>>,
    metadata: Option<TrackMetadata>,
    volume: f32,
    panning: f32,
    is_looping: bool,
    playback_pos: Duration,
    total_duration: Duration,
    is_seeking: bool,
    error: Option<String>,
    album_art: Option<Arc<egui::TextureHandle>>,
    active_tab: Tab,

    // Signal generator state
    signal_generator_panel: SignalGeneratorPanel,

    // Enhanced UI components
    theme_manager: ThemeManager,
    layout_manager: LayoutManager,
    spectrum_visualizer: SpectrumVisualizer,
    album_art_display: AlbumArtDisplay,
    progress_bar: ProgressBar,
    metadata_display: MetadataDisplay,
    waveform_preview: Vec<f32>,
    waveform_dirty: bool,

    // Enhanced controls (legacy - to be replaced)
    _eq_knobs: Vec<CircularKnob>,

    // Accessibility and enhanced controls
    accessibility_manager: AccessibilityManager,
    accessible_volume_slider: AccessibleSlider,
    accessible_eq_knobs: Vec<AccessibleKnob>,
    _file_loading_progress: Option<ProgressIndicator>,
    volume_safety_indicator: VolumeSafetyIndicator,
    error_manager: ErrorManager,

    // Responsive and animation state
    last_frame_time: Instant,
    screen_size: ScreenSize,
    show_keyboard_shortcuts: bool,

    // Dock layout system (Phase 2.1)
    dock_layout_manager: DockLayoutManager,
    enable_dock_layout: bool,

    // Phase 3.1: Hybrid audio backend (TODO: Move into AudioEngine)
    audio_backend: Option<HybridAudioBackend>,
    device_manager: Option<AudioDeviceManager>,
    // web_audio_bridge: Option<WebAudioBridge>, // Replaced by script processor
    script_processor: Option<web_audio_api::node::ScriptProcessorNode>,
    _audio_mode_switching: bool, // Animation state for mode changes
    _last_latency_check: Instant,
    audio_status_message: Option<(String, Instant)>, // (message, timestamp)

    // Phase 3.2: Recording
    recording_panel: RecordingPanel,

    // Phase 1.4: Async file loading
    _async_loader: AsyncAudioLoader,
    _tokio_runtime: Arc<tokio::runtime::Runtime>,
    load_progress: Option<f32>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for AudioPlayerApp {
    fn default() -> Self {
        // Initialize audio engine (replaces ~50 lines of manual setup)
        let audio_engine = Box::new(rusty_audio_core::audio_engine::WebAudioEngine::default())
            as Box<dyn rusty_audio_core::audio_engine::AudioEngineInterface>;

        // Create UI controls for EQ (8 bands)
        let mut eq_knobs = Vec::new();
        let mut accessible_eq_knobs = Vec::new();

        for i in 0..8 {
            // Create corresponding knob controls (legacy and accessible)
            eq_knobs.push(CircularKnob::new(0.0, -40.0..=40.0).radius(20.0));

            let freq = 60.0 * 2.0_f32.powi(i);
            let freq_label = if freq < 1000.0 {
                format!("{:.0} Hz", freq)
            } else {
                format!("{:.1}k Hz", freq / 1000.0)
            };

            accessible_eq_knobs.push(
                AccessibleKnob::new(
                    egui::Id::new(format!("eq_band_{}", i)),
                    0.0,
                    -40.0..=40.0,
                    freq_label,
                )
                .description(format!(
                    "Equalizer gain for {} frequency band",
                    if freq < 1000.0 {
                        format!("{:.0} Hz", freq)
                    } else {
                        format!("{:.1} kHz", freq / 1000.0)
                    }
                ))
                .step_size(0.5),
            );
        }

        Self {
            // Audio Engine (replaces 12 audio fields)
            audio_engine,

            // Playback state
            playback_state: PlaybackState::Stopped,
            current_file: None,
            metadata: None,
            volume: 0.5,
            panning: 0.5,
            is_looping: false,
            playback_pos: Duration::ZERO,
            total_duration: Duration::ZERO,
            is_seeking: false,
            error: None,
            album_art: None,
            active_tab: Tab::Playback,

            // Signal generator
            signal_generator_panel: SignalGeneratorPanel::new(),

            // Enhanced UI components
            theme_manager: ThemeManager::new(Theme::StudioDark),
            layout_manager: LayoutManager::new(),
            spectrum_visualizer: SpectrumVisualizer::new(SpectrumVisualizerConfig::default()),
            album_art_display: AlbumArtDisplay::new(Vec2::new(200.0, 200.0)),
            progress_bar: ProgressBar::new(),
            metadata_display: MetadataDisplay::new(),
            waveform_preview: Vec::new(),
            waveform_dirty: false,

            _eq_knobs: eq_knobs,

            // Accessibility and enhanced controls
            accessibility_manager: AccessibilityManager::new(),
            accessible_volume_slider: AccessibleSlider::new(
                egui::Id::new("volume_slider"),
                0.5,
                0.0..=1.0,
                "Volume",
            )
            .description("Master audio volume control")
            .safety_info("Keep volume below 80% to protect hearing")
            .step_size(0.05),
            accessible_eq_knobs,
            _file_loading_progress: None,
            volume_safety_indicator: VolumeSafetyIndicator::new(),
            error_manager: ErrorManager::new(),

            // Responsive and animation state
            last_frame_time: Instant::now(),
            screen_size: ScreenSize::Desktop,
            show_keyboard_shortcuts: false,

            // Dock layout system
            dock_layout_manager: DockLayoutManager::new(),
            enable_dock_layout: false, // Start with traditional layout, can be toggled

            // Phase 3.1: Hybrid audio backend (TODO: integrate into AudioEngine)
            audio_backend: {
                let mut backend = HybridAudioBackend::new();
                match backend.initialize() {
                    Ok(_) => Some(backend),
                    Err(e) => {
                        eprintln!("Warning: Failed to initialize hybrid audio backend: {}", e);
                        None
                    }
                }
            },
            device_manager: match AudioDeviceManager::new() {
                Ok(dm) => Some(dm),
                Err(e) => {
                    eprintln!("Warning: Failed to initialize device manager: {}", e);
                    None
                }
            },
            script_processor: None,
            _audio_mode_switching: false,
            _last_latency_check: Instant::now(),
            audio_status_message: None,

            // Phase 3.2: Recording
            recording_panel: RecordingPanel::new(),

            // Phase 1.4: Async file loading
            _async_loader: AsyncAudioLoader::new(AsyncLoadConfig::default()),
            _tokio_runtime: Self::build_async_runtime(),
            load_progress: None,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl eframe::App for AudioPlayerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update responsive layout and timing
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Update screen size and responsive layout
        let screen_size_vec = ctx.input(|i| i.screen_rect()).size();
        self.screen_size = ScreenSize::from_width(screen_size_vec.x);
        self.layout_manager
            .update_responsive_layout(screen_size_vec);

        // Apply theme (with accessibility enhancements)
        self.theme_manager.apply_theme(ctx);
        let base_colors = self.theme_manager.get_colors();
        let colors = self
            .accessibility_manager
            .get_accessible_colors(&base_colors);

        // Update animations
        self.layout_manager.update_animations(dt);

        // Update accessibility system
        let ui_builder = egui::UiBuilder::new().max_rect(egui::Rect::EVERYTHING);
        self.accessibility_manager.update(
            &egui::Ui::new(ctx.clone(), egui::Id::new("accessibility_ui"), ui_builder),
            dt,
        );

        // Handle accessibility actions
        let ui_builder = egui::UiBuilder::new().max_rect(egui::Rect::EVERYTHING);
        let accessibility_action =
            self.accessibility_manager
                .handle_keyboard_input(&egui::Ui::new(
                    ctx.clone(),
                    egui::Id::new("accessibility_input"),
                    ui_builder,
                ));

        match accessibility_action {
            AccessibilityAction::EmergencyVolumeReduction => {
                let _emergency_volume = self.accessibility_manager.get_volume_safety_status();
                self.volume = 0.2; // Emergency volume level
                self.audio_engine.set_volume(self.volume);
                self.accessible_volume_slider.set_value(self.volume);
                self.accessibility_manager.announce(
                    "Emergency volume reduction activated".to_string(),
                    ui::accessibility::AnnouncementPriority::Critical,
                );
            }
            AccessibilityAction::ToggleHelp => {
                // Help system is handled within accessibility manager
            }
            AccessibilityAction::ToggleHighContrast => {
                self.accessibility_manager.announce(
                    format!(
                        "High contrast mode {}",
                        if self.accessibility_manager.is_high_contrast_mode() {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    ),
                    ui::accessibility::AnnouncementPriority::Medium,
                );
            }
            AccessibilityAction::AdjustFocusedControl(_delta) => {
                // This would be handled by the focused control itself
            }
            _ => {}
        }

        // Handle keyboard shortcuts (enhanced version)
        self.handle_keyboard_input(ctx);

        // Update audio processing
        self.tick();

        // Update volume safety indicator
        self.volume_safety_indicator.update_volume(self.volume);

        // Update error manager
        self.error_manager.update(dt);

        // Update UI components
        self.update_ui_components(&colors);
        self.handle_signal_generator_routing();

        // Update signal generator
        self.signal_generator_panel.update(dt);

        // Main UI layout - choose between dock layout and traditional layout
        if self.enable_dock_layout && self.screen_size != ScreenSize::Mobile {
            self.draw_dock_layout(ctx, &colors);
        } else if self.screen_size == ScreenSize::Mobile {
            self.draw_mobile_layout(ctx, &colors);
        } else {
            self.draw_desktop_layout(ctx, &colors);
        }

        // Show accessibility help overlay
        let ui_builder = egui::UiBuilder::new().max_rect(egui::Rect::EVERYTHING);
        self.accessibility_manager.show_help_overlay(
            &egui::Ui::new(ctx.clone(), egui::Id::new("help_overlay"), ui_builder),
            &colors,
        );

        // Show error dialogs and handle recovery actions
        let ui_builder = egui::UiBuilder::new().max_rect(egui::Rect::EVERYTHING);
        let recovery_actions = self.error_manager.show_errors(
            &mut egui::Ui::new(ctx.clone(), egui::Id::new("error_display"), ui_builder),
            &colors,
            &mut self.accessibility_manager,
        );

        // Execute recovery actions
        for action in recovery_actions {
            match action {
                RecoveryActionType::Retry => {
                    if self.current_file.is_some() {
                        self.load_current_file();
                    }
                }
                RecoveryActionType::SelectDifferentFile => {
                    self.open_file_dialog();
                }
                RecoveryActionType::ResetSettings => {
                    self.reset_all_settings();
                }
                RecoveryActionType::CheckPermissions => {
                    self.accessibility_manager.announce(
                        "Please check that the file is accessible and not in use by another application".to_string(),
                        ui::accessibility::AnnouncementPriority::High,
                    );
                }
                RecoveryActionType::ContactSupport => {
                    self.show_format_help();
                }
                RecoveryActionType::Dismiss => {
                    // Already handled by error manager
                }
            }
        }

        // Show keyboard shortcuts overlay if requested (legacy)
        if self.show_keyboard_shortcuts {
            self.draw_keyboard_shortcuts_overlay(ctx, &colors);
        }

        // Request repaint for animations
        ctx.request_repaint_after(Duration::from_millis(16)); // ~60 FPS
    }
}

impl AudioPlayerApp {
    #[cfg(not(target_arch = "wasm32"))]
    fn build_async_runtime() -> Arc<tokio::runtime::Runtime> {
        match tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("rusty-audio-async")
            .enable_all()
            .build()
        {
            Ok(runtime) => Arc::new(runtime),
            Err(err) => {
                eprintln!("Failed to create multithreaded runtime: {}", err);
                match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => Arc::new(runtime),
                    Err(fallback_err) => {
                        eprintln!(
                            "Failed to create fallback runtime: {}. Exiting.",
                            fallback_err
                        );
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    fn update_ui_components(&mut self, _colors: &ThemeColors) {
        // Update progress bar
        self.progress_bar.set_progress(
            self.playback_pos.as_secs_f32(),
            self.total_duration.as_secs_f32(),
        );
        // TODO: Re-enable waveform preview once ProgressBar.set_waveform is implemented
        // if self.waveform_dirty {
        //     self.progress_bar
        //         .set_waveform(self.waveform_preview.clone());
        //     self.waveform_dirty = false;
        // }

        // Update metadata display
        if let Some(metadata) = &self.metadata {
            self.metadata_display.set_metadata(
                metadata.title.clone(),
                metadata.artist.clone(),
                metadata.album.clone(),
                metadata.year.clone(),
            );
        }

        // Update album art display
        self.album_art_display
            .set_texture(self.album_art.as_ref().map(|arc| (**arc).clone()));

        // Update spectrum visualizer with data from audio engine
        let spectrum_data = self.audio_engine.get_spectrum();
        self.spectrum_visualizer.update(&spectrum_data);
    }

    fn handle_signal_generator_routing(&mut self) {
        if let Some(intent) = self.signal_generator_panel.take_route_intent() {
            let Some(output) = self.signal_generator_panel.output_snapshot() else {
                self.error = Some("Generate a signal before routing it.".to_string());
                return;
            };

            match intent.mode {
                GeneratorRoutingMode::Recorder => {
                    self.recording_panel.log_generated_take(
                        intent.label.clone(),
                        &output.samples,
                        output.sample_rate,
                        output.channels,
                    );
                    self.audio_status_message = Some((
                        format!("Saved {} as a virtual take", intent.label),
                        Instant::now(),
                    ));
                }
                _ => {
                    if let Err(error) = self.audio_engine.audition_buffer(
                        &output.samples,
                        output.sample_rate,
                        output.channels,
                    ) {
                        self.error =
                            Some(format!("Failed to route generator through deck: {}", error));
                    } else {
                        self.audio_status_message =
                            Some((format!("Routing {}", intent.label), Instant::now()));
                    }
                }
            }
        }
    }

    fn draw_desktop_layout(&mut self, ctx: &egui::Context, colors: &ThemeColors) {
        // Configure for landscape HiDPI layout
        let available_space = ctx.input(|i| i.screen_rect()).size();
        let _is_landscape = available_space.x > available_space.y;

        // Top panel with optimized height for landscape
        egui::TopBottomPanel::top("header")
            .exact_height(60.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.heading(egui::RichText::new("🎵 Rusty Audio - Car Stereo Style").size(18.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Theme selector with better HiDPI sizing
                        egui::ComboBox::from_label("")
                            .selected_text(self.theme_manager.current_theme().display_name())
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                for theme in Theme::all() {
                                    ui.selectable_value(
                                        &mut self.theme_manager,
                                        ThemeManager::new(theme.clone()),
                                        theme.display_name(),
                                    );
                                }
                            });

                        // Layout toggle button
                        let layout_text = if self.enable_dock_layout {
                            "📑"
                        } else {
                            "📑"
                        };
                        if ui
                            .add_sized([40.0, 30.0], egui::Button::new(layout_text))
                            .on_hover_text("Toggle Dock Layout")
                            .clicked()
                        {
                            self.enable_dock_layout = !self.enable_dock_layout;
                        }

                        if ui.add_sized([40.0, 30.0], egui::Button::new("?")).clicked() {
                            self.show_keyboard_shortcuts = !self.show_keyboard_shortcuts;
                        }
                    });
                });
            });

        self.draw_transport_panel(ctx, colors);

        // Tab panel - horizontal layout for landscape optimization
        egui::TopBottomPanel::top("tabs")
            .exact_height(50.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let tab_button_size = [120.0, 35.0];

                    if ui
                        .add_sized(
                            tab_button_size,
                            egui::Button::new("🎵 Playback")
                                .selected(self.active_tab == Tab::Playback),
                        )
                        .clicked()
                    {
                        self.active_tab = Tab::Playback;
                    }
                    ui.separator();

                    if ui
                        .add_sized(
                            tab_button_size,
                            egui::Button::new("🎛️ Effects")
                                .selected(self.active_tab == Tab::Effects),
                        )
                        .clicked()
                    {
                        self.active_tab = Tab::Effects;
                    }
                    ui.separator();

                    if ui
                        .add_sized(
                            tab_button_size,
                            egui::Button::new("📊 EQ").selected(self.active_tab == Tab::Eq),
                        )
                        .clicked()
                    {
                        self.active_tab = Tab::Eq;
                    }
                    ui.separator();

                    if ui
                        .add_sized(
                            tab_button_size,
                            egui::Button::new("🎛️ Generator")
                                .selected(self.active_tab == Tab::Generator),
                        )
                        .clicked()
                    {
                        self.active_tab = Tab::Generator;
                    }
                    ui.separator();

                    if ui
                        .add_sized(
                            tab_button_size,
                            egui::Button::new("🎙️ Recording")
                                .selected(self.active_tab == Tab::Recording),
                        )
                        .clicked()
                    {
                        self.active_tab = Tab::Recording;
                    }
                    ui.separator();

                    if ui
                        .add_sized(
                            tab_button_size,
                            egui::Button::new("⚙️ Settings")
                                .selected(self.active_tab == Tab::Settings),
                        )
                        .clicked()
                    {
                        self.active_tab = Tab::Settings;
                    }
                });
            });

        // Main content area - optimized for landscape
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_width(available_space.x)
                .show(ui, |ui| {
                    ui.set_width(available_space.x - 20.0); // Account for margins

                    match self.active_tab {
                        Tab::Playback => self.draw_playback_panel(ui, colors),
                        Tab::Effects => self.draw_effects_panel(ui, colors),
                        Tab::Eq => self.draw_eq_panel(ui, colors),
                        Tab::Generator => self.draw_generator_panel(ui, colors),
                        Tab::Recording => self.draw_recording_panel(ui, colors),
                        Tab::Settings => self.draw_settings_panel_main(ui, colors),
                    }
                });
        });
    }

    fn draw_mobile_layout(&mut self, ctx: &egui::Context, colors: &ThemeColors) {
        self.draw_transport_panel(ctx, colors);
        // Mobile layout with bottom tab bar
        egui::TopBottomPanel::bottom("mobile_tabs").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Playback, "🎵");
                ui.selectable_value(&mut self.active_tab, Tab::Effects, "🎛️");
                ui.selectable_value(&mut self.active_tab, Tab::Eq, "📊");
                ui.selectable_value(&mut self.active_tab, Tab::Generator, "🎛️");
                ui.selectable_value(&mut self.active_tab, Tab::Recording, "🎙️");
                ui.selectable_value(&mut self.active_tab, Tab::Settings, "⚙️");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.active_tab {
            Tab::Playback => self.draw_mobile_playback_panel(ui, colors),
            Tab::Effects => self.draw_mobile_effects_panel(ui, colors),
            Tab::Eq => self.draw_mobile_eq_panel(ui, colors),
            Tab::Generator => self.draw_mobile_generator_panel(ui, colors),
            Tab::Recording => self.draw_recording_panel(ui, colors),
            Tab::Settings => self.draw_settings_panel_main(ui, colors),
        });
    }

    fn draw_playback_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        // Landscape-optimized layout - side-by-side content
        ui.horizontal(|ui| {
            // Left side: Album art and metadata (1/3 of width)
            ui.allocate_ui(
                egui::Vec2::new(ui.available_width() * 0.35, ui.available_height()),
                |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);

                        // Album art with enhanced display - smaller for landscape
                        let _album_art_response = self.album_art_display.show(ui, colors);

                        ui.add_space(10.0);

                        // Metadata display
                        self.metadata_display.show(ui, colors);
                    });
                },
            );

            ui.separator();

            // Right side: Controls and progress (2/3 of width)
            ui.allocate_ui(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);

                        // Enhanced progress bar - wider for landscape
                        ui.allocate_ui(egui::Vec2::new(ui.available_width() * 0.9, 40.0), |ui| {
                            let progress_response = self.progress_bar.show(ui, colors);
                            if progress_response.changed() {
                                let position_seconds =
                                    self.progress_bar.progress * self.total_duration.as_secs_f32();
                                self.seek_to_position_main(position_seconds);
                            }
                        });

                        ui.add_space(16.0);
                        ui.group(|ui| {
                            ui.label(RichText::new("Session Overview").strong());
                            if let Some(metadata) = &self.metadata {
                                ui.label(
                                    RichText::new(format!(
                                        "{} • {}",
                                        metadata.album, metadata.year
                                    ))
                                    .color(colors.text_secondary),
                                );
                            } else {
                                ui.label(
                                    RichText::new("Load a track to populate the overview.")
                                        .color(colors.text_secondary),
                                );
                            }
                            ui.label(
                                RichText::new(
                                    "Transport and volume controls live in the global dock above.",
                                )
                                .color(colors.text_secondary),
                            );
                        });
                    });
                },
            );

            // Error display
            if let Some(error) = &self.error {
                ui.add_space(10.0);
                ui.label(RichText::new(error).color(colors.error));
            }
        });
    }

    fn draw_mobile_playback_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        ui.vertical_centered(|ui| {
            // Compact album art
            self.album_art_display.size = Vec2::new(150.0, 150.0);
            self.album_art_display.show(ui, colors);

            ui.add_space(8.0);

            // Compact metadata
            self.metadata_display.layout = MetadataLayout::Compact;
            self.metadata_display.show(ui, colors);

            ui.add_space(10.0);

            // Progress bar
            let progress_response = self.progress_bar.show(ui, colors);
            if progress_response.changed() {
                let position_seconds =
                    self.progress_bar.progress * self.total_duration.as_secs_f32();
                self.seek_to_position_main(position_seconds);
            }

            ui.add_space(10.0);

            ui.label(
                RichText::new("Transport, volume, and recording live in the global dock above.")
                    .color(colors.text_secondary),
            );
        });
    }

    fn draw_effects_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("🎛️ Audio Effects & Spectrum").color(colors.text));
            ui.add_space(10.0);

            // Enhanced spectrum visualizer
            ui.group(|ui| {
                ui.label(RichText::new("Spectrum Analyzer").color(colors.text));
                ui.add_space(5.0);
                let spectrum_rect = ui.available_rect_before_wrap();
                self.spectrum_visualizer.draw(ui, spectrum_rect, colors);
            });

            ui.add_space(15.0);

            // Spectrum mode selection
            ui.horizontal(|ui| {
                ui.label("Mode:");
                let current_mode = self.spectrum_visualizer.config().mode.clone();
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", current_mode))
                    .show_ui(ui, |ui| {
                        let mut new_mode = current_mode.clone();
                        ui.selectable_value(&mut new_mode, SpectrumMode::Bars, "Bars");
                        ui.selectable_value(&mut new_mode, SpectrumMode::Line, "Line");
                        ui.selectable_value(&mut new_mode, SpectrumMode::Filled, "Filled");
                        ui.selectable_value(&mut new_mode, SpectrumMode::Circular, "Circular");
                        if new_mode != current_mode {
                            self.spectrum_visualizer.config_mut().mode = new_mode;
                        }
                    });
            });

            ui.add_space(15.0);

            // Audio effects controls placeholder
            ui.group(|ui| {
                ui.label(RichText::new("Audio Effects").color(colors.text));
                ui.add_space(5.0);
                ui.label("Additional audio effects will be implemented here");

                // Placeholder for future effects
                ui.horizontal(|ui| {
                    ui.checkbox(&mut false, "Reverb");
                    ui.checkbox(&mut false, "Chorus");
                    ui.checkbox(&mut false, "Delay");
                });
            });
        });
    }

    fn draw_eq_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("📊 Equalizer").color(colors.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if EnhancedButton::new("Reset All")
                        .style(ButtonStyle::default())
                        .show(ui, colors)
                        .clicked()
                    {
                        for (i, knob) in self.accessible_eq_knobs.iter_mut().enumerate() {
                            if let Err(e) = self.audio_engine.set_eq_band(i, 0.0) {
                                self.error = Some(format!("EQ reset failed: {}", e));
                            }
                            knob.set_value(0.0);
                        }
                        self.accessibility_manager.announce(
                            "Equalizer reset to flat response".to_string(),
                            ui::accessibility::AnnouncementPriority::Medium,
                        );
                    }
                });
            });

            ui.add_space(15.0);

            // EQ bands with accessible knobs
            ui.horizontal(|ui| {
                let eq_knobs_len = self.accessible_eq_knobs.len();
                for (i, knob) in self.accessible_eq_knobs.iter_mut().enumerate() {
                    ui.vertical(|ui| {
                        // Frequency label
                        let freq = 60.0 * 2.0_f32.powi(i as i32);
                        let freq_label = if freq < 1000.0 {
                            format!("{:.0} Hz", freq)
                        } else {
                            format!("{:.1}k Hz", freq / 1000.0)
                        };
                        ui.label(RichText::new(&freq_label).color(colors.text).size(12.0));

                        // Accessible EQ knob
                        let knob_response = knob.show(ui, colors, &mut self.accessibility_manager);
                        if knob_response.changed() {
                            let gain_value = knob.value();
                            if let Err(e) = self.audio_engine.set_eq_band(i, gain_value) {
                                self.error = Some(format!("EQ band {} update failed: {}", i, e));
                            }

                            // Announce EQ changes for accessibility
                            self.accessibility_manager.announce(
                                format!("{} set to {:.1} dB", freq_label, gain_value),
                                ui::accessibility::AnnouncementPriority::Low,
                            );
                        }

                        // Gain value display
                        ui.label(
                            RichText::new(format!("{:.1}dB", knob.value()))
                                .color(colors.text_secondary)
                                .size(10.0),
                        );
                    });

                    if i < eq_knobs_len - 1 {
                        ui.add_space(5.0);
                    }
                }
            });

            ui.add_space(20.0);

            // Master gain control
            ui.horizontal(|ui| {
                ui.label(RichText::new("Master Gain:").color(colors.text));
                ui.add_space(10.0);

                let mut master_gain = self.audio_engine.get_volume();
                if ui
                    .add(
                        egui::Slider::new(&mut master_gain, 0.0..=2.0)
                            .text("dB")
                            .clamp_to_range(true),
                    )
                    .changed()
                {
                    self.audio_engine.set_volume(master_gain);
                    self.accessibility_manager.announce(
                        format!("Master gain set to {:.1}", master_gain),
                        ui::accessibility::AnnouncementPriority::Low,
                    );
                }
            });
        });
    }

    fn draw_mobile_effects_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("🎛️ Effects").size(18.0).color(colors.text));
            ui.add_space(10.0);

            // Compact spectrum visualizer
            let spectrum_rect = ui.available_rect_before_wrap();
            self.spectrum_visualizer.draw(ui, spectrum_rect, colors);

            ui.add_space(10.0);

            // Compact mode selection
            ui.horizontal_centered(|ui| {
                let current_mode = self.spectrum_visualizer.config().mode.clone();
                egui::ComboBox::from_label("Mode")
                    .selected_text(format!("{:?}", current_mode))
                    .show_ui(ui, |ui| {
                        let mut new_mode = current_mode.clone();
                        ui.selectable_value(&mut new_mode, SpectrumMode::Bars, "Bars");
                        ui.selectable_value(&mut new_mode, SpectrumMode::Line, "Line");
                        ui.selectable_value(&mut new_mode, SpectrumMode::Filled, "Filled");
                        ui.selectable_value(&mut new_mode, SpectrumMode::Circular, "Circular");
                        if new_mode != current_mode {
                            self.spectrum_visualizer.config_mut().mode = new_mode;
                        }
                    });
            });
        });
    }

    fn draw_mobile_eq_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        ui.vertical_centered(|ui| {
            ui.horizontal_centered(|ui| {
                ui.label(RichText::new("📊 EQ").size(18.0).color(colors.text));
                ui.add_space(10.0);
                if ui.button("Reset").clicked() {
                    for (i, knob) in self.accessible_eq_knobs.iter_mut().enumerate() {
                        if let Err(e) = self.audio_engine.set_eq_gain(i, 0.0) {
                            self.error = Some(format!("EQ reset failed: {}", e));
                        }
                        knob.set_value(0.0);
                    }
                }
            });

            ui.add_space(10.0);

            // Mobile EQ layout - 2 rows of 4 bands each
            ui.vertical(|ui| {
                // First row (0-3)
                ui.horizontal_centered(|ui| {
                    for (i, knob) in self.accessible_eq_knobs.iter_mut().enumerate().take(4) {
                        ui.vertical(|ui| {
                            let freq = 60.0 * 2.0_f32.powi(i as i32);
                            let freq_label = if freq < 1000.0 {
                                format!("{:.0}", freq)
                            } else {
                                format!("{:.1}k", freq / 1000.0)
                            };
                            ui.label(RichText::new(freq_label).size(10.0));

                            let knob_response =
                                knob.show(ui, colors, &mut self.accessibility_manager);
                            if knob_response.changed() {
                                if let Err(e) = self.audio_engine.set_eq_gain(i, knob.value()) {
                                    self.error =
                                        Some(format!("EQ band {} update failed: {}", i, e));
                                }
                            }
                        });
                    }
                });

                ui.add_space(5.0);

                // Second row (4-7)
                ui.horizontal_centered(|ui| {
                    for (i, knob) in self.accessible_eq_knobs.iter_mut().enumerate().skip(4) {
                        ui.vertical(|ui| {
                            let freq = 60.0 * 2.0_f32.powi(i as i32);
                            let freq_label = if freq < 1000.0 {
                                format!("{:.0}", freq)
                            } else {
                                format!("{:.1}k", freq / 1000.0)
                            };
                            ui.label(RichText::new(freq_label).size(10.0));

                            let knob_response =
                                knob.show(ui, colors, &mut self.accessibility_manager);
                            if knob_response.changed() {
                                if let Err(e) = self.audio_engine.set_eq_gain(i, knob.value()) {
                                    self.error =
                                        Some(format!("EQ band {} update failed: {}", i, e));
                                }
                            }
                        });
                    }
                });
            });
        });
    }

    fn _draw_legacy_effects_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Spectrum");
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 200.0), egui::Sense::hover());
        let painter = ui.painter();
        let color = Color32::from_rgb(0, 128, 255);
        let spectrum = self.audio_engine.get_spectrum();
        let num_points = spectrum.len();
        let point_spacing = rect.width() / num_points as f32;

        let mut points = Vec::with_capacity(num_points);
        for (i, val) in spectrum.iter().enumerate() {
            let x = rect.min.x + i as f32 * point_spacing;
            let y = rect.center().y - val * rect.height() / 2.0;
            points.push(egui::pos2(x, y));
        }

        if points.len() > 1 {
            painter.add(egui::Shape::Path(egui::epaint::PathShape {
                points,
                closed: false,
                fill: Color32::TRANSPARENT,
                stroke: egui::epaint::PathStroke::new(1.0, color),
            }));
        }
    }

    fn _draw_legacy_eq_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Equalizer");
                if ui.button("Reset").clicked() {
                    for i in 0..8 {
                        if let Err(e) = self.audio_engine.set_eq_band(i, 0.0) {
                            self.error = Some(format!("EQ reset failed: {}", e));
                        }
                    }
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                for i in 0..8 {
                    ui.vertical(|ui| {
                        ui.label(format!("{} Hz", 60 * 2_i32.pow(i as u32)));
                        // Get current gain from knob state (or default)
                        let mut gain = if i < self.accessible_eq_knobs.len() {
                            self.accessible_eq_knobs[i].value()
                        } else {
                            0.0
                        };
                        if ui
                            .add(egui::Slider::new(&mut gain, -40.0..=40.0).vertical())
                            .changed()
                        {
                            if let Err(e) = self.audio_engine.set_eq_band(i, gain) {
                                self.error = Some(format!("EQ band {} update failed: {}", i, e));
                            }
                            // Update knob state if it exists
                            if i < self.accessible_eq_knobs.len() {
                                self.accessible_eq_knobs[i].set_value(gain);
                            }
                        }
                    });
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Master Gain:");
                let mut master_gain = self.audio_engine.get_volume();
                if ui
                    .add(egui::Slider::new(&mut master_gain, 0.0..=2.0))
                    .changed()
                {
                    self.audio_engine.set_volume(master_gain);
                }
            });
        });
    }

    fn draw_generator_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        self.signal_generator_panel.show(ui, colors);
    }

    fn draw_mobile_generator_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        // Mobile version of signal generator - simplified controls
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("🎛️ Signal Generator")
                    .size(18.0)
                    .color(colors.text),
            );
            ui.add_space(10.0);

            // Simplified signal generator for mobile
            self.signal_generator_panel.show(ui, colors);
        });
    }

    fn draw_recording_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        // Update level meters if recording
        self.recording_panel.update_levels();

        // Draw the recording panel
        self.recording_panel.draw(ui, colors);
    }

    fn _draw_legacy_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Theme");
        egui::ComboBox::from_label("Select a theme")
            .selected_text(self.theme_manager.current_theme().display_name())
            .show_ui(ui, |ui| {
                for theme in Theme::all() {
                    let mut current_theme = self.theme_manager.current_theme().clone();
                    if ui
                        .selectable_value(&mut current_theme, theme.clone(), theme.display_name())
                        .clicked()
                    {
                        self.theme_manager.set_theme(theme);
                    }
                }
            });
        ui.add_space(20.0);
        ui.label("Audio Device selection is not supported with the web-audio-api backend.");
    }

    fn tick(&mut self) {
        // Get spectrum data from audio engine
        // The AudioEngine internally handles spectrum processing and normalization

        if self.playback_state == PlaybackState::Playing && !self.is_seeking {
            // Use audio_context from engine to get current time
            self.playback_pos =
                Duration::from_secs_f64(self.audio_engine.get_context().current_time());
        }
    }

    fn handle_keyboard_input(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.play_pause_main();
            }
            if i.key_pressed(egui::Key::S) {
                self.stop_playback_main();
            }
            if i.key_pressed(egui::Key::L) {
                self.toggle_loop_main();
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                self.volume = (self.volume + 0.05).min(1.0);
                self.audio_engine.set_volume(self.volume);
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                self.volume = (self.volume - 0.05).max(0.0);
                self.audio_engine.set_volume(self.volume);
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                self.seek_backward();
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                self.seek_forward();
            }
            if i.key_pressed(egui::Key::O) && i.modifiers.ctrl {
                self.open_file_dialog();
            }
            if i.key_pressed(egui::Key::F1) {
                self.show_keyboard_shortcuts = !self.show_keyboard_shortcuts;
            }
        });
    }

    fn seek_backward(&mut self) {
        let new_pos = self.playback_pos.saturating_sub(Duration::from_secs(5));
        self.seek_to_position_main(new_pos.as_secs_f32());
    }

    fn seek_forward(&mut self) {
        let new_pos = self.playback_pos.saturating_add(Duration::from_secs(5));
        self.seek_to_position_main(new_pos.as_secs_f32());
    }

    fn _legacy_handle_keyboard_input(&mut self, ui: &mut egui::Ui) {
        ui.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.play_pause_main();
            }
            if i.key_pressed(egui::Key::S) {
                self.stop_playback_main();
            }
            if i.key_pressed(egui::Key::L) {
                self.is_looping = !self.is_looping;
                if let Err(e) = self.audio_engine.set_loop(self.is_looping) {
                    self.error_manager
                        .add_playback_error(Some(format!("Set loop: {}", e)));
                }
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                self.volume = (self.volume + 0.05).min(1.0);
                self.audio_engine.set_volume(self.volume);
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                self.volume = (self.volume - 0.05).max(0.0);
                self.audio_engine.set_volume(self.volume);
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                let new_pos = self.playback_pos.saturating_sub(Duration::from_secs(5));
                if let Err(e) = self.audio_engine.seek(new_pos) {
                    self.error_manager
                        .add_playback_error(Some(format!("Seek backward: {}", e)));
                } else {
                    self.playback_pos = new_pos;
                }
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                let new_pos = self.playback_pos.saturating_add(Duration::from_secs(5));
                if let Err(e) = self.audio_engine.seek(new_pos) {
                    self.error_manager
                        .add_playback_error(Some(format!("Seek forward: {}", e)));
                } else {
                    self.playback_pos = new_pos;
                }
            }
        });
    }

    fn load_current_file(&mut self) {
        if let Some(handle) = &self.current_file {
            let path = handle.path();
            let filename = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown file");

            // Clear previous errors
            self.error = None;
            self.load_progress = Some(0.0);

            // Load metadata (quick operation)
            if let Ok(tagged_file) = lofty::read_from_path(path) {
                if let Some(tag) = tagged_file.primary_tag() {
                    self.metadata = Some(TrackMetadata {
                        title: tag.title().as_deref().unwrap_or("Unknown Title").into(),
                        artist: tag.artist().as_deref().unwrap_or("Unknown Artist").into(),
                        album: tag.album().as_deref().unwrap_or("Unknown Album").into(),
                        year: tag
                            .year()
                            .map(|y| y.to_string())
                            .unwrap_or_else(|| "----".into()),
                    });
                }
                self.album_art = None; // Album art loaded separately
            }

            self.load_progress = Some(0.3); // Metadata loaded

            // Load audio file via AudioEngine
            let path_str = path.to_str().unwrap_or("");
            match self.audio_engine.load_audio_file(path_str) {
                Ok(duration) => {
                    self.load_progress = Some(0.8); // Audio loaded

                    // Update UI state
                    self.total_duration = duration;

                    // Update waveform preview
                    if let Some(waveform) = self.audio_engine.get_waveform(WAVEFORM_PREVIEW_SAMPLES)
                    {
                        self.waveform_preview = waveform;
                        self.waveform_dirty = true;
                    }

                    // Start playback via AudioEngine
                    if let Err(e) = self.audio_engine.play() {
                        self.error_manager
                            .add_playback_error(Some(format!("Play File: {}", e)));
                        self.error = Some("Failed to start playback".to_string());
                    } else {
                        self.playback_state = PlaybackState::Playing;
                        self.playback_pos = Duration::ZERO;

                        // Announce successful load
                        self.accessibility_manager.announce(
                            format!("Audio file loaded: {}", filename),
                            ui::accessibility::AnnouncementPriority::Medium,
                        );
                    }

                    self.load_progress = None; // Loading complete
                }
                Err(e) => {
                    self.load_progress = None;

                    // Detailed error handling
                    let error_str = e.to_string();
                    if error_str.contains("Permission denied") {
                        self.error_manager
                            .add_permission_error("access", path.to_str().unwrap_or(filename));
                        self.error = Some("Permission denied".to_string());
                    } else if error_str.contains("decode") || error_str.contains("format") {
                        self.error_manager
                            .add_audio_decode_error(filename, Some("audio"));
                        self.error = Some("Failed to decode audio file".to_string());
                    } else {
                        self.error_manager
                            .add_file_load_error(filename, Some(error_str.clone()));
                        self.error = Some(format!("Failed to load file: {}", error_str));
                    }
                }
            }
        }
    }

    fn reset_all_settings(&mut self) {
        // Reset equalizer via AudioEngine
        for i in 0..8 {
            if let Err(e) = self.audio_engine.set_eq_gain(i, 0.0) {
                self.error_manager
                    .add_playback_error(Some(format!("Reset EQ Band {}: {}", i, e)));
            }
            // Sync UI state
            self.accessible_eq_knobs[i].set_value(0.0);
        }

        // Reset volume via AudioEngine
        self.volume = 0.5;
        if let Err(e) = self.audio_engine.set_volume(self.volume) {
            self.error_manager
                .add_playback_error(Some(format!("Reset Volume: {}", e)));
        }
        self.accessible_volume_slider.set_value(self.volume);

        // Reset spectrum visualizer
        self.spectrum_visualizer = SpectrumVisualizer::new(SpectrumVisualizerConfig::default());

        // Announce reset
        self.accessibility_manager.announce(
            "All audio settings have been reset to defaults".to_string(),
            ui::accessibility::AnnouncementPriority::Medium,
        );
    }

    fn show_format_help(&mut self) {
        self.accessibility_manager.announce(
            "Supported audio formats: MP3, WAV, FLAC, OGG, M4A. Ensure your file is not corrupted."
                .to_string(),
            ui::accessibility::AnnouncementPriority::Medium,
        );
    }

    fn _load_album_art(&mut self, ctx: &egui::Context) {
        if let Some(handle) = &self.current_file {
            let path = handle.path();
            if let Ok(tagged_file) = lofty::read_from_path(path) {
                if let Some(picture) = tagged_file.primary_tag().and_then(|t| t.pictures().get(0)) {
                    if let Ok(img) = image::load_from_memory(picture.data()) {
                        let (width, height) = img.dimensions();
                        let rgba = img.to_rgba8();
                        let pixels = rgba.into_raw();
                        let image = egui::ColorImage::from_rgba_unmultiplied(
                            [width as usize, height as usize],
                            &pixels,
                        );
                        self.album_art = Some(Arc::new(ctx.load_texture(
                            "album-art",
                            image,
                            Default::default(),
                        )));
                    }
                }
            }
        }
    }

    // Audio control methods
    fn open_file_dialog(&mut self) {
        if let Some(file) = rfd::FileDialog::new()
            .add_filter("Audio Files", &["mp3", "wav", "flac", "ogg", "m4a"])
            .pick_file()
        {
            self.current_file = Some(Arc::new(rfd::FileHandle::from(file)));
            self.load_current_file();
        }
    }

    fn play_pause_main(&mut self) {
        match self.playback_state {
            PlaybackState::Playing => {
                // Pause playback via AudioEngine
                if let Err(e) = self.audio_engine.pause() {
                    self.error_manager
                        .add_playback_error(Some(format!("Pause: {}", e)));
                } else {
                    self.playback_state = PlaybackState::Paused;
                }
            }
            PlaybackState::Paused | PlaybackState::Stopped => {
                if self.current_file.is_some() {
                    self.load_current_file();
                } else if !self.signal_generator_panel.generated_samples.is_empty() {
                    self.play_generated_signal();
                }
            }
        }
    }

    fn stop_playback_main(&mut self) {
        // Stop playback via AudioEngine
        if let Err(e) = self.audio_engine.stop() {
            self.error_manager
                .add_playback_error(Some(format!("Stop: {}", e)));
        }

        // Update UI state
        self.playback_state = PlaybackState::Stopped;
        self.playback_pos = Duration::ZERO;
    }

    fn toggle_loop_main(&mut self) {
        self.is_looping = !self.is_looping;

        // Set loop state via AudioEngine
        if let Err(e) = self.audio_engine.set_loop(self.is_looping) {
            self.error_manager
                .add_playback_error(Some(format!("Toggle Loop: {}", e)));
            // Revert state on error
            self.is_looping = !self.is_looping;
        }
    }

    fn seek_to_position_main(&mut self, position_seconds: f32) {
        let new_pos =
            Duration::from_secs_f32(position_seconds.clamp(0.0, self.total_duration.as_secs_f32()));

        // Seek via AudioEngine
        if let Err(e) = self.audio_engine.seek(new_pos) {
            self.error_manager
                .add_playback_error(Some(format!("Seek: {}", e)));
        } else {
            // Update UI state
            self.playback_pos = new_pos;
        }
    }

    fn play_generated_signal(&mut self) {
        // Use generated samples directly instead of creating AudioBuffer
        if !self.signal_generator_panel.generated_samples.is_empty() {
            // Get signal parameters first to avoid borrow checker issues
            let samples = self.signal_generator_panel.generated_samples.clone();
            let sample_rate = self.signal_generator_panel.parameters.sample_rate as f32;
            let channels = 1; // Mono signal

            // Update waveform preview
            self.update_waveform_from_samples(&samples, channels);

            // Use audition_buffer instead of load_buffer for generated signals
            match self
                .audio_engine
                .audition_buffer(&samples, sample_rate, channels)
            {
                Ok(_) => {
                    // Start playback via AudioEngine
                    if let Err(e) = self.audio_engine.play() {
                        self.error_manager
                            .add_playback_error(Some(format!("Play Signal: {}", e)));
                    } else {
                        // Update UI state
                        self.playback_state = PlaybackState::Playing;
                        self.total_duration = Duration::from_secs_f32(
                            self.signal_generator_panel.parameters.duration,
                        );
                        self.playback_pos = Duration::ZERO;
                    }
                }
                Err(e) => {
                    self.error_manager
                        .add_playback_error(Some(format!("Load Signal Buffer: {}", e)));
                }
            }
        }
    }

    // TODO: Refactor to not depend on private AudioBuffer type
    // fn update_waveform_from_buffer(&mut self, buffer: &AudioBuffer) {
    //     let preview = Self::downsample_waveform_from_buffer(buffer);
    //     if !preview.is_empty() {
    //         self.waveform_preview = preview;
    //         self.waveform_dirty = true;
    //     }
    // }

    fn update_waveform_from_samples(&mut self, samples: &[f32], channels: usize) {
        let preview = Self::downsample_waveform_from_samples(samples, channels);
        if !preview.is_empty() {
            self.waveform_preview = preview;
            self.waveform_dirty = true;
        }
    }

    // TODO: Refactor to not depend on private AudioBuffer type
    // fn downsample_waveform_from_buffer(buffer: &AudioBuffer) -> Vec<f32> {
    //     let num_channels = buffer.number_of_channels().max(1);
    //     let total_frames = buffer.length().max(1);
    //     let step = (total_frames / WAVEFORM_PREVIEW_SAMPLES).max(1);
    //     let channels: Vec<&[f32]> = (0..num_channels)
    //         .map(|ch| buffer.get_channel_data(ch))
    //         .collect();
    //
    //     let mut waveform = Vec::with_capacity(WAVEFORM_PREVIEW_SAMPLES);
    //     for frame in (0..total_frames).step_by(step) {
    //         let mut sample = 0.0;
    //         for channel in &channels {
    //             if frame < channel.len() {
    //                 sample += channel[frame];
    //             }
    //         }
    //         waveform.push(sample / num_channels as f32);
    //         if waveform.len() >= WAVEFORM_PREVIEW_SAMPLES {
    //             break;
    //         }
    //     }
    //
    //     Self::normalize_waveform(waveform)
    // }

    fn downsample_waveform_from_samples(samples: &[f32], channels: usize) -> Vec<f32> {
        if samples.is_empty() {
            return Vec::new();
        }
        let channels = channels.max(1);
        let frames = samples.len() / channels;
        let step = (frames / WAVEFORM_PREVIEW_SAMPLES).max(1);

        let mut waveform = Vec::with_capacity(WAVEFORM_PREVIEW_SAMPLES);
        for frame in (0..frames).step_by(step) {
            let mut value = 0.0;
            for ch in 0..channels {
                let index = frame * channels + ch;
                if index < samples.len() {
                    value += samples[index];
                }
            }
            waveform.push(value / channels as f32);
            if waveform.len() >= WAVEFORM_PREVIEW_SAMPLES {
                break;
            }
        }

        Self::normalize_waveform(waveform)
    }

    fn normalize_waveform(mut waveform: Vec<f32>) -> Vec<f32> {
        let max = waveform
            .iter()
            .fold(0.0_f32, |acc, &sample| acc.max(sample.abs()));
        if max > 0.0 {
            for sample in &mut waveform {
                *sample /= max;
            }
        }
        waveform
    }

    fn draw_transport_panel(&mut self, ctx: &egui::Context, colors: &ThemeColors) {
        let compact = matches!(self.screen_size, ScreenSize::Mobile | ScreenSize::Tablet);
        let height = if compact { 110.0 } else { 95.0 };
        egui::TopBottomPanel::top("transport_panel")
            .resizable(false)
            .exact_height(height)
            .frame(egui::Frame::NONE.fill(ColorUtils::with_alpha(colors.surface, 0.9)))
            .show(ctx, |ui| {
                ui.set_width(ui.available_width());
                self.render_transport_controls(ui, colors, compact);
            });
    }

    fn render_transport_controls(
        &mut self,
        ui: &mut egui::Ui,
        colors: &ThemeColors,
        compact: bool,
    ) {
        let button_height = if compact { 34.0 } else { 44.0 };
        let primary_width = if compact { 88.0 } else { 110.0 };
        let secondary_width = if compact { 72.0 } else { 92.0 };
        let full_width = ui.available_width();

        ui.vertical(|ui| {
            ui.horizontal_wrapped(|ui| {
                self.transport_button(
                    ui,
                    colors,
                    "📁 Open",
                    primary_width,
                    button_height,
                    false,
                    |this| this.open_file_dialog(),
                );

                let play_label = if self.playback_state == PlaybackState::Playing {
                    "⏸ Pause"
                } else {
                    "▶ Play"
                };
                self.transport_button(
                    ui,
                    colors,
                    play_label,
                    primary_width,
                    button_height,
                    true,
                    |this| this.play_pause_main(),
                );

                self.transport_button(
                    ui,
                    colors,
                    "⏹ Stop",
                    secondary_width,
                    button_height,
                    false,
                    |this| this.stop_playback_main(),
                );

                let loop_label = if self.is_looping {
                    "🔁 Loop On"
                } else {
                    "🔁 Loop Off"
                };
                self.transport_button(
                    ui,
                    colors,
                    loop_label,
                    secondary_width,
                    button_height,
                    false,
                    |this| this.toggle_loop_main(),
                );

                let (record_badge, record_color) = self.recording_panel.status_badge();
                let record_label = if self.recording_panel.is_recording() {
                    format!("{} Stop Rec", record_badge)
                } else {
                    format!("{} Record", record_badge)
                };
                let mut record_button = EnhancedButton::new(record_label).style(ButtonStyle {
                    rounding: 12.0,
                    padding: Vec2::new(16.0, 10.0),
                    min_size: Vec2::new(primary_width.max(96.0), button_height),
                    gradient: true,
                    shadow: true,
                    glow: true,
                });
                if record_button.show(ui, colors).clicked() {
                    self.recording_panel.toggle_recording();
                    self.active_tab = Tab::Recording;
                }
                ui.label(RichText::new(format!("{} Recorder", record_badge)).color(record_color));
            });

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                let slider_width = full_width * if compact { 0.45 } else { 0.55 };
                ui.label(RichText::new("🔊").size(14.0));
                ui.allocate_ui(Vec2::new(slider_width, 30.0), |ui| {
                    let response = self.accessible_volume_slider.show(
                        ui,
                        colors,
                        &mut self.accessibility_manager,
                    );
                    if response.changed() {
                        self.volume = self.accessible_volume_slider.value();
                        self.audio_engine.set_volume(self.volume);

                        if !self.accessibility_manager.is_volume_safe(self.volume) {
                            self.accessibility_manager.announce(
                                "Warning: Volume level may be harmful to hearing".to_string(),
                                ui::accessibility::AnnouncementPriority::High,
                            );
                        }
                    }
                });
                ui.label(format!("{:.0}%", self.volume * 100.0));
                self.volume_safety_indicator.show(ui, colors);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some((badge, color)) = self.backend_health_badge() {
                        ui.label(RichText::new(badge).color(color).strong());
                    }
                    if let Some(device) = self.selected_output_device_label() {
                        ui.label(RichText::new(device).color(colors.text_secondary));
                    }
                });
            });

            let mut clear_status = false;
            if let Some((message, timestamp)) = self.audio_status_message.as_ref() {
                if timestamp.elapsed() < Duration::from_secs(4) {
                    ui.add_space(4.0);
                    ui.label(RichText::new(message).color(colors.text_secondary));
                } else {
                    clear_status = true;
                }
            }
            if clear_status {
                self.audio_status_message = None;
            }
        });
    }

    fn transport_button<F>(
        &mut self,
        ui: &mut egui::Ui,
        colors: &ThemeColors,
        label: &str,
        width: f32,
        height: f32,
        primary: bool,
        mut on_click: F,
    ) where
        F: FnMut(&mut Self),
    {
        let mut button = EnhancedButton::new(label.to_string()).style(ButtonStyle {
            rounding: 12.0,
            padding: Vec2::new(16.0, 10.0),
            min_size: Vec2::new(width, height),
            gradient: primary,
            shadow: true,
            glow: primary,
        });
        if button.show(ui, colors).clicked() {
            on_click(self);
        }
    }

    fn backend_health_badge(&self) -> Option<(String, Color32)> {
        let backend = self.audio_backend.as_ref()?;
        let (label, color) = match backend.health() {
            BackendHealth::Healthy => ("✅ Backend Healthy", Color32::from_rgb(100, 255, 100)),
            BackendHealth::Degraded => ("⚠️ Backend Degraded", Color32::from_rgb(255, 210, 120)),
            BackendHealth::Failed => ("❌ Backend Failed", Color32::from_rgb(255, 120, 120)),
        };
        Some((label.to_string(), color))
    }

    fn selected_output_device_label(&self) -> Option<String> {
        let manager = self.device_manager.as_ref()?;
        let device = manager.selected_output_device()?;
        let latency = manager
            .stream_latency_ms()
            .map(|ms| format!("{:.1} ms", ms))
            .unwrap_or_else(|| "latency n/a".to_string());
        Some(format!("{} • {}", device.name, latency))
    }

    /// Setup hybrid audio mode with ring buffer bridge
    fn setup_hybrid_mode(&mut self) {
        // Only setup if backend is available and in HybridNative mode
        if let Some(backend) = &mut self.audio_backend {
            let mode = backend.mode();
            if mode == HybridMode::HybridNative || mode == HybridMode::AsioOnly {
                // Get ring buffer producer from backend
                if let Some(producer) = backend.take_ring_producer() {
                    // Create ScriptProcessor and connect to destination in a separate scope
                    // to avoid holding an immutable borrow of self.audio_engine while calling connect_output_to (mutable)
                    let script_processor = {
                        let audio_context = self.audio_engine.get_context();

                        // Create ScriptProcessor
                        let script_processor = audio_context.create_script_processor(1024, 2, 2);

                        // Move producer into closure
                        let mut producer = producer;

                        script_processor.set_onaudioprocess(move |e| {
                            let input = &e.input_buffer;
                            let channels = input.number_of_channels();
                            let length = input.length();

                            if let Ok(mut chunk) = producer.write_chunk(length * channels) {
                                let (s1, s2) = chunk.as_mut_slices();
                                let mut written = 0;

                                // Interleave logic: L R L R ...
                                for i in 0..length {
                                    for ch in 0..channels {
                                        let sample = input.get_channel_data(ch)[i];

                                        // Write to s1 then s2
                                        if written < s1.len() {
                                            s1[written] = sample;
                                        } else if written - s1.len() < s2.len() {
                                            s2[written - s1.len()] = sample;
                                        }
                                        written += 1;
                                    }
                                }
                                chunk.commit_all();
                            }
                        });

                        // Connect script processor to destination to keep graph alive
                        // (It outputs silence because we don't write to output buffer)
                        script_processor.connect(&audio_context.destination());

                        script_processor
                    };

                    // Connect to engine output (requires mutable borrow of self.audio_engine)
                    if let Err(e) = self.audio_engine.connect_output_to(&script_processor) {
                        eprintln!("Failed to connect script processor: {}", e);
                    } else {
                        // Disable default output (analyser -> destination) to avoid double path
                        let _ = self.audio_engine.set_output_routing(false);

                        // Keep node alive
                        self.script_processor = Some(script_processor);
                        println!("✅ Hybrid audio bridge connected");
                    }
                } else {
                    eprintln!("⚠️ Ring buffer not available from backend");
                }
            }
        }
    }

    fn draw_settings_panel_main(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        ui.vertical(|ui| {
            ui.heading("Settings");
            ui.add_space(10.0);

            // Theme Settings
            ui.group(|ui| {
                ui.label(RichText::new("🎨 Theme").strong());
                ui.add_space(5.0);
                egui::ComboBox::from_label("")
                    .selected_text(self.theme_manager.current_theme().display_name())
                    .show_ui(ui, |ui| {
                        for theme in Theme::all() {
                            let mut current_theme = self.theme_manager.current_theme().clone();
                            if ui.selectable_value(&mut current_theme, theme.clone(), theme.display_name()).clicked() {
                                self.theme_manager.set_theme(theme);
                            }
                        }
                    });
            });

            ui.add_space(15.0);

            // Audio Backend Settings (Phase 3.1 Enhanced UI)
            let mut should_setup_hybrid = false;

            ui.group(|ui| {
                ui.label(RichText::new("🔊 Audio Backend").strong());
                ui.add_space(5.0);

                if let Some(backend) = &mut self.audio_backend {
                    let current_mode = backend.mode();

                    ui.horizontal(|ui| {
                        ui.label("Mode:");
                        if ui.radio(current_mode == HybridMode::WebAudioOnly, "🌐 Web Audio API").clicked() {
                            if let Err(e) = backend.set_mode(HybridMode::WebAudioOnly) {
                                self.audio_status_message = Some((format!("Failed to switch mode: {}", e), Instant::now()));
                            } else {
                                self.audio_status_message = Some(("Switched to Web Audio API mode".to_string(), Instant::now()));
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("");
                        if ui.radio(current_mode == HybridMode::HybridNative, "🎵 Hybrid (Native + Web)").clicked() {
                            if let Err(e) = backend.set_mode(HybridMode::HybridNative) {
                                self.audio_status_message = Some((format!("Failed to switch mode: {}", e), Instant::now()));
                            } else {
                                self.audio_status_message = Some(("Switched to Hybrid Native mode".to_string(), Instant::now()));
                                // Mark for setup after releasing borrow
                                should_setup_hybrid = true;
                            }
                        }
                    });

                    #[cfg(target_os = "windows")]
                    ui.horizontal(|ui| {
                        ui.label("");
                        if ui.radio(current_mode == HybridMode::AsioOnly, "⚡ ASIO (Professional)").clicked() {
                            if let Err(e) = backend.set_mode(HybridMode::AsioOnly) {
                                self.audio_status_message = Some((format!("Failed to switch mode: {}", e), Instant::now()));
                            } else {
                                self.audio_status_message = Some(("Switched to ASIO mode".to_string(), Instant::now()));
                            }
                        }
                    });

                    // Show mode-specific info
                    ui.add_space(5.0);
                    match current_mode {
                        HybridMode::WebAudioOnly => {
                            ui.label(RichText::new("ℹ️ Browser-compatible mode, ~50-100ms latency").size(11.0).color(colors.text_secondary));
                        }
                        HybridMode::HybridNative => {
                            ui.label(RichText::new("✨ Native hardware + Web effects, ~5-15ms latency").size(11.0).color(Color32::from_rgb(100, 200, 100)));
                        }
                        HybridMode::CpalOnly => {
                            ui.label(RichText::new("⚡ Maximum performance, <5ms latency").size(11.0).color(Color32::from_rgb(100, 255, 100)));
                        }
                        HybridMode::AsioOnly => {
                            ui.label(RichText::new("🎹 Professional ASIO, <3ms latency, Exclusive Mode").size(11.0).color(Color32::from_rgb(150, 255, 255)));
                        }
                    }
                } else {
                    ui.label(RichText::new("⚠️ Audio backend not initialized").color(Color32::from_rgb(255, 200, 100)));
                    ui.label("Using web-audio-api fallback mode");
                }
            });

            // Setup hybrid mode if requested (after releasing borrow)
            if should_setup_hybrid {
                self.setup_hybrid_mode();
            }

            ui.add_space(15.0);

            // Fallback Policy Settings (Phase 3.1.6)
            if let Some(backend) = &mut self.audio_backend {
                ui.group(|ui| {
                    ui.label(RichText::new("🔄 Audio Fallback Policy").strong());
                    ui.add_space(5.0);

                    let current_policy = backend.fallback_policy();
                    let health = backend.health();

                    // Show health status with color coding
                    let (health_text, health_color) = match health {
                        BackendHealth::Healthy => ("✅ Healthy", Color32::from_rgb(100, 255, 100)),
                        BackendHealth::Degraded => ("⚠️ Degraded", Color32::from_rgb(255, 200, 100)),
                        BackendHealth::Failed => ("❌ Failed", Color32::from_rgb(255, 100, 100)),
                    };

                    ui.horizontal(|ui| {
                        ui.label("Status:");
                        ui.label(RichText::new(health_text).color(health_color));
                    });

                    ui.add_space(5.0);

                    // Fallback policy dropdown
                    let policy_text = match current_policy {
                        FallbackPolicy::Manual => "Manual",
                        FallbackPolicy::AutoOnError => "Auto on Error",
                        FallbackPolicy::AutoWithPreference(_) => "Auto with Preference",
                    };

                    egui::ComboBox::from_label("Policy:")
                        .selected_text(policy_text)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(
                                matches!(current_policy, FallbackPolicy::Manual),
                                "🛠️ Manual"
                            ).on_hover_text("You control mode switching. No automatic fallback.").clicked() {
                                backend.set_fallback_policy(FallbackPolicy::Manual);
                                self.audio_status_message = Some(("Fallback policy: Manual".to_string(), Instant::now()));
                            }

                            if ui.selectable_label(
                                matches!(current_policy, FallbackPolicy::AutoOnError),
                                "🔄 Auto on Error"
                            ).on_hover_text("Automatically switch to web audio if native audio fails.").clicked() {
                                backend.set_fallback_policy(FallbackPolicy::AutoOnError);
                                self.audio_status_message = Some(("Fallback policy: Auto on Error".to_string(), Instant::now()));
                            }

                            if ui.selectable_label(
                                matches!(current_policy, FallbackPolicy::AutoWithPreference(_)),
                                "⚙️ Auto with Preference"
                            ).on_hover_text("Try preferred mode, fallback if unavailable.").clicked() {
                                let preferred_mode = backend.mode();
                                backend.set_fallback_policy(FallbackPolicy::AutoWithPreference(preferred_mode));
                                self.audio_status_message = Some(("Fallback policy: Auto with Preference".to_string(), Instant::now()));
                            }
                        });

                    // Show policy description
                    ui.add_space(5.0);
                    let description = match current_policy {
                        FallbackPolicy::Manual => "You have full control. System will not automatically switch audio backends.",
                        FallbackPolicy::AutoOnError => "System will automatically fallback to Web Audio API if native audio encounters errors (device disconnect, excessive underruns).",
                        FallbackPolicy::AutoWithPreference(mode) => {
                            &format!("System will try {:?} first, automatically falling back to Web Audio API if unavailable.", mode)
                        }
                    };

                    ui.label(RichText::new(description).size(11.0).color(colors.text_secondary).italics());
                });
            }

            ui.add_space(15.0);

            // Device Selection (Phase 3.1 Enhanced UI)
            if let Some(backend) = &self.audio_backend {
                if backend.mode() != HybridMode::WebAudioOnly {
                    ui.group(|ui| {
                        ui.label(RichText::new("🎧 Audio Device").strong());
                        ui.add_space(5.0);

                        if let Some(device_manager) = &self.device_manager {
                            // Output device selection
                            ui.label("Output Device:");

                            let selected_device = device_manager.selected_output_device();
                            let selected_name = selected_device.as_ref()
                                .map(|d| d.name.clone())
                                .unwrap_or_else(|| "No device selected".to_string());

                            egui::ComboBox::from_label("")
                                .selected_text(&selected_name)
                                .show_ui(ui, |ui| {
                                    if let Ok(devices) = device_manager.enumerate_output_devices() {
                                        for device in devices {
                                            let icon = if device.is_default { "🔊" } else { "🔉" };
                                            let label = format!("{} {}", icon, device.name);

                                            if ui.selectable_label(
                                                selected_device.as_ref().map(|d| d.id == device.id).unwrap_or(false),
                                                label
                                            ).clicked() {
                                                if let Err(e) = device_manager.select_output_device(&device.id) {
                                                    self.audio_status_message = Some((format!("Failed to select device: {}", e), Instant::now()));
                                                } else {
                                                    self.audio_status_message = Some((format!("Selected device: {}", device.name), Instant::now()));
                                                }
                                            }
                                        }
                                    } else {
                                        ui.label("⚠️ Failed to enumerate devices");
                                    }
                                });

                            // Device info display
                            if let Some(device) = &selected_device {
                                ui.add_space(10.0);
                                ui.group(|ui| {
                                    ui.label(RichText::new("Device Info:").size(12.0).strong());
                                    ui.separator();

                                    ui.horizontal(|ui| {
                                        ui.label("Sample Rate:");
                                        ui.label(format!("{} - {} Hz", device.min_sample_rate, device.max_sample_rate));
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label("Channels:");
                                        ui.label(format!("{}", device.max_output_channels));
                                    });
                                });
                            }

                            // Latency indicator with color coding
                            ui.add_space(10.0);
                            if let Some(latency_ms) = device_manager.stream_latency_ms() {
                                let latency_color = if latency_ms < 10.0 {
                                    Color32::from_rgb(100, 255, 100) // Green: excellent
                                } else if latency_ms < 20.0 {
                                    Color32::from_rgb(200, 255, 100) // Yellow-green: good
                                } else if latency_ms < 50.0 {
                                    Color32::from_rgb(255, 200, 100) // Orange: acceptable
                                } else {
                                    Color32::from_rgb(255, 100, 100) // Red: high
                                };

                                ui.horizontal(|ui| {
                                    ui.label("Latency:");
                                    ui.label(RichText::new(format!("{:.1} ms", latency_ms))
                                        .color(latency_color)
                                        .strong());

                                    // Status indicator
                                    if latency_ms < 10.0 {
                                        ui.label("✅ Excellent");
                                    } else if latency_ms < 20.0 {
                                        ui.label("✨ Good");
                                    } else if latency_ms < 50.0 {
                                        ui.label("⚠️ Acceptable");
                                    } else {
                                        ui.label("❌ High");
                                    }
                                });
                            }
                        } else {
                            ui.label("⚠️ Device manager not initialized");
                        }
                    });

                    ui.add_space(15.0);
                }
            }

            // Show status messages (fade out after 3 seconds)
            if let Some((message, timestamp)) = &self.audio_status_message {
                let elapsed = Instant::now().duration_since(*timestamp).as_secs_f32();
                if elapsed < 3.0 {
                    let alpha = (1.0 - (elapsed / 3.0)).clamp(0.0, 1.0);
                    let mut color = Color32::from_rgb(100, 200, 255);
                    color[3] = (alpha * 255.0) as u8;

                    ui.add_space(5.0);
                    ui.label(RichText::new(message).color(color).size(12.0));
                } else {
                    self.audio_status_message = None;
                }
            }

            ui.add_space(15.0);

            // About section
            ui.group(|ui| {
                ui.label(RichText::new("ℹ️ About").strong());
                ui.add_space(5.0);
                ui.label("Rusty Audio Player v0.1.0");
                ui.label("Built with Rust and egui");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("🎵 Web Audio Graph:");
                    ui.label(RichText::new("Enabled").color(Color32::from_rgb(100, 255, 100)));
                });
                if self.audio_backend.is_some() {
                    ui.horizontal(|ui| {
                        ui.label("🔊 Native Audio:");
                        ui.label(RichText::new("Enabled").color(Color32::from_rgb(100, 255, 100)));
                    });
                }
            });
        });
    }

    fn draw_dock_layout(&mut self, ctx: &egui::Context, colors: &ThemeColors) {
        // Show dock layout with menu bar for workspace switching
        egui::TopBottomPanel::top("dock_menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("View", |ui| {
                    if ui.button("📋 Default Workspace").clicked() {
                        self.dock_layout_manager
                            .switch_workspace(ui::dock_layout::WorkspacePreset::Default);
                        ui.close();
                    }
                    if ui.button("📊 Analyzer Workspace").clicked() {
                        self.dock_layout_manager
                            .switch_workspace(ui::dock_layout::WorkspacePreset::Analyzer);
                        ui.close();
                    }
                    if ui.button("🎛️ Generator Workspace").clicked() {
                        self.dock_layout_manager
                            .switch_workspace(ui::dock_layout::WorkspacePreset::Generator);
                        ui.close();
                    }
                    if ui.button("🎚️ Mixing Workspace").clicked() {
                        self.dock_layout_manager
                            .switch_workspace(ui::dock_layout::WorkspacePreset::Mixing);
                        ui.close();
                    }
                    if ui.button("⏯️ Playback Workspace").clicked() {
                        self.dock_layout_manager
                            .switch_workspace(ui::dock_layout::WorkspacePreset::Playback);
                        ui.close();
                    }
                });

                ui.separator();
                ui.label(format!(
                    "Workspace: {:?}",
                    self.dock_layout_manager.active_workspace()
                ));
            });
        });

        self.draw_transport_panel(ctx, colors);

        // Render the dock layout by temporarily taking ownership
        // This is safe because we're not using dock_layout_manager again in this function
        let mut dock_manager =
            std::mem::replace(&mut self.dock_layout_manager, DockLayoutManager::new());
        dock_manager.show(ctx, self);
        self.dock_layout_manager = dock_manager;
    }

    fn draw_keyboard_shortcuts_overlay(&mut self, ctx: &egui::Context, colors: &ThemeColors) {
        let mut show_shortcuts = self.show_keyboard_shortcuts;
        let mut close_requested = false;

        egui::Window::new("🎹 Keyboard Shortcuts")
            .open(&mut show_shortcuts)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Playback Controls")
                            .size(16.0)
                            .color(colors.text)
                            .strong(),
                    );
                    ui.separator();
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Space").color(colors.accent).strong());
                        ui.label("Play/Pause");
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("S").color(colors.accent).strong());
                        ui.label("Stop");
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("L").color(colors.accent).strong());
                        ui.label("Toggle Loop");
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Ctrl+O").color(colors.accent).strong());
                        ui.label("Open File");
                    });

                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("Volume & Seeking")
                            .size(16.0)
                            .color(colors.text)
                            .strong(),
                    );
                    ui.separator();
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("↑/↓").color(colors.accent).strong());
                        ui.label("Volume Up/Down");
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("←/→").color(colors.accent).strong());
                        ui.label("Seek Backward/Forward (5s)");
                    });

                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("Interface")
                            .size(16.0)
                            .color(colors.text)
                            .strong(),
                    );
                    ui.separator();
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("F1").color(colors.accent).strong());
                        ui.label("Show/Hide This Help");
                    });

                    ui.add_space(15.0);

                    if EnhancedButton::new("Close")
                        .style(ButtonStyle {
                            gradient: true,
                            ..Default::default()
                        })
                        .show(ui, colors)
                        .clicked()
                    {
                        close_requested = true;
                    }
                });
            });

        if close_requested || !show_shortcuts {
            self.show_keyboard_shortcuts = false;
        }
    }
}

// ============================================================================
// Platform-specific entry points
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    // Configure for Windows landscape HiDPI displays
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(1200.0, 800.0)) // Landscape orientation
            .with_min_inner_size(Vec2::new(800.0, 600.0)) // Minimum size
            .with_resizable(true)
            .with_maximize_button(true)
            .with_minimize_button(true)
            .with_close_button(true)
            .with_drag_and_drop(true)
            .with_transparent(false)
            .with_decorations(true),
        multisampling: 4, // Anti-aliasing for crisp HiDPI rendering
        depth_buffer: 8,  // Better rendering quality
        stencil_buffer: 0,
        hardware_acceleration: eframe::HardwareAcceleration::Required,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Rusty Audio - Car Stereo Style Player",
        options,
        Box::new(|cc| {
            // Configure for HiDPI scaling
            cc.egui_ctx.set_pixels_per_point(1.25); // Optimal for HiDPI displays
            cc.egui_ctx.set_zoom_factor(1.0);

            // Enable better text rendering for HiDPI
            let fonts = egui::FontDefinitions::default();

            // Use default system fonts for now - custom font loading can be added later if needed
            // This ensures compatibility without requiring external font files

            cc.egui_ctx.set_fonts(fonts);

            // Configure visual options for HiDPI
            let mut visuals = egui::Visuals::default();
            // Note: window_rounding and menu_rounding are now controlled via Style in egui 0.33
            visuals.button_frame = true;
            visuals.collapsing_header_frame = true;
            visuals.indent_has_left_vline = true;
            visuals.striped = true;
            visuals.slider_trailing_fill = true;

            // Optimize for landscape layout
            visuals.panel_fill = Color32::from_gray(24);
            visuals.window_fill = Color32::from_gray(32);

            cc.egui_ctx.set_visuals(visuals);

            Ok(Box::new(AudioPlayerApp::default()))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = WebHandle)]
pub use rusty_audio_core::web::WebHandle;
