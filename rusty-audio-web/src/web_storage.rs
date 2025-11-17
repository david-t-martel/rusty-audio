//! Web storage utilities
//!
//! Helper functions for working with Web Storage API.

use wasm_bindgen::JsValue;
use web_sys::{window, Storage};

/// Get localStorage
///
/// # Returns
/// Storage object or error
pub fn get_local_storage() -> Result<Storage, JsValue> {
    window()
        .ok_or_else(|| JsValue::from_str("Window not found"))?
        .local_storage()?
        .ok_or_else(|| JsValue::from_str("localStorage not available"))
}

/// Get sessionStorage
///
/// # Returns
/// Storage object or error
pub fn get_session_storage() -> Result<Storage, JsValue> {
    window()
        .ok_or_else(|| JsValue::from_str("Window not found"))?
        .session_storage()?
        .ok_or_else(|| JsValue::from_str("sessionStorage not available"))
}

/// Store value in localStorage
///
/// # Arguments
/// * `key` - Storage key
/// * `value` - Value to store
///
/// # Errors
/// Returns error if storage fails
pub fn local_storage_set(key: &str, value: &str) -> Result<(), JsValue> {
    get_local_storage()?.set_item(key, value)
}

/// Get value from localStorage
///
/// # Arguments
/// * `key` - Storage key
///
/// # Returns
/// Value if found, None otherwise
///
/// # Errors
/// Returns error if storage access fails
pub fn local_storage_get(key: &str) -> Result<Option<String>, JsValue> {
    get_local_storage()?.get_item(key)
}

/// Remove value from localStorage
///
/// # Arguments
/// * `key` - Storage key
///
/// # Errors
/// Returns error if storage fails
pub fn local_storage_remove(key: &str) -> Result<(), JsValue> {
    get_local_storage()?.remove_item(key)
}

/// Clear all localStorage
///
/// # Errors
/// Returns error if storage fails
pub fn local_storage_clear() -> Result<(), JsValue> {
    get_local_storage()?.clear()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_local_storage_operations() {
        let key = "test_key";
        let value = "test_value";

        // Set
        local_storage_set(key, value).unwrap();

        // Get
        let retrieved = local_storage_get(key).unwrap();
        assert_eq!(retrieved, Some(value.to_string()));

        // Remove
        local_storage_remove(key).unwrap();
        let retrieved = local_storage_get(key).unwrap();
        assert_eq!(retrieved, None);
    }
}
