# API Documentation

Complete API reference for the Rusty Audio Auth Service.

## Base URLs

- **Development:** `http://localhost:8787` (local)
- **Staging:** `https://rusty-audio-auth-dev.workers.dev`
- **Production:** `https://api.rusty-audio.com`

## Authentication

Most endpoints require a JWT access token in the Authorization header:

```
Authorization: Bearer <access_token>
```

## Error Responses

All errors follow this format:

```json
{
  "error": "Error Type",
  "message": "Detailed error message",
  "statusCode": 400
}
```

Common status codes:
- `400` - Bad Request (invalid input)
- `401` - Unauthorized (missing/invalid token)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found
- `429` - Too Many Requests (rate limit exceeded)
- `500` - Internal Server Error

## Endpoints

### Health Check

Check service health.

**Endpoint:** `GET /health`

**Authentication:** None

**Response:**
```json
{
  "status": "healthy",
  "service": "rusty-audio-auth",
  "timestamp": "2025-01-16T12:00:00.000Z"
}
```

**Example:**
```bash
curl https://api.rusty-audio.com/health
```

---

### Initiate Authentication

Start OAuth 2.0 authentication flow with PKCE.

**Endpoint:** `POST /api/auth/initiate`

**Authentication:** None

**Rate Limit:** 10 requests/minute

**Request Body:**
```json
{
  "provider": "google" | "github" | "microsoft"
}
```

**Response:**
```json
{
  "authUrl": "https://accounts.google.com/o/oauth2/v2/auth?client_id=...",
  "state": "random_state_string",
  "codeVerifier": "pkce_code_verifier"
}
```

**Fields:**
- `authUrl` - URL to redirect user for OAuth consent
- `state` - CSRF protection token (verify in callback)
- `codeVerifier` - PKCE code verifier (store securely, use in callback)

**Example:**
```bash
curl -X POST https://api.rusty-audio.com/api/auth/initiate \
  -H "Content-Type: application/json" \
  -d '{"provider":"google"}'
```

**Flow:**
1. Call this endpoint with provider
2. Store `codeVerifier` securely on client
3. Open `authUrl` in browser or webview
4. User completes OAuth consent
5. Provider redirects to your callback URL with `code` and `state`
6. Call `/api/auth/callback` with code, state, and codeVerifier

---

### Complete Authentication

Exchange authorization code for access token.

**Endpoint:** `POST /api/auth/callback`

**Authentication:** None

**Rate Limit:** 5 requests/minute

**Request Body:**
```json
{
  "code": "authorization_code_from_provider",
  "state": "state_from_initiate",
  "codeVerifier": "code_verifier_from_initiate",
  "provider": "google" | "github" | "microsoft"
}
```

**Response:**
```json
{
  "accessToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 3600,
  "user": {
    "id": "google_1234567890",
    "email": "user@example.com",
    "name": "John Doe",
    "avatar": "https://lh3.googleusercontent.com/...",
    "provider": "google"
  }
}
```

**Fields:**
- `accessToken` - JWT access token (1 hour expiration)
- `refreshToken` - JWT refresh token (30 days expiration)
- `expiresIn` - Access token lifetime in seconds
- `user` - User profile information

**Example:**
```bash
curl -X POST https://api.rusty-audio.com/api/auth/callback \
  -H "Content-Type: application/json" \
  -d '{
    "code": "4/0AWtgzh7...",
    "state": "abc123...",
    "codeVerifier": "def456...",
    "provider": "google"
  }'
```

**Error Responses:**

Missing fields:
```json
{
  "error": "Bad Request",
  "message": "Missing required fields: code, state, codeVerifier, provider",
  "statusCode": 400
}
```

Invalid authorization code:
```json
{
  "error": "OAuth Error",
  "message": "Failed to exchange authorization code for tokens",
  "statusCode": 400
}
```

---

### Refresh Access Token

Get new access token using refresh token.

**Endpoint:** `POST /api/auth/refresh`

**Authentication:** None (uses refresh token in body)

**Rate Limit:** 20 requests/minute

**Request Body:**
```json
{
  "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response:**
```json
{
  "accessToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 3600
}
```

**Example:**
```bash
curl -X POST https://api.rusty-audio.com/api/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refreshToken":"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."}'
```

**Error Responses:**

Invalid refresh token:
```json
{
  "error": "Unauthorized",
  "message": "Invalid or expired refresh token",
  "statusCode": 401
}
```

---

### Logout

Invalidate user session and tokens.

**Endpoint:** `POST /api/auth/logout`

**Authentication:** None (uses access token in body)

**Rate Limit:** 10 requests/minute

**Request Body:**
```json
{
  "accessToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response:**
```json
{
  "success": true
}
```

**Example:**
```bash
curl -X POST https://api.rusty-audio.com/api/auth/logout \
  -H "Content-Type: application/json" \
  -d '{"accessToken":"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."}'
```

**Note:** This endpoint is idempotent and always returns success, even for invalid tokens.

---

### Get User Profile

Retrieve authenticated user's profile.

**Endpoint:** `GET /api/auth/user`

**Authentication:** Required (Bearer token)

**Rate Limit:** 30 requests/minute

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response:**
```json
{
  "id": "google_1234567890",
  "email": "user@example.com",
  "name": "John Doe",
  "avatar": "https://lh3.googleusercontent.com/...",
  "provider": "google",
  "tier": "free"
}
```

**Fields:**
- `id` - Unique user identifier (provider-specific)
- `email` - User's email address
- `name` - User's display name
- `avatar` - Profile picture URL (optional)
- `provider` - OAuth provider used (google, github, microsoft)
- `tier` - User subscription tier (free, premium)

**Example:**
```bash
curl https://api.rusty-audio.com/api/auth/user \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

**Error Responses:**

Missing token:
```json
{
  "error": "Unauthorized",
  "message": "Valid authentication token required",
  "statusCode": 401
}
```

Invalid token:
```json
{
  "error": "Unauthorized",
  "message": "Invalid token: jwt expired",
  "statusCode": 401
}
```

---

## Complete Authentication Flow

### 1. Initiate Authentication

```javascript
// Client-side JavaScript
const response = await fetch('https://api.rusty-audio.com/api/auth/initiate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ provider: 'google' })
});

