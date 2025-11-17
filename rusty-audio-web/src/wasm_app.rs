//! WASM application module
//!
//! Main WASM application logic and eframe integration.

use eframe::egui;
use wasm_bindgen::prelude::*;

/// WASM application state
pub struct WasmApp {
    frame_count: u64,
}

impl Default for WasmApp {
    fn default() -> Self {
        Self { frame_count: 0 }
    }
}

impl eframe::App for WasmApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rusty Audio Web");
            ui.label(format!("Frame: {}", self.frame_count));
            ui.separator();
            ui.label("WASM application running!");
        });
    }
}

/// Start the WASM application
///
/// # Arguments
/// * `canvas_id` - ID of the HTML canvas element
///
/// # Errors
/// Returns error if app fails to start
#[wasm_bindgen]
pub async fn start_app(canvas_id: &str) -> Result<(), JsValue> {
    crate::init_panic_handler();
    crate::init_logger();

    log::info!("Starting WASM app on canvas: {}", canvas_id);

    let web_options = eframe::WebOptions::default();

    eframe::WebRunner::new()
        .start(
            canvas_id,
            web_options,
            Box::new(|_cc| Ok(Box::new(WasmApp::default()))),
        )
        .await?;

    Ok(())
}
