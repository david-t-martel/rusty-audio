# Security Best Practices for OAuth Authentication

## Overview

This document outlines security best practices for the Rusty Audio OAuth 2.0 implementation, covering token management, CSRF protection, XSS prevention, and secure communication.

## Token Security

### 1. Storage Security

**❌ NEVER DO:**
```javascript
// DON'T store tokens in localStorage
localStorage.setItem('access_token', token); // Vulnerable to XSS

// DON'T store tokens in sessionStorage
sessionStorage.setItem('access_token', token); // Vulnerable to XSS

// DON'T store tokens in cookies without HttpOnly
document.cookie = `token=${token}`; // Accessible to JavaScript
```

**✅ ALWAYS DO:**
```javascript
// DO use IndexedDB with encryption
const tokenStorage = new SecureTokenStorage();
await tokenStorage.storeTokens(accessToken, refreshToken, expiresIn);

// DO encrypt tokens using Web Crypto API
const encrypted = await crypto.subtle.encrypt(
  { name: 'AES-GCM', iv },
  key,
  tokenData
);
```

### 2. Token Encryption

**Implementation:**
```javascript
class SecureTokenStorage {
  async encrypt(data) {
    // 1. Generate encryption key (or retrieve existing)
    const key = await this.getEncryptionKey();

    // 2. Generate random IV (initialization vector)
    const iv = crypto.getRandomValues(new Uint8Array(12));

    // 3. Encrypt data
    const encoder = new TextEncoder();
    const encodedData = encoder.encode(JSON.stringify(data));
    const encryptedData = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      key,
      encodedData
    );

    // 4. Combine IV + encrypted data
    const combined = new Uint8Array(iv.length + encryptedData.byteLength);
    combined.set(iv, 0);
    combined.set(new Uint8Array(encryptedData), iv.length);

    // 5. Base64 encode for storage
    return btoa(String.fromCharCode(...combined));
  }

  async decrypt(encryptedBase64) {
    const key = await this.getEncryptionKey();
    const combined = Uint8Array.from(atob(encryptedBase64), c => c.charCodeAt(0));

    // Extract IV and encrypted data
    const iv = combined.slice(0, 12);
    const encryptedData = combined.slice(12);

    // Decrypt
    const decryptedData = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv },
      key,
      encryptedData
    );

    const decoder = new TextDecoder();
    return JSON.parse(decoder.decode(decryptedData));
  }
}
```

### 3. Token Lifecycle

**Best Practices:**

1. **Short-lived access tokens**: 15-60 minutes
2. **Long-lived refresh tokens**: 30 days
3. **Auto-refresh before expiration**: 5 minutes margin
4. **Rotate refresh tokens**: On each refresh
5. **Revoke on logout**: Clear all tokens

```javascript
// Token refresh strategy
const REFRESH_MARGIN = 5 * 60 * 1000; // 5 minutes

async function ensureValidToken() {
  const timeUntilExpiration = await tokenStorage.getTimeUntilExpiration();

  if (timeUntilExpiration <= REFRESH_MARGIN) {
    // Token expired or expiring soon
    try {
      await authManager.refreshToken();
    } catch (error) {
      // Refresh failed - re-authenticate
      await authManager.logout();
      throw new Error('Session expired');
    }
  }

  return await tokenStorage.getAccessToken();
}
```

## CSRF Protection

### 1. State Parameter

**Purpose**: Prevent Cross-Site Request Forgery attacks

**Implementation:**
```javascript
// Generate cryptographically secure state
function generateState() {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return btoa(String.fromCharCode(...array))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
}

// Store state before OAuth redirect
const state = generateState();
sessionStorage.setItem('oauth_state', state);

// Build authorization URL with state
const authUrl = `${endpoint}?state=${state}&...`;

// Verify state on callback
function verifyState(callbackState) {
  const savedState = sessionStorage.getItem('oauth_state');

  if (callbackState !== savedState) {
    throw new Error('Invalid state - possible CSRF attack');
  }

  // Clear state after verification
  sessionStorage.removeItem('oauth_state');
}
```

### 2. Origin Validation

**Validate postMessage origins:**
```javascript
window.addEventListener('message', (event) => {
  // CRITICAL: Verify origin
  if (event.origin !== window.location.origin) {
    console.warn('Rejected message from untrusted origin:', event.origin);
    return;
  }

  // Process message
  if (event.data.code) {
    handleAuthCallback(event.data.code, event.data.state);
  }
});
```

## XSS Prevention

### 1. Content Security Policy

**Recommended CSP Headers:**
```
Content-Security-Policy:
  default-src 'self';
  script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com;
  style-src 'self' 'unsafe-inline' https://fonts.googleapis.com;
  img-src 'self' data: https:;
  connect-src 'self' https://accounts.google.com https://github.com https://login.microsoftonline.com;
  frame-src 'none';
  object-src 'none';
  base-uri 'self';
  form-action 'self';
  upgrade-insecure-requests;
```

### 2. Input Sanitization

