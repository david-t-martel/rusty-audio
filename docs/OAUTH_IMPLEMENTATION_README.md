# OAuth 2.0 Authentication Implementation

## Quick Start

This implementation provides a complete OAuth 2.0 authentication system for Rusty Audio with Google, GitHub, and Microsoft sign-in support.

### Files Overview

```
static/
├── landing.html              # Pre-auth landing page with OAuth buttons
├── auth-callback.html        # OAuth callback handler
├── js/
│   ├── auth-manager.js       # OAuth flow orchestration
│   ├── token-storage.js      # Encrypted token storage (IndexedDB)
│   ├── session-manager.js    # Session lifecycle management
│   └── feature-gate.js       # Feature access control
└── css/
    └── auth.css              # Authentication UI styles

docs/
├── OAUTH_FLOW_DOCUMENTATION.md      # Complete flow documentation
├── SECURITY_BEST_PRACTICES.md       # Security guidelines
├── OAUTH_TESTING_CHECKLIST.md       # Comprehensive testing guide
└── OAUTH_IMPLEMENTATION_README.md   # This file
```

## Features

### ✅ Complete OAuth 2.0 Flow
- Google OAuth 2.0
- GitHub OAuth 2.0
- Microsoft OAuth 2.0
- Guest mode (limited features)

### 🔐 Security Features
- AES-GCM encryption for token storage
- CSRF protection via state parameter
- XSS prevention
- Origin validation
- HTTPS enforcement
- Secure token lifecycle

### 🔄 Session Management
- Automatic token refresh
- Multi-tab synchronization
- Activity monitoring
- Session timeout
- Graceful error recovery

### 🎯 Feature Gating
- Tier-based access control (Free, Authenticated, Premium)
- Smart upgrade prompts
- Feature lock indicators
- Graceful degradation

### 🎨 User Experience
- Modern, responsive UI (Tailwind CSS)
- Loading states and animations
- Error handling with user-friendly messages
- Popup-based OAuth (non-intrusive)
- Mobile-first design

## Installation

### 1. Include Required Files

Add to your main HTML file:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <!-- Tailwind CSS -->
  <script src="https://cdn.tailwindcss.com"></script>

  <!-- Auth CSS -->
  <link rel="stylesheet" href="/static/css/auth.css">
</head>
<body>
  <!-- Your app content -->

  <!-- Auth Scripts -->
  <script src="/static/js/token-storage.js"></script>
  <script src="/static/js/auth-manager.js"></script>
  <script src="/static/js/session-manager.js"></script>
  <script src="/static/js/feature-gate.js"></script>
</body>
</html>
```

### 2. Configure OAuth Providers

#### Google OAuth 2.0

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing
3. Enable Google+ API
4. Create OAuth 2.0 credentials
5. Add authorized redirect URIs:
   - `http://localhost:8080/auth-callback.html` (development)
   - `https://your-domain.com/auth-callback.html` (production)
6. Copy Client ID

#### GitHub OAuth

1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Click "New OAuth App"
3. Fill in application details:
   - Homepage URL: `https://your-domain.com`
   - Authorization callback URL: `https://your-domain.com/auth-callback.html`
4. Copy Client ID and Client Secret

#### Microsoft OAuth 2.0

