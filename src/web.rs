//! WebAssembly entry point and browser integration
//!
//! This module provides the WASM-specific initialization and runtime
//! for running rusty-audio in a web browser environment.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use eframe::egui;

#[cfg(target_arch = "wasm32")]
use crate::{
    audio::AudioConfig,
    integrated_audio_manager::{IntegratedAudioManager, PlaybackState},
    ui::{
        signal_generator::SignalGeneratorPanel,
        theme::{Theme, ThemeColors, ThemeManager},
    },
};

#[cfg(target_arch = "wasm32")]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
/// Application tabs for navigation
/// Source: Adapted from src/main.rs:93-160 (desktop Tab enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppTab {
    Generator,
    Equalizer,
    Effects,
    Settings,
}

#[cfg(target_arch = "wasm32")]
impl AppTab {
    fn label(&self) -> &'static str {
        match self {
            AppTab::Generator => "🎵 Generator",
            AppTab::Equalizer => "📊 Equalizer",
            AppTab::Effects => "🎛️ Effects",
            AppTab::Settings => "⚙️ Settings",
        }
    }

    fn all() -> &'static [AppTab] {
        &[
            AppTab::Generator,
            AppTab::Equalizer,
            AppTab::Effects,
            AppTab::Settings,
        ]
    }
}

#[cfg(target_arch = "wasm32")]
/// WASM Audio Player Application
///
/// Web-compatible version of the Rusty Audio player using IntegratedAudioManager
/// with Web Audio API backend
struct WasmAudioApp {
    // Audio management
    audio_manager: Option<IntegratedAudioManager>,
    initialization_error: Option<String>,

    // UI state
    signal_generator_panel: SignalGeneratorPanel,
    theme_manager: ThemeManager,
    active_tab: AppTab,

    // EQ state (Phase 2) - 8 bands: 60Hz, 120Hz, 240Hz, 480Hz, 960Hz, 1.9kHz, 3.8kHz, 7.7kHz
    eq_gains: [f32; 8],

    // Playback state
    volume: f32,
    error_message: Option<String>,

    // Status
    last_update: Instant,
}