**Never trust user input:**
```javascript
// ❌ DANGEROUS - XSS vulnerability
element.innerHTML = userInput;

// ✅ SAFE - Use textContent
element.textContent = userInput;

// ✅ SAFE - Sanitize HTML
import DOMPurify from 'dompurify';
element.innerHTML = DOMPurify.sanitize(userInput);

// ✅ SAFE - Use framework escaping (React, Vue, etc.)
<div>{userInput}</div> // Automatically escaped
```

### 3. Avoid eval() and Function()

**Never execute user-provided code:**
```javascript
// ❌ DANGEROUS
eval(userInput);
new Function(userInput)();

// ✅ SAFE - Use JSON.parse for data
const data = JSON.parse(userInput);
```

## HTTPS Requirements

### 1. Production Environment

**MANDATORY:** All OAuth flows must use HTTPS in production

```javascript
// Enforce HTTPS
if (location.protocol !== 'https:' && location.hostname !== 'localhost') {
  location.replace(`https:${location.href.substring(location.protocol.length)}`);
}
```

### 2. Mixed Content Prevention

**Ensure all resources loaded over HTTPS:**
- API endpoints
- CDN resources
- OAuth provider URLs
- Image sources

```html
<!-- ✅ GOOD - Secure resource -->
<script src="https://cdn.example.com/lib.js"></script>

<!-- ❌ BAD - Insecure resource -->
<script src="http://cdn.example.com/lib.js"></script>

<!-- ✅ GOOD - Protocol-relative URL -->
<script src="//cdn.example.com/lib.js"></script>
```

## API Security

### 1. Authentication Headers

**Use Bearer token authentication:**
```javascript
async function authenticatedFetch(url, options = {}) {
  const token = await getValidAccessToken();

  const response = await fetch(url, {
    ...options,
    headers: {
      ...options.headers,
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    }
  });

  // Handle 401 Unauthorized
  if (response.status === 401) {
    // Token invalid - try refresh
    const newToken = await refreshToken();
    return fetch(url, {
      ...options,
      headers: {
        ...options.headers,
        'Authorization': `Bearer ${newToken}`,
        'Content-Type': 'application/json'
      }
    });
  }

  return response;
}
```

### 2. Rate Limiting

**Implement client-side rate limiting:**
```javascript
class RateLimiter {
  constructor(maxRequests = 100, windowMs = 60000) {
    this.maxRequests = maxRequests;
    this.windowMs = windowMs;
    this.requests = [];
  }

  async limit(fn) {
    const now = Date.now();

    // Remove old requests outside window
    this.requests = this.requests.filter(
      time => now - time < this.windowMs
    );

    // Check rate limit
    if (this.requests.length >= this.maxRequests) {
      throw new Error('Rate limit exceeded');
    }

    // Record request
    this.requests.push(now);

    // Execute function
    return await fn();
  }
}

// Usage
const apiLimiter = new RateLimiter(100, 60000);
await apiLimiter.limit(async () => {
  return await fetch('/api/endpoint');
});
```

### 3. Request Validation

**Validate all API responses:**
```javascript
async function validateResponse(response) {
  // Check HTTP status
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }

  // Validate Content-Type
  const contentType = response.headers.get('Content-Type');
  if (!contentType || !contentType.includes('application/json')) {
    throw new Error('Invalid response type');
  }

  // Parse and validate JSON
  const data = await response.json();

  // Validate expected fields
  if (!data || typeof data !== 'object') {
    throw new Error('Invalid response data');
  }

  return data;
}
```

## Session Security

### 1. Session Timeout

**Implement inactivity timeout:**
```javascript
class SessionTimeout {
  constructor(timeoutMs = 30 * 60 * 1000) {
    this.timeoutMs = timeoutMs;
    this.lastActivity = Date.now();
    this.timeoutId = null;

    this.startMonitoring();
  }

  startMonitoring() {
    // Monitor activity
    const events = ['mousedown', 'keydown', 'scroll', 'touchstart'];
    events.forEach(event => {
      window.addEventListener(event, () => this.updateActivity(), {
        passive: true
      });
    });

    // Check for timeout
    this.timeoutId = setInterval(() => {
      const inactiveTime = Date.now() - this.lastActivity;

      if (inactiveTime >= this.timeoutMs) {
        this.handleTimeout();
      }
    }, 60000); // Check every minute
  }

  updateActivity() {
    this.lastActivity = Date.now();
  }

  handleTimeout() {
    console.log('Session timeout due to inactivity');
    // Logout user
    authManager.logout();
  }
}
```

### 2. Concurrent Session Detection

**Detect and handle concurrent sessions:**
```javascript
class SessionConflictDetector {
  constructor() {
    this.sessionId = this.generateSessionId();
    this.checkInterval = null;
  }

  generateSessionId() {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  async startMonitoring() {
    // Store current session ID
    await this.storeSessionId(this.sessionId);

    // Check for conflicts periodically
    this.checkInterval = setInterval(async () => {
      const currentSessionId = await this.getStoredSessionId();

      if (currentSessionId !== this.sessionId) {
        // Another session detected
        this.handleConflict();
      }
    }, 5000);
  }

  handleConflict() {
    alert('Your account is being used in another location. You will be logged out.');
    authManager.logout();
  }
}
```

