/**
 * Feature Gate System
 *
 * Controls access to premium features based on user authentication and tier
 */

class FeatureGate {
  constructor(user = null) {
    this.user = user;
    this.guestMode = localStorage.getItem('guest_mode') === 'true';

    // Feature definitions
    this.features = {
      // Free tier features
      basic_playback: {
        tier: 'free',
        name: 'Basic Playback',
        description: 'Play audio files with standard controls'
      },
      spectrum_visualizer: {
        tier: 'free',
        name: 'Spectrum Visualizer',
        description: 'Real-time frequency spectrum display'
      },
      basic_eq: {
        tier: 'free',
        name: 'Basic Equalizer',
        description: '8-band parametric equalizer'
      },
      volume_control: {
        tier: 'free',
        name: 'Volume Control',
        description: 'Master volume and panning'
      },
      signal_generator: {
        tier: 'free',
        name: 'Signal Generator',
        description: 'Test tone generation'
      },
      local_presets: {
        tier: 'free',
        name: 'Local Presets',
        description: 'Save presets locally'
      },

      // Authenticated user features
      cloud_sync: {
        tier: 'authenticated',
        name: 'Cloud Preset Sync',
        description: 'Synchronize presets across devices',
        icon: '☁️'
      },
      usage_statistics: {
        tier: 'authenticated',
        name: 'Usage Statistics',
        description: 'Track your listening habits',
        icon: '📊'
      },
      profile_customization: {
        tier: 'authenticated',
        name: 'Profile Customization',
        description: 'Customize your profile and settings',
        icon: '👤'
      },

      // Premium features
      ai_features: {
        tier: 'premium',
        name: 'AI Audio Enhancement',
        description: 'AI-powered audio optimization',
        icon: '🤖'
      },
      advanced_effects: {
        tier: 'premium',
        name: 'Advanced Effects',
        description: 'Professional audio effects suite',
        icon: '🎛️'
      },
      noise_reduction: {
        tier: 'premium',
        name: 'Noise Reduction',
        description: 'Intelligent noise cancellation',
        icon: '🔇'
      },
      preset_recommendations: {
        tier: 'premium',
        name: 'Smart Recommendations',
        description: 'AI-powered preset suggestions',
        icon: '✨'
      },
      priority_support: {
        tier: 'premium',
        name: 'Priority Support',
        description: 'Get help faster',
        icon: '🚀'
      },
      early_access: {
        tier: 'premium',
        name: 'Early Access',
        description: 'Try new features first',
        icon: '🎁'
      },
      unlimited_cloud_storage: {
        tier: 'premium',
        name: 'Unlimited Cloud Storage',
        description: 'Store unlimited presets',
        icon: '💾'
      }
    };
  }

  /**
   * Set current user
   */
  setUser(user) {
    this.user = user;
    this.guestMode = !user;
  }

  /**
   * Check if user can access a feature
   */
  canAccess(featureName) {
    const feature = this.features[featureName];

    if (!feature) {
      console.warn(`Unknown feature: ${featureName}`);
      return false;
    }

    // Guest mode - only free features
    if (this.guestMode) {
      return feature.tier === 'free';
    }

    // No user - only free features
    if (!this.user) {
      return feature.tier === 'free';
    }

    // Authenticated users
    if (feature.tier === 'free' || feature.tier === 'authenticated') {
      return true;
    }

    // Premium features
    if (feature.tier === 'premium') {
      return this.isPremiumUser();
    }

    return false;
  }

  /**
   * Check if user has premium tier
   */
  isPremiumUser() {
    if (!this.user) return false;
    return this.user.tier === 'Premium' || this.user.tier === 'premium';
  }

  /**
   * Get all available features for current user
   */
  getAvailableFeatures() {
    return Object.entries(this.features)
      .filter(([name, _]) => this.canAccess(name))
      .map(([name, feature]) => ({ name, ...feature }));
  }

  /**
   * Get all locked features for current user
   */
  getLockedFeatures() {
    return Object.entries(this.features)
      .filter(([name, _]) => !this.canAccess(name))
      .map(([name, feature]) => ({ name, ...feature }));
  }

  /**
   * Get feature by name
   */
  getFeature(featureName) {
    return this.features[featureName];
  }

  /**
   * Check if feature requires upgrade
   */
  requiresUpgrade(featureName) {
    const feature = this.features[featureName];
    if (!feature) return false;

    return !this.canAccess(featureName) && feature.tier === 'premium';
  }

  /**
   * Check if feature requires authentication
   */
  requiresAuth(featureName) {
    const feature = this.features[featureName];
    if (!feature) return false;

    return !this.canAccess(featureName) &&
           (feature.tier === 'authenticated' || feature.tier === 'premium');
  }

  /**
   * Handle feature access attempt
   */
  async handleFeatureAccess(featureName, callback) {
    if (this.canAccess(featureName)) {
      // User has access, execute callback
      if (callback) await callback();
      return true;
    }

    // Feature is locked
    if (this.requiresUpgrade(featureName)) {
      this.showUpgradeModal(featureName);
      return false;
    }

    if (this.requiresAuth(featureName)) {
      this.showAuthModal(featureName);
      return false;
    }

    return false;
  }