#[cfg(target_arch = "wasm32")]
impl Default for WasmAudioApp {
    fn default() -> Self {
        log::info!("Initializing WASM Audio Player...");

        // Initialize audio manager with Web Audio backend
        let audio_config = AudioConfig {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 512,
            exclusive_mode: false, // Not applicable to WASM
            ..Default::default()
        };

        let (audio_manager, initialization_error) =
            match IntegratedAudioManager::new(512, audio_config) {
                Ok(mut manager) => {
                    log::info!("IntegratedAudioManager created successfully");

                    // Initialize output device (Web Audio context)
                    match manager.initialize_output_device(None) {
                        Ok(_) => {
                            log::info!("Web Audio output initialized");
                            (Some(manager), None)
                        }
                        Err(e) => {
                            let error = format!("Failed to initialize Web Audio output: {}", e);
                            log::error!("{}", error);
                            (None, Some(error))
                        }
                    }
                }
                Err(e) => {
                    let error = format!("Failed to create audio manager: {}", e);
                    log::error!("{}", error);
                    (None, Some(error))
                }
            };

        Self {
            audio_manager,
            initialization_error,
            signal_generator_panel: SignalGeneratorPanel::new(),
            theme_manager: ThemeManager::new(Theme::StudioDark),
            active_tab: AppTab::Generator,
            eq_gains: [0.0; 8], // All bands start at 0 dB (flat response)
            volume: 0.5,
            error_message: None,
            last_update: Instant::now(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl eframe::App for WasmAudioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let _dt = (now - self.last_update).as_secs_f32();
        self.last_update = now;

        // Apply theme
        let theme = self.theme_manager.current_theme();
        ctx.set_visuals(theme.to_egui_visuals());

        // Request repaint for animations
        ctx.request_repaint();

        // Top panel with title and status
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎵 Rusty Audio Player (WASM)");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ref manager) = self.audio_manager {
                        let playback_state = manager.playback_state();
                        let status_text = match playback_state {
                            PlaybackState::Playing => "▶ Playing",
                            PlaybackState::Stopped => "⏸ Stopped",
                            PlaybackState::Paused => "⏸ Paused",
                        };
                        ui.label(status_text);
                    } else {
                        ui.colored_label(egui::Color32::RED, "⚠ Audio Unavailable");
                    }
                });
            });
        });

        // Tab navigation panel
        // Source: Adapted from src/main.rs:400-500 (desktop tab switching)
        egui::TopBottomPanel::top("tabs_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for tab in AppTab::all() {
                    let is_active = self.active_tab == *tab;
                    if ui.selectable_label(is_active, tab.label()).clicked() {
                        self.active_tab = *tab;
                        log::info!("Switched to tab: {:?}", tab);
                    }
                }
            });
        });

        // Main content panel with tab-based content
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show initialization error if any
            if let Some(ref error) = self.initialization_error {
                ui.colored_label(
                    egui::Color32::RED,
                    format!("Initialization Error: {}", error),
                );
                ui.label("The audio system failed to initialize. Audio features will not work.");
                ui.separator();
            }

            // Show current error message if any
            if let Some(ref error) = self.error_message {
                ui.colored_label(egui::Color32::YELLOW, format!("Error: {}", error));
                ui.separator();
            }

            // Render active tab content
            match self.active_tab {
                AppTab::Generator => self.draw_generator_panel(ui),
                AppTab::Equalizer => self.draw_equalizer_panel(ui),
                AppTab::Effects => self.draw_effects_panel(ui),
                AppTab::Settings => self.draw_settings_panel(ui),
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
impl WasmAudioApp {
    /// Draw signal generator panel
    /// Source: Refactored from original update() method lines 150-239
    fn draw_generator_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Signal Generator");
        ui.separator();

        // Signal generator panel
        if let Some(ref mut audio_manager) = self.audio_manager {
            // Draw signal generator UI
            let colors = self.theme_manager.current_theme().colors();
            self.signal_generator_panel.show(ui, &colors);

            // Handle signal generator routing intents
            if let Some(intent) = self.signal_generator_panel.take_route_intent() {
                log::info!(
                    "Processing route intent: {} -> {:?}",
                    intent.label,
                    intent.mode
                );

                if let Some(output) = self.signal_generator_panel.output_snapshot() {
                    match audio_manager.play_signal_generator(
                        output.samples,
                        output.sample_rate,
                        false, // Don't loop for now
                    ) {
                        Ok(_) => {
                            log::info!("Signal generator started successfully");
                            self.error_message = None;
                        }
                        Err(e) => {
                            let error = format!("Failed to start playback: {}", e);
                            log::error!("{}", error);
                            self.error_message = Some(error);
                        }
                    }
                }
            }

            // Stop button
            ui.add_space(10.0);
            if ui.button("⏹ Stop Playback").clicked() {
                match audio_manager.stop_signal_generator() {
                    Ok(_) => {
                        log::info!("Playback stopped");
                        self.error_message = None;
                    }
                    Err(e) => {
                        let error = format!("Failed to stop playback: {}", e);
                        log::error!("{}", error);
                        self.error_message = Some(error);
                    }
                }
            }

            // Volume control
            ui.add_space(10.0);
            ui.label("Master Volume:");
            let volume_response =
                ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));

            // Apply volume changes to audio engine
            if volume_response.changed() {
                use crate::integrated_audio_manager::RouteType;
                if let Err(e) =
                    audio_manager.set_route_gain(RouteType::SignalGeneratorPlayback, self.volume)
                {
                    log::error!("Failed to set volume: {}", e);
                }
            }

            // Audio processing (call process periodically)
            if let Err(e) = audio_manager.process() {
                log::error!("Audio processing error: {}", e);
            }
        } else {
            ui.label("Audio system not available");
            ui.label("Please check the console for initialization errors");
        }

        ui.add_space(20.0);
        ui.separator();

        // Status information
        ui.heading("System Information");
        ui.label(format!(
            "Web Audio API: {}",
            if self.audio_manager.is_some() {
                "Active"
            } else {
                "Unavailable"
            }
        ));
        ui.label("Sample Rate: 48000 Hz");
        ui.label("Channels: 2 (Stereo)");
        ui.label("Buffer Size: 512 samples");
    }

    /// Draw equalizer panel
    /// Source: Adapted from src/main.rs:889-982 (AudioPlayerApp::draw_eq_panel)
    fn draw_equalizer_panel(&mut self, ui: &mut egui::Ui) {
        let colors = self.theme_manager.current_theme().colors();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("📊 Equalizer").color(colors.text));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Reset All").clicked() {
                        // Reset all EQ bands to 0.0 dB in UI
                        for i in 0..8 {
                            self.eq_gains[i] = 0.0;
                        }

                        // Reset EQ in audio backend
                        if let Some(ref mut audio_manager) = self.audio_manager {
                            if let Err(e) = audio_manager.reset_eq() {
                                self.error_message = Some(format!("Failed to reset EQ: {}", e));
                                log::error!("Failed to reset EQ: {}", e);
                            } else {
                                log::info!("All EQ bands reset to flat response");
                            }
                        }
                    }
                });
            });

            ui.add_space(15.0);

            // EQ bands with vertical sliders
            ui.horizontal(|ui| {
                for i in 0..8 {
                    ui.vertical(|ui| {
                        // Frequency label
                        let freq = 60.0 * 2.0_f32.powi(i as i32);
                        let freq_label = if freq < 1000.0 {
                            format!("{:.0} Hz", freq)
                        } else {
                            format!("{:.1}k Hz", freq / 1000.0)
                        };
                        ui.label(
                            egui::RichText::new(&freq_label)
                                .color(colors.text)
                                .size(12.0),
                        );

                        // Vertical slider for EQ gain
                        let slider_response = ui.add(
                            egui::Slider::new(&mut self.eq_gains[i], -12.0..=12.0)
                                .vertical()
                                .show_value(false)
                                .fixed_decimals(1),
                        );

                        if slider_response.changed() {
                            let gain_value = self.eq_gains[i];
                            // Set EQ band in audio backend
                            if let Some(ref mut audio_manager) = self.audio_manager {
                                if let Err(e) = audio_manager.set_eq_band(i, gain_value) {
                                    self.error_message =
                                        Some(format!("Failed to set EQ band {}: {}", i, e));
                                    log::error!("Failed to set EQ band {}: {}", i, e);
                                } else {
                                    log::debug!("EQ band {} set to {:.1} dB", i, gain_value);
                                }
                            }
                        }

                        // Gain value display
                        ui.label(
                            egui::RichText::new(format!("{:.1}dB", self.eq_gains[i]))
                                .color(colors.text_secondary)
                                .size(10.0),
                        );
                    });

                    if i < 7 {
                        ui.add_space(5.0);
                    }
                }
            });

            ui.add_space(20.0);

            // Status info
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Status:").color(colors.text));
                let status_text = if self.audio_manager.is_some() {
                    "✓ EQ Active (Web Audio API)"
                } else {
                    "⚠ EQ Not Initialized"
                };
                let status_color = if self.audio_manager.is_some() {
                    egui::Color32::from_rgb(100, 200, 100)
                } else {
                    colors.text_secondary
                };
                ui.label(egui::RichText::new(status_text).color(status_color));
            });
        });
    }

    /// Draw effects/spectrum panel (placeholder for Phase 3)
    /// Will be implemented in Phase 3 with code from src/main.rs:838-887
    fn draw_effects_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎛️ Audio Effects & Spectrum");
        ui.separator();
        ui.add_space(20.0);

        ui.vertical_centered(|ui| {
            ui.label("Real-Time Spectrum Analyzer");
            ui.add_space(10.0);
            ui.label("Coming in Phase 3...");
            ui.add_space(10.0);
            ui.label("Features:");
            ui.label("• Real-time FFT spectrum display");
            ui.label("• Multiple visualization modes (Bars, Line, Filled, Circular)");
            ui.label("• Frequency scale options");
            ui.label("• Peak hold and smoothing");
        });
    }

    /// Draw settings panel (placeholder for Phase 4)
    /// Will be implemented in Phase 4 with code from src/main.rs:1863-1950
    fn draw_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Settings");
        ui.separator();
        ui.add_space(20.0);

        ui.vertical_centered(|ui| {
            ui.label("Application Settings");
            ui.add_space(10.0);
            ui.label("Coming in Phase 4...");
            ui.add_space(10.0);
            ui.label("Features:");
            ui.label("• Theme selection (8 themes)");
            ui.label("• Display settings");
            ui.label("• Settings persistence (localStorage)");
            ui.label("• About information");
        });
    }
}

