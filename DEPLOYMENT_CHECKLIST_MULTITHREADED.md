# Multithreaded WASM Deployment Checklist

Use this checklist to ensure successful deployment of Rusty Audio with SharedArrayBuffer support.

## Pre-Build Checklist

- [ ] Rust toolchain updated to latest stable
  ```bash
  rustup update stable
  ```

- [ ] WASM target installed
  ```bash
  rustup target add wasm32-unknown-unknown
  ```

- [ ] Trunk installed and updated
  ```bash
  cargo install trunk --locked
  ```

- [ ] wasm-bindgen-cli installed (matching Cargo.lock version)
  ```bash
  cargo install wasm-bindgen-cli --force
  ```

- [ ] wasm-opt available (binaryen)
  - Windows: Download from GitHub releases
  - Linux: `apt install binaryen`
  - macOS: `brew install binaryen`

## Configuration Verification

- [ ] `.cargo/config.toml` has WASM threading flags:
  - `+atomics`
  - `+bulk-memory`
  - `+mutable-globals`
  - `--shared-memory`

- [ ] `Trunk.toml` has wasm-opt configuration:
  - `--enable-threads`
  - `--enable-bulk-memory`
  - `--enable-mutable-globals`

- [ ] `static/service-worker.js` includes COOP/COEP headers:
  - `Cross-Origin-Opener-Policy: same-origin`
  - `Cross-Origin-Embedder-Policy: require-corp`

- [ ] `static/_headers` file exists with proper headers

- [ ] Deployment config exists:
  - [ ] `netlify.toml` (for Netlify)
  - [ ] `vercel.json` (for Vercel)
  - [ ] Or custom server config (nginx/apache)

## Build Process

- [ ] Clean previous builds
  ```bash
  trunk clean
  cargo clean
  ```

- [ ] Run production build
  ```bash
  trunk build --release
  ```

- [ ] Verify build output in `dist/` directory:
  - [ ] `index.html` exists
  - [ ] `rusty-audio.js` exists
  - [ ] `rusty-audio_bg.wasm` exists
  - [ ] `service-worker.js` exists
  - [ ] `manifest.webmanifest` exists
  - [ ] `_headers` file exists
  - [ ] `icons/` directory exists

- [ ] Check WASM file for threading features
  ```bash
  wasm-objdump -x dist/rusty-audio_bg.wasm | grep -i "shared"
  ```
  Expected: Should see "shared memory" or atomics features

## Local Testing

- [ ] Test with local server (choose one):

  **Option A: Trunk Serve**
  ```bash
  trunk serve --release
  ```

  **Option B: Python Server with Headers**
  ```bash
  cd dist
  python3 -m http.server 8080 \
    --bind 127.0.0.1
  # Note: Relies on service worker for headers
  ```

- [ ] Open browser and navigate to local server

- [ ] Open DevTools Console and verify:
  ```javascript
  // 1. SharedArrayBuffer available
  console.log('SharedArrayBuffer:', typeof SharedArrayBuffer !== 'undefined');
  // Expected: true

  // 2. Cross-origin isolated
  console.log('crossOriginIsolated:', crossOriginIsolated);
  // Expected: true

  // 3. Service worker registered
  navigator.serviceWorker.getRegistration().then(reg => {
      console.log('Service Worker:', !!reg);
  });
  // Expected: true
  ```

- [ ] Check Network tab in DevTools:
  - [ ] `index.html` has COOP/COEP headers
  - [ ] `rusty-audio_bg.wasm` has correct Content-Type
  - [ ] WASM file loads successfully

- [ ] Test application functionality:
  - [ ] UI renders correctly
  - [ ] Audio playback works
  - [ ] Spectrum analyzer displays
  - [ ] EQ controls functional
  - [ ] No console errors

## Deployment Configuration

### For Netlify

- [ ] `netlify.toml` exists in project root
- [ ] Deploy via CLI:
  ```bash
  netlify deploy --prod
  ```
- [ ] Or connect Git repository in Netlify UI

### For Vercel

- [ ] `vercel.json` exists in project root
- [ ] Deploy via CLI:
  ```bash
  vercel --prod
  ```
- [ ] Or connect Git repository in Vercel UI

### For Cloudflare Pages

- [ ] `static/_headers` file exists
- [ ] Deploy via Wrangler:
  ```bash
  wrangler pages publish dist --project-name rusty-audio
  ```
- [ ] Or connect Git repository in Cloudflare UI

### For Self-Hosted (Nginx/Apache)

- [ ] Server configuration updated with COOP/COEP headers
- [ ] HTTPS enabled (required for service workers)
- [ ] Upload `dist/` contents to web root
- [ ] Restart web server

## Post-Deployment Verification

- [ ] Access deployed URL in browser

- [ ] Check HTTP headers:
  ```bash
  curl -I https://your-deployment-url.com
  ```
  Verify:
  - [ ] `Cross-Origin-Opener-Policy: same-origin`
  - [ ] `Cross-Origin-Embedder-Policy: require-corp`

- [ ] Open browser DevTools Console:
  ```javascript
  // Verify SharedArrayBuffer
  console.log(typeof SharedArrayBuffer !== 'undefined');
  // Expected: true

  // Verify cross-origin isolation
  console.log(crossOriginIsolated);
  // Expected: true
  ```

