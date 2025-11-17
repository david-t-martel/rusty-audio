# Rusty Audio Auth Service

OAuth 2.0 authentication service built on Cloudflare Workers for the Rusty Audio WASM application.

## Overview

This service provides secure authentication using OAuth 2.0 with PKCE (Proof Key for Code Exchange) for the Rusty Audio desktop application. It handles the OAuth flow, token management, and session storage entirely on the edge using Cloudflare Workers and KV storage.

### Features

- **Multiple OAuth Providers**: Google, GitHub, Microsoft
- **PKCE Flow**: Enhanced security with Proof Key for Code Exchange
- **JWT Tokens**: Secure, stateless authentication tokens
- **Session Management**: Server-side session storage in Cloudflare KV
- **Rate Limiting**: Per-endpoint rate limiting to prevent abuse
- **CORS Support**: Cross-origin requests from WASM app
- **Edge Computing**: Low-latency response times worldwide
- **Automatic Scaling**: Handles traffic spikes automatically

## Architecture

```
┌─────────────┐         ┌──────────────────┐         ┌─────────────┐
│  WASM App   │────────▶│  Auth Worker     │────────▶│   OAuth     │
│  (Client)   │◀────────│  (Edge Service)  │◀────────│  Provider   │
└─────────────┘         └──────────────────┘         └─────────────┘
                               │
                               ▼
                        ┌──────────────┐
                        │ Cloudflare   │
                        │   KV Store   │
                        │  (Sessions)  │
                        └──────────────┘
```

## API Endpoints

### POST `/api/auth/initiate`

Initiate OAuth authentication flow.

**Request:**
```json
{
  "provider": "google" | "github" | "microsoft"
}
```

**Response:**
```json
{
  "authUrl": "https://accounts.google.com/o/oauth2/v2/auth?...",
  "state": "random_state_string",
  "codeVerifier": "pkce_code_verifier"
}
```

**Rate Limit:** 10 requests/minute

### POST `/api/auth/callback`

Complete OAuth flow and exchange authorization code for tokens.

**Request:**
```json
{
  "code": "authorization_code",
  "state": "state_from_initiate",
  "codeVerifier": "code_verifier_from_initiate",
  "provider": "google" | "github" | "microsoft"
}
```

**Response:**
```json
{
  "accessToken": "jwt_access_token",
  "refreshToken": "jwt_refresh_token",
  "expiresIn": 3600,
  "user": {
    "id": "provider_user_id",
    "email": "user@example.com",
    "name": "User Name",
    "avatar": "https://avatar.url",
    "provider": "google"
  }
}
```

**Rate Limit:** 5 requests/minute

### POST `/api/auth/refresh`

Refresh access token using refresh token.

**Request:**
```json
{
  "refreshToken": "jwt_refresh_token"
}
```

**Response:**
```json
{
  "accessToken": "new_jwt_access_token",
  "expiresIn": 3600
}
```

**Rate Limit:** 20 requests/minute

### POST `/api/auth/logout`

Logout and invalidate session.

**Request:**
```json
{
  "accessToken": "jwt_access_token"
}
```

**Response:**
```json
{
  "success": true
}
```

**Rate Limit:** 10 requests/minute

### GET `/api/auth/user`

Get authenticated user profile.

**Headers:**
```
Authorization: Bearer <jwt_access_token>
```

**Response:**
```json
{
  "id": "provider_user_id",
  "email": "user@example.com",
  "name": "User Name",
  "avatar": "https://avatar.url",
  "provider": "google",
  "tier": "free" | "premium"
}
```

**Rate Limit:** 30 requests/minute

### GET `/health`

Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "service": "rusty-audio-auth",
  "timestamp": "2025-01-16T12:00:00.000Z"
}
```

## Setup Instructions

### Prerequisites

- [Cloudflare account](https://dash.cloudflare.com/sign-up)
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/) installed
- Node.js 20+ installed
- OAuth application credentials from providers

### 1. Install Dependencies

```bash
cd workers/auth-service
npm install
```

### 2. Configure OAuth Providers

#### Google OAuth

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create new project or select existing
3. Enable Google+ API
4. Create OAuth 2.0 credentials
5. Add authorized redirect URI: `https://rusty-audio.pages.dev/auth-callback`
6. Copy Client ID and Client Secret

#### GitHub OAuth

