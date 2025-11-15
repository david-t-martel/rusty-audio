//! WebAssembly entry point and browser integration
//!
//! This module provides the WASM-specific initialization and runtime
//! for running rusty-audio in a web browser environment.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use eframe::egui;

#[cfg(target_arch = "wasm32")]
use rusty_audio::{
    audio::AudioConfig,
    integrated_audio_manager::{IntegratedAudioManager, PlaybackState},
    ui::{
        signal_generator::SignalGeneratorPanel,
        theme::{Theme, ThemeManager},
    },
};

#[cfg(target_arch = "wasm32")]
use std::time::Instant;

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

        // Main content panel
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

            ui.heading("Signal Generator");
            ui.separator();

            // Signal generator panel
            if let Some(ref mut audio_manager) = self.audio_manager {
                // Draw signal generator UI
                self.signal_generator_panel.show(ui);

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
                let volume_response = ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).text("Volume"));

                // Apply volume changes to audio engine
                if volume_response.changed() {
                    use crate::integrated_audio_manager::RouteType;
                    if let Err(e) = audio_manager.set_route_gain(RouteType::SignalGeneratorPlayback, self.volume) {
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
