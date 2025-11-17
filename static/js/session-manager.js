/**
 * Session Management
 *
 * Handles session initialization, auto-refresh, and multi-tab synchronization
 */

class SessionManager {
  constructor() {
    this.tokenStorage = new SecureTokenStorage();
    this.authManager = null; // Will be set when AuthManager is available
    this.refreshInterval = null;
    this.refreshMargin = 5 * 60 * 1000; // Refresh 5 minutes before expiration
    this.activityTimeout = 30 * 60 * 1000; // 30 minutes of inactivity
    this.lastActivityTime = Date.now();
    this.sessionDuration = 30 * 24 * 60 * 60 * 1000; // 30 days

    // Bind methods for event listeners
    this.handleActivity = this.handleActivity.bind(this);
    this.handleStorageChange = this.handleStorageChange.bind(this);
    this.handleVisibilityChange = this.handleVisibilityChange.bind(this);
  }

  /**
   * Initialize session
   */
  async initSession() {
    console.log('Initializing session...');

    // Check if user is authenticated
    const isValid = await this.tokenStorage.isTokenValid();

    if (!isValid) {
      console.log('No valid session found');
      return false;
    }

    // Setup activity monitoring
    this.setupActivityMonitoring();

    // Setup multi-tab synchronization
    this.setupMultiTabSync();

    // Setup auto-refresh
    await this.setupAutoRefresh();

    // Setup visibility change handler
    this.setupVisibilityHandler();

    console.log('Session initialized successfully');
    return true;
  }

  /**
   * Setup auto token refresh
   */
  async setupAutoRefresh() {
    // Clear existing interval
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
    }

    // Calculate time until refresh needed
    const timeUntilExpiration = await this.tokenStorage.getTimeUntilExpiration();

    if (timeUntilExpiration <= 0) {
      console.log('Token already expired, attempting refresh...');
      await this.refreshSession();
      return;
    }

    // Schedule refresh before expiration
    const timeUntilRefresh = Math.max(0, timeUntilExpiration - this.refreshMargin);

    console.log(
      `Token refresh scheduled in ${Math.round(timeUntilRefresh / 1000 / 60)} minutes`
    );

    setTimeout(async () => {
      await this.refreshSession();
    }, timeUntilRefresh);