1. Go to [GitHub Developer Settings](https://github.com/settings/developers)
2. Click "New OAuth App"
3. Set Authorization callback URL: `https://rusty-audio.pages.dev/auth-callback`
4. Copy Client ID and Client Secret

#### Microsoft OAuth

1. Go to [Azure Portal](https://portal.azure.com/)
2. Register new application
3. Add redirect URI: `https://rusty-audio.pages.dev/auth-callback`
4. Create client secret
5. Copy Application (client) ID and Client Secret

### 3. Create KV Namespaces

```bash
# Development
./scripts/setup-kv.sh dev

# Production
./scripts/setup-kv.sh production
```

Update `wrangler.toml` with the generated KV namespace IDs.

### 4. Set Secrets

```bash
# Development
./scripts/setup-secrets.sh dev

# Production
./scripts/setup-secrets.sh production
```

Or manually:

```bash
# Generate JWT secret
openssl rand -base64 32

# Set secrets
echo "<secret_value>" | wrangler secret put GOOGLE_CLIENT_ID --env production
echo "<secret_value>" | wrangler secret put GOOGLE_CLIENT_SECRET --env production
echo "<secret_value>" | wrangler secret put GITHUB_CLIENT_ID --env production
echo "<secret_value>" | wrangler secret put GITHUB_CLIENT_SECRET --env production
echo "<secret_value>" | wrangler secret put MICROSOFT_CLIENT_ID --env production
echo "<secret_value>" | wrangler secret put MICROSOFT_CLIENT_SECRET --env production
echo "<secret_value>" | wrangler secret put JWT_SECRET --env production
```

### 5. Deploy

```bash
# Development
npm run deploy
# or
./scripts/deploy.sh dev

# Production
npm run deploy:prod
# or
./scripts/deploy.sh production
```

## Development

### Local Development

```bash
# Start local development server
npm run dev

# The worker will be available at http://localhost:8787
```

### Testing

```bash
# Run tests
npm test

# Type checking
npm run type-check

# Linting
npm run lint
```

### Debugging

```bash
# Tail production logs
npm run tail

# Or with wrangler
wrangler tail --env production
```

## Security

### PKCE Flow

This service uses PKCE (RFC 7636) to protect against authorization code interception attacks:

1. Client generates `code_verifier` (random string)
2. Client creates `code_challenge` = BASE64URL(SHA256(code_verifier))
3. Client sends `code_challenge` in authorization request
4. Provider returns authorization code
5. Client sends authorization code + `code_verifier` to token endpoint
6. Provider verifies: SHA256(code_verifier) == code_challenge

### JWT Tokens

- **Access Token**: Short-lived (1 hour), used for API authentication
- **Refresh Token**: Long-lived (30 days), used to obtain new access tokens
- **Signing**: HS256 algorithm with secret key
- **Claims**: User ID, email, provider, tier, expiration

### Rate Limiting

Rate limits are enforced per client IP address:

| Endpoint | Limit |
|----------|-------|
| `/api/auth/initiate` | 10/minute |
| `/api/auth/callback` | 5/minute |
| `/api/auth/refresh` | 20/minute |
| `/api/auth/logout` | 10/minute |
| `/api/auth/user` | 30/minute |

### CORS

Only requests from allowed origins are accepted:
- `http://localhost:8080` (development)
- `https://rusty-audio.pages.dev` (production)
- `https://rusty-audio.com` (custom domain)

## Storage Schema

### KV Namespaces

#### SESSIONS

Stores user sessions with OAuth tokens.

**Key Format:** `session:{uuid}`

**Value:**
```json
{
  "userId": "user_id",
  "provider": "google",
  "accessToken": "oauth_access_token",
  "refreshToken": "oauth_refresh_token",
  "expiresAt": 1234567890000,
  "createdAt": 1234567890000
}
```

**TTL:** 30 days

#### USERS

Stores user profiles.

**Key Format:** `user:{user_id}`

**Value:**
```json
{
  "id": "user_id",
  "email": "user@example.com",
  "name": "User Name",
  "avatar": "https://avatar.url",
  "provider": "google",
  "tier": "free",
  "createdAt": 1234567890000,
  "lastLoginAt": 1234567890000
}
```

**TTL:** 1 year (refreshed on login)

#### RATE_LIMIT

Tracks rate limit counters.

**Key Format:** `ratelimit:{path}:{client_ip}`

**Value:**
```json
{
  "count": 5,
  "resetAt": 1234567890000
}
```

**TTL:** Rate limit window duration

## Monitoring

### Metrics

Monitor worker performance in Cloudflare Dashboard:
- Request count
- Error rate
- Latency (p50, p95, p99)
- KV read/write operations

### Logs

```bash
# Tail live logs
wrangler tail --env production

# Filter by status
wrangler tail --env production --status error
```

### Alerts

Set up alerts in Cloudflare Dashboard for:
- Error rate > 5%
- Latency p95 > 1000ms
- KV operation failures

## Troubleshooting

### Common Issues

**1. "Invalid token" errors**
- Check JWT_SECRET is set correctly
- Verify token hasn't expired
- Ensure clock sync between client/server

**2. Rate limit errors**
- Implement exponential backoff in client
- Check if multiple users behind same IP (NAT)

**3. OAuth callback failures**
- Verify redirect URI matches exactly in provider settings
- Check state parameter matches
- Ensure code_verifier is stored securely on client

**4. KV namespace errors**
- Verify KV namespace IDs in wrangler.toml
- Check KV bindings are correct

## Performance

- **Cold start:** < 50ms
- **Average latency:** < 100ms (p95)
- **KV read latency:** < 50ms
- **OAuth provider latency:** 200-500ms (external)

## Cost Estimation

Cloudflare Workers Free Tier:
- 100,000 requests/day
- 10ms CPU time per request
- Unlimited KV reads
- 1,000 KV writes/day

For Rusty Audio with 1,000 daily active users:
- ~10,000 requests/day (10 per user)
- Well within free tier limits

## License

MIT License - See LICENSE file for details

## Support

For issues or questions:
- GitHub Issues: https://github.com/rusty-audio/rusty-audio/issues
- Documentation: https://docs.rusty-audio.com
