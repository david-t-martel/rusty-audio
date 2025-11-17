# Environment Setup Guide

Complete setup guide for deploying the Rusty Audio Auth Service.

## Quick Start

```bash
# 1. Install dependencies
npm install

# 2. Setup OAuth providers (see sections below)

# 3. Create KV namespaces
./scripts/setup-kv.sh dev

# 4. Set secrets
./scripts/setup-secrets.sh dev

# 5. Deploy
npm run dev  # Local development
npm run deploy  # Deploy to dev environment
```

## Detailed Setup

### 1. Prerequisites

#### Install Required Tools

```bash
# Node.js 20+
node --version  # Should be v20.0.0 or higher

# Install wrangler globally
npm install -g wrangler

# Login to Cloudflare
wrangler login
```

#### Cloudflare Account Setup

1. Create account at https://dash.cloudflare.com/sign-up
2. Note your Account ID (Dashboard → Workers & Pages → Overview)
3. Create API token:
   - Go to: https://dash.cloudflare.com/profile/api-tokens
   - Click "Create Token"
   - Use "Edit Cloudflare Workers" template
   - Save the token securely

### 2. OAuth Provider Setup

#### Google OAuth 2.0

1. **Create Project:**
   - Go to https://console.cloud.google.com/
   - Click "Select a project" → "New Project"
   - Name: "Rusty Audio"
   - Click "Create"

2. **Enable Google+ API:**
   - Navigate to "APIs & Services" → "Library"
   - Search for "Google+ API"
   - Click "Enable"

3. **Create OAuth Credentials:**
   - Go to "APIs & Services" → "Credentials"
   - Click "Create Credentials" → "OAuth client ID"
   - Application type: "Web application"
   - Name: "Rusty Audio Auth"

4. **Configure Authorized Redirect URIs:**
   ```
   Development:
   http://localhost:8080/auth-callback

   Production:
   https://rusty-audio.pages.dev/auth-callback
   https://rusty-audio.com/auth-callback
   ```

5. **Copy Credentials:**
   - Client ID: `xxx.apps.googleusercontent.com`
   - Client Secret: `xxx`
   - Save these securely

#### GitHub OAuth App

1. **Create OAuth App:**
   - Go to https://github.com/settings/developers
   - Click "New OAuth App"
   - Fill in details:
     ```
     Application name: Rusty Audio
     Homepage URL: https://rusty-audio.com
     Authorization callback URL: https://rusty-audio.pages.dev/auth-callback
     ```

2. **For Development:**
   - Create separate OAuth App for localhost
   - Authorization callback URL: `http://localhost:8080/auth-callback`

3. **Copy Credentials:**
   - Client ID: `Iv1.xxx`
   - Generate Client Secret
   - Save these securely

#### Microsoft OAuth (Azure AD)

1. **Register Application:**
   - Go to https://portal.azure.com/
   - Navigate to "Azure Active Directory" → "App registrations"
   - Click "New registration"
   - Name: "Rusty Audio"
   - Supported account types: "Accounts in any organizational directory and personal Microsoft accounts"

2. **Configure Redirect URIs:**
   - Go to "Authentication"
   - Add platform → "Web"
   - Redirect URIs:
     ```
     Development:
     http://localhost:8080/auth-callback

     Production:
     https://rusty-audio.pages.dev/auth-callback
     https://rusty-audio.com/auth-callback
     ```

3. **Create Client Secret:**
   - Go to "Certificates & secrets"
   - Click "New client secret"
   - Description: "Rusty Audio Auth"
   - Expires: 24 months
   - Copy the secret value immediately (shown only once)

4. **Configure API Permissions:**
   - Go to "API permissions"
   - Add permission → "Microsoft Graph" → "Delegated permissions"
   - Select: `User.Read`, `email`, `openid`, `profile`
   - Grant admin consent (if required)

5. **Copy Credentials:**
   - Application (client) ID: `xxx`
   - Client secret: `xxx`
   - Save these securely

### 3. Create KV Namespaces

KV namespaces store session data, user profiles, and rate limit counters.

#### Development Environment

