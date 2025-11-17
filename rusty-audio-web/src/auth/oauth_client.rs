//! OAuth 2.0 client with PKCE support
//!
//! Implements secure OAuth 2.0 authentication flow with PKCE (Proof Key for Code Exchange)
//! for single-page applications running in WASM.

use super::{providers::OAuthProvider, session::Session, token_storage::TokenStorage};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use wasm_bindgen::JsValue;
use web_sys::{window, Window};

/// OAuth error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OAuthError {
    /// Missing window object
    WindowNotFound,
    /// Missing storage
    StorageNotFound,
    /// Invalid state parameter
    InvalidState,
    /// Token exchange failed
    TokenExchangeFailed(String),
    /// Network error
    NetworkError(String),
    /// Invalid response
    InvalidResponse(String),
    /// Not authenticated
    NotAuthenticated,
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OAuthError::WindowNotFound => write!(f, "Window object not found"),
            OAuthError::StorageNotFound => write!(f, "Storage not available"),
            OAuthError::InvalidState => write!(f, "Invalid state parameter"),
            OAuthError::TokenExchangeFailed(msg) => write!(f, "Token exchange failed: {}", msg),
            OAuthError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            OAuthError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            OAuthError::NotAuthenticated => write!(f, "Not authenticated"),
        }
    }
}

impl std::error::Error for OAuthError {}

/// OAuth client with PKCE support
pub struct OAuthClient {
    provider: OAuthProvider,
    client_id: String,
    redirect_uri: String,
    state: String,
    pkce_verifier: String,
    storage: TokenStorage,
}

impl OAuthClient {
    /// Create a new OAuth client
    ///
    /// # Arguments
    /// * `provider` - OAuth provider
    /// * `client_id` - Client ID from provider
    /// * `redirect_uri` - Redirect URI after authentication
    pub fn new(provider: OAuthProvider, client_id: String, redirect_uri: String) -> Self {
        let state = Self::generate_random_string(32);
        let pkce_verifier = Self::generate_random_string(64);
        let storage = TokenStorage::new();

        Self {
            provider,
            client_id,
            redirect_uri,
            state,
            pkce_verifier,
            storage,
        }
    }

