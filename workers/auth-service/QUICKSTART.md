# Quick Start Guide

Get the Rusty Audio Auth Service running in 5 minutes.

## Prerequisites

- Node.js 20+
- Cloudflare account
- OAuth credentials (Google/GitHub/Microsoft)

## Setup (One-Time)

```bash
# 1. Install dependencies
cd workers/auth-service
npm install

# 2. Install wrangler globally
npm install -g wrangler

# 3. Login to Cloudflare
wrangler login

# 4. Create KV namespaces
./scripts/setup-kv.sh production

# 5. Update wrangler.toml with KV IDs (from step 4 output)

# 6. Set secrets
./scripts/setup-secrets.sh production

# 7. Deploy
./scripts/deploy.sh production
```

## OAuth Provider Setup

### Google (5 minutes)

1. Go to https://console.cloud.google.com/
2. Create project → "Rusty Audio"
3. APIs & Services → Credentials → Create OAuth client ID
4. Web application
5. Authorized redirect URI: `https://rusty-audio.pages.dev/auth-callback`
6. Copy Client ID and Secret

### GitHub (2 minutes)

1. Go to https://github.com/settings/developers
2. New OAuth App
3. Callback URL: `https://rusty-audio.pages.dev/auth-callback`
4. Copy Client ID and Secret

### Microsoft (5 minutes)

1. Go to https://portal.azure.com/
2. Azure Active Directory → App registrations → New registration
3. Redirect URI: `https://rusty-audio.pages.dev/auth-callback`
4. Certificates & secrets → New client secret
5. Copy Application ID and Secret

## Test Deployment

```bash
# Health check
curl https://api.rusty-audio.com/health

# Full endpoint tests
./scripts/test-endpoints.sh https://api.rusty-audio.com

# Watch logs
wrangler tail --env production
```

## Client Integration

```typescript
// 1. Initiate auth
const response = await fetch('https://api.rusty-audio.com/api/auth/initiate', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ provider: 'google' })
});

const { authUrl, state, codeVerifier } = await response.json();

// Store securely
sessionStorage.setItem('codeVerifier', codeVerifier);
sessionStorage.setItem('state', state);

// Redirect user
window.location.href = authUrl;

// 2. Handle callback (on redirect back)
const urlParams = new URLSearchParams(window.location.search);
const code = urlParams.get('code');
const state = urlParams.get('state');

// Verify state
if (state !== sessionStorage.getItem('state')) {
  throw new Error('State mismatch');
}

// Exchange code for tokens
const callbackResponse = await fetch('https://api.rusty-audio.com/api/auth/callback', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    code,
    state,
    codeVerifier: sessionStorage.getItem('codeVerifier'),
    provider: 'google'
  })
});

const { accessToken, refreshToken, user } = await callbackResponse.json();

// Store tokens
localStorage.setItem('accessToken', accessToken);
localStorage.setItem('refreshToken', refreshToken);

// 3. Make authenticated requests
const userResponse = await fetch('https://api.rusty-audio.com/api/auth/user', {
  headers: {
    'Authorization': `Bearer ${localStorage.getItem('accessToken')}`
  }
});

const userProfile = await userResponse.json();
```

## Common Commands

```bash
# Deploy to production
npm run deploy:prod

# Deploy to development
npm run deploy

# Local development
npm run dev

# Watch logs
npm run tail

# Run tests
npm test

# Type check
npm run type-check

# Lint
npm run lint
```

## Troubleshooting

**"Namespace not found" error:**
```bash
wrangler kv:namespace list
# Update wrangler.toml with correct IDs
```

**OAuth callback fails:**
- Check redirect URI matches exactly
- Verify CORS origins in src/middleware/cors.ts

**Rate limit errors:**
- Adjust limits in src/middleware/ratelimit.ts
- Implement exponential backoff in client

## Documentation

- `README.md` - Full documentation
- `API.md` - API reference
- `SECURITY.md` - Security guidelines
- `SETUP.md` - Detailed setup
- `DEPLOYMENT.md` - Deployment guide

## Support

- Issues: https://github.com/rusty-audio/rusty-audio/issues
- Docs: https://docs.rusty-audio.com

## Next Steps

1. Integrate with Rusty Audio WASM app
2. Add user profile management
3. Implement premium features
4. Set up monitoring alerts
5. Configure custom domain

---

**Quick Reference:**

| Task | Command |
|------|---------|
| Deploy prod | `./scripts/deploy.sh production` |
| Create KV | `./scripts/setup-kv.sh production` |
| Set secrets | `./scripts/setup-secrets.sh production` |
| Test endpoints | `./scripts/test-endpoints.sh <url>` |
| Watch logs | `wrangler tail --env production` |
| Health check | `curl https://api.rusty-audio.com/health` |
