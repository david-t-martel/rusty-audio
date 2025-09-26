use eframe::{egui, NativeOptions};
use egui::{
    load::SizedTexture, Color32, RichText, Vec2,
};
use image::GenericImageView;
use lofty::{file::TaggedFileExt, tag::Accessor};
use rfd::FileHandle;
use std::sync::Arc;
use std::time::{Duration, Instant};
use web_audio_api::context::{AudioContext, BaseAudioContext};
use web_audio_api::node::{
    AudioNode, AudioScheduledSourceNode, BiquadFilterNode, BiquadFilterType, AnalyserNode,
};

mod ui;
mod ui_extensions;
mod audio_performance;
pub mod testing;

use ui::{
    theme::{Theme, ThemeManager, ThemeColors},
    layout::{LayoutManager, PanelConfig, PanelType, DockSide},
    spectrum::{SpectrumVisualizer, SpectrumVisualizerConfig, SpectrumMode},
    components::{AlbumArtDisplay, ProgressBar, MetadataDisplay, MetadataLayout, ProgressBarStyle},
    controls::{EnhancedSlider, CircularKnob, EnhancedButton, SliderOrientation, SliderStyle, ButtonStyle},
    utils::{ScreenSize, AnimationState, ResponsiveSize},
    signal_generator::{SignalGeneratorPanel, GeneratorState},
    accessibility::{AccessibilityManager, AccessibilityAction},
    enhanced_controls::{AccessibleSlider, AccessibleKnob},
    enhanced_button::{AccessibleButton, ProgressIndicator, VolumeSafetyIndicator},
    error_handling::{ErrorManager, RecoveryActionType},
};
use testing::signal_generators::*;

#[derive(Debug, Clone, PartialEq)]
enum Tab {
    Playback,
    Effects,
    Eq,
    Generator,
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

struct AudioPlayerApp {
    audio_context: AudioContext,
    source_node: Option<web_audio_api::node::AudioBufferSourceNode>,
    gain_node: web_audio_api::node::GainNode,
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

    // Enhanced controls (legacy - to be replaced)
    volume_slider: EnhancedSlider,
    eq_knobs: Vec<CircularKnob>,

    // Accessibility and enhanced controls
    accessibility_manager: AccessibilityManager,
    accessible_volume_slider: AccessibleSlider,
    accessible_eq_knobs: Vec<AccessibleKnob>,
    file_loading_progress: Option<ProgressIndicator>,
    volume_safety_indicator: VolumeSafetyIndicator,
    error_manager: ErrorManager,

    // Audio processing
    spectrum: Vec<f32>,
    eq_bands: Vec<BiquadFilterNode>,
    analyser: AnalyserNode,
    spectrum_processor: audio_performance::SpectrumProcessor,

    // Responsive and animation state
    last_frame_time: Instant,
    screen_size: ScreenSize,
    show_keyboard_shortcuts: bool,
}

impl Default for AudioPlayerApp {
    fn default() -> Self {
        let audio_context = AudioContext::default();
        let analyser = audio_context.create_analyser();
        let gain_node = audio_context.create_gain();
        gain_node.gain().set_value(0.5);

        let mut eq_bands = Vec::new();
        let mut eq_knobs = Vec::new();
        let mut accessible_eq_knobs = Vec::new();

        for i in 0..8 {
            let mut band = audio_context.create_biquad_filter();
            band.set_type(BiquadFilterType::Peaking);
            band.frequency().set_value(60.0 * 2.0_f32.powi(i));
            band.q().set_value(1.0);
            band.gain().set_value(0.0);
            eq_bands.push(band);

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
                .description(format!("Equalizer gain for {} frequency band",
                    if freq < 1000.0 { format!("{:.0} Hz", freq) } else { format!("{:.1} kHz", freq / 1000.0) }))
                .step_size(0.5)
            );
        }

