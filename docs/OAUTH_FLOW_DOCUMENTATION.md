# OAuth 2.0 Authentication Flow Documentation

## Overview

Rusty Audio implements a complete OAuth 2.0 authentication system supporting Google, GitHub, and Microsoft sign-in. The system provides secure token management, automatic token refresh, and multi-tab session synchronization.

## Architecture

### Components

1. **Landing Page** (`static/landing.html`)
   - Pre-authentication entry point
   - OAuth provider selection
   - Guest mode option
   - Feature comparison

2. **Auth Manager** (`static/js/auth-manager.js`)
   - OAuth flow orchestration
   - Token exchange
   - Authentication state management

3. **Token Storage** (`static/js/token-storage.js`)
   - Encrypted token storage using IndexedDB
   - Web Crypto API for AES-GCM encryption
   - Secure key management

4. **Session Manager** (`static/js/session-manager.js`)
   - Session initialization
   - Auto token refresh
   - Multi-tab synchronization
   - Activity monitoring

5. **Feature Gate** (`static/js/feature-gate.js`)
   - Feature access control
   - Tier-based permissions
   - Upgrade prompts

6. **OAuth Callback Handler** (`static/auth-callback.html`)
   - Receives OAuth redirect
   - Extracts authorization code
   - CSRF protection via state parameter

## Authentication Flow

### 1. Initial Landing

```
User visits landing page
  ↓
Check for existing session (IndexedDB)
  ↓
If valid session exists → Redirect to app
  ↓
If no session → Show OAuth buttons
```

### 2. OAuth Initiation

```
User clicks "Sign in with [Provider]"
  ↓
Generate random state parameter (CSRF protection)
  ↓
Store state in sessionStorage
  ↓
Build authorization URL with:
  - client_id
  - redirect_uri
  - response_type: code
  - scope
  - state
  ↓
Open OAuth provider in popup window
```

### 3. OAuth Provider Flow

```
User authenticates with provider
  ↓
Provider redirects to auth-callback.html with:
  - code (authorization code)
  - state (for verification)
  ↓
Callback page validates state parameter
  ↓
Send code + state to parent window via postMessage
  ↓
Close popup
```

### 4. Token Exchange

```
Parent window receives code
  ↓
Send code to backend /api/auth/token
  ↓
Backend exchanges code for tokens:
  - access_token
  - refresh_token
  - expires_in
  ↓
Backend returns tokens + user info
  ↓
Store tokens in IndexedDB (encrypted)
  ↓
Initialize session
  ↓
Redirect to app
```

### 5. Session Management

```
Session initialized
  ↓
Start auto-refresh timer
  ↓
Monitor user activity
  ↓
Listen for multi-tab events
  ↓
Before token expiration:
  - Send refresh_token to backend
  - Receive new access_token
  - Update IndexedDB
  - Broadcast to other tabs
```

## Security Features

### 1. Token Encryption

All tokens are encrypted before storage using Web Crypto API:

```javascript
// Generate AES-GCM 256-bit key
const key = await crypto.subtle.generateKey(
  { name: 'AES-GCM', length: 256 },
  true,
  ['encrypt', 'decrypt']
);

// Encrypt token data
const iv = crypto.getRandomValues(new Uint8Array(12));
const encrypted = await crypto.subtle.encrypt(
  { name: 'AES-GCM', iv },
  key,
  encodedData
);
```

### 2. CSRF Protection

State parameter prevents CSRF attacks:

```javascript
// Generate cryptographically secure state
const array = new Uint8Array(32);
crypto.getRandomValues(array);
const state = btoa(String.fromCharCode(...array));

// Store for verification
sessionStorage.setItem('oauth_state', state);

// Verify on callback
if (callbackState !== savedState) {
  throw new Error('Invalid state - possible CSRF attack');
}
```

### 3. Secure Storage

- **IndexedDB** for encrypted token storage
- **SessionStorage** for temporary OAuth state
- **No LocalStorage** for sensitive data (to prevent XSS attacks)