```bash
cd workers/auth-service

# Run setup script
./scripts/setup-kv.sh dev

# Or manually:
wrangler kv:namespace create "SESSIONS" --env dev
wrangler kv:namespace create "USERS" --env dev
wrangler kv:namespace create "RATE_LIMIT" --env dev
```

Copy the namespace IDs from the output.

#### Production Environment

```bash
./scripts/setup-kv.sh production

# Or manually:
wrangler kv:namespace create "SESSIONS" --env production
wrangler kv:namespace create "USERS" --env production
wrangler kv:namespace create "RATE_LIMIT" --env production
```

#### Update wrangler.toml

Edit `wrangler.toml` and replace the placeholder IDs:

```toml
# Development
[[env.dev.kv_namespaces]]
binding = "SESSIONS"
id = "abc123def456"  # Replace with actual ID

[[env.dev.kv_namespaces]]
binding = "USERS"
id = "ghi789jkl012"  # Replace with actual ID

[[env.dev.kv_namespaces]]
binding = "RATE_LIMIT"
id = "mno345pqr678"  # Replace with actual ID

# Production (similar)
```

### 4. Set Secrets

Secrets are encrypted environment variables managed by Cloudflare.

#### Generate JWT Secret

```bash
# Generate a strong random secret (minimum 32 bytes)
openssl rand -base64 32
```

#### Development Environment

```bash
# Interactive setup
./scripts/setup-secrets.sh dev

# Or manually:
echo "your_google_client_id" | wrangler secret put GOOGLE_CLIENT_ID --env dev
echo "your_google_client_secret" | wrangler secret put GOOGLE_CLIENT_SECRET --env dev
echo "your_github_client_id" | wrangler secret put GITHUB_CLIENT_ID --env dev
echo "your_github_client_secret" | wrangler secret put GITHUB_CLIENT_SECRET --env dev
echo "your_microsoft_client_id" | wrangler secret put MICROSOFT_CLIENT_ID --env dev
echo "your_microsoft_client_secret" | wrangler secret put MICROSOFT_CLIENT_SECRET --env dev
echo "your_jwt_secret" | wrangler secret put JWT_SECRET --env dev
```

#### Production Environment

```bash
# Interactive setup
./scripts/setup-secrets.sh production

# Or manually (same commands with --env production)
```

#### Verify Secrets

```bash
# List all secrets
wrangler secret list --env dev
wrangler secret list --env production

# Should show:
# GOOGLE_CLIENT_ID
# GOOGLE_CLIENT_SECRET
# GITHUB_CLIENT_ID
# GITHUB_CLIENT_SECRET
# MICROSOFT_CLIENT_ID
# MICROSOFT_CLIENT_SECRET
# JWT_SECRET
```

### 5. Configure GitHub Secrets (CI/CD)

For automated deployments, add secrets to your GitHub repository:

1. Go to GitHub repository → Settings → Secrets and variables → Actions
2. Add the following secrets:

```
CLOUDFLARE_API_TOKEN=<your_cloudflare_api_token>
CLOUDFLARE_ACCOUNT_ID=<your_cloudflare_account_id>

# Development
DEV_GOOGLE_CLIENT_ID=<dev_google_client_id>
DEV_GOOGLE_CLIENT_SECRET=<dev_google_client_secret>
DEV_GITHUB_CLIENT_ID=<dev_github_client_id>
DEV_GITHUB_CLIENT_SECRET=<dev_github_client_secret>
DEV_MICROSOFT_CLIENT_ID=<dev_microsoft_client_id>
DEV_MICROSOFT_CLIENT_SECRET=<dev_microsoft_client_secret>
DEV_JWT_SECRET=<dev_jwt_secret>

# Production
PROD_GOOGLE_CLIENT_ID=<prod_google_client_id>
PROD_GOOGLE_CLIENT_SECRET=<prod_google_client_secret>
PROD_GITHUB_CLIENT_ID=<prod_github_client_id>
PROD_GITHUB_CLIENT_SECRET=<prod_github_client_secret>
PROD_MICROSOFT_CLIENT_ID=<prod_microsoft_client_id>
PROD_MICROSOFT_CLIENT_SECRET=<prod_microsoft_client_secret>
PROD_JWT_SECRET=<prod_jwt_secret>
```

