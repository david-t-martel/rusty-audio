# Deployment Guide

Complete deployment guide for the Rusty Audio Auth Service on Cloudflare Workers.

## Quick Deployment Checklist

- [ ] Install dependencies: `npm install`
- [ ] Configure OAuth providers (Google, GitHub, Microsoft)
- [ ] Create KV namespaces: `./scripts/setup-kv.sh production`
- [ ] Set secrets: `./scripts/setup-secrets.sh production`
- [ ] Update `wrangler.toml` with KV namespace IDs
- [ ] Configure GitHub Actions secrets
- [ ] Deploy: `./scripts/deploy.sh production`
- [ ] Test endpoints: `./scripts/test-endpoints.sh https://api.rusty-audio.com`
- [ ] Configure custom domain (optional)
- [ ] Set up monitoring and alerts

## Pre-Deployment

### 1. Install Dependencies

```bash
cd workers/auth-service
npm install
```

Verify installation:
```bash
npm run type-check
npm run lint
```

### 2. Configure OAuth Providers

Create OAuth applications for each provider:

#### Google
- Console: https://console.cloud.google.com/
- Create OAuth 2.0 credentials
- Authorized redirect URI: `https://rusty-audio.pages.dev/auth-callback`
- Copy Client ID and Secret

#### GitHub
- Settings: https://github.com/settings/developers
- Create OAuth App
- Authorization callback URL: `https://rusty-audio.pages.dev/auth-callback`
- Copy Client ID and Secret

#### Microsoft
- Portal: https://portal.azure.com/
- Register application
- Redirect URI: `https://rusty-audio.pages.dev/auth-callback`
- Copy Application ID and Client Secret

See `SETUP.md` for detailed instructions.

### 3. Create KV Namespaces

```bash
# Production namespaces
./scripts/setup-kv.sh production

# Output will show namespace IDs like:
# SESSIONS: abc123def456
# USERS: ghi789jkl012
# RATE_LIMIT: mno345pqr678
```

Update `wrangler.toml` with these IDs:
```toml
[[env.production.kv_namespaces]]
binding = "SESSIONS"
id = "abc123def456"  # Replace with actual ID

[[env.production.kv_namespaces]]
binding = "USERS"
id = "ghi789jkl012"  # Replace with actual ID

[[env.production.kv_namespaces]]
binding = "RATE_LIMIT"
id = "mno345pqr678"  # Replace with actual ID
```

### 4. Set Secrets

```bash
# Interactive secret setup
./scripts/setup-secrets.sh production

# Or manually:
echo "your_google_client_id" | wrangler secret put GOOGLE_CLIENT_ID --env production
echo "your_google_client_secret" | wrangler secret put GOOGLE_CLIENT_SECRET --env production
echo "your_github_client_id" | wrangler secret put GITHUB_CLIENT_ID --env production
echo "your_github_client_secret" | wrangler secret put GITHUB_CLIENT_SECRET --env production
echo "your_microsoft_client_id" | wrangler secret put MICROSOFT_CLIENT_ID --env production
echo "your_microsoft_client_secret" | wrangler secret put MICROSOFT_CLIENT_SECRET --env production

# Generate JWT secret (minimum 32 bytes)
JWT_SECRET=$(openssl rand -base64 32)
echo "$JWT_SECRET" | wrangler secret put JWT_SECRET --env production
```

Verify secrets:
```bash
wrangler secret list --env production
```

### 5. Configure GitHub Actions (CI/CD)

Add secrets to GitHub repository (Settings → Secrets and variables → Actions):

```
CLOUDFLARE_API_TOKEN=<your_cloudflare_api_token>
CLOUDFLARE_ACCOUNT_ID=<your_cloudflare_account_id>

PROD_GOOGLE_CLIENT_ID=<google_client_id>
PROD_GOOGLE_CLIENT_SECRET=<google_client_secret>
PROD_GITHUB_CLIENT_ID=<github_client_id>
PROD_GITHUB_CLIENT_SECRET=<github_client_secret>
PROD_MICROSOFT_CLIENT_ID=<microsoft_client_id>
PROD_MICROSOFT_CLIENT_SECRET=<microsoft_client_secret>
PROD_JWT_SECRET=<jwt_secret>
```

## Deployment

### Option 1: Manual Deployment

```bash
# Deploy to production
./scripts/deploy.sh production

# Or with npm:
npm run deploy:prod
```

### Option 2: Automated Deployment (GitHub Actions)

Deployment is triggered automatically on:
- Push to `main` branch (production)
- Pull request (development)
- Manual trigger via GitHub Actions UI

**Workflow file:** `.github/workflows/deploy-auth-worker.yml`

