# Rusty Audio - Troubleshooting Guide

Quick reference for diagnosing and fixing common deployment and runtime issues.

---

## Quick Diagnostics

### 1-Minute Health Check

```bash
# 1. Start server
cd C:/Users/david/rusty-audio/dist
node server.cjs

# 2. In another terminal, validate deployment
cd C:/Users/david/rusty-audio/dist
node validate-deployment.cjs

# 3. Open in browser
# Chrome: http://localhost:8080/browser-test.html
# Check: Cross-Origin Isolated = Yes
# Check: SharedArrayBuffer = Supported
```

If all green, skip to [Manual Testing](#manual-testing).

---

## Common Issues

### Issue 1: Server Won't Start

**Error:** `node: command not found` or `Cannot find module`

**Solutions:**
```bash
# Check Node.js installed
node --version  # Should show v18+ or v20+

# If not installed, install Node.js from https://nodejs.org/

# Check file exists
ls -la C:/Users/david/rusty-audio/dist/server.cjs

# Try running with full path
"C:\Program Files\nodejs\node.exe" server.cjs
```

**Error:** `Error: listen EADDRINUSE: address already in use :::8080`

**Solutions:**
```bash
# Port 8080 is in use. Kill existing process:

# Windows:
netstat -ano | findstr :8080
taskkill /PID <PID> /F

# WSL/Linux:
lsof -ti:8080 | xargs kill -9

# Or use different port:
# Edit server.cjs, change PORT = 8080 to PORT = 8081
```

---

### Issue 2: WASM File Not Loading

**Symptoms:**
- Console: "Failed to fetch"
- Console: "Incorrect response MIME type"
- Network tab shows 404 for `.wasm` file

**Check 1: File Exists**
```bash
ls -la C:/Users/david/rusty-audio/dist/pkg/rusty_audio_bg.wasm
# Should show ~10-11 MB file

# If missing, rebuild:
cd C:/Users/david/rusty-audio
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/rusty_audio.wasm \
  --out-dir dist/pkg --target web
```

**Check 2: Correct MIME Type**
```bash
curl -I http://localhost:8080/pkg/rusty_audio_bg.wasm | grep -i content-type
# Should show: Content-Type: application/wasm

# If wrong, check server.cjs has:
# '.wasm': 'application/wasm'
```

**Check 3: Server Headers**
```bash
curl -I http://localhost:8080/pkg/rusty_audio_bg.wasm
# Should include:
# - Content-Type: application/wasm
# - Cross-Origin-Opener-Policy: same-origin
# - Cross-Origin-Embedder-Policy: require-corp
```

**Check 4: Path in index.html**
```bash
grep "rusty_audio" C:/Users/david/rusty-audio/dist/index.html
# Should show: import init from './pkg/rusty_audio.js'
# Path must match file location
```

---

### Issue 3: SharedArrayBuffer Not Available

**Symptoms:**
- Console: `crossOriginIsolated = false`
- Console: "SharedArrayBuffer is not defined"
- Audio worker pool fails to initialize

**Critical:** SharedArrayBuffer requires COOP/COEP headers!

**Check Headers:**
```bash
curl -I http://localhost:8080/ | grep -i cross-origin

# MUST show:
# Cross-Origin-Opener-Policy: same-origin
# Cross-Origin-Embedder-Policy: require-corp
```

**Solutions:**

1. **Verify using correct server:**
   ```bash
   # ❌ DON'T USE: python -m http.server
   # ✅ USE: node server.cjs
   ```

2. **Check browser support:**
   - Chrome 92+ ✅
   - Firefox 79+ ✅
   - Safari 15.2+ ⚠️ (may have restrictions)
   - Edge 92+ ✅

3. **Test in browser console:**
   ```javascript
   console.log('crossOriginIsolated:', crossOriginIsolated);
   console.log('SharedArrayBuffer:', typeof SharedArrayBuffer);
   // Both should show true/function
   ```

4. **Try different browser:**
   - Firefox has best WASM support
   - Chrome usually works
   - Safari may have issues

5. **Check for browser extensions:**
   - Disable ad blockers
   - Try Incognito/Private mode
   - Extensions can interfere with headers

---

### Issue 4: Black Screen / No UI

**Symptoms:**
- Page loads, canvas exists, but shows only black
- No egui interface visible

**Check 1: Console Errors**
```javascript
// Open DevTools (F12) → Console
// Look for:
// ❌ WebGL errors
// ❌ WASM initialization errors
// ❌ JavaScript exceptions
```

**Check 2: WebGL Support**
```javascript
// In browser console:
const canvas = document.createElement('canvas');
const gl = canvas.getContext('webgl2');
console.log('WebGL2:', gl !== null);

// If false, try WebGL1:
const gl1 = canvas.getContext('webgl');
console.log('WebGL1:', gl1 !== null);
```

**Check 3: Browser Graphics Drivers**
- Update GPU drivers
- Try hardware acceleration toggle:
  - Chrome: `chrome://settings` → Advanced → System
  - Toggle "Use hardware acceleration when available"

**Check 4: Canvas Size**
```javascript
// In browser console:
const canvas = document.getElementById('rusty-audio-canvas');
console.log('Canvas:', canvas);
console.log('Width:', canvas.width, 'Height:', canvas.height);
// Should show dimensions > 0
```

**Solutions:**

1. **Try different browser**
2. **Check GPU blacklist:** `chrome://gpu/` (Chrome/Edge)
3. **Disable extensions**
4. **Update graphics drivers**
5. **Try software rendering** (slower but compatible)

---

### Issue 5: No Audio Output

**Symptoms:**
- UI loads correctly
- Signal generator shows but no sound
- Audio context errors in console

**Check 1: Audio Context State**
```javascript
// In browser console:
const AudioContext = window.AudioContext || window.webkitAudioContext;
const ctx = new AudioContext();
console.log('Audio context state:', ctx.state);
// Should show: "running" or "suspended"

// If suspended, resume:
ctx.resume().then(() => console.log('Audio resumed'));
```

**Check 2: User Interaction Required**
Modern browsers require user interaction before audio:
1. Click anywhere on the page
2. Click "Generate" button in signal generator
3. Audio should start

**Check 3: System Audio**
- Check system volume is not muted
- Check browser tab is not muted (right-click tab)
- Check audio output device is working

**Check 4: Permissions**
```javascript
// Check permissions:
navigator.permissions.query({name: 'microphone'})
  .then(result => console.log('Mic permission:', result.state));
// Note: Speaker output doesn't need permission, but mic might
```

**Solutions:**

1. **Click on page** (user interaction)
2. **Check browser audio settings**
3. **Try different audio output device**
4. **Restart browser**
5. **Test with simple Web Audio:**
   ```javascript
   const ctx = new (window.AudioContext || window.webkitAudioContext)();
   const osc = ctx.createOscillator();
   osc.connect(ctx.destination);
   osc.start();
   osc.stop(ctx.currentTime + 0.5);
   // Should hear beep
   ```

---

### Issue 6: Poor Performance / Lag

**Symptoms:**
- UI stutters or freezes
- Frame rate <30 FPS
- High CPU usage

**Check 1: Performance Monitoring**
```bash
# Open browser test console:
# http://localhost:8080/browser-test.html
# Click "Measure Performance"
# Check FPS value
```

**Check 2: Browser Task Manager**
- Chrome: Shift+Esc
- Check CPU and memory usage
- Look for runaway processes

**Check 3: Worker Pool**
```javascript
// In console:
console.log('Hardware Concurrency:', navigator.hardwareConcurrency);
// Shows number of CPU cores available for workers
```

**Solutions:**

1. **Close other tabs** (free up memory)
2. **Disable browser extensions**
3. **Try Incognito mode** (clean environment)
4. **Check CPU usage** (other apps using CPU?)
5. **Reduce audio processing:**
   - Lower sample rate (if configurable)
   - Disable EQ temporarily
   - Reduce FFT size

**Performance Targets:**
- WASM load: <3s
- FPS: >30 (ideal: 60)
- Audio latency: <50ms
- Memory: Stable (no continuous growth)

---

### Issue 7: Memory Leaks

**Symptoms:**
- Memory usage continuously increases
- Browser becomes slow over time
- Tab crashes after extended use

**Check Memory Usage:**
```javascript
// In console (Chrome):
console.log(performance.memory);
// Shows: usedJSHeapSize, totalJSHeapSize, jsHeapSizeLimit

// Monitor for 1 minute:
setInterval(() => {
  const mb = (performance.memory.usedJSHeapSize / 1024 / 1024).toFixed(2);
  console.log('Memory:', mb, 'MB');
}, 5000);
// Should not continuously increase
```

**Take Heap Snapshot:**
1. DevTools → Memory tab
2. Take snapshot
3. Use app for 1 minute
4. Take another snapshot
5. Compare (look for retained objects)

**Solutions:**

1. **Reload page** (temporary fix)
2. **Report issue** (likely bug in WASM code)
3. **Monitor specific features:**
   - Does leak happen with specific feature?
   - Test each tab individually
4. **Check worker cleanup:**
   - Are workers being properly terminated?

---

### Issue 8: Service Worker Issues

**Symptoms:**
- Console: "Service Worker registration failed"
- PWA install not available
- Caching not working

**Check Registration:**
```javascript
// In console:
navigator.serviceWorker.getRegistrations()
  .then(regs => console.log('Registered SWs:', regs));

// Check for errors:
navigator.serviceWorker.register('/service-worker.js')
  .then(reg => console.log('SW registered:', reg))
  .catch(err => console.error('SW error:', err));
```

**Solutions:**

1. **Service Worker is optional** - app works without it
2. **Requires localhost or HTTPS** (we have localhost ✓)
3. **Clear previous registrations:**
   ```javascript
   navigator.serviceWorker.getRegistrations()
     .then(regs => regs.forEach(reg => reg.unregister()));
   ```
4. **Check SW file exists:**
   ```bash
   curl http://localhost:8080/service-worker.js
   # Should return JavaScript code
   ```

---

## Browser-Specific Issues

### Chrome/Edge (Chromium)

**Best support expected. If issues:**
1. Check `chrome://flags` for experimental features
2. Try disabling flags that might interfere
3. Check `chrome://gpu/` for graphics issues
4. Update to latest version

### Firefox

**Good support. Known issues:**
1. WebGPU may not be available → Should fallback to WebGL2
2. Check `about:config` for `dom.postMessage.sharedArrayBuffer.bypassCOOP_COEP.insecure.enabled`
3. Update to latest version

### Safari

**Limited support. Known issues:**
1. SharedArrayBuffer may not work (security restrictions)
2. WebGPU not supported → Fallback to WebGL2
3. Web Audio API quirks
4. May need user interaction for audio

**If Safari doesn't work:**
- This is expected due to stricter security
- Test in Chrome/Firefox instead
- Production deployment may need Safari-specific handling

---

## Testing Strategy

### Progressive Testing

1. **Start Simple:**
   ```bash
   # Just check server
   curl http://localhost:8080/
   ```

2. **Add Validation:**
   ```bash
   # Run automated tests
   node validate-deployment.cjs
   ```

3. **Browser Test Console:**
   ```
   # Open test page
   http://localhost:8080/browser-test.html
   # Check all diagnostics green
   ```

4. **Main Application:**
   ```
   # Open main app
   http://localhost:8080/index.html
   # Test features one by one
   ```

### Isolate Issues

If something doesn't work:

1. **Check one thing at a time**
2. **Test in browser console first**
3. **Verify headers before blaming code**
4. **Try different browser**
5. **Check for console errors**

---

## Quick Reference

### Important URLs

- **Main App:** http://localhost:8080/index.html
- **Test Console:** http://localhost:8080/browser-test.html
- **Service Worker:** http://localhost:8080/service-worker.js
- **WASM Binary:** http://localhost:8080/pkg/rusty_audio_bg.wasm

### Important Commands

```bash
# Start server
cd C:/Users/david/rusty-audio/dist && node server.cjs

# Validate deployment
cd C:/Users/david/rusty-audio/dist && node validate-deployment.cjs

# Check headers
curl -I http://localhost:8080/

# Check WASM file
curl -I http://localhost:8080/pkg/rusty_audio_bg.wasm

# Rebuild WASM
cd C:/Users/david/rusty-audio
cargo build --release --target wasm32-unknown-unknown
```

### Console Checks

```javascript
// Must be true for SharedArrayBuffer:
crossOriginIsolated

// Must be defined:
typeof WebAssembly !== 'undefined'
typeof SharedArrayBuffer !== 'undefined'
typeof Worker !== 'undefined'

// Audio context should work:
new (window.AudioContext || window.webkitAudioContext)()
```

---

## Getting Help

### Information to Collect

When reporting issues, include:

1. **Browser & Version:**
   ```javascript
   navigator.userAgent
   ```

2. **Feature Detection:**
   ```javascript
   console.log('crossOriginIsolated:', crossOriginIsolated);
   console.log('SharedArrayBuffer:', typeof SharedArrayBuffer);
   console.log('WebAssembly:', typeof WebAssembly);
   ```

3. **Console Errors:**
   - Full error message
   - Stack trace
   - Network tab (failed requests)

4. **Server Headers:**
   ```bash
   curl -I http://localhost:8080/
   ```

5. **WASM File Info:**
   ```bash
   ls -lh dist/pkg/rusty_audio_bg.wasm
   ```

---

## Success Checklist

Before declaring "it works":

- [ ] Server starts without errors
- [ ] Validation script passes (9/9 tests)
- [ ] Browser test console shows all green
- [ ] `crossOriginIsolated = true` in console
- [ ] WASM file loads (check Network tab)
- [ ] UI renders (not black screen)
- [ ] Audio context initializes
- [ ] Signal generator produces sound
- [ ] Spectrum analyzer displays
- [ ] EQ controls are responsive
- [ ] Performance is acceptable (>30 FPS)
- [ ] No memory leaks (stable over 1 minute)
- [ ] Works in at least 2 browsers

---

*Last Updated: 2025-11-17*
*Deployment Version: Local Dev Server*
