# Workspace Migration Overview: Desktop/Web Application Separation

## Executive Summary

This document provides a high-level overview of the complete architecture redesign for splitting Rusty Audio into separate desktop and web applications with OAuth 2.0 authentication.

**Quick Links:**
- [Workspace Structure](WORKSPACE_ARCHITECTURE.md) - Detailed crate organization
- [OAuth Implementation](OAUTH_ARCHITECTURE.md) - Authentication system
- [API Design](API_SPECIFICATION.md) - Backend API (OpenAPI spec)
- [Deployment](DEPLOYMENT_ARCHITECTURE.md) - Infrastructure and CI/CD
- [Migration Guide](MIGRATION_GUIDE.md) - Step-by-step implementation
- [Security](SECURITY_THREAT_MODEL.md) - Threat model and mitigations

---

## Current vs. Target Architecture

### Current State (Monolithic)
```
rusty-audio/
├── src/
│   ├── main.rs           # Desktop app
│   ├── web.rs            # WASM app
│   ├── lib.rs            # Shared code
│   ├── audio/            # Mixed desktop/web code
│   └── ui/               # Mixed desktop/web code
└── Cargo.toml            # Single package
```

**Problems:**
- Code duplication between desktop and web
- Complex cfg gates (#[cfg(not(target_arch = "wasm32"))])
- Difficult to maintain separate feature sets
- No clean compilation boundaries

---

### Target State (Workspace)
```
rusty-audio/
├── Cargo.toml                   # Workspace root
├── rusty-audio-core/            # Shared library
│   ├── src/
│   │   ├── audio/               # Platform-agnostic DSP
│   │   ├── ui/                  # Shared UI components
│   │   └── security/            # Common security
│   └── Cargo.toml
├── rusty-audio-desktop/         # Native desktop app
│   ├── src/
│   │   ├── main.rs              # Desktop entry point
│   │   ├── audio/               # CPAL, ASIO backends
│   │   └── platform/            # Windows, Linux, macOS
│   └── Cargo.toml
└── rusty-audio-web/             # WASM web app
    ├── src/
    │   ├── lib.rs               # WASM entry point
    │   ├── auth/                # OAuth 2.0
    │   └── audio/               # Web Audio API
    ├── static/                  # PWA assets
    └── Cargo.toml
```

**Benefits:**
- Zero code duplication
- Clean platform separation
- Independent feature development
- Optimized binary sizes

---

## Key Features by Platform

### Desktop Application (Local-First)

**Features:**
- Standalone native binary (no network required)
- Professional audio backends (ASIO on Windows)
- Full file system access
- Audio recording to disk
- No authentication (privacy-focused)
- Works completely offline

**Platforms:**
- Windows (x64) - ASIO support
- Linux (x64) - AppImage, Flatpak, .deb
- macOS (x64 + ARM) - Universal binary, DMG

**Distribution:**
- GitHub Releases
- Direct download
- Package managers (Homebrew, Chocolatey)

---

### Web Application (Cloud-Synced)

**Features:**
- Browser-based (no installation)
- OAuth 2.0 authentication (Google, GitHub, Microsoft)
- Cloud-synced presets across devices
- PWA support (offline mode)
- Premium tier with usage analytics
- Mobile-friendly UI

**Access:**
- Any modern browser
- Installable PWA
- Works on mobile devices
- Cross-device sync

**Hosting:**
- Cloudflare Pages (frontend)
- Cloudflare Workers (backend API)
- Cloudflare D1 (database)
- Global edge network (low latency)

---

## OAuth 2.0 Authentication

### Why OAuth 2.0 with PKCE?

**Security Benefits:**
- Industry-standard authentication
- No password management
- PKCE prevents code interception
- Multi-provider support
- Secure token storage (encrypted)

**User Benefits:**
- One-click login with existing accounts
- No new passwords to remember
- Sync presets across devices
- Premium features unlocked

### Supported Providers

1. **Google** - Most popular
2. **GitHub** - Developer-friendly
3. **Microsoft** - Enterprise users

### Authentication Flow

```
User clicks "Sign In"
  → Redirect to provider (Google/GitHub/MS)
  → User authenticates
  → Redirect back with code
  → Exchange code for token (PKCE verification)
  → Store encrypted token in browser
  → Load application with user session
```

**Security Features:**
- PKCE code challenge/verifier
- State parameter (CSRF protection)
- AES-256-GCM token encryption
- 30-day session expiration
- Auto-refresh before expiration

---

## Backend API Design

### Architecture

```
Browser → Cloudflare Pages (WASM)
        → Cloudflare Workers (API)
        → Cloudflare D1 (Database)
```

### Core Endpoints

**Authentication:**
- `POST /api/auth/callback` - Exchange OAuth code for token
- `POST /api/auth/refresh` - Refresh expiring token
- `POST /api/auth/logout` - Invalidate session
- `GET /api/auth/user` - Get user profile

**Presets:**
- `GET /api/presets` - List user's presets
- `POST /api/presets` - Create preset (10 limit for free tier)
- `PUT /api/presets/:id` - Update preset
- `DELETE /api/presets/:id` - Delete preset

**Usage (Premium):**
- `GET /api/usage` - Get analytics
- `POST /api/usage` - Log usage event

### Data Models

**User:**
```rust
struct User {
    id: Uuid,
    email: String,
    name: String,
    avatar_url: Option<String>,
    provider: OAuthProvider,  // Google, GitHub, Microsoft
    tier: UserTier,           // Free, Premium
    created_at: DateTime<Utc>,
}
```

**Audio Preset:**
```rust
struct AudioPreset {
    id: Uuid,
    user_id: Uuid,
    name: String,
    eq_settings: [EqBand; 8],  // 60Hz - 7680Hz
    effects_config: Option<EffectsConfig>,
    is_public: bool,
    created_at: DateTime<Utc>,
}
```

---

## Deployment Strategy

### Desktop Deployment

**Build Process:**
```bash
# Windows
cargo build --release --package rusty-audio-desktop
# Output: rusty-audio.exe (~15-20 MB)

# Linux
cargo build --release --package rusty-audio-desktop
# Create AppImage, .deb, Flatpak

# macOS
# Build universal binary (x64 + ARM)
# Create signed and notarized DMG
```

**CI/CD:**
- GitHub Actions for all platforms
- Automated releases on tags
- Code signing (Windows, macOS)
- Notarization (macOS)

---

### Web Deployment

**Build Process:**
```bash
# Build WASM
cd rusty-audio-web
wasm-pack build --release --target web
# Output: rusty_audio_web.wasm (~2-3 MB gzipped)

# Deploy to Cloudflare
wrangler pages deploy dist
wrangler deploy  # Workers
```

**Infrastructure:**
- **Frontend:** Cloudflare Pages (global CDN)
- **Backend:** Cloudflare Workers (serverless)
- **Database:** Cloudflare D1 (SQLite at edge)
- **Sessions:** Cloudflare KV (key-value store)

**CI/CD:**
- Automatic deployment on main branch merge
- Preview deployments for PRs
- Staging environment for testing
- Zero-downtime deployments

---

## Security Highlights

### Web Application Security

**OAuth Security:**
- PKCE code challenge/verifier
- State parameter validation
- Authorization code single-use
- Token expiration and rotation

**API Security:**
- HTTPS only (HSTS enforced)
- CORS restricted to known origins
- Rate limiting on all endpoints
- Prepared statements (no SQL injection)
- Content Security Policy headers

**Token Storage:**
- AES-256-GCM encryption
- Stored in IndexedDB (not LocalStorage)
- Auto-logout after 30 days
- Secure deletion on logout

### Desktop Application Security

**File Handling:**
- File format validation
- Size limits enforced
- Path canonicalization (prevent traversal)
- Memory-safe audio decoding

**Audio Safety:**
- Maximum volume/gain limits
- Hearing damage protection
- Soft-limiter on master output

---

## Cost Estimation

### Cloudflare Free Tier

**Limits:**
- Pages: 500 builds/month, unlimited bandwidth
- Workers: 100,000 requests/day
- D1: 5 GB storage, 5M rows read/day
- KV: 1 GB storage, 100K reads/day

**Capacity:** ~1,000 daily active users

### Paid Tier (10,000 users)

| Service | Monthly Cost |
|---------|--------------|
| Pages | $0 |
| Workers | $5 |
| D1 | $5 |
| KV | $5 |
| **Total** | **$15/month** |

**Unit Economics:** $0.0015 per user/month

---

## Migration Timeline

### Week 1: Foundation
- Create workspace structure
- Extract shared code to core library
- Test core library independently

### Week 2: Application Separation
- Create desktop crate with CPAL/ASIO
- Create web crate with Web Audio API
- Test both applications

### Week 3: Authentication
- Implement OAuth 2.0 PKCE flow
- Secure token storage
- Backend API development
- Integration testing

### Week 4: Deployment
- CI/CD pipeline setup
- Staging deployment
- Security audit
- Production launch

**Total Time:** 3-4 weeks
**Risk Level:** Medium (mitigated by testing)

---

## Success Criteria

The migration is complete when:

**Technical:**
- [ ] All tests pass in all three crates
- [ ] Zero code duplication verified
- [ ] Desktop builds on Windows, Linux, macOS
- [ ] WASM bundle < 3 MB (gzipped)
- [ ] OAuth works with all providers
- [ ] Security audit passes

**Functional:**
- [ ] Desktop: Playback, EQ, recording work
- [ ] Web: Login, preset sync, playback work
- [ ] CI/CD: Automated builds/deployments
- [ ] Performance: Within 5% of baseline

**Quality:**
- [ ] Documentation complete
- [ ] Monitoring configured
- [ ] Incident response plan documented

---

## Performance Targets

### Desktop Application

| Metric | Target |
|--------|--------|
| Binary Size | < 25 MB |
| Startup Time | < 2 seconds |
| Audio Latency (ASIO) | < 10ms |
| CPU Usage (playback) | < 5% |
| Memory Usage | < 100 MB |

### Web Application

| Metric | Target |
|--------|--------|
| WASM Size | < 3 MB (gzipped) |
| Load Time (3G) | < 3 seconds |
| Time to Interactive | < 5 seconds |
| API Latency (p95) | < 200ms |
| Lighthouse Score | > 90 |

---

## Technology Stack Summary

### Core Library
- egui 0.33.0 (UI framework)
- rustfft (DSP)
- serde (serialization)
- rayon (parallelism)

### Desktop
- eframe + wgpu (native GUI)
- cpal + ASIO (audio I/O)
- symphonia (audio decoding)
- tokio (async runtime)

### Web
- wasm-bindgen (WASM bindings)
- gloo-net (HTTP client)
- rexie (IndexedDB)
- sha2 + aes-gcm (crypto)
- Web Audio API (via web-sys)

### Backend
- Cloudflare Workers (TypeScript/JavaScript)
- Cloudflare D1 (SQLite)
- Cloudflare KV (sessions)
- Cloudflare Pages (CDN)

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| WASM performance issues | Benchmark early, optimize critical paths |
| OAuth complexity | Use proven patterns, test thoroughly |
| Desktop audio backend issues | Maintain CPAL + ASIO fallback |
| Migration breaks features | Comprehensive tests, phased rollout |
| Cloudflare costs exceed budget | Monitor usage, rate limiting |
| Security breach | Defense-in-depth, penetration testing |

---

## Next Steps

1. **Review Architecture Documents**
   - Read all 6 architecture documents in detail
   - Understand separation strategy
   - Review security model

2. **Approve Design**
   - Technical review by team
   - Security review
   - Cost/benefit analysis

3. **Begin Migration**
   - Follow MIGRATION_GUIDE.md step-by-step
   - Phase 1: Workspace setup (Week 1)
   - Phase 2-4: Application separation (Weeks 2-3)
   - Phase 5: OAuth implementation (Week 3)
   - Phase 6: Deployment (Week 4)

4. **Testing and Validation**
   - Comprehensive test suite
   - Security audit
   - Performance benchmarking

5. **Launch**
   - Staging deployment
   - Production deployment
   - Monitoring and incident response

---

## Document Index

| Document | Purpose | Pages |
|----------|---------|-------|
| [WORKSPACE_ARCHITECTURE.md](WORKSPACE_ARCHITECTURE.md) | Workspace structure, module dependencies | 40+ |
| [OAUTH_ARCHITECTURE.md](OAUTH_ARCHITECTURE.md) | OAuth 2.0 PKCE flow, token storage | 50+ |
| [API_SPECIFICATION.md](API_SPECIFICATION.md) | OpenAPI spec, endpoints, errors | 35+ |
| [DEPLOYMENT_ARCHITECTURE.md](DEPLOYMENT_ARCHITECTURE.md) | CI/CD, Cloudflare, monitoring | 45+ |
| [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) | Step-by-step migration plan | 60+ |
| [SECURITY_THREAT_MODEL.md](SECURITY_THREAT_MODEL.md) | Threats, mitigations, checklist | 55+ |
| **WORKSPACE_MIGRATION_OVERVIEW.md** | **This document (summary)** | **15** |

**Total:** ~300 pages of comprehensive architecture documentation

---

## Support and Questions

For questions about specific aspects:

- **Workspace structure:** See WORKSPACE_ARCHITECTURE.md
- **OAuth implementation:** See OAUTH_ARCHITECTURE.md
- **API design:** See API_SPECIFICATION.md
- **Deployment:** See DEPLOYMENT_ARCHITECTURE.md
- **Migration steps:** See MIGRATION_GUIDE.md
- **Security:** See SECURITY_THREAT_MODEL.md

---

## Conclusion

This architecture redesign provides:

**Clean Separation:**
- Desktop: Local-first, no auth, privacy-focused
- Web: Cloud-synced, OAuth-gated, cross-device
- Core: Shared library, zero duplication

**Production-Ready:**
- Industry-standard OAuth 2.0 with PKCE
- Defense-in-depth security model
- Cloudflare edge deployment
- Comprehensive monitoring

**Future-Proof:**
- Extensible for mobile, CLI, VST
- Scalable infrastructure (free → 100k users)
- Clean API for third-party integrations
- Modern PWA capabilities

**Ready for Implementation:**
- 6 comprehensive architecture documents
- Step-by-step migration guide
- Security threat model
- Complete API specification
- Deployment playbooks

The architecture is comprehensive, secure, and ready for development to begin.
