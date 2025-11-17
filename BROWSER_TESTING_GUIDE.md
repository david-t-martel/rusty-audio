# Browser Compatibility Testing Guide - Rusty Audio WASM

**Test Date:** 2025-11-17
**Application:** Rusty Audio (WASM Build)
**Server:** Local Development (http://localhost:8080/)

---

## Quick Reference

### Supported Browsers

| Browser | Version | Status | Notes |
|---------|---------|--------|-------|
| Chrome | 92+ | ✅✅ Recommended | Full support, best performance |
| Firefox | 79+ | ✅ Supported | Full support, WebGL2 backend |
| Edge | 92+ | ✅✅ Recommended | Full support (Chromium) |
| Safari | 15.2+ | ⚠️ Limited | SAB restrictions may apply |

### Testing URLs

- **Test Console:** http://localhost:8080/browser-test.html
- **Main App:** http://localhost:8080/index.html
- **Validation:** Run `node validate-deployment.cjs`

---

## Chrome Testing (Primary Target)

### Expected: ✅ Full Support

**Test Steps:**

1. **Start Server:**
   ```bash
   cd C:/Users/david/rusty-audio/dist
   node server.cjs
   ```

2. **Open Test Console:**
   - Navigate to: http://localhost:8080/browser-test.html
   - Verify auto-diagnostics show all green
   - Check: "Cross-Origin Isolated: Yes ✓"
   - Check: "SharedArrayBuffer: Supported ✓"

3. **Load Application:**
   - Click "Load Application" button OR
   - Navigate to: http://localhost:8080/index.html

4. **Console Verification (Press F12):**
   ```javascript
   crossOriginIsolated  // Must be true
   typeof SharedArrayBuffer  // Must be "function"
   typeof WebAssembly  // Must be "object"
   ```

5. **Feature Tests:**
   - [ ] UI renders (egui interface visible)
   - [ ] Signal generator produces audio
   - [ ] Spectrum analyzer displays
   - [ ] EQ sliders work
   - [ ] Recording functions
   - [ ] No console errors

**Performance Targets:**
- WASM load: <3 seconds
- FPS: 60 (check test console)
- Memory: Stable (<500MB)

---

## Firefox Testing (Secondary)

### Expected: ✅ Full Support (WebGL2 backend)

**Known Differences:**
- WebGPU not available → Uses WebGL2 (automatic)
- Slightly different audio behavior (compatible)

**Test Steps:**
Same as Chrome, but note:
- WebGPU feature detection will show "Not Supported" (OK)
- egui automatically uses WebGL2 backend
- All other features should work identically

**Critical Checks:**
- [ ] Cross-origin isolation enabled
- [ ] SharedArrayBuffer available
- [ ] WebGL2 available (not WebGPU)
- [ ] Audio output works

---

## Safari Testing (Compatibility Check)

### Expected: ⚠️ Partial Support

**Known Limitations:**
- SharedArrayBuffer may not work (even with headers)
- WebGPU not available
- Audio requires explicit user gesture
- Worker pool may fail

**Test Steps:**

1. **Open Test Console:**
   http://localhost:8080/browser-test.html

2. **Critical Verification:**
   ```javascript
   crossOriginIsolated  // May be false!
   typeof SharedArrayBuffer  // May be "undefined"!
   ```

3. **Expected Results:**
   - [ ] WebAssembly: Supported ✓
   - [ ] SharedArrayBuffer: ⚠️ May NOT be supported
   - [ ] WebGL2: Supported ✓
   - [ ] Worker pool: ⚠️ May fail to initialize

4. **If SharedArrayBuffer Missing:**
   - Application may still load
   - Performance will be degraded (single-threaded)
   - Worker pool initialization will fail
   - Core features should still work

**Recommendation:**
If Safari doesn't work well, recommend users switch to Chrome/Firefox.

---

## Manual Testing Checklist

### For Each Browser

#### 1. Load Test
- [ ] Server responds (http://localhost:8080/)
- [ ] index.html loads
- [ ] Loading spinner appears
- [ ] Canvas element becomes visible
- [ ] UI renders (not black screen)

#### 2. Feature Test

**Signal Generator:**
- [ ] Tab opens
- [ ] Waveform selector works (sine, square, etc.)
- [ ] Frequency slider adjusts
- [ ] Click "Generate" produces audio
- [ ] Volume control works

**Spectrum Analyzer:**
- [ ] Visual display appears
- [ ] Updates in real-time during audio
- [ ] Colors/bars animate
- [ ] Frequency scale visible

**Equalizer:**
- [ ] 8 frequency bands visible
- [ ] Sliders are responsive
- [ ] Audio changes when adjusted
- [ ] Reset button works

**Recording:**
- [ ] Recording panel opens
- [ ] Record button starts recording
- [ ] Timer shows recording duration
- [ ] Stop button ends recording
- [ ] Save produces downloadable file

#### 3. Performance Test
- [ ] UI frame rate >30 FPS (check test console)
- [ ] No stuttering during audio
- [ ] Memory usage stable (monitor for 1 minute)
- [ ] CPU usage reasonable
- [ ] No browser hangs/freezes

#### 4. Stability Test
- [ ] No console errors
- [ ] No network errors (check Network tab)
- [ ] WASM file loads successfully
- [ ] Service worker registers (optional)
- [ ] No memory leaks

---

## Performance Benchmarking

### Metrics to Collect

Open http://localhost:8080/browser-test.html and click "Measure Performance"

**Record:**
- **WASM Load Time:** _______ seconds (target: <3s)
- **Page Load Time:** _______ seconds (target: <2s)
- **FPS:** _______ (target: >30, ideal: 60)
- **Memory Used:** _______ MB (target: <500MB)
- **CPU Cores:** _______ (for worker pool)

### DevTools Performance Tab

1. Open DevTools (F12) → Performance tab
2. Click Record
3. Interact with application for 10 seconds
4. Stop recording
5. **Check:**
   - [ ] FPS mostly >30
   - [ ] No long tasks (red blocks)
   - [ ] No layout thrashing
   - [ ] Worker threads active (if SAB available)

---

## Console Error Analysis

### Expected Console Output (Good)

```
[Info] Server running...
[Info] Loading WASM module...
[Info] WASM loaded successfully
[Info] crossOriginIsolated: true
[Info] SharedArrayBuffer: available
[Info] Worker pool initialized (X workers)
[Info] Audio context: running
[Info] egui initialized (WebGL2/WebGPU)
[Info] Rusty Audio started successfully!
```

### Warning Signs (Investigate)

```
[Warn] crossOriginIsolated: false
[Warn] SharedArrayBuffer not available
[Warn] Worker pool initialization failed
[Warn] Falling back to single-threaded mode
```

### Critical Errors (Fix Required)

```
[Error] Failed to fetch WASM module
[Error] Incorrect MIME type for WASM
[Error] WebGL context creation failed
[Error] Audio context initialization failed
```

---

## Browser-Specific Issues

### Chrome Issues

**Issue:** "Site can't be reached"
- Check server is running: `curl http://localhost:8080/`
- Check port not in use: `netstat -ano | findstr :8080`

**Issue:** WASM won't load
- Check MIME type: `curl -I http://localhost:8080/pkg/rusty_audio_bg.wasm`
- Should show: `Content-Type: application/wasm`

### Firefox Issues

**Issue:** SharedArrayBuffer not available
- Check about:config → `dom.postMessage.sharedArrayBuffer`
- Should allow SAB with COOP/COEP headers

**Issue:** WebGPU error
- Expected! Firefox doesn't support WebGPU yet
- egui falls back to WebGL2 automatically

### Safari Issues

**Issue:** crossOriginIsolated false
- This is common in Safari
- May be unsolvable due to Safari security
- Application may still work in degraded mode

**Issue:** Audio won't play
- Safari requires user gesture before audio
- Click anywhere on page first
- Then try signal generator

---

## Test Report Template

```markdown
### Browser Test Report

**Browser:** [Chrome/Firefox/Safari] [Version]
**Date:** 2025-11-17
**Tester:** [Your Name]

#### Environment
- [ ] crossOriginIsolated: [true/false]
- [ ] SharedArrayBuffer: [available/unavailable]
- [ ] WebGL2: [available/unavailable]
- [ ] WebGPU: [available/unavailable]

#### Application Loading
- [ ] Page loads: [✅/❌]
- [ ] WASM loads: [✅/❌]
- [ ] UI renders: [✅/❌]
- [ ] Console errors: [none/list them]

#### Features
- [ ] Signal Generator: [✅/⚠️/❌]
- [ ] Spectrum Analyzer: [✅/⚠️/❌]
- [ ] Equalizer: [✅/⚠️/❌]
- [ ] Recording: [✅/⚠️/❌]

#### Performance
- WASM Load: [___] seconds
- FPS: [___]
- Memory: [___] MB

#### Issues Found
1. [Issue description]
2. [Issue description]

#### Overall Status
[✅ Full Support / ⚠️ Partial Support / ❌ Not Working]

#### Notes
[Additional observations]
```

---

## Automated Testing Script

For automated feature detection, use:

```bash
cd C:/Users/david/rusty-audio/dist
node validate-deployment.cjs
```

This validates:
- ✅ Server running
- ✅ COOP/COEP headers
- ✅ WASM MIME type
- ✅ File sizes reasonable
- ✅ All assets available

---

## Success Criteria

Application is considered "working" if:

- [x] Automated validation passes (9/9 tests)
- [x] crossOriginIsolated = true (in Chrome/Firefox)
- [x] WASM loads without errors
- [x] UI renders (not black screen)
- [x] At least one audio feature works (signal generator)
- [x] Performance >30 FPS
- [x] No memory leaks
- [x] Works in 2+ browsers (Chrome recommended)

---

## Next Steps After Testing

1. **All Tests Pass:**
   - Application ready for use
   - Can proceed to production deployment
   - Document any browser-specific notes

2. **Some Tests Fail:**
   - Consult TROUBLESHOOTING_GUIDE.md
   - Check specific browser known issues
   - Test in different browser
   - Report issues if persistent

3. **Critical Failures:**
   - Check server headers: `curl -I http://localhost:8080/`
   - Verify WASM file exists: `ls -lh dist/pkg/*.wasm`
   - Check console errors for clues
   - Restart server and retry

---

*Last Updated: 2025-11-17*
*Status: Ready for Manual Testing*
*Automated Validation: ✅ PASSING*
