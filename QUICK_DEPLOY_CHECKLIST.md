# Quick Deployment Checklist

## Pre-Deployment

- [ ] **Build the app**
  ```bash
  trunk build --release
  ```

- [ ] **Verify build output**
  ```bash
  ls -lh dist/
  # Check for: index.html, *.wasm, *.js, service-worker.js, _headers
  ```

- [ ] **Test locally first**
  ```bash
  trunk serve --open
  # Open: http://127.0.0.1:8080
  # Verify app loads without errors
  ```

## Deployment (Choose One)

### Cloudflare Pages (Recommended)

```bash
# Option 1: Wrangler CLI
wrangler pages deploy dist --project-name rusty-audio

# Option 2: GitHub Integration
git push origin main
# Cloudflare will auto-deploy from main branch
```

**Post-Deploy:**
```bash
# Verify headers
curl -I https://rusty-audio.pages.dev/ | grep -i cross-origin
```

### Netlify

```bash
netlify deploy --prod --dir=dist
```

**Post-Deploy:**
```bash
curl -I https://your-app.netlify.app/ | grep -i cross-origin
```

### Vercel

```bash
vercel --prod
```

**Note:** Add `vercel.json` with headers (see WASM_THREADING_SETUP.md)

## Post-Deployment Verification

### 1. Test Headers
```bash
curl -I https://your-domain.com/ | grep -i cross-origin

# Expected output:
# Cross-Origin-Opener-Policy: same-origin
# Cross-Origin-Embedder-Policy: require-corp
# Cross-Origin-Resource-Policy: cross-origin
```

### 2. Open Test Page
```
https://your-domain.com/test-threading.html
```

**Expected Results:**
- ✅ WebAssembly: Supported
- ✅ SharedArrayBuffer: Available
- ✅ Cross-Origin Isolated: Yes
- ✅ Service Worker API: Supported
- ✅ Web Audio API: Supported
- ✅ WebGL: Supported

### 3. Browser Console Check
```javascript
// Open DevTools (F12), paste in console:
console.log({
  isolated: window.crossOriginIsolated,
  sab: typeof SharedArrayBuffer !== 'undefined',
  wasm: typeof WebAssembly !== 'undefined',
  threads: navigator.hardwareConcurrency
});

// Expected output:
// { isolated: true, sab: true, wasm: true, threads: 8 }
```

### 4. Service Worker Check
```javascript
navigator.serviceWorker.getRegistration().then(reg => {
  console.log('SW State:', reg?.active?.state);
  console.log('SW Scope:', reg?.scope);
});

// Expected output:
// SW State: activated
// SW Scope: https://your-domain.com/
```

### 5. Performance Check
```javascript
// Check WASM load time
performance.getEntriesByType('navigation')[0].domContentLoadedEventEnd

// Should be < 2000ms on decent connection
```

## Browser Testing Matrix

Test on these browsers (minimum):

- [ ] **Chrome 92+** (Desktop)
  - Open: https://your-domain.com/
  - Check: Console shows no errors
  - Verify: Audio playback works

- [ ] **Firefox 79+** (Desktop)
  - Same checks as Chrome

- [ ] **Safari 15.2+** (macOS/iOS)
  - Same checks as Chrome
  - Note: May need user interaction for audio

- [ ] **Mobile Chrome** (Android)
  - Test on actual device if possible
  - Check: Touch controls work
  - Verify: Audio playback works

- [ ] **Mobile Safari** (iOS)
  - Test on actual device if possible
  - Check: PWA install works
  - Verify: Audio requires user tap

## Troubleshooting

### Issue: SharedArrayBuffer Not Available

**Quick Fix:**
1. Hard refresh: `Ctrl+Shift+R` (Windows) or `Cmd+Shift+R` (Mac)
2. Clear service worker:
   ```javascript
   navigator.serviceWorker.getRegistrations().then(r =>
     r.forEach(reg => reg.unregister())
   );
   ```
3. Reload page

**Verify Headers:**
```bash
curl -I https://your-domain.com/ | grep -E "(COOP|COEP|CORP)"
```

### Issue: Service Worker Not Activating

**Quick Fix:**
1. Open DevTools → Application → Service Workers
2. Click "Unregister" on old service worker
3. Click "Update" to install new one
4. Hard refresh page

### Issue: WASM Load Timeout

**Quick Fix:**
1. Check network speed (DevTools → Network tab)
2. Verify WASM file size: `ls -lh dist/*.wasm`
3. If file is huge (>10MB), check build flags
4. Consider enabling CDN compression

## CDN-Specific Configuration

### Cloudflare Pages

**Automatic:** `_headers` file is recognized automatically

**Optional:** Custom domain settings
```bash
# In Cloudflare dashboard:
# Pages → Your Project → Custom domains → Add domain
```

### Netlify

**Automatic:** `_headers` file works out of the box

**Optional:** Custom headers in `netlify.toml`
```toml
[[headers]]
  for = "/*"
  [headers.values]
    Cross-Origin-Opener-Policy = "same-origin"
    Cross-Origin-Embedder-Policy = "require-corp"
```

### Vercel

**Required:** Create `vercel.json` in project root:
```json
{
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        { "key": "Cross-Origin-Opener-Policy", "value": "same-origin" },
        { "key": "Cross-Origin-Embedder-Policy", "value": "require-corp" },
        { "key": "Cross-Origin-Resource-Policy", "value": "cross-origin" }
      ]
    }
  ]
}
```

## Success Criteria

✅ **Deployment Successful When:**

1. [ ] App loads at https://your-domain.com/
2. [ ] No errors in browser console
3. [ ] `window.crossOriginIsolated === true`
4. [ ] All tests pass at `/test-threading.html`
5. [ ] Service Worker is active
6. [ ] Audio playback works
7. [ ] PWA install prompt appears (mobile)

## Quick Reference

| Check | Command |
|-------|---------|
| Build | `trunk build --release` |
| Test Locally | `trunk serve --open` |
| Check Headers | `curl -I https://your-domain.com/ \| grep -i cross` |
| Test Page | `https://your-domain.com/test-threading.html` |
| Unregister SW | DevTools → Application → Service Workers → Unregister |
| Clear Cache | DevTools → Application → Clear storage |

## Contact/Support

- **Documentation:** See `WASM_THREADING_SETUP.md` for detailed info
- **Browser Support:** See `BROWSER_COMPATIBILITY.md` for compatibility matrix
- **Summary:** See `MULTITHREADED_WASM_SUMMARY.md` for implementation details

---

**Last Updated:** 2025-11-16
**Version:** 1.0
**Status:** ✅ Ready for Production
