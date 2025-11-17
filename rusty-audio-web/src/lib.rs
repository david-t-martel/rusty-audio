// Rusty Audio Web - WASM/PWA Application
//
// This module provides the WASM entry point and OAuth authentication
// for the web-based audio player.

#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![cfg_attr(not(test), warn(clippy::unwrap_used, clippy::expect_used))]

//! # Rusty Audio Web
//!
//! WASM/PWA application for Rusty Audio with OAuth authentication.

use wasm_bindgen::prelude::*;

// Re-export core library
pub use rusty_audio_core::*;

// OAuth authentication module
#[cfg(feature = "auth")]
pub mod auth;

// Web-specific modules
pub mod wasm_app;
pub mod web_storage;

/// Initialize WASM panic handler
pub fn init_panic_handler() {
    console_error_panic_hook::set_once();
}

/// Initialize WASM logger
pub fn init_logger() {
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
}

/// WASM entry point for the web application
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    init_panic_handler();
    init_logger();

    log::info!("Rusty Audio Web v{} starting...", rusty_audio_core::version());

    Ok(())
}

/// WebHandle for WASM exports
#[wasm_bindgen(js_name = WebHandle)]
pub struct WebHandle {
    #[cfg(feature = "auth")]
    auth_client: Option<auth::OAuthClient>,
}

#[wasm_bindgen]
impl WebHandle {
    /// Create a new WebHandle
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        init_panic_handler();
        init_logger();

        Self {
            #[cfg(feature = "auth")]
            auth_client: None,
        }
    }

    /// Initialize OAuth authentication
    #[cfg(feature = "auth")]
    #[wasm_bindgen(js_name = initAuth)]
    pub fn init_auth(&mut self, provider: &str, client_id: &str, redirect_uri: &str) -> Result<(), JsValue> {
        use auth::{OAuthClient, OAuthProvider};

        let provider_enum = match provider {
            "google" => OAuthProvider::Google,
            "github" => OAuthProvider::GitHub,
            "microsoft" => OAuthProvider::Microsoft,
            _ => return Err(JsValue::from_str(&format!("Unknown provider: {}", provider))),
        };

        self.auth_client = Some(OAuthClient::new(
            provider_enum,
            client_id.to_string(),
            redirect_uri.to_string(),
        ));

        Ok(())
    }

    /// Start OAuth login flow
    #[cfg(feature = "auth")]
    #[wasm_bindgen(js_name = login)]
    pub async fn login(&self) -> Result<String, JsValue> {
        let client = self.auth_client.as_ref()
            .ok_or_else(|| JsValue::from_str("Auth not initialized"))?;

        client.initiate_auth().await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Handle OAuth callback
    #[cfg(feature = "auth")]
    #[wasm_bindgen(js_name = handleCallback)]
    pub async fn handle_callback(&self, code: &str) -> Result<JsValue, JsValue> {
        let client = self.auth_client.as_ref()
            .ok_or_else(|| JsValue::from_str("Auth not initialized"))?;

        let session = client.handle_callback(code).await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&session)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Logout and clear session
    #[cfg(feature = "auth")]
    #[wasm_bindgen(js_name = logout)]
    pub async fn logout(&self) -> Result<(), JsValue> {
        let client = self.auth_client.as_ref()
            .ok_or_else(|| JsValue::from_str("Auth not initialized"))?;

        client.logout().await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get library version
    #[wasm_bindgen(js_name = getVersion)]
    pub fn get_version(&self) -> String {
        rusty_audio_core::version().to_string()
    }
}

impl Default for WebHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_web_handle_creation() {
        let handle = WebHandle::new();
        assert!(!handle.get_version().is_empty());
    }

    #[wasm_bindgen_test]
    fn test_panic_handler_init() {
        init_panic_handler();
        // Should not panic
    }
}