const { authUrl, state, codeVerifier } = await response.json();

// Store codeVerifier securely (e.g., sessionStorage)
sessionStorage.setItem('codeVerifier', codeVerifier);
sessionStorage.setItem('oauthState', state);

// Redirect user to authUrl
window.location.href = authUrl;
```

### 2. Handle Callback

```javascript
// On your callback page (e.g., /auth-callback)
const urlParams = new URLSearchParams(window.location.search);
const code = urlParams.get('code');
const state = urlParams.get('state');

// Verify state matches
const storedState = sessionStorage.getItem('oauthState');
if (state !== storedState) {
  throw new Error('State mismatch - possible CSRF attack');
}

// Retrieve code verifier
const codeVerifier = sessionStorage.getItem('codeVerifier');

// Exchange code for tokens
const response = await fetch('https://api.rusty-audio.com/api/auth/callback', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    code,
    state,
    codeVerifier,
    provider: 'google'
  })
});

const { accessToken, refreshToken, user } = await response.json();

// Store tokens securely
localStorage.setItem('accessToken', accessToken);
localStorage.setItem('refreshToken', refreshToken);
localStorage.setItem('user', JSON.stringify(user));

// Clean up
sessionStorage.removeItem('codeVerifier');
sessionStorage.removeItem('oauthState');

// Redirect to app
window.location.href = '/';
```

### 3. Make Authenticated Requests

```javascript
// Get user profile
const response = await fetch('https://api.rusty-audio.com/api/auth/user', {
  headers: {
    'Authorization': `Bearer ${localStorage.getItem('accessToken')}`
  }
});

const user = await response.json();
```

### 4. Handle Token Expiration

```javascript
async function makeAuthenticatedRequest(url, options = {}) {
  const accessToken = localStorage.getItem('accessToken');

  let response = await fetch(url, {
    ...options,
    headers: {
      ...options.headers,
      'Authorization': `Bearer ${accessToken}`
    }
  });

  // If token expired, refresh and retry
  if (response.status === 401) {
    const refreshToken = localStorage.getItem('refreshToken');

    const refreshResponse = await fetch('https://api.rusty-audio.com/api/auth/refresh', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refreshToken })
    });

    if (refreshResponse.ok) {
      const { accessToken: newAccessToken } = await refreshResponse.json();
      localStorage.setItem('accessToken', newAccessToken);

      // Retry original request
      response = await fetch(url, {
        ...options,
        headers: {
          ...options.headers,
          'Authorization': `Bearer ${newAccessToken}`
        }
      });
    } else {
      // Refresh failed, logout
      localStorage.clear();
      window.location.href = '/login';
    }
  }

  return response;
}
```

### 5. Logout

```javascript
async function logout() {
  const accessToken = localStorage.getItem('accessToken');

  await fetch('https://api.rusty-audio.com/api/auth/logout', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ accessToken })
  });

  // Clear local storage
  localStorage.clear();

  // Redirect to login
  window.location.href = '/login';
}
```

## Rate Limiting

Rate limits are enforced per client IP address. When rate limit is exceeded, you'll receive a 429 response:

```json
{
  "error": "Too Many Requests",
  "message": "Rate limit exceeded",
  "statusCode": 429,
  "retryAfter": 45
}
```

The `Retry-After` header indicates seconds to wait before retrying.

### Rate Limit Guidelines

| Endpoint | Limit | Window |
|----------|-------|--------|
| `/api/auth/initiate` | 10 | 1 minute |
| `/api/auth/callback` | 5 | 1 minute |
| `/api/auth/refresh` | 20 | 1 minute |
| `/api/auth/logout` | 10 | 1 minute |
| `/api/auth/user` | 30 | 1 minute |

**Best Practices:**
- Implement exponential backoff for retries
- Cache user profile instead of frequent `/user` calls
- Refresh tokens only when needed (on 401 errors)
- Use access tokens until they expire (don't refresh proactively)

## CORS

Cross-Origin Resource Sharing is configured for:
- `http://localhost:8080` (development)
- `http://localhost:3000` (development)
- `https://rusty-audio.pages.dev` (production)
- `https://rusty-audio.com` (production)

Requests from other origins will be rejected.

## Security Best Practices

1. **Never expose refresh tokens:** Store in httpOnly cookies or secure storage
2. **Validate state parameter:** Always verify state matches to prevent CSRF
3. **Store code verifier securely:** Use sessionStorage, never localStorage
4. **Implement token rotation:** Refresh access tokens before expiration
5. **Handle errors gracefully:** Don't expose sensitive error details to users
6. **Use HTTPS only:** Never send tokens over HTTP
7. **Implement logout:** Always provide logout functionality
8. **Monitor for suspicious activity:** Log authentication events

## Support

For API issues or questions:
- GitHub Issues: https://github.com/rusty-audio/rusty-audio/issues
- Documentation: https://docs.rusty-audio.com
- Email: api-support@rusty-audio.com
