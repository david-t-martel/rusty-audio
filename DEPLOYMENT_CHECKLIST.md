# Rusty Audio - Deployment Checklist

Quick reference checklist for deploying Rusty Audio to production.

## Pre-Deployment Verification

- [ ] All tests pass: `cargo test --all-features --all-targets`
- [ ] Clippy checks pass: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Security audit clean: `cargo audit`
- [ ] Code formatted: `cargo fmt --all -- --check`
- [ ] Documentation up to date
- [ ] CHANGELOG.md updated
- [ ] Version number correct in Cargo.toml
- [ ] Git working directory clean
- [ ] On main branch with latest changes

## Desktop Deployment

### Windows

- [ ] Run packaging script: `.\scripts\package-windows.ps1 -Version X.Y.Z -CreatePortable`
- [ ] Optional: Create MSI installer with `-CreateMSI` flag
- [ ] Test executable runs without errors
- [ ] Verify portable ZIP contains all necessary files
- [ ] Check SHA256 checksums generated
- [ ] Upload to GitHub Releases

**Expected Artifacts**:
- `dist\windows\rusty-audio.exe`
- `dist\windows\rusty-audio-portable-windows-x64-X.Y.Z.zip`
- `dist\windows\rusty-audio-setup-X.Y.Z.msi` (if created)
- `dist\windows\SHA256SUMS.txt`

### Linux

- [ ] Run packaging script: `VERSION=X.Y.Z ./scripts/package-linux.sh`
- [ ] Test tarball extraction and binary execution
- [ ] Test AppImage (if created)
- [ ] Test .deb installation (if created)
- [ ] Test .rpm installation (if created)
- [ ] Check SHA256 checksums generated
- [ ] Upload to GitHub Releases

**Expected Artifacts**:
- `dist/linux/rusty-audio`
- `dist/linux/rusty-audio-X.Y.Z-linux-x86_64.tar.gz`
- `dist/linux/RustyAudio-X.Y.Z-x86_64.AppImage` (if appimagetool available)
- `dist/linux/rusty-audio_X.Y.Z_amd64.deb` (if dpkg-deb available)
- `dist/linux/rusty-audio-X.Y.Z-1.x86_64.rpm` (if rpmbuild available)
- `dist/linux/SHA256SUMS.txt`

### macOS

- [ ] Run packaging script: `VERSION=X.Y.Z ./scripts/package-macos.sh`
- [ ] Test universal binary on Intel Mac (if available)
- [ ] Test universal binary on Apple Silicon Mac (if available)
- [ ] Verify app bundle launches
- [ ] Test DMG installation
- [ ] Optional: Code sign app bundle
- [ ] Optional: Notarize for distribution
- [ ] Check SHA256 checksums generated
- [ ] Upload to GitHub Releases

**Expected Artifacts**:
- `dist/macos/rusty-audio` (universal binary)
- `dist/macos/Rusty Audio.app` (app bundle)
- `dist/macos/RustyAudio-X.Y.Z-macOS-universal.dmg`
- `dist/macos/SHA256SUMS.txt`

## WASM/PWA Deployment

### GitHub Pages

- [ ] Build PWA: `VERSION=X.Y.Z ENVIRONMENT=production CDN_PROVIDER=github-pages ./scripts/deploy-pwa-cdn.sh`
- [ ] Verify build artifacts in `dist/pwa/`
- [ ] Check WASM size (<15MB recommended)
- [ ] Test locally: `cd dist/pwa && python3 -m http.server 8080`
- [ ] Verify manifest.webmanifest present
- [ ] Verify service-worker.js present
- [ ] Verify _headers file present
- [ ] Verify icons directory present
- [ ] Push to gh-pages branch (or use GitHub Actions)
- [ ] Wait 5-10 minutes for deployment
- [ ] Test live site: https://[username].github.io/rusty-audio
- [ ] Test PWA installation ("Add to Home Screen")
- [ ] Test offline functionality

### Cloudflare Pages (Optional)

- [ ] Install Wrangler: `npm install -g wrangler`
- [ ] Login: `wrangler login`
- [ ] Build: `CDN_PROVIDER=cloudflare ./scripts/deploy-pwa-cdn.sh`
- [ ] Deploy: `cd dist/pwa && wrangler pages publish . --project-name rusty-audio`
- [ ] Verify deployment: https://rusty-audio.pages.dev
- [ ] Optional: Configure custom domain

### Netlify (Optional)

- [ ] Install CLI: `npm install -g netlify-cli`
- [ ] Login: `netlify login`
- [ ] Build: `CDN_PROVIDER=netlify ./scripts/deploy-pwa-cdn.sh`
- [ ] Deploy: `cd dist/pwa && netlify deploy --prod --dir .`
- [ ] Verify deployment: https://rusty-audio.netlify.app
- [ ] Optional: Configure custom domain

## Docker Deployment

- [ ] Build images: `docker-compose build`
- [ ] Test WASM server: `docker-compose up wasm`
- [ ] Verify access: http://localhost:8080
- [ ] Run health checks
- [ ] Optional: Push to Docker Hub or GitHub Container Registry
- [ ] Optional: Deploy to production container orchestration (Kubernetes, ECS, etc.)

## CI/CD Pipeline

### GitHub Actions

- [ ] Verify workflows present:
  - `.github/workflows/production-deploy.yml`
  - `.github/workflows/release.yml`