    // Also check periodically (every 5 minutes)
    this.refreshInterval = setInterval(async () => {
      const remaining = await this.tokenStorage.getTimeUntilExpiration();

      if (remaining <= this.refreshMargin && remaining > 0) {
        await this.refreshSession();
      }
    }, 5 * 60 * 1000);
  }

  /**
   * Refresh session tokens
   */
  async refreshSession() {
    console.log('Refreshing session...');

    if (!this.authManager) {
      this.authManager = new AuthManager();
    }

    try {
      await this.authManager.refreshToken();
      console.log('Session refreshed successfully');

      // Broadcast refresh to other tabs
      this.broadcastSessionUpdate('refresh');

      // Reschedule next refresh
      await this.setupAutoRefresh();
    } catch (error) {
      console.error('Session refresh failed:', error);
      await this.endSession();
      throw error;
    }
  }

  /**
   * Setup activity monitoring
   */
  setupActivityMonitoring() {
    // Track user activity
    const activityEvents = [
      'mousedown',
      'keydown',
      'scroll',
      'touchstart',
      'click'
    ];

    activityEvents.forEach(event => {
      window.addEventListener(event, this.handleActivity, { passive: true });
    });

    // Check for inactivity periodically
    setInterval(() => {
      const inactiveTime = Date.now() - this.lastActivityTime;

      if (inactiveTime >= this.activityTimeout) {
        console.log('User inactive, session will be maintained but refreshes may pause');
        // Optionally implement auto-logout on inactivity
        // this.endSession();
      }
    }, 60 * 1000); // Check every minute
  }

  /**
   * Handle user activity
   */
  handleActivity() {
    this.lastActivityTime = Date.now();
  }

  /**
   * Setup multi-tab synchronization
   */
  setupMultiTabSync() {
    // Listen for storage events from other tabs
    window.addEventListener('storage', this.handleStorageChange);

    // Announce this tab
    this.broadcastSessionUpdate('tab_opened');
  }

  /**
   * Handle storage changes from other tabs
   */
  async handleStorageChange(event) {
    // Logout event from another tab
    if (event.key === 'logout_event') {
      console.log('Logout detected from another tab');
      await this.tokenStorage.clearTokens();
      window.location.href = '/';
      return;
    }

    // Session update from another tab
    if (event.key === 'session_update') {
      const update = JSON.parse(event.newValue || '{}');

      if (update.type === 'refresh') {
        console.log('Token refresh detected from another tab');
        // Reload session state
        await this.setupAutoRefresh();
      }
    }
  }

  /**
   * Broadcast session update to other tabs
   */
  broadcastSessionUpdate(type, data = {}) {
    const update = {
      type,
      timestamp: Date.now(),
      ...data
    };

    localStorage.setItem('session_update', JSON.stringify(update));
    // Remove immediately to trigger event in other tabs
    localStorage.removeItem('session_update');
  }

  /**
   * Setup visibility change handler
   */
  setupVisibilityHandler() {
    document.addEventListener('visibilitychange', this.handleVisibilityChange);
  }

  /**
   * Handle page visibility changes
   */
  async handleVisibilityChange() {
    if (!document.hidden) {
      console.log('Page became visible, checking session...');

      // Check if token is still valid
      const isValid = await this.tokenStorage.isTokenValid();

      if (!isValid) {
        console.log('Session expired while tab was hidden');
        await this.endSession();
        return;
      }

      // Check if refresh is needed
      const timeUntilExpiration = await this.tokenStorage.getTimeUntilExpiration();

      if (timeUntilExpiration <= this.refreshMargin) {
        await this.refreshSession();
      }
    }
  }

  /**
   * End session and cleanup
   */
  async endSession() {
    console.log('Ending session...');

    // Clear refresh interval
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
      this.refreshInterval = null;
    }

    // Remove event listeners
    const activityEvents = [
      'mousedown',
      'keydown',
      'scroll',
      'touchstart',
      'click'
    ];

    activityEvents.forEach(event => {
      window.removeEventListener(event, this.handleActivity);
    });

    window.removeEventListener('storage', this.handleStorageChange);
    document.removeEventListener('visibilitychange', this.handleVisibilityChange);

    // Clear tokens
    await this.tokenStorage.clearTokens();

    // Redirect to landing page
    window.location.href = '/';
  }

  /**
   * Get session statistics
   */
  async getSessionStats() {
    const stats = await this.tokenStorage.getStats();
    const timeUntilExpiration = await this.tokenStorage.getTimeUntilExpiration();
    const inactiveTime = Date.now() - this.lastActivityTime;

    return {
      ...stats,
      timeUntilExpiration,
      timeUntilExpirationMinutes: Math.round(timeUntilExpiration / 1000 / 60),
      inactiveTime,
      inactiveMinutes: Math.round(inactiveTime / 1000 / 60),
      lastActivityTime: this.lastActivityTime
    };
  }

  /**
   * Extend session (reset activity timer)
   */
  extendSession() {
    this.lastActivityTime = Date.now();
    console.log('Session extended');
  }

  /**
   * Check if session is active
   */
  async isSessionActive() {
    const isValid = await this.tokenStorage.isTokenValid();
    const inactiveTime = Date.now() - this.lastActivityTime;

    return isValid && inactiveTime < this.activityTimeout;
  }

  /**
   * Force session refresh
   */
  async forceRefresh() {
    await this.refreshSession();
  }
}

/**
 * Global session guard for protected pages
 */
class SessionGuard {
  constructor() {
    this.sessionManager = new SessionManager();
    this.authManager = new AuthManager();
  }

  /**
   * Require authentication for current page
   */
  async requireAuth() {
    const isAuthenticated = await this.authManager.isAuthenticated();

    if (!isAuthenticated) {
      // Check if guest mode is allowed
      const guestMode = localStorage.getItem('guest_mode');

      if (!guestMode) {
        // Redirect to landing page
        window.location.href = '/';
        throw new Error('Authentication required');
      }
    }

    // Initialize session
    await this.sessionManager.initSession();

    return isAuthenticated;
  }

  /**
   * Check feature access
   */
  async canAccessFeature(featureName) {
    const user = await this.authManager.getCurrentUser();

    if (!user) {
      // Guest mode - check if feature is available
      return this.isFeatureAvailableForGuests(featureName);
    }

    // Check user tier
    return this.isFeatureAvailableForTier(featureName, user.tier);
  }

  /**
   * Check if feature is available for guests
   */
  isFeatureAvailableForGuests(featureName) {
    const guestFeatures = [
      'basic_playback',
      'spectrum_visualizer',
      'basic_eq'
    ];

    return guestFeatures.includes(featureName);
  }

  /**
   * Check if feature is available for user tier
   */
  isFeatureAvailableForTier(featureName, tier) {
    const premiumFeatures = [
      'cloud_sync',
      'ai_features',
      'advanced_effects',
      'preset_recommendations',
      'noise_reduction'
    ];

    if (premiumFeatures.includes(featureName)) {
      return tier === 'Premium' || tier === 'premium';
    }

    // All other features available to all authenticated users
    return true;
  }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { SessionManager, SessionGuard };
}