        Self {
            audio_context,
            source_node: None,
            gain_node,
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
            theme_manager: ThemeManager::new(Theme::Mocha),
            layout_manager: LayoutManager::new(),
            spectrum_visualizer: SpectrumVisualizer::new(SpectrumVisualizerConfig::default()),
            album_art_display: AlbumArtDisplay::new(Vec2::new(200.0, 200.0)),
            progress_bar: ProgressBar::new(),
            metadata_display: MetadataDisplay::new(),

            // Enhanced controls (legacy - to be replaced)
            volume_slider: EnhancedSlider::new(0.5, 0.0..=1.0)
                .orientation(SliderOrientation::Horizontal)
                .style(SliderStyle::default()),
            eq_knobs,

            // Accessibility and enhanced controls
            accessibility_manager: AccessibilityManager::new(),
            accessible_volume_slider: AccessibleSlider::new(
                egui::Id::new("volume_slider"),
                0.5,
                0.0..=1.0,
                "Volume"
            )
            .description("Master audio volume control")
            .safety_info("Keep volume below 80% to protect hearing")
            .step_size(0.05),
            accessible_eq_knobs,
            file_loading_progress: None,
            volume_safety_indicator: VolumeSafetyIndicator::new(),
            error_manager: ErrorManager::new(),

            // Audio processing
            spectrum: vec![0.0; 1024],
            eq_bands,
            analyser,
            spectrum_processor: audio_performance::SpectrumProcessor::new(2048),

            // Responsive and animation state
            last_frame_time: Instant::now(),
            screen_size: ScreenSize::Desktop,
            show_keyboard_shortcuts: false,
        }
    }
}