- [ ] Check Network tab for WASM file:
  - [ ] Status: 200 OK
  - [ ] Content-Type: `application/wasm`
  - [ ] COOP/COEP headers present

- [ ] Test application features:
  - [ ] Page loads without errors
  - [ ] UI is responsive
  - [ ] Audio playback works
  - [ ] File loading works
  - [ ] All panels functional (Playback, Effects, EQ, etc.)

## Performance Verification

- [ ] Run Lighthouse audit:
  ```bash
  lighthouse https://your-deployment-url.com \
    --output html \
    --output-path ./lighthouse-report.html
  ```

- [ ] Check PWA score (target: 90+)

- [ ] Verify load times:
  - [ ] First Contentful Paint < 2s
  - [ ] Time to Interactive < 3s
  - [ ] Total bundle size < 5MB

- [ ] Check WASM bundle size:
  ```bash
  ls -lh dist/rusty-audio_bg.wasm
  ```

- [ ] Memory usage in DevTools:
  - Open Performance Monitor
  - Check WASM memory usage
  - Monitor for memory leaks

## Security Verification

- [ ] All security headers present:
  - [ ] `Cross-Origin-Opener-Policy: same-origin`
  - [ ] `Cross-Origin-Embedder-Policy: require-corp`
  - [ ] `X-Content-Type-Options: nosniff`
  - [ ] `X-Frame-Options: DENY`
  - [ ] `Referrer-Policy: no-referrer`

- [ ] Content Security Policy (CSP) configured:
  - [ ] `script-src` allows 'wasm-unsafe-eval'
  - [ ] `worker-src` allows 'self'

- [ ] HTTPS enabled and enforced

- [ ] Service worker served over HTTPS

## Browser Compatibility Testing

Test in multiple browsers to ensure compatibility:

- [ ] Chrome/Chromium (latest):
  - [ ] SharedArrayBuffer available
  - [ ] Application works
  - [ ] No console errors

- [ ] Firefox (latest):
  - [ ] SharedArrayBuffer available
  - [ ] Application works
  - [ ] No console errors

- [ ] Safari (latest macOS):
  - [ ] SharedArrayBuffer available (Safari 15.2+)
  - [ ] Application works
  - [ ] No console errors

- [ ] Edge (latest):
  - [ ] SharedArrayBuffer available
  - [ ] Application works
  - [ ] No console errors

- [ ] Mobile browsers:
  - [ ] Chrome Mobile (Android)
  - [ ] Safari (iOS 15.2+)
  - [ ] Test PWA installation

## Troubleshooting Checklist

If SharedArrayBuffer is undefined:

- [ ] Verify COOP/COEP headers in Network tab
- [ ] Check `crossOriginIsolated` in console
- [ ] Force service worker update:
  ```javascript
  navigator.serviceWorker.getRegistrations().then(regs => {
      regs.forEach(reg => reg.unregister());
  });
  ```
- [ ] Hard reload: Ctrl+Shift+R (Cmd+Shift+R on Mac)
- [ ] Clear site data in DevTools

If WASM fails to load:

- [ ] Check Content-Type is `application/wasm`
- [ ] Verify WASM file downloaded completely
- [ ] Check console for specific error messages
- [ ] Rebuild from scratch:
  ```bash
  trunk clean && cargo clean && trunk build --release
  ```

If performance is poor:

- [ ] Profile with Chrome DevTools Performance tab
- [ ] Check WASM bundle size (should be 2-5 MB)
- [ ] Verify wasm-opt ran (check build output)
- [ ] Consider reducing thread pool size in Rust code

## Rollback Plan

If deployment fails:

- [ ] Revert to previous deployment:
  - Netlify: Use UI to rollback
  - Vercel: `vercel rollback`
  - Cloudflare: Use UI to rollback
  - Self-hosted: Restore previous `dist/` backup

- [ ] Notify users if service is disrupted

- [ ] Document issues encountered

- [ ] Create GitHub issue with error details

## Documentation Updates

- [ ] Update README.md with deployment instructions

- [ ] Add deployment URL to documentation

- [ ] Update changelog with deployment notes

- [ ] Tag release in Git:
  ```bash
  git tag -a v0.1.0 -m "Multithreaded WASM deployment"
  git push origin v0.1.0
  ```

## Monitoring Setup (Optional but Recommended)

- [ ] Set up error tracking (Sentry, etc.)

- [ ] Configure uptime monitoring

- [ ] Set up performance monitoring

- [ ] Create alerts for:
  - [ ] High error rates
  - [ ] Slow load times
  - [ ] Deployment failures

## Final Checklist

- [ ] All configuration files committed to Git
- [ ] Deployment is live and accessible
- [ ] SharedArrayBuffer is available in all tested browsers
- [ ] No console errors or warnings
- [ ] Performance metrics meet targets
- [ ] Security headers verified
- [ ] Documentation updated
- [ ] Team notified of deployment

---

**Deployment Date**: _____________

**Deployed By**: _____________

**Deployment URL**: _____________

**Notes**:
_____________________________________________
_____________________________________________
_____________________________________________

**Issues Encountered**:
_____________________________________________
_____________________________________________
_____________________________________________

**Resolutions**:
_____________________________________________
_____________________________________________
_____________________________________________