    /// Generate random string for state and PKCE
    fn generate_random_string(length: usize) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..62);
                if idx < 26 {
                    (b'A' + idx) as char
                } else if idx < 52 {
                    (b'a' + (idx - 26)) as char
                } else {
                    (b'0' + (idx - 52)) as char
                }
            })
            .collect()
    }

    /// Generate PKCE challenge from verifier
    fn generate_pkce_challenge(verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let result = hasher.finalize();
        URL_SAFE_NO_PAD.encode(result)
    }

    /// Initiate OAuth authentication flow
    ///
    /// # Returns
    /// Authorization URL to redirect to
    ///
    /// # Errors
    /// Returns error if window is not available
    pub async fn initiate_auth(&self) -> Result<String, OAuthError> {
        let config = self.provider.config();
        let challenge = Self::generate_pkce_challenge(&self.pkce_verifier);

        // Store state and verifier in session storage
        self.storage
            .store_temp("oauth_state", &self.state)
            .await
            .map_err(|_| OAuthError::StorageNotFound)?;
        self.storage
            .store_temp("pkce_verifier", &self.pkce_verifier)
            .await
            .map_err(|_| OAuthError::StorageNotFound)?;

        // Build authorization URL
        let scopes = config.default_scopes.join(" ");
        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
            config.auth_endpoint,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scopes),
            &self.state,
            &challenge
        );

        // Store provider for callback
        self.storage
            .store_temp(
                "oauth_provider",
                &serde_json::to_string(&self.provider).unwrap(),
            )
            .await
            .map_err(|_| OAuthError::StorageNotFound)?;

        Ok(auth_url)
    }

    /// Handle OAuth callback with authorization code
    ///
    /// # Arguments
    /// * `code` - Authorization code from OAuth provider
    ///
    /// # Returns
    /// Session with access token
    ///
    /// # Errors
    /// Returns error if token exchange fails
    pub async fn handle_callback(&self, code: &str) -> Result<Session, OAuthError> {
        // Verify state
        let stored_state = self
            .storage
            .retrieve_temp("oauth_state")
            .await
            .map_err(|_| OAuthError::InvalidState)?;
        if stored_state != self.state {
            return Err(OAuthError::InvalidState);
        }

        // Retrieve PKCE verifier
        let verifier = self
            .storage
            .retrieve_temp("pkce_verifier")
            .await
            .map_err(|_| OAuthError::TokenExchangeFailed("Missing verifier".to_string()))?;

        // Exchange code for token
        let config = self.provider.config();
        let token_response = self
            .exchange_code_for_token(code, &verifier, config.token_endpoint)
            .await?;

        // Create session
        let session = Session {
            provider: self.provider,
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(token_response.expires_in),
            user_info: None,
        };

        // Store session
        self.storage
            .store_session(&session)
            .await
            .map_err(|e| OAuthError::StorageNotFound)?;

        // Clean up temporary storage
        let _ = self.storage.remove_temp("oauth_state").await;
        let _ = self.storage.remove_temp("pkce_verifier").await;
        let _ = self.storage.remove_temp("oauth_provider").await;

        Ok(session)
    }

    /// Exchange authorization code for access token
    async fn exchange_code_for_token(
        &self,
        code: &str,
        verifier: &str,
        token_endpoint: &str,
    ) -> Result<TokenResponse, OAuthError> {
        // In a real implementation, this would make an HTTP request
        // For now, we'll simulate it
        log::info!("Exchanging code for token at {}", token_endpoint);

        // This is a placeholder - actual implementation would use reqwest
        // with CORS proxy or backend endpoint
        Err(OAuthError::TokenExchangeFailed(
            "Token exchange not yet implemented - requires backend endpoint".to_string(),
        ))
    }

    /// Refresh access token
    ///
    /// # Arguments
    /// * `refresh_token` - Refresh token from previous session
    ///
    /// # Returns
    /// New session with refreshed token
    ///
    /// # Errors
    /// Returns error if refresh fails
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<Session, OAuthError> {
        let config = self.provider.config();

        // In a real implementation, this would make an HTTP request
        log::info!("Refreshing token at {}", config.token_endpoint);

        Err(OAuthError::TokenExchangeFailed(
            "Token refresh not yet implemented - requires backend endpoint".to_string(),
        ))
    }

    /// Logout and clear session
    ///
    /// # Errors
    /// Returns error if logout fails
    pub async fn logout(&self) -> Result<(), OAuthError> {
        // Clear stored session
        self.storage
            .clear_session()
            .await
            .map_err(|_| OAuthError::StorageNotFound)?;

        // Revoke token if provider supports it
        if let Some(revoke_endpoint) = self.provider.config().revoke_endpoint {
            log::info!("Revoking token at {}", revoke_endpoint);
            // In a real implementation, this would make an HTTP request
        }

        Ok(())
    }

    /// Get current session if available
    ///
    /// # Returns
    /// Current session or None if not authenticated
    pub async fn get_session(&self) -> Option<Session> {
        self.storage.retrieve_session().await.ok()
    }
}

/// Token response from OAuth provider
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    expires_in: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_string() {
        let s1 = OAuthClient::generate_random_string(32);
        let s2 = OAuthClient::generate_random_string(32);
        assert_eq!(s1.len(), 32);
        assert_eq!(s2.len(), 32);
        assert_ne!(s1, s2); // Should be different
    }

    #[test]
    fn test_generate_pkce_challenge() {
        let verifier = "test_verifier_12345";
        let challenge = OAuthClient::generate_pkce_challenge(verifier);
        assert!(!challenge.is_empty());
        // Challenge should be base64-url-safe
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
        assert!(!challenge.contains('='));
    }

    #[test]
    fn test_oauth_client_creation() {
        let client = OAuthClient::new(
            OAuthProvider::Google,
            "test_client_id".to_string(),
            "http://localhost:8080/callback".to_string(),
        );
        assert_eq!(client.provider, OAuthProvider::Google);
        assert_eq!(client.client_id, "test_client_id");
    }
}