### 4. Token Refresh

Automatic token refresh 5 minutes before expiration:

```javascript
const timeUntilExpiration = await tokenStorage.getTimeUntilExpiration();
const refreshMargin = 5 * 60 * 1000; // 5 minutes

if (timeUntilExpiration <= refreshMargin) {
  await authManager.refreshToken();
}
```

## API Endpoints

### Required Backend Endpoints

#### 1. Get OAuth Configuration

```
GET /api/auth/config

Response:
{
  "google": {
    "clientId": "your-google-client-id"
  },
  "github": {
    "clientId": "your-github-client-id"
  },
  "microsoft": {
    "clientId": "your-microsoft-client-id"
  }
}
```

#### 2. Exchange Authorization Code

```
POST /api/auth/token

Request:
{
  "code": "authorization_code",
  "provider": "google" | "github" | "microsoft",
  "redirectUri": "https://your-app.com/auth-callback.html"
}

Response:
{
  "access_token": "...",
  "refresh_token": "...",
  "expires_in": 3600,
  "user": {
    "id": "user-id",
    "email": "user@example.com",
    "name": "User Name",
    "avatar": "https://...",
    "tier": "free" | "premium"
  }
}
```

#### 3. Refresh Access Token

```
POST /api/auth/refresh

Request:
{
  "refreshToken": "refresh_token"
}

Response:
{
  "access_token": "...",
  "refresh_token": "...",
  "expires_in": 3600
}
```

#### 4. Logout

```
POST /api/auth/logout

Headers:
Authorization: Bearer access_token

Response:
{
  "success": true
}
```

## Multi-Tab Synchronization

The session manager synchronizes authentication state across browser tabs:

### Logout Synchronization

```javascript
// Tab 1: User logs out
localStorage.setItem('logout_event', Date.now().toString());
localStorage.removeItem('logout_event'); // Trigger storage event

// Tab 2: Receives event
window.addEventListener('storage', (event) => {
  if (event.key === 'logout_event') {
    // Clear local session
    await tokenStorage.clearTokens();
    window.location.href = '/';
  }
});
```

### Token Refresh Synchronization

```javascript
// Tab 1: Refreshes token
localStorage.setItem('session_update', JSON.stringify({
  type: 'refresh',
  timestamp: Date.now()
}));
localStorage.removeItem('session_update');

// Tab 2: Receives event
window.addEventListener('storage', (event) => {
  if (event.key === 'session_update') {
    // Reload tokens from IndexedDB
    await setupAutoRefresh();
  }
});
```

## Feature Gating

### Feature Tiers

1. **Free Tier** (Guest + Authenticated)
   - Basic playback
   - Spectrum visualizer
   - Basic EQ controls
   - Signal generator
   - Local presets

2. **Authenticated Tier**
   - All free features
   - Cloud preset sync
   - Usage statistics
   - Profile customization

3. **Premium Tier**
   - All authenticated features
   - AI audio enhancement
   - Advanced effects
   - Noise reduction
   - Smart recommendations
   - Priority support
   - Early access

### Usage

```javascript
const featureGate = new FeatureGate(currentUser);

// Check access
if (featureGate.canAccess('cloud_sync')) {
  // Enable cloud sync
}

// Handle feature click
await featureGate.handleFeatureAccess('ai_features', async () => {
  // Feature callback
});

// Apply to UI element
featureGate.applyFeatureGate(element, 'advanced_effects');
```

## Error Handling

### User-Friendly Error Messages

```javascript
const ERROR_MESSAGES = {
  'invalid_token': 'Your session has expired. Please sign in again.',
  'network_error': 'Connection error. Please check your internet.',
  'oauth_error': 'Authentication failed. Please try again.',
  'rate_limit': 'Too many requests. Please wait and try again.',
  'popup_blocked': 'Popup blocked. Please allow popups for authentication.'
};
```

### Error Recovery

1. **Token Expired**
   - Attempt automatic refresh
   - If refresh fails, redirect to login

