# Playwright Tests - Quick Start Guide

Get up and running with E2E tests in 5 minutes.

## 1. Prerequisites Check

```bash
# Verify you're in the rusty-audio directory
pwd

# Should show: .../rusty-audio
```

## 2. Build WASM Application

```bash
# Build with Trunk (recommended)
trunk build --release

# Verify build succeeded
ls -la dist/*.wasm
ls -la dist/pkg/*.wasm
```

**Expected output:**
```
dist/rusty_audio_bg.wasm  (or dist/pkg/rusty_audio_bg.wasm)
```

## 3. Install Test Dependencies

```bash
cd tests/
npm install
npm run install-browsers
```

## 4. Run Tests

### Option A: Run All Tests
```bash
npm test
```

### Option B: Run Specific Browser
```bash
npm run test:chromium    # Fastest, most compatible
npm run test:firefox     # Good coverage
npm run test:webkit      # Safari testing
```

### Option C: Performance Only
```bash
npm run test:performance
```

## 5. View Results

```bash
# Open HTML report
npm run report
```

## Common Commands Cheat Sheet

```bash
# Development
npm run test:headed          # See tests run in browser
npm run test:debug           # Debug with Playwright Inspector
npm run test:ui              # Interactive test runner

# Specific tests
npx playwright test wasm-loading.spec.ts           # One file
npx playwright test -g "should load WASM"          # One test

# CI simulation
CI=true npm test             # Run as CI would

# Generate code
npm run codegen              # Record new tests
```

## Expected Test Results

**Passing tests (Chromium):**
- ✅ WASM Loading: 14/14 tests
- ✅ Multithreading: 15/15 tests
- ✅ Audio Functionality: 18/18 tests
- ✅ UI Rendering: 16/16 tests
- ✅ Performance: 12/12 tests

**Total: ~75 tests**

## Troubleshooting

### Problem: "WASM binary not found"
```bash
# Solution: Build first
trunk build --release
```

### Problem: "SharedArrayBuffer not available"
```bash
# Solution: Use correct server headers
npx http-server dist -p 8080 --cors \
  -H "Cross-Origin-Opener-Policy: same-origin" \
  -H "Cross-Origin-Embedder-Policy: require-corp"
```

### Problem: Tests timeout
```bash
# Solution: Run with fewer workers
npx playwright test --workers=1
```

### Problem: Browser not installed
```bash
# Solution: Install browsers
npm run install-browsers
```

## Next Steps

1. **Read full documentation**: `tests/README.md`
2. **Check CI workflow**: `.github/workflows/playwright-e2e.yml`
3. **Review test helpers**: `tests/helpers/wasm-fixtures.ts`
4. **Add new tests**: Use `tests/e2e/*.spec.ts` as templates

## Performance Targets

| Metric | Target | Warning | Critical |
|--------|--------|---------|----------|
| Init Time | < 2s | < 2.5s | < 3s |
| FPS | 60 | 55+ | 50+ |
| Memory | < 100MB | < 150MB | < 200MB |
| Audio Latency | < 30ms | < 50ms | < 100ms |

## Support

- **Issues**: Check `tests/README.md` troubleshooting section
- **CI Logs**: View GitHub Actions workflow runs
- **Playwright Docs**: https://playwright.dev

---

**Happy Testing!** 🎭
