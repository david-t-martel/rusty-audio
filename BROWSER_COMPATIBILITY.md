# Browser Compatibility Matrix

## Overview

Rusty Audio requires modern browser features including WebAssembly, Web Audio API, and optionally SharedArrayBuffer for multithreaded performance.

## Feature Requirements

| Feature | Required | Purpose |
|---------|----------|---------|
| WebAssembly | ✅ Required | Core application runtime |
| Web Audio API | ✅ Required | Audio playback and processing |
| Canvas/WebGL | ✅ Required | UI rendering via egui |
| Service Workers | ⚠️ Recommended | Offline support, caching |
| SharedArrayBuffer | ⚠️ Optional | Multithreaded WASM performance |
| Cross-Origin Isolation | ⚠️ Optional | Required for SharedArrayBuffer |

## Desktop Browser Support

### Chrome / Chromium / Edge

| Version | WASM | Threads | Status | Notes |
|---------|------|---------|--------|-------|
| 92+ | ✅ | ✅ | **Full Support** | Recommended. Full threading with HTTPS + COOP/COEP |
| 80-91 | ✅ | ⚠️ | **Partial** | Threads require origin trial or flags |
| 60-79 | ✅ | ❌ | **Single-threaded** | WASM supported, no threading |
| <60 | ❌ | ❌ | **Not Supported** | No WebAssembly support |

**Recommended Version**: Chrome 92+ (released July 2021)

**Enabling Threading**:
- Ensure HTTPS (required for cross-origin isolation)
- Verify headers: COOP: same-origin, COEP: require-corp
- Check: `window.crossOriginIsolated === true`

**Known Issues**:
- Chrome 60-67: Limited WASM performance
- Chrome 68-91: SharedArrayBuffer disabled by default (Spectre mitigation)

---

### Firefox

| Version | WASM | Threads | Status | Notes |
|---------|------|---------|--------|-------|
| 79+ | ✅ | ✅ | **Full Support** | Full threading with HTTPS + COOP/COEP |
| 72-78 | ✅ | ⚠️ | **Experimental** | Requires `javascript.options.shared_memory` flag |
| 52-71 | ✅ | ❌ | **Single-threaded** | WASM supported, no threading |
| <52 | ❌ | ❌ | **Not Supported** | No WebAssembly support |

**Recommended Version**: Firefox 79+ (released July 2020)

**Enabling Threading**:
- Ensure HTTPS
- Verify COOP/COEP headers
- For Firefox 72-78: Set `javascript.options.shared_memory = true` in about:config

**Known Issues**:
- Firefox 72-78: Threading behind flag, not recommended for production
- Firefox ESR 78: May have limited threading support

---

### Safari

| Version | WASM | Threads | Status | Notes |
|---------|------|---------|--------|-------|
| 15.2+ | ✅ | ✅ | **Full Support** | macOS 12.1+, iOS 15.2+ |
| 14.0-15.1 | ✅ | ⚠️ | **Experimental** | Limited thread support |
| 11.0-13.1 | ✅ | ❌ | **Single-threaded** | WASM supported, no threading |
| <11.0 | ❌ | ❌ | **Not Supported** | No WebAssembly support |

**Recommended Version**: Safari 15.2+ (macOS 12.1+, iOS 15.2+)

**Platform Requirements**:
- macOS: 12.1 (Monterey) or later
- iOS: 15.2 or later
- iPadOS: 15.2 or later

**Known Issues**:
- Safari 14.0-15.1: Inconsistent SharedArrayBuffer behavior
- Safari < 14: Some egui rendering issues reported
- iOS Safari: AudioContext requires user interaction to start

---

### Opera

| Version | WASM | Threads | Status | Notes |
|---------|------|---------|--------|-------|
| 78+ | ✅ | ✅ | **Full Support** | Based on Chromium 92+ |
| 47-77 | ✅ | ⚠️ | **Varies** | Depends on Chromium version |
| <47 | ❌ | ❌ | **Not Supported** | No WebAssembly support |

**Note**: Opera uses Chromium, so support mirrors Chrome versions.

---

### Edge (Legacy)

| Version | WASM | Threads | Status | Notes |
|---------|------|---------|--------|-------|
| Legacy Edge | ⚠️ | ❌ | **Deprecated** | EdgeHTML engine, limited WASM |
| Modern Edge 92+ | ✅ | ✅ | **Full Support** | Same as Chrome (Chromium-based) |

