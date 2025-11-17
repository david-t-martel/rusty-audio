//! OAuth provider configurations
//!
//! Defines OAuth provider endpoints and configurations for Google, GitHub, and Microsoft.

use serde::{Deserialize, Serialize};

/// OAuth provider enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OAuthProvider {
    /// Google OAuth
    Google,
    /// GitHub OAuth
    GitHub,
    /// Microsoft OAuth
    Microsoft,
}

/// Provider configuration
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Authorization endpoint
    pub auth_endpoint: &'static str,
    /// Token endpoint
    pub token_endpoint: &'static str,
    /// Revoke endpoint (optional)
    pub revoke_endpoint: Option<&'static str>,
    /// Scopes to request
    pub default_scopes: &'static [&'static str],
}

impl OAuthProvider {
    /// Get provider configuration
    ///
    /// # Returns
    /// Provider-specific configuration
    pub fn config(&self) -> ProviderConfig {
        match self {
            OAuthProvider::Google => ProviderConfig {
                auth_endpoint: "https://accounts.google.com/o/oauth2/v2/auth",
                token_endpoint: "https://oauth2.googleapis.com/token",
                revoke_endpoint: Some("https://oauth2.googleapis.com/revoke"),
                default_scopes: &["openid", "email", "profile"],
            },
            OAuthProvider::GitHub => ProviderConfig {
                auth_endpoint: "https://github.com/login/oauth/authorize",
                token_endpoint: "https://github.com/login/oauth/access_token",
                revoke_endpoint: None,
                default_scopes: &["user:email"],
            },
            OAuthProvider::Microsoft => ProviderConfig {
                auth_endpoint: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
                token_endpoint: "https://login.microsoftonline.com/common/oauth2/v2.0/token",
                revoke_endpoint: None,
                default_scopes: &["openid", "email", "profile"],
            },
        }
    }

    /// Get provider name
    ///
    /// # Returns
    /// Human-readable provider name
    pub fn name(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "Google",
            OAuthProvider::GitHub => "GitHub",
            OAuthProvider::Microsoft => "Microsoft",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_config() {
        let config = OAuthProvider::Google.config();
        assert!(config.auth_endpoint.contains("google.com"));
        assert!(!config.default_scopes.is_empty());
    }

    #[test]
    fn test_github_config() {
        let config = OAuthProvider::GitHub.config();
        assert!(config.auth_endpoint.contains("github.com"));
        assert!(!config.default_scopes.is_empty());
    }

    #[test]
    fn test_microsoft_config() {
        let config = OAuthProvider::Microsoft.config();
        assert!(config.auth_endpoint.contains("microsoftonline.com"));
        assert!(!config.default_scopes.is_empty());
    }

    #[test]
    fn test_provider_names() {
        assert_eq!(OAuthProvider::Google.name(), "Google");
        assert_eq!(OAuthProvider::GitHub.name(), "GitHub");
        assert_eq!(OAuthProvider::Microsoft.name(), "Microsoft");
    }
}
