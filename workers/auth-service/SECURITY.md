# Security Audit Checklist

This document outlines security considerations and best practices for the Rusty Audio Auth Service.

## ✅ Security Checklist

### Authentication & Authorization

- [x] **PKCE Implementation**: OAuth 2.0 with PKCE (RFC 7636) for enhanced security
- [x] **State Parameter**: Random state generation to prevent CSRF attacks
- [x] **Code Verifier**: Cryptographically secure random code verifier (64 bytes)
- [x] **Code Challenge**: SHA-256 hashing of code verifier
- [x] **JWT Signing**: HS256 algorithm with strong secret key
- [x] **Token Expiration**: Access tokens expire after 1 hour
- [x] **Refresh Tokens**: Long-lived tokens (30 days) for token renewal
- [x] **Token Verification**: Signature, expiration, issuer, audience validation

### Data Protection

- [x] **Secrets Management**: All secrets stored in Cloudflare environment variables
- [x] **KV Encryption**: Cloudflare KV data encrypted at rest
- [x] **HTTPS Only**: All endpoints served over HTTPS
- [x] **Secure Headers**: CORS, Content-Type security headers
- [x] **Session TTL**: Sessions expire after 30 days
- [x] **Password Storage**: N/A (OAuth only, no passwords stored)

### Input Validation

- [x] **Provider Validation**: Only allowed providers (google, github, microsoft)
- [x] **Email Validation**: RFC-compliant email format validation
- [x] **String Sanitization**: XSS prevention via input sanitization
- [x] **Request Size Limits**: Cloudflare Workers default limits (100KB)
- [x] **JSON Parsing**: Safe JSON parsing with error handling

### Rate Limiting

- [x] **Per-IP Limiting**: Rate limits enforced per client IP address
- [x] **Endpoint-Specific**: Different limits per endpoint
- [x] **Sliding Window**: Time-based window reset
- [x] **Retry-After Header**: Client guidance on retry timing

### Network Security

- [x] **CORS Policy**: Whitelist of allowed origins
- [x] **HTTPS Enforcement**: Cloudflare automatic HTTPS redirect
- [x] **TLS 1.2+**: Modern TLS versions only
- [x] **Certificate Validation**: Cloudflare managed certificates

### Logging & Monitoring

- [x] **Error Logging**: All errors logged to console
- [x] **Request Logging**: Available via wrangler tail
- [x] **No PII Logging**: Sensitive data not logged
- [x] **Audit Trail**: Session creation/deletion logged

### Code Security

- [x] **Type Safety**: TypeScript with strict mode
- [x] **Dependency Scanning**: Regular npm audit checks
- [x] **No Eval**: No dynamic code execution
- [x] **Constant-Time Comparison**: Timing-safe string comparison for tokens
- [x] **Error Handling**: Graceful error handling, no stack traces exposed

## 🔐 Security Best Practices

### JWT Secret Management

**Generate Strong Secret:**
```bash
# Minimum 256-bit (32 bytes) secret
openssl rand -base64 32
```

**Rotate Secrets:**
- Rotate JWT_SECRET every 90 days
- Update all environments simultaneously
- Invalidate existing sessions on rotation

### OAuth Provider Configuration

**Redirect URI Whitelist:**
- Only add exact redirect URIs
- Use HTTPS in production
- Avoid wildcard patterns

**Scope Minimization:**
- Request only necessary scopes
- Google: `openid`, `email`, `profile`
- GitHub: `read:user`, `user:email`
- Microsoft: `openid`, `email`, `profile`, `User.Read`

### Rate Limiting Strategy

**Recommended Limits:**
```typescript
/api/auth/initiate:  10 requests/minute
/api/auth/callback:  5 requests/minute
/api/auth/refresh:   20 requests/minute
/api/auth/logout:    10 requests/minute
/api/auth/user:      30 requests/minute
```

**Adjust Based on:**
- User behavior patterns
- Legitimate use cases (e.g., token refresh on app startup)
- Attack mitigation requirements

### CORS Configuration

**Allowed Origins:**
```typescript
const ALLOWED_ORIGINS = [
  'http://localhost:8080',        // Development only
  'https://rusty-audio.pages.dev', // Production
  'https://rusty-audio.com',       // Custom domain
];
```

**Never Allow:**
- `*` wildcard origin
- Untrusted domains
- HTTP origins in production

## 🚨 Security Incidents

### Response Plan

1. **Detection**: Monitor logs for suspicious activity
2. **Analysis**: Identify attack vector and scope
3. **Containment**: Rate limit aggressive IPs, rotate secrets if needed
4. **Eradication**: Fix vulnerability, deploy patch
5. **Recovery**: Restore normal operations
6. **Post-Mortem**: Document incident, improve defenses

### Suspicious Activity Indicators

- Unusual rate of failed login attempts
- High volume from single IP
- Requests with malformed tokens
- CORS violations from unknown origins
- Abnormal geographic distribution