## OAuth Provider Configuration

### 1. Redirect URI Validation

**Register exact redirect URIs with OAuth providers:**

```
Production:
  https://rusty-audio.com/auth-callback.html

Development:
  http://localhost:8080/auth-callback.html

DO NOT use wildcards:
  ❌ https://rusty-audio.com/* (insecure)
  ✅ https://rusty-audio.com/auth-callback.html (secure)
```

### 2. Scope Minimization

**Request only necessary scopes:**

```javascript
// ✅ GOOD - Minimal scopes
const googleScopes = 'openid profile email';

// ❌ BAD - Excessive scopes
const googleScopes = 'openid profile email https://www.googleapis.com/auth/drive';
```

### 3. Client Secret Protection

**NEVER expose client secrets in frontend:**

```javascript
// ❌ DANGEROUS - Client secret in frontend
const clientSecret = 'your-secret-here'; // NEVER DO THIS

// ✅ SAFE - Use backend for token exchange
const response = await fetch('/api/auth/token', {
  method: 'POST',
  body: JSON.stringify({ code, provider })
});
```

## Audit Logging

### 1. Security Events

**Log security-relevant events:**

```javascript
class SecurityLogger {
  log(event, details = {}) {
    const logEntry = {
      timestamp: new Date().toISOString(),
      event,
      userId: this.getCurrentUserId(),
      sessionId: this.getSessionId(),
      ip: this.getClientIP(),
      userAgent: navigator.userAgent,
      ...details
    };

    // Send to backend
    this.sendToBackend(logEntry);

    // Local storage for debugging
    console.log('[Security]', logEntry);
  }

  logAuthAttempt(provider, success) {
    this.log('auth_attempt', { provider, success });
  }

  logTokenRefresh(success) {
    this.log('token_refresh', { success });
  }

  logLogout(reason) {
    this.log('logout', { reason });
  }

  logSuspiciousActivity(description) {
    this.log('suspicious_activity', { description });
  }
}
```

### 2. Failed Authentication Tracking

**Track and limit failed attempts:**

```javascript
class FailedAuthTracker {
  constructor(maxAttempts = 5, lockoutDuration = 15 * 60 * 1000) {
    this.maxAttempts = maxAttempts;
    this.lockoutDuration = lockoutDuration;
    this.attempts = [];
  }

  recordFailure() {
    const now = Date.now();

    // Remove old attempts
    this.attempts = this.attempts.filter(
      time => now - time < this.lockoutDuration
    );

    // Add current attempt
    this.attempts.push(now);

    // Check if locked out
    if (this.attempts.length >= this.maxAttempts) {
      throw new Error(
        `Too many failed attempts. Please try again in ${Math.ceil(this.lockoutDuration / 60000)} minutes.`
      );
    }
  }

  reset() {
    this.attempts = [];
  }

  isLockedOut() {
    return this.attempts.length >= this.maxAttempts;
  }

  getTimeUntilUnlock() {
    if (!this.isLockedOut()) return 0;

    const oldestAttempt = Math.min(...this.attempts);
    const unlockTime = oldestAttempt + this.lockoutDuration;
    return Math.max(0, unlockTime - Date.now());
  }
}
```

## Security Checklist

### Pre-Deployment

- [ ] All tokens stored encrypted in IndexedDB
- [ ] CSRF protection via state parameter implemented
- [ ] Origin validation for postMessage
- [ ] Content Security Policy configured
- [ ] All API calls use HTTPS
- [ ] Client secrets never exposed in frontend
- [ ] Input sanitization implemented
- [ ] XSS prevention measures in place
- [ ] Rate limiting implemented
- [ ] Session timeout configured
- [ ] Audit logging enabled

### Production Monitoring

- [ ] Monitor failed authentication attempts
- [ ] Track token refresh success rate
- [ ] Alert on suspicious activity
- [ ] Review security logs regularly
- [ ] Update dependencies regularly
- [ ] Rotate OAuth credentials periodically
- [ ] Test security measures continuously

## Incident Response

### 1. Token Compromise

**If tokens are compromised:**

1. **Immediate Actions:**
   - Revoke all user tokens on backend
   - Force logout all sessions
   - Rotate OAuth credentials
   - Invalidate refresh tokens

2. **Investigation:**
   - Review audit logs
   - Identify compromise vector
   - Assess scope of breach

3. **Remediation:**
   - Patch vulnerability
   - Notify affected users
   - Document incident

### 2. XSS Attack Detected

**If XSS vulnerability found:**

1. **Immediate Actions:**
   - Patch vulnerability
   - Deploy fix urgently
   - Clear affected sessions

2. **User Protection:**
   - Force password reset (if applicable)
   - Notify users
   - Provide security guidance

## References

- [OAuth 2.0 RFC 6749](https://tools.ietf.org/html/rfc6749)
- [OAuth 2.0 Security Best Practices](https://tools.ietf.org/html/draft-ietf-oauth-security-topics)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Web Crypto API](https://www.w3.org/TR/WebCryptoAPI/)
- [Content Security Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)
