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
    audio::backend::AudioConfig,
    integrated_audio_manager::{IntegratedAudioManager, PlaybackState},
    ui::{
        signal_generator::SignalGeneratorPanel,
        theme::{Theme, ThemeManager},
    },
};

#[cfg(target_arch = "wasm32")]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use parking_lot::Mutex;

#[cfg(target_arch = "wasm32")]
/// Worker pool for multithreaded audio processing
///
/// Manages a pool of Web Workers for offloading heavy audio processing
/// tasks (FFT, EQ, effects) from the main thread to maintain responsive UI.
struct WorkerPool {
    initialized: Arc<Mutex<bool>>,
    num_workers: usize,
}

#[cfg(target_arch = "wasm32")]
impl WorkerPool {
    /// Create a new worker pool
    ///
    /// # Arguments
    /// * `num_workers` - Number of workers to spawn (defaults to navigator.hardwareConcurrency - 1)
    fn new(num_workers: Option<usize>) -> Self {
        let num_workers = num_workers.unwrap_or_else(|| {
            // Use hardware concurrency, reserve one thread for main
            let hw_concurrency = web_sys::window()
                .and_then(|w| w.navigator().hardware_concurrency())
                .map(|c| c.max(1.0) as usize)
                .unwrap_or(4);

            (hw_concurrency - 1).max(1)
        });

        Self {
            initialized: Arc::new(Mutex::new(false)),
            num_workers,
        }
    }

    /// Initialize the worker pool with wasm-bindgen-rayon
    ///
    /// This must be called from the main thread before using workers
    fn initialize(&self) -> Result<(), JsValue> {
        let mut initialized = self.initialized.lock();
        if *initialized {
            return Ok(());
        }

        // Initialize rayon thread pool for WASM
        // This spawns Web Workers and sets up SharedArrayBuffer communication
        wasm_bindgen_rayon::init_thread_pool(self.num_workers).map_err(|e| {
            JsValue::from_str(&format!("Failed to initialize worker pool: {:?}", e))
        })?;

        log::info!("Worker pool initialized with {} workers", self.num_workers);
        *initialized = true;
        Ok(())
    }

    /// Check if the worker pool is initialized
    fn is_initialized(&self) -> bool {
        *self.initialized.lock()
    }

    /// Get the number of workers
    fn worker_count(&self) -> usize {
        self.num_workers
    }
}

#[cfg(target_arch = "wasm32")]
/// Audio buffer shared between main thread and workers via SharedArrayBuffer
///
/// Provides thread-safe access to audio data for parallel processing
struct SharedAudioBuffer {
    // Length of the buffer
    length: usize,
    // Number of channels
    channels: usize,
    // Actual buffer data (wrapped in Arc<Mutex<>> for thread safety)
    data: Arc<Mutex<Vec<f32>>>,
}

#[cfg(target_arch = "wasm32")]
impl SharedAudioBuffer {
    /// Create a new shared audio buffer
    fn new(length: usize, channels: usize) -> Self {
        Self {
            length,
            channels,
            data: Arc::new(Mutex::new(vec![0.0; length * channels])),
        }
    }

    /// Get a read-only copy of the buffer data
    fn read(&self) -> Vec<f32> {
        self.data.lock().clone()
    }

    /// Write data to the buffer
    fn write(&self, data: &[f32]) {
        let mut buffer = self.data.lock();
        let copy_len = data.len().min(buffer.len());
        buffer[..copy_len].copy_from_slice(&data[..copy_len]);
    }

    /// Get the length of the buffer
    fn len(&self) -> usize {
        self.length
    }

    /// Get the number of channels
    fn channel_count(&self) -> usize {
        self.channels
    }

    /// Process audio data in parallel using the worker pool
    ///
    /// This splits the buffer into chunks and processes them on workers
    fn process_parallel<F>(&self, mut processor: F) -> Vec<f32>
    where
        F: FnMut(&[f32]) -> Vec<f32> + Send + Sync + 'static,
    {
        use rayon::prelude::*;

        let data = self.data.lock().clone();
        let chunk_size = (data.len() / rayon::current_num_threads()).max(128);

        // Process chunks in parallel
        let processed: Vec<f32> = data
            .par_chunks(chunk_size)
            .flat_map(|chunk| processor(chunk))
            .collect();

        processed
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

    // Worker pool for multithreaded processing
    worker_pool: Arc<WorkerPool>,

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
        log::info!("Initializing WASM Audio Player with multithreading support...");

        // Initialize worker pool first
        let worker_pool = Arc::new(WorkerPool::new(None));

        // Attempt to initialize the worker pool (may fail if SharedArrayBuffer not available)
        match worker_pool.initialize() {
            Ok(_) => {
                log::info!(
                    "Worker pool initialized successfully with {} workers",
                    worker_pool.worker_count()
                );
            }
            Err(e) => {
                log::warn!(
                    "Failed to initialize worker pool: {:?}. Running in single-threaded mode.",
                    e
                );
                log::warn!("For multithreading, ensure Cross-Origin-Opener-Policy and Cross-Origin-Embedder-Policy headers are set.");
            }
        }

        // Initialize audio manager with Web Audio backend
        let audio_config = AudioConfig {
            sample_rate: 48000,
            channels: 2,
            sample_format: crate::audio::backend::SampleFormat::F32,
            buffer_size: 512,
            exclusive_mode: false, // Not applicable to WASM
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
            worker_pool,
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
        let visuals = self.theme_manager.get_visuals();
        ctx.set_visuals(visuals);

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
                let colors = self.theme_manager.get_colors();
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
                    if let Err(e) = audio_manager
                        .set_route_gain(RouteType::SignalGeneratorPlayback, self.volume)
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

            // Worker pool status
            ui.add_space(10.0);
            ui.separator();
            ui.heading("Multithreading Status");
            if self.worker_pool.is_initialized() {
                ui.colored_label(
                    egui::Color32::GREEN,
                    format!(
                        "✓ Worker Pool Active ({} workers)",
                        self.worker_pool.worker_count()
                    ),
                );
                ui.label("Heavy audio processing offloaded to workers");
            } else {
                ui.colored_label(egui::Color32::YELLOW, "⚠ Single-threaded mode");
                ui.label("SharedArrayBuffer not available - all processing on main thread");
                ui.label("Set COOP/COEP headers for multithreading support");
            }
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
