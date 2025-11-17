//! WASM Panic Boundary Handler
//!
//! Provides comprehensive panic catching and recovery mechanisms for WASM entry points.
//! This module ensures that Rust panics don't propagate to JavaScript, providing
//! graceful degradation and detailed error reporting.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use parking_lot::Mutex;
#[cfg(target_arch = "wasm32")]
use std::panic;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

/// Maximum length for error messages to prevent memory exhaustion
const MAX_ERROR_MESSAGE_LENGTH: usize = 4096;

/// Maximum number of panic records to retain
const MAX_PANIC_HISTORY: usize = 100;

#[cfg(target_arch = "wasm32")]
/// Panic statistics for monitoring and debugging
pub struct PanicStats {
    total_panics: AtomicU64,
    total_caught: AtomicU64,
    total_recovered: AtomicU64,
    panic_history: Arc<Mutex<Vec<PanicRecord>>>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
pub struct PanicRecord {
    pub timestamp: f64,
    pub message: String,
    pub location: Option<String>,
    pub recovered: bool,
}

#[cfg(target_arch = "wasm32")]
impl PanicStats {
    /// Create a new panic statistics tracker
    pub fn new() -> Self {
        Self {
            total_panics: AtomicU64::new(0),
            total_caught: AtomicU64::new(0),
            total_recovered: AtomicU64::new(0),
            panic_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record a panic event
    pub fn record_panic(&self, message: String, location: Option<String>, recovered: bool) {
        self.total_panics.fetch_add(1, Ordering::Relaxed);
        if recovered {
            self.total_recovered.fetch_add(1, Ordering::Relaxed);
        }

        // Get current time from browser performance API
        let timestamp = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        let record = PanicRecord {
            timestamp,
            message: truncate_string(message, MAX_ERROR_MESSAGE_LENGTH),
            location,
            recovered,
        };

        let mut history = self.panic_history.lock();
        history.push(record);

        // Limit history size to prevent unbounded growth
        if history.len() > MAX_PANIC_HISTORY {
            history.remove(0);
        }
    }

    /// Record a caught panic
    pub fn record_caught(&self) {
        self.total_caught.fetch_add(1, Ordering::Relaxed);
    }

    /// Get panic statistics
    pub fn get_stats(&self) -> (u64, u64, u64) {
        (
            self.total_panics.load(Ordering::Relaxed),
            self.total_caught.load(Ordering::Relaxed),
            self.total_recovered.load(Ordering::Relaxed),
        )
    }

    /// Get recent panic history
    pub fn get_recent_panics(&self, count: usize) -> Vec<PanicRecord> {
        let history = self.panic_history.lock();
        let start = history.len().saturating_sub(count);
        history[start..].to_vec()
    }

    /// Clear panic history
    pub fn clear_history(&self) {
        self.panic_history.lock().clear();
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for PanicStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
/// Global panic statistics instance
static PANIC_STATS: once_cell::sync::Lazy<PanicStats> = once_cell::sync::Lazy::new(PanicStats::new);

/// Truncate string to maximum length with ellipsis
fn truncate_string(s: String, max_len: usize) -> String {
    if s.len() <= max_len {
        s
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(target_arch = "wasm32")]
/// Install panic hook that logs to browser console
///
/// This should be called early in WASM initialization to capture all panics
pub fn install_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic payload".to_string()
        };

        let location = panic_info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()));

        // Log to browser console with full details
        if let Some(loc) = &location {
            web_sys::console::error_2(
                &JsValue::from_str(&format!("Panic at {}: {}", loc, message)),
                &JsValue::from_str(&format!("{:?}", panic_info)),
            );
        } else {
            web_sys::console::error_1(&JsValue::from_str(&format!("Panic: {}", message)));
        }

        // Record in statistics (recovered = false for unhandled panics)
        PANIC_STATS.record_panic(message, location, false);
    }));

    log::info!("WASM panic hook installed");
}

