//! WebAssembly entry point and browser integration
//! 
//! This module provides the WASM-specific initialization and runtime
//! for running rusty-audio in a web browser environment.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use eframe::egui;

#[cfg(target_arch = "wasm32")]
/// Minimal WASM application stub
/// This will be expanded to match AudioPlayerApp functionality
struct WasmAudioApp {
    message: String,
}

#[cfg(target_arch = "wasm32")]
impl Default for WasmAudioApp {
    fn default() -> Self {
        Self {
            message: "Rusty Audio - WASM Version Loading...".to_string(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl eframe::App for WasmAudioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎵 Rusty Audio Player");
            ui.separator();
            ui.label(&self.message);
            ui.add_space(20.0);
            ui.label("WASM Version - UI Framework Active");
            ui.label("Audio functionality coming soon...");
            
            if ui.button("Click me!").clicked() {
                self.message = format!("Button clicked at {:?}!", std::time::Instant::now());
                log::info!("Button clicked in WASM app");
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
        console_log::init_with_level(log::Level::Debug)
            .expect("Failed to initialize console logging");
        
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
            Some(self.runner.panic_summary().map(|s| s.message()).unwrap_or_default())
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
