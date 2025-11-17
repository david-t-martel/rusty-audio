/**
 * OAuth 2.0 Authentication Manager
 *
 * Handles authentication flows for Google, GitHub, and Microsoft
 */

class AuthManager {
  constructor() {
    this.tokenStorage = new SecureTokenStorage();
    this.apiBaseUrl = '/api/auth'; // Backend OAuth endpoints

    // OAuth Configuration
    this.config = {
      google: {
        clientId: '', // Set via backend config
        authEndpoint: 'https://accounts.google.com/o/oauth2/v2/auth',
        tokenEndpoint: 'https://oauth2.googleapis.com/token',
        scope: 'openid profile email',
        responseType: 'code',
        redirectUri: `${window.location.origin}/auth-callback.html`
      },
      github: {
        clientId: '', // Set via backend config
        authEndpoint: 'https://github.com/login/oauth/authorize',
        tokenEndpoint: 'https://github.com/login/oauth/access_token',
        scope: 'read:user user:email',
        responseType: 'code',
        redirectUri: `${window.location.origin}/auth-callback.html`
      },
      microsoft: {
        clientId: '', // Set via backend config
        authEndpoint: 'https://login.microsoftonline.com/common/oauth2/v2.0/authorize',
        tokenEndpoint: 'https://login.microsoftonline.com/common/oauth2/v2.0/token',
        scope: 'openid profile email',
        responseType: 'code',
        redirectUri: `${window.location.origin}/auth-callback.html`
      }
    };

    // Load configuration from backend
    this.loadConfig();
  }

  /**
   * Load OAuth configuration from backend
   */
  async loadConfig() {
    try {
      const response = await fetch(`${this.apiBaseUrl}/config`);
      if (response.ok) {
        const config = await response.json();
        this.config.google.clientId = config.google?.clientId || '';
        this.config.github.clientId = config.github?.clientId || '';
        this.config.microsoft.clientId = config.microsoft?.clientId || '';
      }
    } catch (error) {
      console.error('Failed to load OAuth config:', error);
    }
  }

