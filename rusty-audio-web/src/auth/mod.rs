//! OAuth authentication module for WASM
//!
//! Provides OAuth 2.0 authentication with PKCE support for secure
//! authentication in single-page applications.

pub mod oauth_client;
pub mod providers;
pub mod session;
pub mod token_storage;

pub use oauth_client::{OAuthClient, OAuthError};
pub use providers::OAuthProvider;
pub use session::{Session, SessionManager};
pub use token_storage::TokenStorage;