**Note**: Legacy Edge (EdgeHTML) is no longer supported by Microsoft. Use modern Edge (Chromium).

---

## Mobile Browser Support

### Mobile Chrome (Android)

| Version | WASM | Threads | Status | Notes |
|---------|------|---------|--------|-------|
| 92+ | ✅ | ✅ | **Full Support** | Requires HTTPS + COOP/COEP |
| 60-91 | ✅ | ❌ | **Single-threaded** | WASM only, no threading |
| <60 | ❌ | ❌ | **Not Supported** | No WebAssembly |

**Android Version**: Android 5.0+ recommended
**Performance**: Varies by device CPU/RAM

---

### Mobile Safari (iOS/iPadOS)

| Version | WASM | Threads | Status | Notes |
|---------|------|---------|--------|-------|
| 15.2+ | ✅ | ✅ | **Full Support** | iOS/iPadOS 15.2+ |
| 11.0-15.1 | ✅ | ❌ | **Single-threaded** | WASM only |
| <11.0 | ❌ | ❌ | **Not Supported** | No WebAssembly |

**iOS Version**: iOS 15.2+ recommended
**Known Issues**:
- iOS < 15: No SharedArrayBuffer support
- AudioContext requires user gesture to start
- Background tab suspension may affect audio

---

### Firefox Mobile (Android)

| Version | WASM | Threads | Status | Notes |
|---------|------|---------|--------|-------|
| 79+ | ✅ | ✅ | **Full Support** | Requires HTTPS + COOP/COEP |
| 52-78 | ✅ | ❌ | **Single-threaded** | WASM only |
| <52 | ❌ | ❌ | **Not Supported** | No WebAssembly |

---

### Samsung Internet

| Version | WASM | Threads | Status | Notes |
|---------|------|---------|--------|-------|
| 16.0+ | ✅ | ✅ | **Full Support** | Based on Chromium 92+ |
| 5.0-15.0 | ✅ | ⚠️ | **Varies** | Depends on Chromium version |
| <5.0 | ❌ | ❌ | **Not Supported** | No WebAssembly |

---

## Platform-Specific Considerations

### Windows

- **Best Performance**: Chrome 92+, Edge 92+, Firefox 79+
- **Minimum**: Chrome 60+, Edge 79+, Firefox 52+
- **WebGPU**: Chrome 113+ (experimental egui_wgpu support)

### macOS

- **Best Performance**: Safari 15.2+, Chrome 92+, Firefox 79+
- **Minimum**: Safari 11+, Chrome 60+, Firefox 52+
- **Metal Support**: Safari preferred for best GPU performance

### Linux

- **Best Performance**: Chrome 92+, Firefox 79+
- **Minimum**: Chrome 60+, Firefox 52+
- **Wayland**: Some egui rendering issues, X11 recommended

### iOS/iPadOS

- **Best Performance**: Safari 15.2+
- **Minimum**: Safari 11+
- **Note**: All browsers on iOS use WebKit (Safari engine)
- **PWA Support**: Excellent on iOS 15.2+

### Android

- **Best Performance**: Chrome 92+, Firefox 79+
- **Minimum**: Chrome 60+, Firefox 52+
- **Samsung Internet**: 16.0+ recommended
- **Performance**: Varies significantly by device

---

## Testing Your Browser

### Quick Feature Detection

Open the console (F12) on [your deployment URL] and run:

```javascript
console.log({
  userAgent: navigator.userAgent,
  wasm: typeof WebAssembly !== 'undefined',
  threads: typeof SharedArrayBuffer !== 'undefined',
  crossOriginIsolated: window.crossOriginIsolated,
  serviceWorker: 'serviceWorker' in navigator,
  audioContext: typeof AudioContext !== 'undefined',
  hardwareConcurrency: navigator.hardwareConcurrency
});
```

### Expected Output (Fully Supported Browser)

```javascript
{
  userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64)...",
  wasm: true,
  threads: true,
  crossOriginIsolated: true,
  serviceWorker: true,
  audioContext: true,
  hardwareConcurrency: 8
}
```

### Fallback Behavior