#[cfg(target_arch = "wasm32")]
/// WebHandle manages the eframe WebRunner instance for browser deployment
#[derive(Clone)]
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebHandle {
    /// Create a new WebHandle and initialize logging
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Set up panic hook for better error messages in browser console
        console_error_panic_hook::set_once();

        // Initialize logging to browser console
        if let Err(err) = console_log::init_with_level(log::Level::Debug) {
            web_sys::console::error_1(&format!("Console logging init failed: {}", err).into());
        }

        log::info!("Rusty Audio WASM initializing...");

        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    /// Start the application on the specified canvas element
    ///
    /// # Arguments
    /// * `canvas` - The HTML canvas element to render into
    #[wasm_bindgen]
    pub async fn start(&self, canvas: web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
        log::info!("Starting Rusty Audio on canvas: {:?}", canvas.id());

        let web_options = eframe::WebOptions::default();

        self.runner
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    // Configure for web
                    cc.egui_ctx.set_pixels_per_point(1.0);
                    cc.egui_ctx.set_zoom_factor(1.0);

                    log::info!("Creating WasmAudioApp instance");

                    Ok(Box::new(WasmAudioApp::default()))
                }),
            )
            .await
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    /// Destroy the web runner and clean up resources
    #[wasm_bindgen]
    pub fn destroy(&self) {
        log::info!("Destroying Rusty Audio WASM instance");
        self.runner.destroy();
    }

    /// Check if the application has panicked
    #[wasm_bindgen]
    pub fn has_panicked(&self) -> bool {
        self.runner.has_panicked()
    }

    /// Get panic message if the application has panicked
    #[wasm_bindgen]
    pub fn panic_message(&self) -> Option<String> {
        if self.runner.has_panicked() {
            Some(
                self.runner
                    .panic_summary()
                    .map(|s| s.message())
                    .unwrap_or_default(),
            )
        } else {
            None
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for WebHandle {
    fn default() -> Self {
        Self::new()
    }
}