**To trigger manual deployment:**
1. Go to GitHub → Actions → "Deploy Auth Worker"
2. Click "Run workflow"
3. Select branch and environment

### Deployment Output

Successful deployment will show:
```
✅ Production deployment complete
🔗 URL: https://api.rusty-audio.com
🏥 Running health check...
✅ Health check passed
🎉 Deployment successful!
```

## Post-Deployment

### 1. Verify Deployment

```bash
# Health check
curl https://api.rusty-audio.com/health

# Should return:
# {"status":"healthy","service":"rusty-audio-auth","timestamp":"..."}
```

### 2. Test Endpoints

```bash
# Run full endpoint test suite
./scripts/test-endpoints.sh https://api.rusty-audio.com

# Should show:
# ✓ Health Check
# ✓ Initiate Auth (Google)
# ✓ Rate Limiting
# ...
# ✓ All tests passed!
```

### 3. Test OAuth Flow

```bash
# 1. Initiate auth
curl -X POST https://api.rusty-audio.com/api/auth/initiate \
  -H "Content-Type: application/json" \
  -d '{"provider":"google"}'

# 2. Open authUrl in browser and complete OAuth
# 3. Test callback (replace with actual values)
curl -X POST https://api.rusty-audio.com/api/auth/callback \
  -H "Content-Type: application/json" \
  -d '{
    "code": "authorization_code",
    "state": "state_value",
    "codeVerifier": "verifier_value",
    "provider": "google"
  }'
```

### 4. Monitor Logs

```bash
# Tail production logs
wrangler tail --env production

# Filter errors
wrangler tail --env production --status error
```

### 5. Configure Custom Domain (Optional)

**Prerequisites:**
- Domain added to Cloudflare
- DNS managed by Cloudflare

**Steps:**

1. **Add Worker Route:**
   - Cloudflare Dashboard → Workers & Pages → rusty-audio-auth-prod
   - Triggers → Routes → Add route
   - Pattern: `api.rusty-audio.com/*`
   - Zone: `rusty-audio.com`

2. **Update wrangler.toml:**
   ```toml
   [env.production]
   routes = [
     { pattern = "api.rusty-audio.com/*", zone_name = "rusty-audio.com" }
   ]
   ```

3. **Update OAuth Redirect URIs:**
   - Add `https://api.rusty-audio.com/auth-callback` to all OAuth providers

4. **Test Custom Domain:**
   ```bash
   curl https://api.rusty-audio.com/health
   ```

## Monitoring & Observability

### Cloudflare Dashboard

Access metrics at: Workers & Pages → rusty-audio-auth-prod → Metrics

**Key Metrics:**
- Requests per second
- Error rate (target: < 1%)
- Latency (p50, p95, p99)
- CPU time
- KV operations

### Real-Time Logs

```bash
# Live log stream
wrangler tail --env production

# With sampling (100% of requests)
wrangler tail --env production --sampling-rate 1

# Filter by HTTP status
wrangler tail --env production --status error
wrangler tail --env production --status ok
```

### Alerts

Set up alerts in Cloudflare Dashboard:

1. Workers & Pages → rusty-audio-auth-prod → Alerts
2. Create alert:
   - **High Error Rate:** Error rate > 5% for 5 minutes
   - **High Latency:** p95 latency > 1000ms for 5 minutes
   - **Request Spike:** Requests > 10,000/minute

Notification channels:
- Email
- Webhook (Slack, PagerDuty, etc.)

### Performance Targets

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Availability | 99.9% | < 99.5% |
| Error Rate | < 1% | > 5% |
| Latency (p95) | < 500ms | > 1000ms |
| Auth Success Rate | > 95% | < 90% |

## Rollback Procedure

### Quick Rollback

```bash
# List deployments
wrangler deployments list --env production

# Rollback to previous deployment
wrangler rollback --env production --deployment-id <previous_deployment_id>
```

### Manual Rollback

1. Identify last known good commit
2. Checkout commit: `git checkout <commit_hash>`
3. Deploy: `./scripts/deploy.sh production`
4. Verify: `./scripts/test-endpoints.sh https://api.rusty-audio.com`

### Emergency Rollback

If immediate rollback is needed:

```bash
# Deploy last stable version
git checkout main~1  # Go back one commit
npm run deploy:prod

# Or deploy from specific tag
git checkout v1.0.0
npm run deploy:prod
```

## Scaling & Performance

### Automatic Scaling

Cloudflare Workers automatically scale based on demand:
- No configuration needed
- Handles traffic spikes automatically
- Global edge network (200+ locations)

### Performance Optimization