If `threads: false`:
- Application runs in single-threaded mode
- Audio processing still works
- Performance may be reduced on complex audio graphs
- UI remains responsive

---

## Recommended Browsers by Use Case

### Best Overall Experience
1. **Chrome/Edge 92+** (Windows, macOS, Linux)
2. **Firefox 79+** (Windows, macOS, Linux)
3. **Safari 15.2+** (macOS, iOS)

### Best Performance
1. **Chrome 100+** with hardware acceleration
2. **Safari 15.2+** on macOS (Metal optimization)
3. **Firefox 100+** with WebRender

### Best Mobile Experience
1. **Safari 15.2+** (iOS/iPadOS)
2. **Chrome 92+** (Android)
3. **Samsung Internet 16.0+** (Samsung devices)

### Development/Testing
1. **Chrome Dev/Canary** (latest features)
2. **Firefox Nightly** (experimental features)
3. **Safari Technology Preview** (upcoming iOS/macOS features)

---

## Browser Flags for Testing

### Chrome/Edge

Enable experimental WASM features:

```
chrome://flags/#enable-webassembly-threads
chrome://flags/#enable-webassembly-simd
```

### Firefox

Enable SharedArrayBuffer (Firefox 72-78):

```
about:config
javascript.options.shared_memory = true
javascript.options.wasm_simd = true
```

### Safari

Enable experimental features:

```
Safari → Develop → Experimental Features
- WebAssembly Streaming API
- WebAssembly SIMD
```

---

## Known Issues & Workarounds

### Issue: SharedArrayBuffer Unavailable

**Symptoms**: `TypeError: SharedArrayBuffer is not defined`

**Causes**:
- HTTP instead of HTTPS
- Missing COOP/COEP headers
- Browser doesn't support cross-origin isolation

**Workarounds**:
1. Deploy with HTTPS
2. Verify headers in Network tab
3. Use fallback single-threaded mode
4. Update browser to supported version

### Issue: Audio Not Playing (Mobile Safari)

**Symptoms**: Silent playback on iOS

**Cause**: AudioContext requires user interaction

**Workaround**:
- Add "Play" button that user must click
- Resume AudioContext on first user interaction

### Issue: Canvas Not Rendering

**Symptoms**: Black screen, no UI

**Causes**:
- WebGL not enabled
- GPU blacklisted
- Browser too old

**Workarounds**:
1. Enable hardware acceleration
2. Update GPU drivers
3. Use WebGL fallback renderer

---

## Browser Market Share (2024)

Reference for prioritizing browser support:

- Chrome: ~65% (Desktop + Mobile)
- Safari: ~20% (macOS + iOS)
- Firefox: ~3%
- Edge: ~5%
- Samsung Internet: ~2.5%
- Opera: ~2%
- Others: ~2.5%

**Recommendation**: Prioritize Chrome, Safari, Edge for 90%+ coverage.

---

## Future Compatibility

### Upcoming Features

- **WebGPU**: Chrome 113+, Safari 18+ (egui experimental support)
- **WebCodecs**: Enhanced audio/video processing
- **WebTransport**: Low-latency audio streaming
- **File System Access API**: Better file picker integration

### Deprecations

- **Legacy Edge**: End of support (use Chromium Edge)
- **Internet Explorer**: No support (use any modern browser)

---

## Testing Matrix

Recommended testing across:

1. **Primary**: Latest Chrome, Firefox, Safari
2. **Secondary**: Edge, Safari iOS, Chrome Android
3. **Edge Cases**: Older browser versions (warn users)

---

## Support Policy

- ✅ **Fully Supported**: Latest 2 major versions of Chrome, Firefox, Safari, Edge
- ⚠️ **Best Effort**: Older versions with WASM support (single-threaded mode)
- ❌ **Not Supported**: IE, browsers without WASM, browsers <2 years old

---

## Resources

- [Can I Use: WebAssembly](https://caniuse.com/wasm)
- [Can I Use: SharedArrayBuffer](https://caniuse.com/sharedarraybuffer)
- [MDN: Browser Compatibility](https://developer.mozilla.org/en-US/docs/Web/API/SharedArrayBuffer#browser_compatibility)
- [Chrome Platform Status](https://chromestatus.com/features)
- [WebKit Feature Status](https://webkit.org/status/)
- [Firefox Platform Status](https://platform-status.mozilla.org/)