  /**
   * Generate a random state parameter for CSRF protection
   */
  generateState() {
    const array = new Uint8Array(32);
    crypto.getRandomValues(array);
    return btoa(String.fromCharCode(...array))
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=+$/, '');
  }

  /**
   * Generate PKCE code verifier and challenge
   */
  async generatePKCE() {
    const verifier = this.generateState();

    // Create SHA-256 hash of verifier
    const encoder = new TextEncoder();
    const data = encoder.encode(verifier);
    const hash = await crypto.subtle.digest('SHA-256', data);

    // Base64 URL encode the hash
    const challenge = btoa(String.fromCharCode(...new Uint8Array(hash)))
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=+$/, '');

    return { verifier, challenge };
  }

  /**
   * Build authorization URL
   */
  buildAuthUrl(provider) {
    const config = this.config[provider];
    const state = this.generateState();

    // Store state for verification
    sessionStorage.setItem('oauth_state', state);
    sessionStorage.setItem('oauth_provider', provider);

    const params = new URLSearchParams({
      client_id: config.clientId,
      redirect_uri: config.redirectUri,
      response_type: config.responseType,
      scope: config.scope,
      state: state
    });

    // Add provider-specific parameters
    if (provider === 'microsoft') {
      params.append('response_mode', 'query');
    }

    return `${config.authEndpoint}?${params.toString()}`;
  }

  /**
   * Sign in with Google
   */
  async signInWithGoogle() {
    console.log('Initiating Google sign-in...');

    if (!this.config.google.clientId) {
      throw new Error('Google OAuth not configured');
    }

    const authUrl = this.buildAuthUrl('google');

    // Open OAuth flow in popup
    const popup = window.open(
      authUrl,
      'Google Sign In',
      'width=500,height=600,scrollbars=yes'
    );

    if (!popup) {
      throw new Error('Popup blocked. Please allow popups for authentication.');
    }

    // Monitor popup for completion
    return this.monitorPopup(popup);
  }

  /**
   * Sign in with GitHub
   */
  async signInWithGitHub() {
    console.log('Initiating GitHub sign-in...');

    if (!this.config.github.clientId) {
      throw new Error('GitHub OAuth not configured');
    }

    const authUrl = this.buildAuthUrl('github');

    const popup = window.open(
      authUrl,
      'GitHub Sign In',
      'width=500,height=600,scrollbars=yes'
    );

    if (!popup) {
      throw new Error('Popup blocked. Please allow popups for authentication.');
    }

    return this.monitorPopup(popup);
  }

  /**
   * Sign in with Microsoft
   */
  async signInWithMicrosoft() {
    console.log('Initiating Microsoft sign-in...');

    if (!this.config.microsoft.clientId) {
      throw new Error('Microsoft OAuth not configured');
    }

    const authUrl = this.buildAuthUrl('microsoft');

    const popup = window.open(
      authUrl,
      'Microsoft Sign In',
      'width=500,height=600,scrollbars=yes'
    );

    if (!popup) {
      throw new Error('Popup blocked. Please allow popups for authentication.');
    }

    return this.monitorPopup(popup);
  }

  /**
   * Monitor OAuth popup for completion
   */
  monitorPopup(popup) {
    return new Promise((resolve, reject) => {
      // Check if popup was closed
      const checkClosed = setInterval(() => {
        if (popup.closed) {
          clearInterval(checkClosed);
          reject(new Error('Authentication cancelled'));
        }
      }, 500);

      // Listen for message from popup
      const messageHandler = (event) => {
        if (event.origin !== window.location.origin) return;

        if (event.data.code) {
          clearInterval(checkClosed);
          window.removeEventListener('message', messageHandler);
          popup.close();
          resolve(event.data);
        } else if (event.data.error) {
          clearInterval(checkClosed);
          window.removeEventListener('message', messageHandler);
          popup.close();
          reject(new Error(event.data.error));
        }
      };

      window.addEventListener('message', messageHandler);
    });
  }

  /**
   * Handle OAuth callback
   */
  async handleCallback(code, state) {
    // Verify state parameter
    const savedState = sessionStorage.getItem('oauth_state');
    const provider = sessionStorage.getItem('oauth_provider');

    if (state !== savedState) {
      throw new Error('Invalid state parameter - possible CSRF attack');
    }

    // Clear state
    sessionStorage.removeItem('oauth_state');
    sessionStorage.removeItem('oauth_provider');

    // Exchange code for tokens via backend
    const response = await fetch(`${this.apiBaseUrl}/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        code,
        provider,
        redirectUri: this.config[provider].redirectUri
      })
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || 'Failed to exchange code for tokens');
    }

    const data = await response.json();

    // Store tokens securely
    await this.tokenStorage.storeTokens(
      data.access_token,
      data.refresh_token,
      data.expires_in
    );

    // Store user info
    if (data.user) {
      await this.tokenStorage.storeUserInfo(data.user);
    }

    return data;
  }

  /**
   * Refresh access token
   */
  async refreshToken() {
    const refreshToken = await this.tokenStorage.getRefreshToken();

    if (!refreshToken) {
      throw new Error('No refresh token available');
    }

    const response = await fetch(`${this.apiBaseUrl}/refresh`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ refreshToken })
    });

    if (!response.ok) {
      // Refresh failed - user needs to sign in again
      await this.logout();
      throw new Error('Session expired. Please sign in again.');
    }

    const data = await response.json();

    // Store new tokens
    await this.tokenStorage.storeTokens(
      data.access_token,
      data.refresh_token,
      data.expires_in
    );

    return data.access_token;
  }

  /**
   * Logout user
   */
  async logout() {
    console.log('Logging out...');

    // Get current access token for backend logout
    const accessToken = await this.tokenStorage.getAccessToken();

    // Clear local tokens
    await this.tokenStorage.clearTokens();

    // Notify backend
    if (accessToken) {
      try {
        await fetch(`${this.apiBaseUrl}/logout`, {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${accessToken}`
          }
        });
      } catch (error) {
        console.error('Backend logout failed:', error);
      }
    }

    // Broadcast logout to other tabs
    localStorage.setItem('logout_event', Date.now().toString());
    localStorage.removeItem('logout_event');

    // Redirect to landing page
    window.location.href = '/';
  }

  /**
   * Get current user information
   */
  async getCurrentUser() {
    return await this.tokenStorage.getUserInfo();
  }

  /**
   * Check if user is authenticated
   */
  async isAuthenticated() {
    return await this.tokenStorage.isTokenValid();
  }

  /**
   * Get valid access token (auto-refresh if needed)
   */
  async getValidAccessToken() {
    const isValid = await this.tokenStorage.isTokenValid();

    if (isValid) {
      return await this.tokenStorage.getAccessToken();
    }

    // Token expired, try to refresh
    try {
      return await this.refreshToken();
    } catch (error) {
      console.error('Token refresh failed:', error);
      return null;
    }
  }

  /**
   * Make authenticated API request
   */
  async authenticatedFetch(url, options = {}) {
    const token = await this.getValidAccessToken();

    if (!token) {
      throw new Error('Not authenticated');
    }

    const headers = {
      ...options.headers,
      'Authorization': `Bearer ${token}`
    };

    const response = await fetch(url, {
      ...options,
      headers
    });

    // Handle 401 Unauthorized
    if (response.status === 401) {
      // Try to refresh token and retry once
      try {
        const newToken = await this.refreshToken();
        headers.Authorization = `Bearer ${newToken}`;

        return await fetch(url, {
          ...options,
          headers
        });
      } catch (error) {
        // Refresh failed, logout
        await this.logout();
        throw new Error('Authentication expired');
      }
    }

    return response;
  }
}

// Error messages for user-friendly display
const ERROR_MESSAGES = {
  'invalid_token': 'Your session has expired. Please sign in again.',
  'network_error': 'Connection error. Please check your internet connection.',
  'oauth_error': 'Authentication failed. Please try again.',
  'rate_limit': 'Too many requests. Please wait a moment and try again.',
  'invalid_state': 'Security verification failed. Please try signing in again.',
  'popup_blocked': 'Popup blocked. Please allow popups for authentication.',
  'authentication_cancelled': 'Sign-in was cancelled.',
  'session_expired': 'Your session has expired. Please sign in again.'
};

/**
 * Get user-friendly error message
 */
function getErrorMessage(error) {
  if (typeof error === 'string') {
    return ERROR_MESSAGES[error] || error;
  }

  if (error instanceof Error) {
    const message = error.message.toLowerCase();
    for (const [key, userMessage] of Object.entries(ERROR_MESSAGES)) {
      if (message.includes(key.replace('_', ' '))) {
        return userMessage;
      }
    }
    return error.message;
  }

  return 'An unexpected error occurred. Please try again.';
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { AuthManager, ERROR_MESSAGES, getErrorMessage };
}