- [ ] Check workflow syntax: `gh workflow list`
- [ ] Test manual trigger: `gh workflow run production-deploy.yml`
- [ ] Monitor workflow execution
- [ ] Verify all jobs pass
- [ ] Check artifacts uploaded

### Automated Release

- [ ] Run release script: `./scripts/release.sh patch` (or `minor`, `major`)
- [ ] Review version bump
- [ ] Review CHANGELOG.md updates
- [ ] Confirm push to remote
- [ ] Monitor GitHub Actions workflows
- [ ] Verify GitHub Release created
- [ ] Verify all artifacts attached to release
- [ ] Verify checksums included

## Monitoring Setup

- [ ] Run monitoring setup: `./scripts/setup-monitoring.sh`
- [ ] Review `monitoring-config.toml`
- [ ] Integrate `src/monitoring.rs` into application
- [ ] Optional: Configure Sentry DSN
- [ ] Test health check: `./health-check.sh`
- [ ] Test metrics export: `./export-metrics.sh`
- [ ] Verify logging configuration

## Post-Deployment Verification

### Desktop Applications

- [ ] Download release artifacts from GitHub
- [ ] Verify checksums match
- [ ] Test Windows binary on Windows 10/11
- [ ] Test Linux binary on Ubuntu 20.04+
- [ ] Test macOS app on macOS 11+
- [ ] Verify audio playback works
- [ ] Test EQ functionality
- [ ] Test spectrum analyzer
- [ ] Test recording feature
- [ ] Test theme switching
- [ ] Check for crashes or errors
- [ ] Verify About dialog shows correct version

### WASM/PWA

- [ ] Access live URL in Chrome
- [ ] Access live URL in Firefox
- [ ] Access live URL in Safari (desktop)
- [ ] Access live URL in Edge
- [ ] Test mobile browsers (iOS Safari, Chrome Android)
- [ ] Verify PWA installs correctly
- [ ] Test offline functionality
- [ ] Check service worker caching
- [ ] Verify audio playback in browser
- [ ] Test EQ and effects
- [ ] Check performance (60 FPS minimum)
- [ ] Run Lighthouse audit (score >90 recommended)
- [ ] Verify CSP headers not blocking functionality
- [ ] Test on slow network (3G simulation)

### Docker Containers

- [ ] Pull images from registry
- [ ] Start containers
- [ ] Verify health checks pass
- [ ] Test HTTP endpoints respond
- [ ] Check logs for errors
- [ ] Monitor resource usage (CPU, memory)
- [ ] Test container restart recovery

## Communication

- [ ] Update project README.md with new version
- [ ] Update documentation website
- [ ] Create release announcement (GitHub Discussions)
- [ ] Optional: Post on social media (Twitter, Reddit, etc.)
- [ ] Optional: Update project website
- [ ] Optional: Send newsletter to subscribers
- [ ] Optional: Submit to relevant directories (Product Hunt, etc.)

## Rollback Plan

If issues are discovered post-deployment:

### Desktop

- [ ] Create hotfix branch from last stable tag
- [ ] Apply fixes
- [ ] Release hotfix version (X.Y.Z+1)
- [ ] Upload new binaries
- [ ] Update GitHub Release notes

### WASM/PWA

- [ ] Revert gh-pages branch to previous commit
- [ ] Or redeploy previous working version
- [ ] Clear CDN cache if using custom CDN
- [ ] Verify rollback successful

### Docker

- [ ] Deploy previous image tag
- [ ] Verify health checks pass
- [ ] Monitor for errors

## Security

- [ ] Verify no secrets committed to repository
- [ ] Check .gitignore includes sensitive files
- [ ] Verify API keys not exposed in client-side code
- [ ] Review CSP headers for WASM deployment
- [ ] Check CORS configuration
- [ ] Verify HTTPS enforced (for PWA)
- [ ] Review dependencies for vulnerabilities: `cargo audit`

## Performance

- [ ] Benchmark audio latency (<20ms target)
- [ ] Check UI frame rate (>60 FPS target)
- [ ] Measure startup time (<5s desktop, <10s WASM)
- [ ] Profile memory usage (<500MB desktop, <250MB WASM)
- [ ] Test with large audio files (>100MB)
- [ ] Verify CPU usage reasonable (<30% idle, <80% active)

## Final Sign-Off

- [ ] All critical features working
- [ ] No blockers or critical bugs
- [ ] Performance meets targets
- [ ] Security review passed
- [ ] Documentation complete
- [ ] Team approval obtained
- [ ] User acceptance testing completed (if applicable)

---

## Quick Commands Reference

```bash
# Desktop Builds
.\scripts\package-windows.ps1 -Version 0.1.0 -CreatePortable
VERSION=0.1.0 ./scripts/package-linux.sh
VERSION=0.1.0 ./scripts/package-macos.sh

# WASM/PWA
VERSION=0.1.0 ENVIRONMENT=production ./scripts/deploy-pwa-cdn.sh

# Docker
docker-compose up wasm
docker-compose up test

# Release
./scripts/release.sh patch

# Monitoring
./scripts/setup-monitoring.sh
./health-check.sh
./export-metrics.sh

# GitHub Actions
gh workflow run production-deploy.yml
gh workflow run release.yml

# Testing
cargo test --all-features --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo audit
```

---

**Last Updated**: January 16, 2025