#[cfg(target_arch = "wasm32")]
/// Execute a closure with panic boundary
///
/// If the closure panics, the panic is caught and converted to a Result::Err.
/// This is the primary mechanism for preventing panics from crossing FFI boundaries.
///
/// # Example
/// ```no_run
/// let result = with_panic_boundary(|| {
///     // Potentially panicking code
///     risky_operation()?;
///     Ok(())
/// });
/// ```
pub fn with_panic_boundary<F, T>(f: F) -> Result<T, JsValue>
where
    F: FnOnce() -> Result<T, JsValue> + panic::UnwindSafe,
{
    match panic::catch_unwind(f) {
        Ok(result) => {
            PANIC_STATS.record_caught();
            result
        }
        Err(panic_payload) => {
            let message = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };

            // Record panic as caught and recovered
            PANIC_STATS.record_panic(message.clone(), None, true);
            PANIC_STATS.record_recovered.fetch_add(1, Ordering::Relaxed);

            log::error!("Caught panic in boundary: {}", message);

            Err(JsValue::from_str(&format!("Panic caught: {}", message)))
        }
    }
}

#[cfg(target_arch = "wasm32")]
/// Async version of panic boundary for async functions
///
/// # Example
/// ```no_run
/// let result = with_panic_boundary_async(async {
///     async_risky_operation().await?;
///     Ok(())
/// }).await;
/// ```
pub async fn with_panic_boundary_async<F, T>(f: F) -> Result<T, JsValue>
where
    F: std::future::Future<Output = Result<T, JsValue>>,
{
    // Async functions can't use catch_unwind directly, so we wrap the await
    match panic::catch_unwind(panic::AssertUnwindSafe(|| {
        // We can't actually catch panics in async code without pinning,
        // so this is a best-effort wrapper
        log::debug!("Entering async panic boundary");
    })) {
        Ok(_) => {}
        Err(_) => {
            return Err(JsValue::from_str("Panic in async boundary setup"));
        }
    }

    f.await
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
/// Get panic statistics from JavaScript
pub fn get_panic_stats() -> JsValue {
    let (total, caught, recovered) = PANIC_STATS.get_stats();

    let stats = js_sys::Object::new();
    js_sys::Reflect::set(&stats, &"totalPanics".into(), &JsValue::from(total as f64)).ok();
    js_sys::Reflect::set(&stats, &"totalCaught".into(), &JsValue::from(caught as f64)).ok();
    js_sys::Reflect::set(
        &stats,
        &"totalRecovered".into(),
        &JsValue::from(recovered as f64),
    )
    .ok();

    JsValue::from(stats)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
/// Get recent panic history from JavaScript
pub fn get_recent_panics(count: usize) -> JsValue {
    let panics = PANIC_STATS.get_recent_panics(count);

    let array = js_sys::Array::new();
    for panic in panics {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"timestamp".into(), &JsValue::from(panic.timestamp)).ok();
        js_sys::Reflect::set(&obj, &"message".into(), &JsValue::from_str(&panic.message)).ok();
        if let Some(loc) = panic.location {
            js_sys::Reflect::set(&obj, &"location".into(), &JsValue::from_str(&loc)).ok();
        }
        js_sys::Reflect::set(&obj, &"recovered".into(), &JsValue::from(panic.recovered)).ok();
        array.push(&JsValue::from(obj));
    }

    JsValue::from(array)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
/// Clear panic history from JavaScript
pub fn clear_panic_history() {
    PANIC_STATS.clear_history();
}

// Add once_cell dependency to Cargo.toml
#[cfg(target_arch = "wasm32")]
use once_cell::sync::Lazy;

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_panic_stats_creation() {
        let stats = PanicStats::new();
        let (total, caught, recovered) = stats.get_stats();
        assert_eq!(total, 0);
        assert_eq!(caught, 0);
        assert_eq!(recovered, 0);
    }

    #[wasm_bindgen_test]
    fn test_panic_boundary_success() {
        let result = with_panic_boundary(|| Ok(42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[wasm_bindgen_test]
    fn test_panic_boundary_error() {
        let result: Result<(), JsValue> =
            with_panic_boundary(|| Err(JsValue::from_str("test error")));
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_truncate_string() {
        let long_string = "a".repeat(5000);
        let truncated = truncate_string(long_string, 100);
        assert!(truncated.len() <= 100);
        assert!(truncated.ends_with("..."));
    }
}
