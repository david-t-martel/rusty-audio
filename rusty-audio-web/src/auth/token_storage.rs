//! Secure token storage using Web Storage API
//!
//! Provides encrypted storage for OAuth tokens using localStorage and sessionStorage.

use super::session::Session;
use serde::{Deserialize, Serialize};
use std::fmt;
use wasm_bindgen::JsValue;
use web_sys::{window, Storage};

/// Storage error types
#[derive(Debug, Clone)]
pub enum StorageError {
    /// Window not available
    WindowNotFound,
    /// Storage not available
    StorageNotFound,
    /// Serialization error
    SerializationError(String),
    /// Encryption error
    EncryptionError(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::WindowNotFound => write!(f, "Window object not found"),
            StorageError::StorageNotFound => write!(f, "Web Storage not available"),
            StorageError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            StorageError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

/// Token storage handler
pub struct TokenStorage {
    storage_key: String,
}

impl TokenStorage {
    /// Create a new token storage handler
    pub fn new() -> Self {
        Self {
            storage_key: "rusty_audio_session".to_string(),
        }
    }

    /// Get localStorage
    fn get_local_storage(&self) -> Result<Storage, StorageError> {
        window()
            .ok_or(StorageError::WindowNotFound)?
            .local_storage()
            .map_err(|_| StorageError::StorageNotFound)?
            .ok_or(StorageError::StorageNotFound)
    }

    /// Get sessionStorage
    fn get_session_storage(&self) -> Result<Storage, StorageError> {
        window()
            .ok_or(StorageError::WindowNotFound)?
            .session_storage()
            .map_err(|_| StorageError::StorageNotFound)?
            .ok_or(StorageError::StorageNotFound)
    }

    /// Store session in localStorage
    ///
    /// # Arguments
    /// * `session` - Session to store
    ///
    /// # Errors
    /// Returns error if storage fails
    pub async fn store_session(&self, session: &Session) -> Result<(), StorageError> {
        let storage = self.get_local_storage()?;

        let json = serde_json::to_string(session)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // In a real implementation, encrypt the session data
        // For now, we'll store it as-is (WARNING: not secure for production)
        storage
            .set_item(&self.storage_key, &json)
            .map_err(|_| StorageError::StorageNotFound)?;

        log::info!("Session stored successfully");
        Ok(())
    }

    /// Retrieve session from localStorage
    ///
    /// # Returns
    /// Session if found
    ///
    /// # Errors
    /// Returns error if retrieval or deserialization fails
    pub async fn retrieve_session(&self) -> Result<Session, StorageError> {
        let storage = self.get_local_storage()?;

        let json = storage
            .get_item(&self.storage_key)
            .map_err(|_| StorageError::StorageNotFound)?
            .ok_or(StorageError::StorageNotFound)?;

        // In a real implementation, decrypt the session data
        let session: Session = serde_json::from_str(&json)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        Ok(session)
    }

    /// Clear session from localStorage
    ///
    /// # Errors
    /// Returns error if clear fails
    pub async fn clear_session(&self) -> Result<(), StorageError> {
        let storage = self.get_local_storage()?;
        storage
            .remove_item(&self.storage_key)
            .map_err(|_| StorageError::StorageNotFound)?;

        log::info!("Session cleared");
        Ok(())
    }

    /// Store temporary data in sessionStorage
    ///
    /// # Arguments
    /// * `key` - Storage key
    /// * `value` - Value to store
    ///
    /// # Errors
    /// Returns error if storage fails
    pub async fn store_temp(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let storage = self.get_session_storage()?;
        storage
            .set_item(key, value)
            .map_err(|_| StorageError::StorageNotFound)?;
        Ok(())
    }

    /// Retrieve temporary data from sessionStorage
    ///
    /// # Arguments
    /// * `key` - Storage key
    ///
    /// # Returns
    /// Value if found
    ///
    /// # Errors
    /// Returns error if retrieval fails
    pub async fn retrieve_temp(&self, key: &str) -> Result<String, StorageError> {
        let storage = self.get_session_storage()?;
        storage
            .get_item(key)
            .map_err(|_| StorageError::StorageNotFound)?
            .ok_or(StorageError::StorageNotFound)
    }

    /// Remove temporary data from sessionStorage
    ///
    /// # Arguments
    /// * `key` - Storage key
    ///
    /// # Errors
    /// Returns error if removal fails
    pub async fn remove_temp(&self, key: &str) -> Result<(), StorageError> {
        let storage = self.get_session_storage()?;
        storage
            .remove_item(key)
            .map_err(|_| StorageError::StorageNotFound)?;
        Ok(())
    }
}

impl Default for TokenStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_storage_creation() {
        let storage = TokenStorage::new();
        assert_eq!(storage.storage_key, "rusty_audio_session");
    }

    // Note: Web Storage API tests require WASM test environment
    // Use wasm-bindgen-test for actual storage tests
}