### Emergency Actions

**Revoke All Sessions:**
```bash
# Delete all sessions from KV
wrangler kv:key delete --all --namespace-id=<SESSIONS_ID>
```

**Rotate JWT Secret:**
```bash
# Generate new secret
NEW_SECRET=$(openssl rand -base64 32)

# Update secret
echo "$NEW_SECRET" | wrangler secret put JWT_SECRET --env production
```

**Enable IP Blocking:**
```typescript
// Add to middleware/ratelimit.ts
const BLOCKED_IPS = [
  '123.45.67.89',
  '98.76.54.32',
];

if (BLOCKED_IPS.includes(clientIp)) {
  return new Response('Forbidden', { status: 403 });
}
```

## 🔍 Security Testing

### Manual Testing

**1. Test PKCE Flow:**
```bash
# Initiate auth
curl -X POST https://api.rusty-audio.com/api/auth/initiate \
  -H "Content-Type: application/json" \
  -d '{"provider":"google"}'

# Should return authUrl, state, codeVerifier
```

**2. Test JWT Verification:**
```bash
# Get user with invalid token
curl https://api.rusty-audio.com/api/auth/user \
  -H "Authorization: Bearer invalid_token"

# Should return 401 Unauthorized
```

**3. Test Rate Limiting:**
```bash
# Send 20 requests rapidly
for i in {1..20}; do
  curl -X POST https://api.rusty-audio.com/api/auth/initiate \
    -H "Content-Type: application/json" \
    -d '{"provider":"google"}'
done

# Should eventually return 429 Too Many Requests
```

**4. Test CORS:**
```bash
# Request from unauthorized origin
curl https://api.rusty-audio.com/api/auth/user \
  -H "Origin: https://evil.com" \
  -H "Authorization: Bearer <token>"

# Should deny or not include CORS headers
```

### Automated Security Scanning

**Dependency Scanning:**
```bash
npm audit
npm audit fix
```

**TypeScript Security:**
```bash
npm run type-check
```

**Secrets Detection:**
```bash
# Install gitleaks
brew install gitleaks

# Scan for secrets
gitleaks detect --source . --verbose
```

## 📊 Security Metrics

### KPIs to Monitor

- **Authentication Success Rate**: > 95%
- **Token Validation Failure Rate**: < 1%
- **Rate Limit Hits**: < 0.1% of requests
- **Error Rate**: < 0.5%
- **Mean Time to Detect (MTTD)**: < 5 minutes
- **Mean Time to Respond (MTTR)**: < 30 minutes

### Alerting Thresholds

```yaml
alerts:
  - name: High Error Rate
    condition: error_rate > 5%
    severity: critical

  - name: Authentication Failures
    condition: auth_failure_rate > 10%
    severity: warning

  - name: Rate Limit Abuse
    condition: rate_limit_hits > 100/hour from single IP
    severity: warning

  - name: Latency Spike
    condition: p95_latency > 1000ms
    severity: warning
```

## 🛡️ Compliance

### GDPR Considerations

- **Right to Access**: Users can retrieve their profile via GET /api/auth/user
- **Right to Erasure**: Implement user deletion endpoint (future)
- **Data Minimization**: Only collect necessary OAuth profile data
- **Consent**: OAuth flow includes user consent screen
- **Data Retention**: Sessions expire after 30 days

### OAuth 2.0 Compliance

- ✅ RFC 6749: OAuth 2.0 Authorization Framework
- ✅ RFC 7636: PKCE for OAuth Public Clients
- ✅ RFC 7519: JSON Web Tokens (JWT)
- ✅ RFC 6750: Bearer Token Usage

## 🔄 Regular Security Tasks

### Weekly

- [ ] Review error logs for anomalies
- [ ] Check rate limit violations
- [ ] Monitor authentication success rates

### Monthly

- [ ] Update dependencies (npm update)
- [ ] Run security audit (npm audit)
- [ ] Review access logs for suspicious patterns
- [ ] Test backup/recovery procedures

### Quarterly

- [ ] Rotate JWT secret
- [ ] Review and update rate limits
- [ ] Security training for team
- [ ] Penetration testing (optional)

### Annually

- [ ] Full security audit
- [ ] OAuth provider credential rotation
- [ ] Incident response plan review
- [ ] Compliance assessment (GDPR, etc.)

## 📚 References

- [OAuth 2.0 Security Best Practices](https://tools.ietf.org/html/draft-ietf-oauth-security-topics)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Cloudflare Workers Security](https://developers.cloudflare.com/workers/platform/security/)
- [JWT Best Practices](https://tools.ietf.org/html/rfc8725)

## 📞 Security Contact

For security issues, please contact:
- Email: security@rusty-audio.com
- GitHub Security Advisories: https://github.com/rusty-audio/rusty-audio/security