2. **Network Error**
   - Show retry button
   - Implement exponential backoff

3. **OAuth Error**
   - Display user-friendly message
   - Provide "Try Again" option

4. **Popup Blocked**
   - Detect popup blocker
   - Show instructions to enable

## Testing Checklist

### Authentication Flow

- [ ] Google sign-in works
- [ ] GitHub sign-in works
- [ ] Microsoft sign-in works
- [ ] Guest mode works
- [ ] OAuth callback handles errors
- [ ] State verification prevents CSRF
- [ ] Tokens are stored encrypted
- [ ] Session initializes correctly

### Token Management

- [ ] Access token retrieval works
- [ ] Refresh token works
- [ ] Tokens expire correctly
- [ ] Auto-refresh triggers before expiration
- [ ] Expired tokens trigger re-authentication
- [ ] Token encryption/decryption works

### Multi-Tab Sync

- [ ] Logout in one tab logs out all tabs
- [ ] Token refresh syncs to other tabs
- [ ] Opening new tab loads existing session
- [ ] Closing all tabs doesn't affect session

### Feature Gating

- [ ] Free features accessible to all
- [ ] Authenticated features require sign-in
- [ ] Premium features show upgrade modal
- [ ] Feature gates work correctly
- [ ] Upgrade flow works

### Error Handling

- [ ] Invalid tokens handled gracefully
- [ ] Network errors show retry option
- [ ] OAuth errors display user-friendly messages
- [ ] Popup blocker detected and handled
- [ ] Session expiration handled properly

## Best Practices

### Security

1. **Never store tokens in localStorage** - Use IndexedDB with encryption
2. **Always validate state parameter** - Prevents CSRF attacks
3. **Use HTTPS in production** - Required for secure cookies and OAuth
4. **Implement token rotation** - Refresh tokens regularly
5. **Clear tokens on logout** - Ensure complete cleanup

### Performance

1. **Lazy load auth modules** - Only load when needed
2. **Cache OAuth config** - Reduce backend calls
3. **Debounce activity tracking** - Avoid excessive updates
4. **Use passive event listeners** - Improve scroll performance

### User Experience

1. **Show loading states** - Keep users informed
2. **Handle popup blockers** - Provide clear instructions
3. **Implement auto-refresh** - Seamless token renewal
4. **Support guest mode** - Lower barrier to entry
5. **Graceful degradation** - Work without authentication

## Browser Compatibility

### Required APIs

- **IndexedDB** - Token storage
- **Web Crypto API** - Encryption
- **SessionStorage** - OAuth state
- **LocalStorage** - Multi-tab sync
- **PostMessage** - Popup communication
- **Popup Windows** - OAuth flow

### Supported Browsers

- Chrome 60+
- Firefox 55+
- Safari 11+
- Edge 79+

### Fallbacks

If Web Crypto API unavailable:
- Store tokens unencrypted with warning
- Recommend browser upgrade

If popups blocked:
- Provide instructions to enable
- Consider redirect-based flow

## Troubleshooting

### Common Issues

**Issue**: Popup blocked
**Solution**:
```javascript
const popup = window.open(authUrl, 'OAuth', 'width=500,height=600');
if (!popup) {
  alert('Please allow popups for authentication');
}
```

**Issue**: State mismatch
**Solution**: Clear sessionStorage and retry

**Issue**: Tokens not persisting
**Solution**: Check IndexedDB is enabled

**Issue**: Multi-tab logout not working
**Solution**: Verify localStorage events firing

## Future Enhancements

1. **Biometric Authentication**
   - WebAuthn support
   - Touch ID / Face ID

2. **Social Login**
   - Twitter OAuth
   - Apple Sign In

3. **Advanced Security**
   - Rate limiting
   - Device fingerprinting
   - Suspicious activity detection

4. **Session Management**
   - Active sessions list
   - Remote logout
   - Session history

5. **Analytics**
   - Login success rate
   - Authentication method preferences
   - Session duration tracking