impl eframe::App for AudioPlayerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update responsive layout and timing
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Update screen size and responsive layout
        let screen_size_vec = ctx.screen_rect().size();
        self.screen_size = ScreenSize::from_width(screen_size_vec.x);
        self.layout_manager.update_responsive_layout(screen_size_vec);

        // Apply theme (with accessibility enhancements)
        self.theme_manager.apply_theme(ctx);
        let base_colors = self.theme_manager.get_colors();
        let colors = self.accessibility_manager.get_accessible_colors(&base_colors);

        // Update animations
        self.layout_manager.update_animations(dt);

        // Update accessibility system
        self.accessibility_manager.update(
            &egui::Ui::new(
                ctx.clone(),
                egui::LayerId::background(),
                egui::Id::new("accessibility_ui"),
                egui::Rect::EVERYTHING,
                egui::Rect::EVERYTHING,
            ),
            dt
        );

        // Handle accessibility actions
        let accessibility_action = self.accessibility_manager.handle_keyboard_input(
            &egui::Ui::new(
                ctx.clone(),
                egui::LayerId::background(),
                egui::Id::new("accessibility_input"),
                egui::Rect::EVERYTHING,
                egui::Rect::EVERYTHING,
            )
        );

        match accessibility_action {
            AccessibilityAction::EmergencyVolumeReduction => {
                let emergency_volume = self.accessibility_manager.get_volume_safety_status();
                self.volume = 0.2; // Emergency volume level
                self.gain_node.gain().set_value(self.volume);
                self.accessible_volume_slider.set_value(self.volume);
                self.accessibility_manager.announce(
                    "Emergency volume reduction activated".to_string(),
                    ui::accessibility::AnnouncementPriority::Critical,
                );
            },
            AccessibilityAction::ToggleHelp => {
                // Help system is handled within accessibility manager
            },
            AccessibilityAction::ToggleHighContrast => {
                self.accessibility_manager.announce(
                    format!("High contrast mode {}",
                        if self.accessibility_manager.is_high_contrast_mode() { "enabled" } else { "disabled" }),
                    ui::accessibility::AnnouncementPriority::Medium,
                );
            },
            AccessibilityAction::AdjustFocusedControl(delta) => {
                // This would be handled by the focused control itself
            },
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

        // Update signal generator
        self.signal_generator_panel.update(dt);

        // Main UI layout using the new layout system
        if self.screen_size == ScreenSize::Mobile {
            self.draw_mobile_layout(ctx, &colors);
        } else {
            self.draw_desktop_layout(ctx, &colors);
        }

        // Show accessibility help overlay
        self.accessibility_manager.show_help_overlay(
            &egui::Ui::new(
                ctx.clone(),
                egui::LayerId::top(),
                egui::Id::new("help_overlay"),
                egui::Rect::EVERYTHING,
                egui::Rect::EVERYTHING,
            ),
            &colors
        );

        // Show error dialogs and handle recovery actions
        let recovery_actions = self.error_manager.show_errors(
            &mut egui::Ui::new(
                ctx.clone(),
                egui::LayerId::top(),
                egui::Id::new("error_display"),
                egui::Rect::EVERYTHING,
                egui::Rect::EVERYTHING,
            ),
            &colors,
            &mut self.accessibility_manager
        );

        // Execute recovery actions
        for action in recovery_actions {
            match action {
                RecoveryActionType::Retry => {
                    if self.current_file.is_some() {
                        self.load_current_file();
                    }
                },
                RecoveryActionType::SelectDifferentFile => {
                    self.open_file_dialog();
                },
                RecoveryActionType::ResetSettings => {
                    self.reset_all_settings();
                },
                RecoveryActionType::CheckPermissions => {
                    self.accessibility_manager.announce(
                        "Please check that the file is accessible and not in use by another application".to_string(),
                        ui::accessibility::AnnouncementPriority::High,
                    );
                },
                RecoveryActionType::ContactSupport => {
                    self.show_format_help();
                },
                RecoveryActionType::Dismiss => {
                    // Already handled by error manager
                },
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
    fn update_ui_components(&mut self, colors: &ThemeColors) {
        // Update progress bar
        self.progress_bar.set_progress(
            self.playback_pos.as_secs_f32(),
            self.total_duration.as_secs_f32()
        );

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
        self.album_art_display.set_texture(self.album_art.as_ref().map(|arc| (**arc).clone()));

        // Update spectrum visualizer
        self.spectrum_visualizer.update(&self.spectrum);

        // Update volume slider
        self.volume_slider.set_value(self.volume);
    }

    fn draw_desktop_layout(&mut self, ctx: &egui::Context, colors: &ThemeColors) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎵 Rusty Audio");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Theme selector
                    egui::ComboBox::from_label("")
                        .selected_text(self.theme_manager.current_theme().display_name())
                        .show_ui(ui, |ui| {
                            for theme in Theme::all() {
                                ui.selectable_value(
                                    &mut self.theme_manager,
                                    ThemeManager::new(theme.clone()),
                                    theme.display_name()
                                );
                            }
                        });

                    if ui.button("?").clicked() {
                        self.show_keyboard_shortcuts = !self.show_keyboard_shortcuts;
                    }
                });
            });

            ui.separator();

            // Tab selection
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Playback, "🎵 Playback");
                ui.selectable_value(&mut self.active_tab, Tab::Effects, "🎛️ Effects");
                ui.selectable_value(&mut self.active_tab, Tab::Eq, "📊 EQ");
                ui.selectable_value(&mut self.active_tab, Tab::Generator, "🎛️ Generator");
                ui.selectable_value(&mut self.active_tab, Tab::Settings, "⚙️ Settings");
            });

            ui.separator();

            match self.active_tab {
                Tab::Playback => self.draw_playback_panel(ui, colors),
                Tab::Effects => self.draw_effects_panel(ui, colors),
                Tab::Eq => self.draw_eq_panel(ui, colors),
                Tab::Generator => self.draw_generator_panel(ui, colors),
                Tab::Settings => self.draw_settings_panel_main(ui, colors),
            }
        });
    }

    fn draw_mobile_layout(&mut self, ctx: &egui::Context, colors: &ThemeColors) {
        // Mobile layout with bottom tab bar
        egui::TopBottomPanel::bottom("mobile_tabs").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Playback, "🎵");
                ui.selectable_value(&mut self.active_tab, Tab::Effects, "🎛️");
                ui.selectable_value(&mut self.active_tab, Tab::Eq, "📊");
                ui.selectable_value(&mut self.active_tab, Tab::Generator, "🎛️");
                ui.selectable_value(&mut self.active_tab, Tab::Settings, "⚙️");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Playback => self.draw_mobile_playback_panel(ui, colors),
                Tab::Effects => self.draw_mobile_effects_panel(ui, colors),
                Tab::Eq => self.draw_mobile_eq_panel(ui, colors),
                Tab::Generator => self.draw_mobile_generator_panel(ui, colors),
                Tab::Settings => self.draw_settings_panel_main(ui, colors),
            }
        });
    }

    fn draw_playback_panel(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);

            // Album art with enhanced display
            let album_art_response = self.album_art_display.show(ui, colors);

            ui.add_space(10.0);

            // Metadata display
            self.metadata_display.show(ui, colors);

            ui.add_space(15.0);

            // Enhanced progress bar
            let progress_response = self.progress_bar.show(ui, colors);
            if progress_response.changed() {
                let position_seconds = self.progress_bar.progress * self.total_duration.as_secs_f32();
                self.seek_to_position_main(position_seconds);
            }

            ui.add_space(15.0);

            // Control buttons with enhanced styling
            ui.horizontal_centered(|ui| {
                let mut open_button = EnhancedButton::new("📁 Open")
                    .style(ButtonStyle {
                        glow: true,
                        ..Default::default()
                    });
                if open_button.show(ui, colors).clicked() {
                    self.open_file_dialog();
                }

                ui.add_space(10.0);

                let play_pause_text = if self.playback_state == PlaybackState::Playing { "⏸️ Pause" } else { "▶️ Play" };
                let mut play_pause_button = EnhancedButton::new(play_pause_text)
                    .style(ButtonStyle {
                        gradient: true,
                        glow: true,
                        ..Default::default()
                    });
                if play_pause_button.show(ui, colors).clicked() {
                    self.play_pause_main();
                }

                ui.add_space(10.0);

                let mut stop_button = EnhancedButton::new("⏹️ Stop");
                if stop_button.show(ui, colors).clicked() {
                    self.stop_playback_main();
                }

                ui.add_space(10.0);

                let loop_text = if self.is_looping { "🔁 Loop: On" } else { "🔁 Loop: Off" };
                let mut loop_button = EnhancedButton::new(loop_text)
                    .style(ButtonStyle {
                        gradient: self.is_looping,
                        ..Default::default()
                    });
                if loop_button.show(ui, colors).clicked() {
                    self.toggle_loop_main();
                }
            });

            ui.add_space(20.0);

            // Volume control with accessible slider
            ui.horizontal_centered(|ui| {
                ui.label("🔊 Volume:");
                ui.add_space(10.0);

                ui.vertical(|ui| {
                    let volume_response = self.accessible_volume_slider.show(ui, colors, &mut self.accessibility_manager);
                    if volume_response.changed() {
                        self.volume = self.accessible_volume_slider.value();
                        self.gain_node.gain().set_value(self.volume);

                        // Check volume safety
                        if !self.accessibility_manager.is_volume_safe(self.volume) {
                            self.accessibility_manager.announce(
                                "Warning: Volume level may be harmful to hearing".to_string(),
                                ui::accessibility::AnnouncementPriority::High,
                            );
                        }
                    }

                    // Show volume safety indicator
                    self.volume_safety_indicator.show(ui, colors);
                });
            });

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
                let position_seconds = self.progress_bar.progress * self.total_duration.as_secs_f32();
                self.seek_to_position_main(position_seconds);
            }

            ui.add_space(10.0);

            // Compact controls
            ui.horizontal_centered(|ui| {
                if ui.button("📁").clicked() { self.open_file_dialog(); }
                ui.add_space(15.0);

                let play_pause_icon = if self.playback_state == PlaybackState::Playing { "⏸️" } else { "▶️" };
                if ui.button(play_pause_icon).clicked() { self.play_pause_main(); }
                ui.add_space(15.0);

                if ui.button("⏹️").clicked() { self.stop_playback_main(); }
                ui.add_space(15.0);

                let loop_icon = if self.is_looping { "🔁" } else { "🔁" };
                if ui.button(loop_icon).clicked() { self.toggle_loop_main(); }
            });

            ui.add_space(10.0);

            // Compact volume
            ui.horizontal_centered(|ui| {
                ui.label("🔊");
                let volume_response = self.volume_slider.show(ui, colors);
                if volume_response.changed() {
                    self.volume = self.volume_slider.value();
                    self.gain_node.gain().set_value(self.volume);
                }
            });
        });
    }


    fn draw_legacy_effects_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Spectrum");
        let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 200.0), egui::Sense::hover());
        let painter = ui.painter();
        let color = Color32::from_rgb(0, 128, 255);
        let num_points = self.spectrum.len();
        let point_spacing = rect.width() / num_points as f32;

        let mut points = Vec::with_capacity(num_points);
        for (i, val) in self.spectrum.iter().enumerate() {
            let x = rect.min.x + i as f32 * point_spacing;
            let y = rect.center().y - val * rect.height() / 2.0;
            points.push(egui::pos2(x, y));
        }

        if points.len() > 1 {
            painter.add(egui::Shape::Path(egui::epaint::PathShape {
                points,
                closed: false,
                fill: Color32::TRANSPARENT,
                stroke: egui::Stroke::new(1.0, color),
            }));
        }
    }


    fn draw_legacy_eq_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Equalizer");
                if ui.button("Reset").clicked() {
                    for band in &mut self.eq_bands {
                        band.gain().set_value(0.0);
                    }
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                for (i, band) in self.eq_bands.iter_mut().enumerate() {
                    ui.vertical(|ui| {
                        ui.label(format!("{} Hz", 60 * 2_i32.pow(i as u32)));
                        let mut gain = band.gain().value();
                        if ui.add(egui::Slider::new(&mut gain, -40.0..=40.0).vertical()).changed() {
                            band.gain().set_value(gain);
                        }
                    });
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Master Gain:");
                let mut master_gain = self.gain_node.gain().value();
                if ui.add(egui::Slider::new(&mut master_gain, 0.0..=2.0)).changed() {
                    self.gain_node.gain().set_value(master_gain);
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
            ui.label(RichText::new("🎛️ Signal Generator").size(18.0).color(colors.text));
            ui.add_space(10.0);

            // Simplified signal generator for mobile
            self.signal_generator_panel.show(ui, colors);
        });
    }

    fn draw_legacy_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Theme");
        egui::ComboBox::from_label("Select a theme")
            .selected_text(self.theme_manager.current_theme().display_name())
            .show_ui(ui, |ui| {
                for theme in Theme::all() {
                    let mut current_theme = self.theme_manager.current_theme().clone();
                    if ui.selectable_value(&mut current_theme, theme.clone(), theme.display_name()).clicked() {
                        self.theme_manager.set_theme(theme);
                    }
                }
            });
        ui.add_space(20.0);
        ui.label("Audio Device selection is not supported with the web-audio-api backend.");
    }



    fn tick(&mut self) {
        // Use optimized spectrum processor for better performance
        let spectrum_data = self.spectrum_processor.process_spectrum(&mut self.analyser);

        // Copy the optimized spectrum data (already normalized to 0-1 range)
        self.spectrum = spectrum_data.to_vec();

        if self.playback_state == PlaybackState::Playing && !self.is_seeking {
            self.playback_pos = Duration::from_secs_f64(self.audio_context.current_time());
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
                self.gain_node.gain().set_value(self.volume);
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                self.volume = (self.volume - 0.05).max(0.0);
                self.gain_node.gain().set_value(self.volume);
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
        if let Some(source_node) = &mut self.source_node {
            let new_pos = self.playback_pos.saturating_sub(Duration::from_secs(5));
            self.seek_to_position_main(new_pos.as_secs_f32());
        }
    }

    fn seek_forward(&mut self) {
        if let Some(source_node) = &mut self.source_node {
            let new_pos = self.playback_pos.saturating_add(Duration::from_secs(5));
            self.seek_to_position_main(new_pos.as_secs_f32());
        }
    }



    fn legacy_handle_keyboard_input(&mut self, ui: &mut egui::Ui) {
        ui.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.play_pause_main();
            }
            if i.key_pressed(egui::Key::S) {
                self.stop();
            }
            if i.key_pressed(egui::Key::L) {
                self.is_looping = !self.is_looping;
                if let Some(source_node) = &mut self.source_node {
                    source_node.set_loop(self.is_looping);
                }
            }
            if i.key_pressed(egui::Key::ArrowUp) {
                self.volume = (self.volume + 0.05).min(1.0);
                self.gain_node.gain().set_value(self.volume);
            }
            if i.key_pressed(egui::Key::ArrowDown) {
                self.volume = (self.volume - 0.05).max(0.0);
                self.gain_node.gain().set_value(self.volume);
            }
            if i.key_pressed(egui::Key::ArrowLeft) {
                if let Some(source_node) = &mut self.source_node {
                    let new_pos = self.playback_pos.saturating_sub(Duration::from_secs(5));
                    source_node.stop();
                    source_node.start_at(self.audio_context.current_time() + new_pos.as_secs_f64());
                    self.playback_pos = new_pos;
                }
            }
            if i.key_pressed(egui::Key::ArrowRight) {
                if let Some(source_node) = &mut self.source_node {
                    let new_pos = self.playback_pos.saturating_add(Duration::from_secs(5));
                    source_node.stop();
                    source_node.start_at(self.audio_context.current_time() + new_pos.as_secs_f64());
                    self.playback_pos = new_pos;
                }
            }
        });
    }


    fn load_current_file(&mut self) {
        if let Some(handle) = &self.current_file {
            let path = handle.path();
            let filename = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown file");

            // Clear previous errors
            self.error = None;

            // Load metadata
            if let Ok(tagged_file) = lofty::read_from_path(path) {
                if let Some(tag) = tagged_file.primary_tag() {
                    self.metadata = Some(TrackMetadata {
                        title: tag.title().as_deref().unwrap_or("Unknown Title").into(),
                        artist: tag.artist().as_deref().unwrap_or("Unknown Artist").into(),
                        album: tag.album().as_deref().unwrap_or("Unknown Album").into(),
                        year: tag.year().map(|y| y.to_string()).unwrap_or_else(|| "----".into()),
                    });
                }
                // Album art will be loaded separately when context is available
                self.album_art = None;
            }

            // Load and decode audio
            match std::fs::File::open(path) {
                Ok(file) => {
                    match self.audio_context.decode_audio_data_sync(file) {
                        Ok(buffer) => {
                            self.total_duration = Duration::from_secs_f64(buffer.duration());

                            let mut source_node = self.audio_context.create_buffer_source();
                            source_node.set_buffer(buffer);

                            source_node.connect(&self.gain_node);
                            let mut previous_node: &dyn AudioNode = &self.gain_node;
                            for band in &self.eq_bands {
                                previous_node.connect(band);
                                previous_node = band;
                            }
                            previous_node.connect(&self.analyser);
                            self.analyser.connect(&self.audio_context.destination());

                            source_node.start();
                            self.source_node = Some(source_node);
                            self.playback_state = PlaybackState::Playing;
                            self.playback_pos = Duration::ZERO;

                            // Announce successful load
                            self.accessibility_manager.announce(
                                format!("Audio file loaded: {}", filename),
                                ui::accessibility::AnnouncementPriority::Medium,
                            );
                        },
                        Err(decode_error) => {
                            self.error_manager.add_audio_decode_error(
                                filename,
                                Some("audio") // Could be enhanced to detect actual format
                            );
                            self.error = Some("Failed to decode audio file".to_string());
                        }
                    }
                },
                Err(io_error) => {
                    if io_error.kind() == std::io::ErrorKind::PermissionDenied {
                        self.error_manager.add_permission_error("access", path.to_str().unwrap_or(filename));
                    } else {
                        self.error_manager.add_file_load_error(
                            filename,
                            Some(format!("IO Error: {}", io_error))
                        );
                    }
                    self.error = Some("Failed to open audio file".to_string());
                }
            }
        }
    }

    fn reset_all_settings(&mut self) {
        // Reset equalizer
        for (band, knob) in self.eq_bands.iter_mut().zip(self.accessible_eq_knobs.iter_mut()) {
            band.gain().set_value(0.0);
            knob.set_value(0.0);
        }

        // Reset volume to safe level
        self.volume = 0.5;
        self.gain_node.gain().set_value(self.volume);
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
            "Supported audio formats: MP3, WAV, FLAC, OGG, M4A. Ensure your file is not corrupted.".to_string(),
            ui::accessibility::AnnouncementPriority::Medium,
        );
    }

    fn load_album_art(&mut self, ctx: &egui::Context) {
        if let Some(handle) = &self.current_file {
            let path = handle.path();
            if let Ok(tagged_file) = lofty::read_from_path(path) {
                if let Some(picture) = tagged_file.primary_tag().and_then(|t| t.pictures().get(0)) {
                    if let Ok(img) = image::load_from_memory(picture.data()) {
                        let (width, height) = img.dimensions();
                        let rgba = img.to_rgba8();
                        let pixels = rgba.into_raw();
                        let image = egui::ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &pixels);
                        self.album_art = Some(Arc::new(ctx.load_texture("album-art", image, Default::default())));
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
                self.playback_state = PlaybackState::Paused;
                if let Some(source) = &mut self.source_node {
                    source.stop();
                }
            },
            PlaybackState::Paused | PlaybackState::Stopped => {
                if self.current_file.is_some() {
                    self.load_current_file();
                } else if !self.signal_generator_panel.generated_samples.is_empty() {
                    self.play_generated_signal();
                }
            },
        }
    }

    fn stop_playback_main(&mut self) {
        self.playback_state = PlaybackState::Stopped;
        self.playback_pos = Duration::ZERO;
        if let Some(source) = &mut self.source_node {
            source.stop();
        }
        self.source_node = None;
    }

    fn toggle_loop_main(&mut self) {
        self.is_looping = !self.is_looping;
        if let Some(source_node) = &mut self.source_node {
            source_node.set_loop(self.is_looping);
        }
    }

    fn seek_to_position_main(&mut self, position_seconds: f32) {
        if let Some(source_node) = &mut self.source_node {
            let new_pos = Duration::from_secs_f32(position_seconds.clamp(0.0, self.total_duration.as_secs_f32()));

            // For simplicity, restart playback at the new position
            source_node.stop();

            if self.current_file.is_some() {
                self.playback_pos = new_pos;
                self.load_current_file();
            } else if !self.signal_generator_panel.generated_samples.is_empty() {
                self.playback_pos = new_pos;
                self.play_generated_signal();
            }
        }
    }

    fn play_generated_signal(&mut self) {
        if let Some(buffer) = self.signal_generator_panel.create_audio_buffer(&self.audio_context) {
            // Stop any currently playing source
            if let Some(source) = &mut self.source_node {
                source.stop();
            }

            // Create new source node
            let mut source_node = self.audio_context.create_buffer_source();
            source_node.set_buffer(buffer);

            // Connect to audio graph
            source_node.connect(&self.gain_node);
            let mut previous_node: &dyn AudioNode = &self.gain_node;
            for band in &self.eq_bands {
                previous_node.connect(band);
                previous_node = band;
            }
            previous_node.connect(&self.analyser);
            self.analyser.connect(&self.audio_context.destination());

            // Start playback
            source_node.start();
            self.source_node = Some(source_node);
            self.playback_state = PlaybackState::Playing;

            // Set duration for progress tracking
            self.total_duration = Duration::from_secs_f32(self.signal_generator_panel.parameters.duration);
            self.playback_pos = Duration::ZERO;
        }
    }






    fn draw_settings_panel_main(&mut self, ui: &mut egui::Ui, colors: &ThemeColors) {
        ui.vertical(|ui| {
            ui.heading("Settings");
            ui.add_space(10.0);

            ui.group(|ui| {
                ui.label("Theme");
                egui::ComboBox::from_label("Select a theme")
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

            ui.group(|ui| {
                ui.label("Audio Settings");
                ui.label("Audio device selection is not supported with the web-audio-api backend.");
            });

            ui.add_space(15.0);

            ui.group(|ui| {
                ui.label("About");
                ui.label("Rusty Audio Player v0.1.0");
                ui.label("Built with Rust and egui");
            });
        });
    }

}

fn main() -> Result<(), eframe::Error> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(450.0, 600.0))
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "Rusty Audio",
        options,
        Box::new(|_cc| Box::new(AudioPlayerApp::default())),
    )
}