**KV Reads:**
- Cache KV reads in worker memory
- Use batch operations when possible
- Set appropriate TTLs

**Rate Limiting:**
- Adjust limits based on usage patterns
- Monitor rate limit hits in logs
- Consider IP allowlist for trusted clients

**CORS:**
- Minimize allowed origins
- Cache CORS preflight responses

## Security Hardening

### Rotate Secrets

Rotate JWT_SECRET every 90 days:

```bash
# Generate new secret
NEW_JWT_SECRET=$(openssl rand -base64 32)

# Set new secret
echo "$NEW_JWT_SECRET" | wrangler secret put JWT_SECRET --env production

# Note: This invalidates all existing JWTs
# Users will need to re-authenticate
```

### Update OAuth Credentials

If OAuth credentials are compromised:

1. Generate new credentials in provider console
2. Update secrets:
   ```bash
   echo "new_client_id" | wrangler secret put GOOGLE_CLIENT_ID --env production
   echo "new_client_secret" | wrangler secret put GOOGLE_CLIENT_SECRET --env production
   ```
3. Test authentication flow

### Enable Security Headers

Add to `src/middleware/cors.ts`:

```typescript
'X-Content-Type-Options': 'nosniff',
'X-Frame-Options': 'DENY',
'X-XSS-Protection': '1; mode=block',
'Referrer-Policy': 'strict-origin-when-cross-origin',
```

## Troubleshooting

### Common Issues

**1. "Namespace not found" error**

```bash
# Verify KV namespace IDs
wrangler kv:namespace list

# Check wrangler.toml matches
cat wrangler.toml | grep "id ="
```

**2. OAuth callback fails**

- Verify redirect URI exactly matches in provider settings
- Check CORS allowed origins
- Ensure state parameter matches

**3. High error rate**

```bash
# Check logs
wrangler tail --env production --status error

# Common causes:
# - Invalid secrets
# - KV namespace misconfiguration
# - Rate limiting too aggressive
```

**4. Slow response times**

- Check KV operation latency in dashboard
- Verify OAuth provider response times
- Consider caching user profiles

### Debug Mode

Enable detailed logging:

```typescript
// src/index.ts
console.log('Request:', request.method, request.url);
console.log('Headers:', Object.fromEntries(request.headers));
```

Watch logs:
```bash
wrangler tail --env production
```

## Maintenance Windows

Schedule maintenance for:
- Secret rotation: Every 90 days
- Dependency updates: Monthly
- KV cleanup: Quarterly (remove expired sessions)

**Announce maintenance:**
1. Post to status page
2. Send user notifications
3. Update health endpoint during maintenance

## Cost Management

### Cloudflare Workers Pricing

**Free Tier:**
- 100,000 requests/day
- 10ms CPU time/request
- Sufficient for ~1,000 DAU

**Paid Plan ($5/month):**
- 10 million requests/month
- 50ms CPU time/request
- Suitable for 10,000+ DAU

### Estimated Costs

| Users | Requests/Day | Cost |
|-------|--------------|------|
| 1,000 | 10,000 | Free |
| 10,000 | 100,000 | $5/month |
| 100,000 | 1,000,000 | $15/month |

**KV Storage:**
- Read: 1 million free/day
- Write: 1,000 free/day
- Storage: 1GB free

## Compliance

### GDPR

- User data stored in KV (encrypted at rest)
- Right to access: GET /api/auth/user
- Right to erasure: Implement user deletion endpoint
- Data retention: 30 days (session TTL)

### Data Residency

Cloudflare Workers run globally. For EU-only:

```toml
[env.production]
routes = [
  { pattern = "api.rusty-audio.com/*", zone_name = "rusty-audio.com" }
]

# EU-only workers (requires Enterprise plan)
workers_dev = false
```

## Support & Resources

- **Documentation:** `README.md`, `API.md`, `SECURITY.md`
- **Cloudflare Docs:** https://developers.cloudflare.com/workers/
- **GitHub Issues:** https://github.com/rusty-audio/rusty-audio/issues
- **Community:** https://community.cloudflare.com/

## Deployment History

Track deployments:

```bash
# List recent deployments
wrangler deployments list --env production

# View deployment details
wrangler deployments view <deployment-id> --env production
```

## Next Steps

After successful deployment:

1. [ ] Integrate auth service with Rusty Audio WASM app
2. [ ] Implement user profile management
3. [ ] Add premium tier features
4. [ ] Set up user analytics
5. [ ] Configure backup/disaster recovery
6. [ ] Plan for multi-region deployment (if needed)

---

**Last Updated:** 2025-01-16

**Version:** 1.0.0

**Deployed By:** Rusty Audio Team
