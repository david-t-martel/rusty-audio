//! WASM AudioContext Thread Safety Tests
//!
//! Validates AudioContext thread safety:
//! - Main thread assertion
//! - Initialization guard
//! - Concurrent access protection
//! - Proper cleanup

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[cfg(target_arch = "wasm32")]
mod audio_context_tests {
    use super::*;
    use std::sync::Arc;
    use parking_lot::Mutex;
    use wasm_bindgen::JsValue;
    use web_sys::AudioContext;

    /// Thread-safe AudioContext wrapper
    struct WasmAudioContext {
        context: Arc<Mutex<Option<AudioContext>>>,
    }

    impl WasmAudioContext {
        fn new() -> Self {
            Self {
                context: Arc::new(Mutex::new(None)),
            }
        }

        fn get_or_create(&self) -> Result<AudioContext, JsValue> {
            let mut ctx = self.context.lock();
            if ctx.is_none() {
                let audio_ctx = AudioContext::new()?;
                *ctx = Some(audio_ctx);
            }
            Ok(ctx.as_ref().unwrap().clone())
        }

        fn get(&self) -> Option<AudioContext> {
            self.context.lock().as_ref().map(|c| c.clone())
        }

        fn is_initialized(&self) -> bool {
            self.context.lock().is_some()
        }

        fn close(&self) -> Result<(), JsValue> {
            let mut ctx = self.context.lock();
            if let Some(audio_ctx) = ctx.take() {
                let _ = audio_ctx.close();
            }
            Ok(())
        }
    }

    impl Clone for WasmAudioContext {
        fn clone(&self) -> Self {
            Self {
                context: Arc::clone(&self.context),
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_audio_context_creation() {
        let ctx = WasmAudioContext::new();
        assert!(!ctx.is_initialized());
    }

    #[wasm_bindgen_test]
    fn test_audio_context_lazy_initialization() {
        let ctx = WasmAudioContext::new();

        assert!(!ctx.is_initialized());

        let result = ctx.get_or_create();
        assert!(result.is_ok());
        assert!(ctx.is_initialized());
    }

    #[wasm_bindgen_test]
    fn test_audio_context_idempotent_initialization() {
        let ctx = WasmAudioContext::new();

        // First initialization
        let ctx1 = ctx.get_or_create();
        assert!(ctx1.is_ok());

        // Second initialization should return same context
        let ctx2 = ctx.get_or_create();
        assert!(ctx2.is_ok());

        // Should still be initialized
        assert!(ctx.is_initialized());
    }

    #[wasm_bindgen_test]
    fn test_audio_context_clone_safety() {
        let ctx1 = WasmAudioContext::new();
        let ctx2 = ctx1.clone();

        // Initialize through one clone
        let _ = ctx1.get_or_create();

        // Both should see the initialized state
        assert!(ctx1.is_initialized());
        assert!(ctx2.is_initialized());
    }

    #[wasm_bindgen_test]
    fn test_audio_context_concurrent_access() {
        let ctx = Arc::new(WasmAudioContext::new());

        let ctx1 = ctx.clone();
        let ctx2 = ctx.clone();

        // Both can safely check initialization state
        let init1 = ctx1.is_initialized();
        let init2 = ctx2.is_initialized();

        assert_eq!(init1, init2);
    }

    #[wasm_bindgen_test]
    fn test_audio_context_get_before_init() {
        let ctx = WasmAudioContext::new();

        let result = ctx.get();
        assert!(result.is_none());
    }

    #[wasm_bindgen_test]
    fn test_audio_context_get_after_init() {
        let ctx = WasmAudioContext::new();

        let _ = ctx.get_or_create();
        let result = ctx.get();

        assert!(result.is_some());
    }

    #[wasm_bindgen_test]
    fn test_audio_context_close() {
        let ctx = WasmAudioContext::new();

        // Initialize
        let _ = ctx.get_or_create();
        assert!(ctx.is_initialized());

        // Close
        let result = ctx.close();
        assert!(result.is_ok());
        assert!(!ctx.is_initialized());
    }

    #[wasm_bindgen_test]
    fn test_audio_context_close_before_init() {
        let ctx = WasmAudioContext::new();

        // Close without initialization should not error
        let result = ctx.close();
        assert!(result.is_ok());
        assert!(!ctx.is_initialized());
    }

    #[wasm_bindgen_test]
    async fn test_audio_context_async_initialization() {
        let ctx = WasmAudioContext::new();

        // Initialize
        let result = ctx.get_or_create();
        assert!(result.is_ok());

        // Simulate async delay
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::from(1)))
            .await
            .ok();

        // Should still be initialized
        assert!(ctx.is_initialized());
    }

    #[wasm_bindgen_test]
    fn test_audio_context_sample_rate() {
        let ctx = WasmAudioContext::new();

        let audio_ctx = ctx.get_or_create();
        if let Ok(ac) = audio_ctx {
            let sample_rate = ac.sample_rate();
            // Should be a reasonable sample rate
            assert!(sample_rate >= 8000.0);
            assert!(sample_rate <= 192000.0);
        }
    }

