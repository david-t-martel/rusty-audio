//! Session management for OAuth authentication
//!
//! Manages user sessions with secure token storage.

use super::providers::OAuthProvider;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// OAuth provider
    pub provider: OAuthProvider,
    /// Access token
    pub access_token: String,
    /// Refresh token (optional)
    pub refresh_token: Option<String>,
    /// Token expiration time
    pub expires_at: DateTime<Utc>,
    /// User information (optional)
    pub user_info: Option<UserInfo>,
}

impl Session {
    /// Check if session is expired
    ///
    /// # Returns
    /// True if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// Check if session needs refresh (within 5 minutes of expiry)
    ///
    /// # Returns
    /// True if session should be refreshed
    pub fn needs_refresh(&self) -> bool {
        let refresh_threshold = self.expires_at - chrono::Duration::minutes(5);
        Utc::now() >= refresh_threshold
    }

    /// Time until expiration
    ///
    /// # Returns
    /// Duration until session expires
    pub fn time_until_expiry(&self) -> chrono::Duration {
        self.expires_at - Utc::now()
    }
}

/// User information from OAuth provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// User ID
    pub id: String,
    /// Email address
    pub email: Option<String>,
    /// Display name
    pub name: Option<String>,
    /// Profile picture URL
    pub picture: Option<String>,
}

/// Session manager for handling multiple sessions
#[derive(Debug, Default)]
pub struct SessionManager {
    current_session: Option<Session>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            current_session: None,
        }
    }

    /// Set current session
    ///
    /// # Arguments
    /// * `session` - Session to set as current
    pub fn set_session(&mut self, session: Session) {
        self.current_session = Some(session);
    }

    /// Get current session
    ///
    /// # Returns
    /// Current session if available and not expired
    pub fn get_session(&self) -> Option<&Session> {
        self.current_session.as_ref().filter(|s| !s.is_expired())
    }

    /// Clear current session
    pub fn clear_session(&mut self) {
        self.current_session = None;
    }

    /// Check if authenticated
    ///
    /// # Returns
    /// True if there is a valid session
    pub fn is_authenticated(&self) -> bool {
        self.get_session().is_some()
    }

    /// Get access token if authenticated
    ///
    /// # Returns
    /// Access token or None
    pub fn get_access_token(&self) -> Option<&str> {
        self.get_session().map(|s| s.access_token.as_str())
    }

    /// Check if session needs refresh
    ///
    /// # Returns
    /// True if session should be refreshed
    pub fn needs_refresh(&self) -> bool {
        self.current_session
            .as_ref()
            .map(|s| s.needs_refresh())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_expiry() {
        let session = Session {
            provider: OAuthProvider::Google,
            access_token: "test_token".to_string(),
            refresh_token: None,
            expires_at: Utc::now() + chrono::Duration::hours(1),
            user_info: None,
        };

        assert!(!session.is_expired());
        assert!(!session.needs_refresh());
    }

    #[test]
    fn test_session_expired() {
        let session = Session {
            provider: OAuthProvider::Google,
            access_token: "test_token".to_string(),
            refresh_token: None,
            expires_at: Utc::now() - chrono::Duration::hours(1),
            user_info: None,
        };

        assert!(session.is_expired());
    }

    #[test]
    fn test_session_needs_refresh() {
        let session = Session {
            provider: OAuthProvider::Google,
            access_token: "test_token".to_string(),
            refresh_token: None,
            expires_at: Utc::now() + chrono::Duration::minutes(3),
            user_info: None,
        };

        assert!(!session.is_expired());
        assert!(session.needs_refresh());
    }

    #[test]
    fn test_session_manager() {
        let mut manager = SessionManager::new();
        assert!(!manager.is_authenticated());

        let session = Session {
            provider: OAuthProvider::Google,
            access_token: "test_token".to_string(),
            refresh_token: None,
            expires_at: Utc::now() + chrono::Duration::hours(1),
            user_info: None,
        };

        manager.set_session(session);
        assert!(manager.is_authenticated());
        assert!(manager.get_access_token().is_some());

        manager.clear_session();
        assert!(!manager.is_authenticated());
    }
}