1. Go to [Azure Portal](https://portal.azure.com/)
2. Navigate to Azure Active Directory > App Registrations
3. Click "New registration"
4. Configure redirect URI: `https://your-domain.com/auth-callback.html`
5. Copy Application (client) ID

### 3. Backend API Setup

Implement these endpoints:

```javascript
// GET /api/auth/config
// Returns OAuth client IDs
{
  "google": { "clientId": "..." },
  "github": { "clientId": "..." },
  "microsoft": { "clientId": "..." }
}

// POST /api/auth/token
// Exchanges authorization code for tokens
Request: { code, provider, redirectUri }
Response: { access_token, refresh_token, expires_in, user }

// POST /api/auth/refresh
// Refreshes access token
Request: { refreshToken }
Response: { access_token, refresh_token, expires_in }

// POST /api/auth/logout
// Revokes tokens
Headers: Authorization: Bearer {token}
Response: { success: true }
```

### 4. Configure Landing Page

Update landing page to be your entry point:

```javascript
// In your main app entry point
async function checkAuth() {
  const sessionManager = new SessionManager();
  const isAuthenticated = await sessionManager.initSession();

  if (!isAuthenticated) {
    // Redirect to landing page
    window.location.href = '/landing.html';
  } else {
    // Load main app
    loadApp();
  }
}

checkAuth();
```

## Usage

### Basic Authentication

```javascript
// Initialize auth manager
const authManager = new AuthManager();

// Sign in with Google
document.getElementById('google-signin').addEventListener('click', async () => {
  try {
    await authManager.signInWithGoogle();
    // User will be redirected after successful auth
  } catch (error) {
    console.error('Google sign-in failed:', error);
  }
});

// Check if user is authenticated
const isAuthenticated = await authManager.isAuthenticated();

// Get current user
const user = await authManager.getCurrentUser();
console.log(user); // { id, email, name, avatar, tier }

// Logout
await authManager.logout();
```

### Session Management

```javascript
// Initialize session
const sessionManager = new SessionManager();
await sessionManager.initSession();

// Get session stats
const stats = await sessionManager.getSessionStats();
console.log(stats);
// {
//   hasTokens: true,
//   hasUserInfo: true,
//   isTokenValid: true,
//   timeUntilExpiration: 3540000,
//   timeUntilExpirationMinutes: 59,
//   inactiveMinutes: 5
// }

// Manually refresh session
await sessionManager.forceRefresh();

// End session
await sessionManager.endSession();
```

### Feature Gating

```javascript
// Initialize feature gate with current user
const user = await authManager.getCurrentUser();
const featureGate = new FeatureGate(user);

// Check if user can access a feature
if (featureGate.canAccess('cloud_sync')) {
  // Enable cloud sync
  enableCloudSync();
}

// Handle feature access with automatic modals
await featureGate.handleFeatureAccess('ai_features', () => {
  // This callback only runs if user has access
  enableAIFeatures();
});

// Apply to UI element (disable if locked)
const aiButton = document.getElementById('ai-enhance-btn');
featureGate.applyFeatureGate(aiButton, 'ai_features');

// Get all available features for current user
const features = featureGate.getAvailableFeatures();
console.log(features);
// [
//   { name: 'basic_playback', tier: 'free', ... },
//   { name: 'cloud_sync', tier: 'authenticated', ... }
// ]
```

### Making Authenticated API Calls

```javascript
// Use authenticated fetch (auto-handles token refresh)
const response = await authManager.authenticatedFetch('/api/user/presets', {
  method: 'POST',
  body: JSON.stringify({ name: 'My Preset', settings: {...} })
});

const data = await response.json();
```

### Protected Route Guard

```javascript
// Protect routes that require authentication
const sessionGuard = new SessionGuard();

// Require authentication for current page
try {
  const isAuthenticated = await sessionGuard.requireAuth();

  if (isAuthenticated) {
    console.log('User is signed in');
  } else {
    console.log('User is in guest mode');
  }
} catch (error) {
  // User was redirected to landing page
  console.log('Authentication required');
}

// Check feature access
const canUseAI = await sessionGuard.canAccessFeature('ai_features');
if (!canUseAI) {
  console.log('User needs to upgrade for AI features');
}
```

## Customization

### Theming

Customize colors in landing.html:

```javascript
tailwind.config = {
  theme: {
    extend: {
      colors: {
        primary: {
          500: '#8b5cf6', // Change to your brand color
          600: '#7c3aed',
          700: '#6d28d9',
        }
      }
    }
  }
}
```

### Feature Tiers

Modify feature definitions in `feature-gate.js`:

```javascript
this.features = {
  // Add your custom features
  custom_feature: {
    tier: 'premium',
    name: 'Custom Feature',
    description: 'Description of your feature',
    icon: '🎉'
  }
};
```

### Session Timeout

Adjust timeout in `session-manager.js`:

```javascript
constructor() {
  this.activityTimeout = 30 * 60 * 1000; // 30 minutes
  this.sessionDuration = 30 * 24 * 60 * 60 * 1000; // 30 days
}
```

### Auto-Refresh Margin

Change when tokens refresh in `session-manager.js`:

```javascript
this.refreshMargin = 5 * 60 * 1000; // 5 minutes before expiration
```

## Security Checklist

Before deploying to production:

- [ ] All tokens stored encrypted in IndexedDB
- [ ] CSRF protection enabled (state parameter)
- [ ] HTTPS enforced in production
- [ ] Client secrets never exposed in frontend
- [ ] Origin validation for postMessage
- [ ] Content Security Policy configured
- [ ] Input sanitization implemented
- [ ] Rate limiting configured
- [ ] Audit logging enabled
- [ ] OAuth redirect URIs registered exactly

## Troubleshooting

### Popup Blocked

**Problem**: OAuth popup is blocked by browser

**Solution**:
```javascript
// Detect and handle popup blocking
const popup = window.open(authUrl, 'OAuth', 'width=500,height=600');
if (!popup || popup.closed || typeof popup.closed === 'undefined') {
  alert('Popup blocked. Please allow popups for authentication.');
}
```

### State Mismatch

**Problem**: "Invalid state" error

**Solution**:
1. Clear sessionStorage
2. Ensure state is generated before redirect
3. Check for sessionStorage being cleared
4. Verify callback is from correct OAuth flow

### Tokens Not Persisting

**Problem**: User logged out after page refresh

**Solution**:
1. Check IndexedDB is enabled
2. Verify browser is not in private mode
3. Check storage quota not exceeded
4. Ensure encryption key is stored

### Multi-Tab Logout Not Working

**Problem**: Logout in one tab doesn't affect others

**Solution**:
1. Verify localStorage events are firing
2. Check event listener is registered
3. Ensure storage event handler is working
4. Test in same origin

## Performance Tips

1. **Lazy Load Auth Modules**
   ```javascript
   // Only load when needed
   const { AuthManager } = await import('./js/auth-manager.js');
   ```

2. **Optimize Token Checks**
   ```javascript
   // Cache token validity check
   let cachedValidity = null;
   let cacheTime = 0;

   async function isTokenValid() {
     const now = Date.now();
     if (cachedValidity && now - cacheTime < 60000) {
       return cachedValidity;
     }
     cachedValidity = await tokenStorage.isTokenValid();
     cacheTime = now;
     return cachedValidity;
   }
   ```

3. **Debounce Activity Tracking**
   ```javascript
   let activityTimeout;
   function handleActivity() {
     clearTimeout(activityTimeout);
     activityTimeout = setTimeout(() => {
       sessionManager.extendSession();
     }, 1000);
   }
   ```

## Testing

Run the test suite:

```bash
# Unit tests
npm test

# Integration tests
npm run test:integration

# E2E tests
npm run test:e2e

# Coverage report
npm run test:coverage
```

See `OAUTH_TESTING_CHECKLIST.md` for comprehensive testing guide.

## Browser Support

| Browser | Version | Status |
|---------|---------|--------|
| Chrome | 60+ | ✅ Full Support |
| Firefox | 55+ | ✅ Full Support |
| Safari | 11+ | ✅ Full Support |
| Edge | 79+ | ✅ Full Support |
| iOS Safari | 11+ | ✅ Full Support |
| Chrome Android | Latest | ✅ Full Support |

### Required APIs
- IndexedDB
- Web Crypto API
- SessionStorage
- LocalStorage
- PostMessage
- Popup Windows

## Migration from Existing Auth

If you have an existing authentication system:

1. **Add OAuth alongside existing auth**
   ```javascript
   if (hasLegacySession()) {
     // Migrate to OAuth
     await migrateLegacySession();
   }
   ```

2. **Migrate user data**
   ```javascript
   async function migrateLegacySession() {
     const legacyToken = getLegacyToken();
     // Exchange legacy token for OAuth token
     const response = await fetch('/api/migrate', {
       headers: { 'Authorization': `Bearer ${legacyToken}` }
     });
     const { access_token, refresh_token, expires_in } = await response.json();
     await tokenStorage.storeTokens(access_token, refresh_token, expires_in);
   }
   ```

3. **Support both temporarily**
   ```javascript
   const isOAuthUser = await authManager.isAuthenticated();
   const isLegacyUser = hasLegacySession();

   if (!isOAuthUser && isLegacyUser) {
     // Prompt to migrate
     showMigrationPrompt();
   }
   ```

## API Reference

### AuthManager

```javascript
class AuthManager {
  async signInWithGoogle()
  async signInWithGitHub()
  async signInWithMicrosoft()
  async handleCallback(code, state)
  async refreshToken()
  async logout()
  async getCurrentUser()
  async isAuthenticated()
  async getValidAccessToken()
  async authenticatedFetch(url, options)
}
```

### SecureTokenStorage

```javascript
class SecureTokenStorage {
  async init()
  async storeTokens(accessToken, refreshToken, expiresIn)
  async getAccessToken()
  async getRefreshToken()
  async isTokenValid()
  async getTimeUntilExpiration()
  async storeUserInfo(userInfo)
  async getUserInfo()
  async clearTokens()
  async getStats()
}
```

### SessionManager

```javascript
class SessionManager {
  async initSession()
  async refreshSession()
  async endSession()
  async getSessionStats()
  async isSessionActive()
  async forceRefresh()
  extendSession()
}
```

### FeatureGate

```javascript
class FeatureGate {
  canAccess(featureName)
  isPremiumUser()
  getAvailableFeatures()
  getLockedFeatures()
  requiresUpgrade(featureName)
  requiresAuth(featureName)
  async handleFeatureAccess(featureName, callback)
  applyFeatureGate(element, featureName)
}
```

## Support

For issues and questions:

1. Check `OAUTH_FLOW_DOCUMENTATION.md` for detailed flow information
2. Review `SECURITY_BEST_PRACTICES.md` for security guidance
3. Use `OAUTH_TESTING_CHECKLIST.md` for debugging
4. Open an issue on GitHub

## License

This implementation is part of the Rusty Audio project.

## Changelog

### v1.0.0 (2024-11-16)
- Initial OAuth 2.0 implementation
- Google, GitHub, Microsoft support
- Encrypted token storage
- Multi-tab synchronization
- Feature gating system
- Comprehensive documentation
