# Rusty Audio - Complete Deployment Guide

Comprehensive deployment documentation for all platforms: Desktop (Windows/Linux/macOS), WASM/PWA, Docker, CI/CD, monitoring, and release automation.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Desktop Deployment](#desktop-deployment)
4. [WASM/PWA Deployment](#wasmpwa-deployment)
5. [Docker Deployment](#docker-deployment)
6. [CI/CD Pipeline](#cicd-pipeline)
7. [Monitoring & Observability](#monitoring--observability)
8. [Release Process](#release-process)
9. [Rollback Procedures](#rollback-procedures)
10. [Troubleshooting](#troubleshooting)

---

## Overview

### Deployment Targets

| Target | Platform | Distribution | Status |
|--------|----------|--------------|--------|
| Desktop | Windows x64 | Portable ZIP, MSI | ✅ Production Ready |
| Desktop | Linux x64 | AppImage, .deb, .rpm, tarball | ✅ Production Ready |
| Desktop | macOS Universal | DMG, .app bundle | ✅ Production Ready |
| WASM/PWA | Web Browsers | GitHub Pages, Cloudflare, Netlify | ✅ Production Ready |
| Docker | Linux containers | Multi-stage builds | ✅ Production Ready |

### Infrastructure Files

```
.github/workflows/
├── production-deploy.yml    # Main CI/CD pipeline
├── release.yml              # Automated release workflow
└── deploy-pwa.yml           # PWA-specific deployment (legacy)

scripts/
├── package-windows.ps1      # Windows packaging
├── package-linux.sh         # Linux packaging
├── package-macos.sh         # macOS packaging
├── deploy-pwa-cdn.sh        # PWA/CDN deployment
├── setup-monitoring.sh      # Monitoring configuration
└── release.sh               # Release automation

Dockerfile                   # Multi-stage Docker build
docker-compose.yml           # Docker services
.dockerignore                # Docker exclusions
```

---

## Quick Start

### Prerequisites Check

```bash
# Verify Rust toolchain
rustc --version
cargo --version

# Verify WASM target
rustup target list --installed | grep wasm32

# Verify tools
wasm-pack --version  # For WASM builds
docker --version     # For containerization
```

### One-Command Deployments

```bash
# Windows Portable
.\scripts\package-windows.ps1 -Version 0.1.0 -CreatePortable

# Linux All Packages
VERSION=0.1.0 ./scripts/package-linux.sh

# macOS Universal Binary + DMG
VERSION=0.1.0 ./scripts/package-macos.sh

# WASM/PWA to GitHub Pages
VERSION=0.1.0 ENVIRONMENT=production CDN_PROVIDER=github-pages ./scripts/deploy-pwa-cdn.sh

# Docker WASM Server
docker-compose up wasm

# Full Release (version bump + builds + publish)
./scripts/release.sh patch  # or minor, major, 1.0.0
```

---

## Desktop Deployment

### Windows

#### Portable Distribution (No Installation)

```powershell
# Build portable ZIP
.\scripts\package-windows.ps1 -Version 0.1.0 -CreatePortable

# Output: dist\windows\rusty-audio-portable-windows-x64-0.1.0.zip
# Contents:
#   - rusty-audio.exe
#   - README.md, LICENSE
#   - PORTABLE-README.txt (installation instructions)
```

**Features**:
- No admin rights required
- Run from USB drive
- Self-contained (no dependencies)
- ~15-25MB compressed

**System Requirements**:
- Windows 10/11 (64-bit)
- 4GB RAM minimum
- DirectX 12 or Vulkan support
- 100MB disk space

#### MSI Installer (Windows Installer)

```powershell
# Install WiX Toolset 3.11+
# Download: https://wixtoolset.org/releases/

# Build MSI
.\scripts\package-windows.ps1 -Version 0.1.0 -CreateMSI -CreatePortable

# Output: dist\windows\rusty-audio-setup-0.1.0.msi
```

**Features**:
- Start Menu shortcuts
- Desktop shortcut
- Proper uninstaller
- Windows integration

**Installation**:
```powershell
# Silent install
msiexec /i rusty-audio-setup-0.1.0.msi /quiet

# With logging
msiexec /i rusty-audio-setup-0.1.0.msi /l*v install.log
```

### Linux

#### AppImage (Universal Linux Binary)

```bash
# Build AppImage
VERSION=0.1.0 ./scripts/package-linux.sh

# Output: dist/linux/RustyAudio-0.1.0-x86_64.AppImage

# Install appimagetool (if not present)
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x appimagetool-x86_64.AppImage
sudo mv appimagetool-x86_64.AppImage /usr/local/bin/appimagetool
```

**Usage**:
```bash
# Make executable
chmod +x RustyAudio-0.1.0-x86_64.AppImage

# Run
./RustyAudio-0.1.0-x86_64.AppImage

# Integrate with desktop (optional)
./RustyAudio-0.1.0-x86_64.AppImage --appimage-install
```

#### Debian/Ubuntu (.deb)

```bash
# Build .deb package
VERSION=0.1.0 ./scripts/package-linux.sh

# Output: dist/linux/rusty-audio_0.1.0_amd64.deb

# Install
sudo dpkg -i rusty-audio_0.1.0_amd64.deb
sudo apt-get install -f  # Fix dependencies

# Or use gdebi for automatic dependency resolution
sudo gdebi rusty-audio_0.1.0_amd64.deb
```

#### Fedora/RHEL (.rpm)

```bash
# Build .rpm package
VERSION=0.1.0 ./scripts/package-linux.sh

# Output: dist/linux/rusty-audio-0.1.0-1.x86_64.rpm

# Install (Fedora)
sudo dnf install rusty-audio-0.1.0-1.x86_64.rpm

# Install (RHEL/CentOS)
sudo yum install rusty-audio-0.1.0-1.x86_64.rpm
```

#### Tarball (Manual Installation)

```bash
# Build tarball
VERSION=0.1.0 ./scripts/package-linux.sh

# Output: dist/linux/rusty-audio-0.1.0-linux-x86_64.tar.gz

# Install manually
tar -xzf rusty-audio-0.1.0-linux-x86_64.tar.gz
sudo cp rusty-audio /usr/local/bin/
```

### macOS

#### Universal Binary (Intel + Apple Silicon)

```bash
# Build universal binary with DMG
VERSION=0.1.0 ./scripts/package-macos.sh

# Outputs:
#   - dist/macos/rusty-audio (universal binary)
#   - dist/macos/Rusty Audio.app (app bundle)
#   - dist/macos/RustyAudio-0.1.0-macOS-universal.dmg
```

**Installation from DMG**:
1. Open `RustyAudio-0.1.0-macOS-universal.dmg`
2. Drag "Rusty Audio.app" to Applications folder
3. Eject DMG
4. Launch from Applications or Spotlight

**First Launch (Gatekeeper)**:
```bash
# If "App is damaged" warning appears:
# Method 1: System Preferences
# 1. Open System Preferences → Security & Privacy
# 2. Click "Open Anyway"

# Method 2: Remove quarantine attribute
xattr -d com.apple.quarantine "/Applications/Rusty Audio.app"

# Method 3: Temporarily disable Gatekeeper (not recommended)
sudo spctl --master-disable
```

#### Code Signing (For Distribution)

```bash
# Check signing identities
security find-identity -v -p codesigning

# Sign app bundle
codesign --force --deep \
    --sign "Developer ID Application: Your Name (TEAMID)" \
    "Rusty Audio.app"

# Verify signature
codesign --verify --deep --strict "Rusty Audio.app"
spctl -a -vv "Rusty Audio.app"
```

#### Notarization (Required for macOS 10.15+)

```bash
# 1. Create app-specific password at appleid.apple.com

# 2. Store in keychain
xcrun notarytool store-credentials "AC_PASSWORD" \
    --apple-id "your@email.com" \
    --team-id "TEAMID" \
    --password "app-specific-password"

# 3. Create signed zip/dmg
ditto -c -k --keepParent "Rusty Audio.app" RustyAudio.zip

# 4. Submit for notarization
xcrun notarytool submit RustyAudio.zip \
    --keychain-profile "AC_PASSWORD" \
    --wait

# 5. Staple notarization ticket
xcrun stapler staple "Rusty Audio.app"

# 6. Verify
spctl -a -vv "Rusty Audio.app"
```

---

## WASM/PWA Deployment

### Build Process

```bash
# Configure environment
export VERSION=0.1.0
export ENVIRONMENT=production  # or staging, development
export CDN_PROVIDER=github-pages  # or cloudflare, netlify

# Run deployment script
chmod +x scripts/deploy-pwa-cdn.sh
./scripts/deploy-pwa-cdn.sh
```

**Build Steps**:
1. ✅ Check prerequisites (rustc, wasm-pack, wasm-opt)
2. ✅ Clean previous builds
3. ✅ Build WASM with wasm-pack
4. ✅ Optimize with wasm-opt (Oz level)
5. ✅ Copy static assets (manifest, service worker, icons)
6. ✅ Generate build manifest
7. ✅ Create gzip/brotli compressed versions
8. ✅ Configure CDN-specific headers
9. ✅ Generate deployment report

**Output**: `dist/pwa/` directory ready for deployment

### GitHub Pages (Free CDN)

#### Automatic Deployment

```yaml
# Triggered automatically on:
# - Push to main branch
# - Git tags (v*.*.*)
# - Manual workflow dispatch

# Workflow: .github/workflows/production-deploy.yml
```

**Manual Deployment**:
```bash
# Build PWA
./scripts/deploy-pwa-cdn.sh

# Deploy to gh-pages branch
cd dist/pwa
git init
git add .
git commit -m "Deploy v0.1.0"
git branch -M gh-pages
git remote add origin https://github.com/<username>/rusty-audio.git
git push -u origin gh-pages --force
```

**Configuration**:
1. Go to **Settings → Pages**
2. Source: **Deploy from a branch**
3. Branch: **gh-pages** / **(root)**
4. Save

**Custom Domain** (optional):
```bash
# Add CNAME file
echo "rusty-audio.example.com" > dist/pwa/CNAME

# Configure DNS
# Type: CNAME
# Name: rusty-audio
# Value: <username>.github.io
```

**Access**: https://[username].github.io/rusty-audio

### Cloudflare Pages

```bash
# Install Wrangler CLI
npm install -g wrangler

# Login
wrangler login

# Build PWA
CDN_PROVIDER=cloudflare ./scripts/deploy-pwa-cdn.sh

# Deploy
cd dist/pwa
wrangler pages publish . --project-name rusty-audio
```

**Configuration** (wrangler.toml):
```toml
name = "rusty-audio"
pages_build_output_dir = "dist/pwa"

[env.production]
compatibility_date = "2024-01-01"
```

**Features**:
- ✅ Automatic HTTP/3
- ✅ Global CDN (200+ locations)
- ✅ Unlimited bandwidth (Free plan)
- ✅ Free SSL certificates
- ✅ Web Analytics

**Access**: https://rusty-audio.pages.dev

### Netlify

```bash
# Install Netlify CLI
npm install -g netlify-cli

# Login
netlify login

# Build PWA
CDN_PROVIDER=netlify ./scripts/deploy-pwa-cdn.sh

# Deploy
cd dist/pwa
netlify deploy --prod --dir .
```

**Configuration** (netlify.toml):
```toml
[build]
  command = "./scripts/deploy-pwa-cdn.sh"
  publish = "dist/pwa"

[[headers]]
  for = "/*"
  [headers.values]
    X-Frame-Options = "DENY"
    X-Content-Type-Options = "nosniff"

[[headers]]
  for = "/*.wasm"
  [headers.values]
    Content-Type = "application/wasm"
    Cache-Control = "public, max-age=31536000, immutable"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200
```

**Features**:
- ✅ Automatic HTTPS
- ✅ Continuous deployment
- ✅ Branch previews
- ✅ Form handling
- ✅ Serverless functions

**Access**: https://rusty-audio.netlify.app

### PWA Features Verification

```bash
# Check PWA setup
./scripts/verify-pwa-setup.sh

# Lighthouse audit (requires Chrome/Node.js)
npm install -g lighthouse
lighthouse https://[your-domain] --view
```

**PWA Checklist**:
- ✅ HTTPS enabled
- ✅ manifest.webmanifest present
- ✅ Service worker registered
- ✅ Icons (192x192, 512x512)
- ✅ Installable
- ✅ Offline capable
- ✅ Fast load time (<3s)

---

## Docker Deployment

### Available Images

```bash
# Build all targets
docker build --target desktop-linux -t rusty-audio:linux .
docker build --target wasm-server -t rusty-audio:wasm .
docker build --target testing -t rusty-audio:test .
docker build --target development -t rusty-audio:dev .
docker build --target cross-windows -t rusty-audio:windows .
```

### Docker Compose Services

```bash
# Start WASM server (http://localhost:8080)
docker-compose up wasm

# Run tests
docker-compose up test

# Development environment
docker-compose up dev

# Build Linux binary
docker-compose up build-linux

# Cross-compile for Windows
docker-compose up build-windows

# Run benchmarks
docker-compose up benchmark

# Security audit
docker-compose up security
```

### Production Deployment

```bash
# Build production image
docker build --target wasm-server -t rusty-audio:0.1.0 .

# Run container
docker run -d -p 8080:80 --name rusty-audio rusty-audio:0.1.0

# Check health
docker exec rusty-audio curl -f http://localhost/health || exit 1

# View logs
docker logs -f rusty-audio
```

---

## CI/CD Pipeline

### Production Deployment Workflow

**File**: `.github/workflows/production-deploy.yml`

**Triggers**:
- Push to `main` branch
- Git tags matching `v*.*.*`
- Manual workflow dispatch

**Jobs**:
1. **Preflight**: Version determination, Cargo.toml validation, security audit
2. **Quality**: Format check, Clippy, security scan
3. **Test**: Multi-platform (Ubuntu, Windows, macOS) × Rust (stable)
4. **Build Desktop**: Windows x64, Linux x64, macOS Universal
5. **Build WASM**: Production-optimized WASM + PWA assets
6. **Deploy GitHub Pages**: Automatic deployment (main branch only)
7. **Create Release**: GitHub Release with all artifacts (tags only)
8. **Notify**: Deployment status summary

### Release Workflow

**File**: `.github/workflows/release.yml`

**Triggers**:
- Git tags matching `v*.*.*`

**Jobs**:
1. **Prepare**: Extract version from tag, generate changelog
2. **Build**: All platforms (Windows x64, Linux x64, macOS x64, macOS ARM64, WASM)
3. **Release**: Create GitHub Release with:
   - Changelog
   - Desktop binaries (4 platforms)
   - WASM/PWA archive
   - SHA256 checksums
4. **Notify**: Post-release notifications

### Manual Workflow Triggering

```bash
# Via GitHub CLI
gh workflow run production-deploy.yml --ref main

# With inputs
gh workflow run production-deploy.yml \
    -f deploy_desktop=true \
    -f deploy_wasm=true \
    -f version=0.2.0

# List workflow runs
gh run list --workflow=production-deploy.yml

# View specific run
gh run view 1234567890
```

**Web Interface**:
1. Navigate to **Actions** tab
2. Select workflow (e.g., "Production Deployment")
3. Click **Run workflow**
4. Select branch and options
5. Click **Run workflow**

---

## Monitoring & Observability

### Setup

```bash
# Run monitoring setup script
chmod +x scripts/setup-monitoring.sh
./scripts/setup-monitoring.sh
```

**Created Files**:
- `monitoring-config.toml`: Metrics configuration
- `src/monitoring.rs`: Monitoring Rust module
- `analytics.json`: Privacy-preserving analytics
- `health-check.sh`: Health check script
- `export-metrics.sh`: Metrics export utility

### Integration

```rust
// Add to Cargo.toml
// (monitoring.rs module code provided by setup script)

// In main.rs:
mod monitoring;
use monitoring::MetricsCollector;

let metrics = MetricsCollector::new();

// Update metrics
metrics.update_audio_metrics(latency_ms, cpu_usage);
metrics.update_ui_metrics(frame_time);

// Export snapshot
let snapshot = metrics.snapshot();
println!("{:?}", snapshot);
```

### Health Checks

```bash
# Run health check
./health-check.sh

# Output:
# ✅ Process running
# ✅ HTTP endpoint healthy
# ✅ Memory OK: 2048MB available
# ✅ CPU load: 1.5
```

### Sentry Integration (Optional)

```bash
# Get Sentry DSN from sentry.io
export SENTRY_DSN="https://key@o123456.ingest.sentry.io/123456"

# Update monitoring-config.toml
[error_tracking]
sentry_dsn = "https://key@o123456.ingest.sentry.io/123456"
environment = "production"
sample_rate = 1.0
```

### Metrics Export

```bash
# Export current metrics to JSON
./export-metrics.sh

# Output: metrics-20240116-143022.json
# {
#   "timestamp": "2024-01-16T14:30:22Z",
#   "metrics": {
#     "audio": {...},
#     "ui": {...},
#     "memory": {...}
#   }
# }
```

---

## Release Process

### Semantic Versioning

Format: `MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]`

- **MAJOR**: Breaking changes (0.x.y → 1.0.0)
- **MINOR**: New features, backward compatible (1.0.0 → 1.1.0)
- **PATCH**: Bug fixes, backward compatible (1.1.0 → 1.1.1)
- **PRERELEASE**: Alpha, beta, rc (1.0.0-beta.1)
- **BUILD**: Build metadata (1.0.0+20240116)

### Automated Release

```bash
# Make script executable
chmod +x scripts/release.sh

# Patch release (0.1.0 → 0.1.1)
./scripts/release.sh patch

# Minor release (0.1.0 → 0.2.0)
./scripts/release.sh minor

# Major release (0.1.0 → 1.0.0)
./scripts/release.sh major

# Specific version
./scripts/release.sh 1.0.0-beta.1
```

**Automated Steps**:
1. ✅ Pre-release checks (branch, uncommitted changes, remote sync)
2. ✅ Update version in Cargo.toml
3. ✅ Update Cargo.lock
4. ✅ Run full test suite
5. ✅ Run Clippy checks
6. ✅ Generate CHANGELOG.md
7. ✅ Build release artifacts (desktop + WASM)
8. ✅ Commit version changes
9. ✅ Create git tag
10. ✅ Push changes and tag (with confirmation)

**Post-Release** (automatic via GitHub Actions):
1. ✅ Multi-platform builds
2. ✅ GitHub Release creation
3. ✅ PWA deployment to GitHub Pages
4. ✅ Artifacts uploaded

### Release Checklist

**Pre-Release**:
- [ ] All tests pass: `cargo test --all-features`
- [ ] Clippy clean: `cargo clippy -- -D warnings`
- [ ] Security audit: `cargo audit`
- [ ] Documentation updated
- [ ] CHANGELOG.md reviewed
- [ ] Breaking changes documented

**Release**:
- [ ] Run `./scripts/release.sh <type>`
- [ ] Verify git tag created
- [ ] Push tag to GitHub
- [ ] Monitor GitHub Actions workflows
- [ ] Verify builds complete successfully

**Post-Release**:
- [ ] GitHub Release published
- [ ] WASM deployed to GitHub Pages
- [ ] Test downloads work
- [ ] Verify checksums
- [ ] Update documentation website
- [ ] Announce release (social media, forums)

### Manual Release (Fallback)

```bash
# 1. Update version
nano Cargo.toml  # Change version = "0.2.0"
cargo check      # Update Cargo.lock

# 2. Update CHANGELOG.md
cat >> CHANGELOG.md <<EOF
## [0.2.0] - $(date +%Y-%m-%d)
### Added
- New feature X
### Fixed
- Bug Y
EOF

# 3. Commit and tag
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: release v0.2.0"
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin main --tags

# 4. GitHub Actions will handle the rest
```

---

## Rollback Procedures

### Rolling Back a Release

#### Option 1: Revert Git Tag (Recommended)

```bash
# 1. Delete remote tag
git push --delete origin v0.2.0

# 2. Delete local tag
git tag -d v0.2.0

# 3. Delete GitHub Release (via web UI or CLI)
gh release delete v0.2.0 --yes

# 4. Optionally revert commit
git revert <commit-hash>
git push origin main
```

#### Option 2: Create Hotfix Release

```bash
# 1. Create hotfix branch from last stable tag
git checkout -b hotfix/0.1.1 v0.1.0

# 2. Apply fixes
# ... make changes ...

# 3. Release hotfix
./scripts/release.sh 0.1.1

# 4. Merge back to main
git checkout main
git merge hotfix/0.1.1
git push origin main
```

### Rolling Back PWA Deployment

#### GitHub Pages

```bash
# 1. Find previous working commit in gh-pages branch
git checkout gh-pages
git log --oneline

# 2. Reset to previous commit
git reset --hard <previous-commit-hash>
git push origin gh-pages --force

# Or use GitHub Actions to redeploy previous version
gh workflow run deploy-pwa.yml --ref <previous-tag>
```

#### Cloudflare Pages

```bash
# 1. List deployments
wrangler pages deployment list --project-name rusty-audio

# 2. Promote previous deployment
wrangler pages deployment promote <deployment-id> --project-name rusty-audio
```

#### Netlify

```bash
# 1. List deployments
netlify deploy:list

# 2. Restore previous deployment
netlify deploy:restore <deploy-id>
```

---

## Troubleshooting

### Desktop Build Issues

#### Windows: Missing ASIO SDK

**Error**: `CPAL_ASIO_DIR environment variable not set`

**Solution**:
```toml
# .cargo/config.toml
[env]
CPAL_ASIO_DIR = "C:\\path\\to\\asio_sdk"
```

#### Linux: ALSA Development Files Missing

**Error**: `Package alsa was not found in the pkg-config search path`

**Solution**:
```bash
sudo apt-get update
sudo apt-get install -y libasound2-dev pkg-config
```

#### macOS: Rust Target Missing

**Error**: `error[E0463]: can't find crate for 'core'`

**Solution**:
```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
```

### WASM Build Issues

#### wasm-pack Not Found

**Solution**:
```bash
cargo install wasm-pack --locked
```

#### wasm-opt Optimization Fails

**Solution**:
```bash
# Ubuntu/Debian
sudo apt-get install binaryen

# macOS
brew install binaryen

# Windows
choco install binaryen
```

#### WASM Binary Too Large

**Solutions**:
1. Enable `opt-level = "z"` in Cargo.toml
2. Use `wasm-opt -Oz`
3. Disable default features: `--no-default-features`
4. Strip debug symbols
5. Use `lto = "fat"` in release profile

### Deployment Issues

#### GitHub Pages Returns 404

**Solutions**:
1. Check Settings → Pages → Source is correct
2. Verify `gh-pages` branch exists
3. Ensure `index.html` is at root
4. Wait 5-10 minutes for propagation
5. Check GitHub Actions logs

#### WASM Fails to Load in Browser

**Solutions**:
1. Verify MIME type: `application/wasm` in headers
2. Check CORS headers
3. Inspect browser console for errors
4. Test locally first: `python3 -m http.server`
5. Verify CSP allows `'wasm-unsafe-eval'`

#### Service Worker Not Registering

**Solutions**:
1. HTTPS required (except localhost)
2. Check `service-worker.js` path
3. Verify scope matches app location
4. Clear browser cache
5. Check DevTools → Application → Service Workers

### Performance Issues

#### High Audio Latency

**Solutions**:
- Use ASIO drivers (Windows)
- Reduce buffer size in settings
- Close other audio applications
- Check CPU usage (<80%)
- Disable spectrum analyzer temporarily

#### Low Frame Rate

**Solutions**:
- Update GPU drivers
- Reduce spectrum analyzer resolution
- Check GPU usage in Task Manager
- Disable anti-aliasing
- Close other GPU-intensive apps

### Docker Issues

#### No Space Left on Device

**Solution**:
```bash
# Clean up Docker
docker system prune -a --volumes

# Check disk usage
docker system df

# Increase Docker disk limit (Docker Desktop)
# Settings → Resources → Disk image size
```

#### Build Fails with Network Timeout

**Solution**:
```bash
# Increase Docker build timeout
docker build --network=host --build-arg BUILDKIT_PROGRESS=plain .

# Use proxy if behind firewall
docker build --build-arg http_proxy=$http_proxy --build-arg https_proxy=$https_proxy .
```

---

## Support & Contact

- **GitHub Issues**: https://github.com/david-t-martel/rusty-audio/issues
- **Discussions**: https://github.com/david-t-martel/rusty-audio/discussions
- **Wiki**: https://github.com/david-t-martel/rusty-audio/wiki
- **Security**: security@example.com (for vulnerabilities only)

## Additional Resources

- **PWA Deployment**: See `DEPLOYMENT.md` for detailed PWA instructions
- **Performance Guide**: `PERFORMANCE_GUIDE.md`
- **User Manual**: `USER_MANUAL.md`
- **Contributing**: `CONTRIBUTING.md`
- **Architecture**: `ARCHITECTURE_SUMMARY.md`

---

**Last Updated**: January 16, 2025
**Version**: 1.0.0
**Maintained By**: Rusty Audio Team