  /**
   * Show upgrade modal
   */
  showUpgradeModal(featureName) {
    const feature = this.features[featureName];
    const modal = this.createModal({
      title: '🔒 Premium Feature',
      message: `<strong>${feature.name}</strong> is a premium feature.`,
      description: feature.description,
      buttons: [
        {
          text: 'Upgrade to Premium',
          class: 'bg-gradient-to-r from-purple-600 to-purple-700 text-white',
          onClick: () => this.redirectToUpgrade()
        },
        {
          text: 'Maybe Later',
          class: 'bg-gray-700 text-white',
          onClick: () => this.closeModal()
        }
      ]
    });

    this.displayModal(modal);
  }

  /**
   * Show authentication modal
   */
  showAuthModal(featureName) {
    const feature = this.features[featureName];
    const modal = this.createModal({
      title: '🔐 Sign In Required',
      message: `<strong>${feature.name}</strong> requires a free account.`,
      description: feature.description,
      buttons: [
        {
          text: 'Sign In',
          class: 'bg-gradient-to-r from-blue-600 to-blue-700 text-white',
          onClick: () => this.redirectToAuth()
        },
        {
          text: 'Cancel',
          class: 'bg-gray-700 text-white',
          onClick: () => this.closeModal()
        }
      ]
    });

    this.displayModal(modal);
  }

  /**
   * Create modal HTML
   */
  createModal({ title, message, description, buttons }) {
    const buttonsHtml = buttons.map(btn => `
      <button
        class="${btn.class} px-6 py-3 rounded-lg font-semibold transition hover:opacity-90"
        data-action="${btn.text}"
      >
        ${btn.text}
      </button>
    `).join('');

    return `
      <div id="feature-gate-modal" class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-75">
        <div class="bg-gray-800 rounded-xl shadow-2xl max-w-md w-full mx-4 p-8 animate-fade-in">
          <h3 class="text-2xl font-bold mb-4">${title}</h3>
          <p class="text-lg mb-2">${message}</p>
          <p class="text-gray-400 mb-6">${description}</p>
          <div class="flex space-x-4">
            ${buttonsHtml}
          </div>
        </div>
      </div>
    `;
  }

  /**
   * Display modal
   */
  displayModal(html) {
    // Remove existing modal
    this.closeModal();

    // Add modal to DOM
    const div = document.createElement('div');
    div.innerHTML = html;
    document.body.appendChild(div.firstElementChild);

    // Add event listeners
    const modal = document.getElementById('feature-gate-modal');
    modal.querySelectorAll('button').forEach(btn => {
      btn.addEventListener('click', () => {
        const action = btn.getAttribute('data-action');
        if (action === 'Upgrade to Premium') {
          this.redirectToUpgrade();
        } else if (action === 'Sign In') {
          this.redirectToAuth();
        } else {
          this.closeModal();
        }
      });
    });

    // Close on background click
    modal.addEventListener('click', (e) => {
      if (e.target === modal) {
        this.closeModal();
      }
    });

    // Close on Escape key
    const escapeHandler = (e) => {
      if (e.key === 'Escape') {
        this.closeModal();
        document.removeEventListener('keydown', escapeHandler);
      }
    };
    document.addEventListener('keydown', escapeHandler);
  }

  /**
   * Close modal
   */
  closeModal() {
    const modal = document.getElementById('feature-gate-modal');
    if (modal) {
      modal.remove();
    }
  }

  /**
   * Redirect to upgrade page
   */
  redirectToUpgrade() {
    window.location.href = '/upgrade';
  }

  /**
   * Redirect to authentication page
   */
  redirectToAuth() {
    // Store current page for redirect after auth
    sessionStorage.setItem('auth_redirect', window.location.pathname);
    window.location.href = '/';
  }

  /**
   * Add lock icon to feature element
   */
  addLockIcon(element, featureName) {
    if (!this.canAccess(featureName)) {
      const lockIcon = document.createElement('span');
      lockIcon.className = 'lock-icon';
      lockIcon.innerHTML = '🔒';
      lockIcon.style.cssText = 'margin-left: 8px; font-size: 0.9em; opacity: 0.7;';
      element.appendChild(lockIcon);

      // Make element clickable to show modal
      element.style.cursor = 'pointer';
      element.addEventListener('click', (e) => {
        e.preventDefault();
        e.stopPropagation();
        this.handleFeatureAccess(featureName);
      });
    }
  }

  /**
   * Disable feature element if locked
   */
  applyFeatureGate(element, featureName) {
    if (!this.canAccess(featureName)) {
      element.disabled = true;
      element.classList.add('locked-feature');
      element.title = `Requires ${this.features[featureName]?.tier} access`;

      // Add visual indicator
      element.style.opacity = '0.5';
      element.style.cursor = 'not-allowed';

      // Show modal on click
      element.addEventListener('click', (e) => {
        e.preventDefault();
        e.stopPropagation();
        this.handleFeatureAccess(featureName);
      });
    }
  }

  /**
   * Create feature badge
   */
  createFeatureBadge(tier) {
    const badges = {
      free: { text: 'Free', class: 'bg-gray-600' },
      authenticated: { text: 'Signed In', class: 'bg-blue-600' },
      premium: { text: 'Premium', class: 'bg-purple-600' }
    };

    const badge = badges[tier] || badges.free;

    return `
      <span class="${badge.class} text-white text-xs px-2 py-1 rounded-full font-semibold">
        ${badge.text}
      </span>
    `;
  }
}

// Export for use in other modules
if (typeof module !== 'undefined' && module.exports) {
  module.exports = FeatureGate;
}