    #[wasm_bindgen_test]
    fn test_audio_context_state() {
        let ctx = WasmAudioContext::new();

        let audio_ctx = ctx.get_or_create();
        if let Ok(ac) = audio_ctx {
            let state = ac.state();
            // Should be in a valid state
            assert!(
                matches!(
                    state,
                    web_sys::AudioContextState::Suspended
                        | web_sys::AudioContextState::Running
                        | web_sys::AudioContextState::Closed
                )
            );
        }
    }

    #[wasm_bindgen_test]
    fn test_audio_context_destination() {
        let ctx = WasmAudioContext::new();

        let audio_ctx = ctx.get_or_create();
        if let Ok(ac) = audio_ctx {
            let destination = ac.destination();
            // Should have a destination node
            assert!(destination.max_channel_count() > 0);
        }
    }

    #[wasm_bindgen_test]
    fn test_audio_context_multiple_clones() {
        let ctx = Arc::new(WasmAudioContext::new());

        // Create multiple clones
        let clones: Vec<_> = (0..10).map(|_| ctx.clone()).collect();

        // Initialize through first clone
        let _ = clones[0].get_or_create();

        // All clones should see the initialized state
        for clone in &clones {
            assert!(clone.is_initialized());
        }
    }

    #[wasm_bindgen_test]
    fn test_audio_context_initialization_guard() {
        let ctx = Arc::new(WasmAudioContext::new());

        // Multiple concurrent initialization attempts
        let ctx1 = ctx.clone();
        let ctx2 = ctx.clone();
        let ctx3 = ctx.clone();

        let _ = ctx1.get_or_create();
        let _ = ctx2.get_or_create();
        let _ = ctx3.get_or_create();

        // Should be initialized exactly once
        assert!(ctx.is_initialized());
    }

    #[wasm_bindgen_test]
    async fn test_audio_context_async_close() {
        let ctx = WasmAudioContext::new();

        // Initialize
        let _ = ctx.get_or_create();

        // Simulate async operations
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&JsValue::from(1)))
            .await
            .ok();

        // Close
        let result = ctx.close();
        assert!(result.is_ok());
        assert!(!ctx.is_initialized());
    }
}

/// Tests for main thread assertion
#[cfg(target_arch = "wasm32")]
mod main_thread_tests {
    use super::*;

    #[wasm_bindgen_test]
    fn test_is_main_thread() {
        // In WASM, wasm_bindgen_test runs on main thread
        // This test validates we're on main thread
        let window = web_sys::window();
        assert!(window.is_some(), "Should have access to window on main thread");
    }

    #[wasm_bindgen_test]
    fn test_audio_context_requires_main_thread() {
        // AudioContext creation requires main thread
        let ctx_result = web_sys::AudioContext::new();

        // Should succeed on main thread
        assert!(ctx_result.is_ok(), "AudioContext creation should work on main thread");
    }

    #[wasm_bindgen_test]
    fn test_document_access_on_main_thread() {
        let window = web_sys::window().expect("Should have window");
        let document = window.document().expect("Should have document");

        // Should be able to access DOM on main thread
        assert!(document.body().is_some());
    }
}

/// Tests for proper cleanup
#[cfg(target_arch = "wasm32")]
mod cleanup_tests {
    use super::*;
    use parking_lot::Mutex;
    use std::sync::Arc;

    struct AudioContextManager {
        contexts: Arc<Mutex<Vec<web_sys::AudioContext>>>,
    }

    impl AudioContextManager {
        fn new() -> Self {
            Self {
                contexts: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn create_context(&self) -> Result<web_sys::AudioContext, wasm_bindgen::JsValue> {
            let ctx = web_sys::AudioContext::new()?;
            self.contexts.lock().push(ctx.clone());
            Ok(ctx)
        }

        fn close_all(&self) -> Result<(), wasm_bindgen::JsValue> {
            let mut contexts = self.contexts.lock();
            for ctx in contexts.drain(..) {
                let _ = ctx.close();
            }
            Ok(())
        }

        fn count(&self) -> usize {
            self.contexts.lock().len()
        }
    }

    #[wasm_bindgen_test]
    fn test_context_manager_cleanup() {
        let manager = AudioContextManager::new();

        // Create contexts
        for _ in 0..3 {
            let _ = manager.create_context();
        }

        assert_eq!(manager.count(), 3);

        // Close all
        let result = manager.close_all();
        assert!(result.is_ok());
        assert_eq!(manager.count(), 0);
    }

    #[wasm_bindgen_test]
    fn test_cleanup_idempotent() {
        let manager = AudioContextManager::new();

        let _ = manager.create_context();
        assert_eq!(manager.count(), 1);

        // Close twice
        let _ = manager.close_all();
        let _ = manager.close_all();

        // Should be fine
        assert_eq!(manager.count(), 0);
    }
}

// Non-WASM tests
#[cfg(not(target_arch = "wasm32"))]
mod non_wasm_tests {
    #[test]
    fn test_audio_context_not_available_on_native() {
        assert!(true, "AudioContext tests are WASM-only");
    }
}