### 6. Deploy

#### Local Development

```bash
# Start local development server
npm run dev

# Worker runs at http://localhost:8787
# Test endpoint: curl http://localhost:8787/health
```

#### Deploy to Development

```bash
# Deploy to dev environment
npm run deploy

# Or with script:
./scripts/deploy.sh dev
```

#### Deploy to Production

```bash
# Deploy to production environment
npm run deploy:prod

# Or with script:
./scripts/deploy.sh production
```

### 7. Configure Custom Domain (Optional)

1. **Add Domain to Cloudflare:**
   - Go to Cloudflare Dashboard
   - Add site: `rusty-audio.com`
   - Update nameservers

2. **Add Worker Route:**
   - Go to Workers & Pages → rusty-audio-auth-prod
   - Triggers → Routes → Add route
   - Route: `api.rusty-audio.com/*`
   - Zone: `rusty-audio.com`

3. **Update wrangler.toml:**
   ```toml
   [env.production]
   routes = [
     { pattern = "api.rusty-audio.com/*", zone_name = "rusty-audio.com" }
   ]
   ```

4. **Update OAuth Redirect URIs:**
   - Add `https://api.rusty-audio.com/auth-callback` to all providers

### 8. Testing

#### Health Check

```bash
# Local
curl http://localhost:8787/health

# Dev
curl https://rusty-audio-auth-dev.workers.dev/health

# Production
curl https://api.rusty-audio.com/health
```

#### Test OAuth Flow

```bash
# 1. Initiate auth
curl -X POST https://api.rusty-audio.com/api/auth/initiate \
  -H "Content-Type: application/json" \
  -d '{"provider":"google"}'

# Should return: { authUrl, state, codeVerifier }

# 2. Open authUrl in browser, complete OAuth
# 3. Exchange code for tokens (see README.md for full example)
```

### 9. Monitoring

#### View Logs

```bash
# Tail production logs
npm run tail

# Or with wrangler:
wrangler tail --env production

# Filter by status:
wrangler tail --env production --status error
```

#### Cloudflare Dashboard

- Go to Workers & Pages → rusty-audio-auth-prod
- Metrics tab: View request count, error rate, latency
- Real-time Logs: See live requests

### 10. Troubleshooting

#### "Namespace not found" Error

```bash
# Verify KV namespace IDs
wrangler kv:namespace list

# Compare with wrangler.toml
```

#### "Secret not set" Error

```bash
# List secrets
wrangler secret list --env production

# Set missing secret
echo "value" | wrangler secret put SECRET_NAME --env production
```

#### OAuth Callback Fails

- Verify redirect URI matches exactly in provider settings
- Check CORS allowed origins in `src/middleware/cors.ts`
- Ensure APP_URL and REDIRECT_URI in wrangler.toml are correct

#### Rate Limit Issues

- Check client IP in logs
- Adjust rate limits in `src/middleware/ratelimit.ts`
- Clear rate limit KV: `wrangler kv:key delete <key> --namespace-id=<RATE_LIMIT_ID>`

## Environment Checklist

Before deploying to production:

- [ ] All OAuth providers configured with production redirect URIs
- [ ] KV namespaces created and IDs updated in wrangler.toml
- [ ] All secrets set via wrangler secret put
- [ ] GitHub Actions secrets configured
- [ ] Custom domain configured (if using)
- [ ] CORS allowed origins updated for production URLs
- [ ] Health check passes
- [ ] Test OAuth flow with each provider
- [ ] Monitoring and alerts configured

## Security Reminders

- Never commit secrets to version control
- Use different credentials for dev/prod
- Rotate JWT_SECRET every 90 days
- Enable 2FA on Cloudflare account
- Regularly review access logs
- Keep dependencies updated (npm audit)

## Support

For issues during setup:
- Check wrangler docs: https://developers.cloudflare.com/workers/
- GitHub issues: https://github.com/rusty-audio/rusty-audio/issues
- Cloudflare community: https://community.cloudflare.com/
