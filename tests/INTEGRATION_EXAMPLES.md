# Playwright Test Integration Examples

Real-world examples of running and integrating the test suite.

## Example 1: Local Development Workflow

```bash
# Terminal 1: Start dev server with correct headers
cd rusty-audio
npx http-server dist -p 8080 --cors -c-1 \
  -H "Cross-Origin-Opener-Policy: same-origin" \
  -H "Cross-Origin-Embedder-Policy: require-corp"

# Terminal 2: Run tests in watch mode
cd tests/
npm test -- --watch

# Make changes to WASM app, rebuild, tests re-run automatically
```

## Example 2: Pre-Commit Testing

```bash
# Quick smoke test before committing
cd rusty-audio
trunk build --release && cd tests && npm run test:chromium
```

## Example 3: CI/CD Pipeline (GitHub Actions)

The workflow automatically:
1. Builds WASM on push/PR
2. Runs tests in parallel across browsers
3. Generates performance report
4. Comments results on PR

View results:
- Actions tab → Playwright E2E Tests
- Check artifacts for detailed reports

## Example 4: Performance Regression Detection

```bash
# Run performance benchmarks
cd tests/
npm run test:performance

# Check results
cat performance-data/performance-summary.json

# Example output:
{
  "chromium": {
    "tests": 12,
    "avgFPS": 59.2,
    "avgMemory": 94.3,
    "avgLoadTime": 1823
  }
}

# Compare with baseline in E2E_TEST_SUMMARY.md
```

## Example 5: Debugging Failing Test

```bash
# Run test in debug mode
cd tests/
npx playwright test --debug -g "should load WASM binary"

# Steps:
# 1. Playwright Inspector opens
# 2. Step through test actions
# 3. Inspect page state
# 4. Check console logs
# 5. Take screenshots
```

## Example 6: Visual Regression Testing

```bash
# Take baseline screenshots
npm run test:chromium ui-rendering.spec.ts

# Make UI changes
# Run again and compare
npm run test:chromium ui-rendering.spec.ts

# Check screenshots/ directory for diffs
```

## Example 7: Cross-Browser Compatibility Check

```bash
# Run all browsers sequentially
npm run test:chromium && \
npm run test:firefox && \
npm run test:webkit

# Or use the helper script
./run-tests.sh all
```

## Example 8: Mobile Device Testing

```bash
# Test mobile layout
npm run test:mobile

# Or specific mobile device
npx playwright test --project=mobile-chrome
```

## Example 9: Custom Test Suite

Create `tests/e2e/custom.spec.ts`:

```typescript
import { test, expect } from '@playwright/test';
import { waitForWasmInit } from '../helpers/wasm-fixtures';

test.describe('Custom Feature Tests', () => {
  test('should load custom audio file', async ({ page }) => {
    await page.goto('/');
    await waitForWasmInit(page);

    // Custom test logic here
    // ...
  });
});
```

Run with:
```bash
npx playwright test custom.spec.ts
```

## Example 10: CI Integration with Performance Budgets

Add to `.github/workflows/playwright-e2e.yml`:

```yaml
- name: Check performance budgets
  run: |
    LOAD_TIME=$(cat performance-data/performance-metrics.json | jq '.[0].metrics.loadTime')
    if [ $LOAD_TIME -gt 3000 ]; then
      echo "Performance budget exceeded: ${LOAD_TIME}ms > 3000ms"
      exit 1
    fi
```

## Example 11: Headless vs Headed Testing

```bash
# Headless (default, for CI)
npm test

# Headed (see browser, for debugging)
npm run test:headed

# Specific browser headed
npx playwright test --project=firefox --headed
```

## Example 12: Generating Test Code

```bash
# Start codegen with running app
npm run codegen

# Steps:
# 1. Browser opens to localhost:8080
# 2. Perform actions (click, type, etc.)
# 3. Code is generated automatically
# 4. Copy to test file
```

## Example 13: Parallel vs Serial Execution

```bash
# Parallel (faster, default for non-audio tests)
npx playwright test --workers=4

# Serial (for audio tests to avoid device conflicts)
npx playwright test audio-functionality.spec.ts --workers=1

# Auto-detect (recommended)
npm test  # Uses config settings
```

## Example 14: Test Reporting

```bash
# Generate and open HTML report
npm test
npm run report

# View specific test trace
npx playwright show-trace test-results/.../trace.zip

# Generate custom report
npx playwright test --reporter=json > results.json
```

## Example 15: Docker-based Testing

Create `Dockerfile.playwright`:

```dockerfile
FROM mcr.microsoft.com/playwright:v1.40.0-jammy

WORKDIR /app

# Copy WASM build
COPY dist/ dist/

# Copy tests
COPY tests/ tests/

# Install dependencies
WORKDIR /app/tests
RUN npm ci

# Run tests
CMD ["npm", "test"]
```

Run with:
```bash
# Build WASM
trunk build --release

# Run tests in Docker
docker build -f Dockerfile.playwright -t rusty-audio-tests .
docker run rusty-audio-tests
```

## Example 16: Continuous Performance Monitoring

Set up performance tracking:

```bash
# Run daily performance test
# Save results with date
DATE=$(date +%Y%m%d)
npm run test:performance
cp performance-data/performance-summary.json \
   performance-history/perf-${DATE}.json

# Plot trends over time
# (requires custom script)
```

## Example 17: Screenshot Comparison

```bash
# Take reference screenshots
npm run test:chromium ui-rendering.spec.ts
mv screenshots screenshots-baseline

# Make changes, test again
npm run test:chromium ui-rendering.spec.ts

# Compare
diff -r screenshots-baseline screenshots
```

## Example 18: Testing Specific Scenarios

```bash
# Test only threading features
npx playwright test multithreading.spec.ts

# Test only initialization
npx playwright test -g "initialization"

# Test only performance critical paths
npx playwright test -g "benchmark"

# Test single function
npx playwright test -g "should load WASM binary successfully"
```

## Example 19: Local vs CI Configuration

```typescript
// playwright.config.ts
export default defineConfig({
  workers: process.env.CI ? 1 : 2,
  retries: process.env.CI ? 2 : 1,
  use: {
    baseURL: process.env.TEST_URL || 'http://localhost:8080',
  },
});
```

Run locally:
```bash
npm test
```

Run as CI would:
```bash
CI=true npm test
```

## Example 20: Integration with PR Workflow

```yaml
# .github/workflows/pr-check.yml
name: PR Check

on: pull_request

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run E2E tests
        run: |
          trunk build --release
          cd tests && npm ci
          npm test
      - name: Comment results
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const summary = fs.readFileSync('tests/playwright-report/summary.json');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              body: `## Test Results\n\`\`\`json\n${summary}\n\`\`\``
            });
```

## Tips and Best Practices

### 1. Speed Up Tests
```bash
# Use fewer workers
npx playwright test --workers=2

# Skip slow tests during dev
npx playwright test --grep-invert "@slow"

# Run only changed tests
npx playwright test --only-changed
```

### 2. Debug Flaky Tests
```bash
# Run test 10 times
npx playwright test --repeat-each=10 flaky.spec.ts

# Increase timeout
npx playwright test --timeout=90000

# Add debug logs
DEBUG=pw:api npm test
```

### 3. Optimize CI
```bash
# Cache Playwright browsers
# (See .github/workflows/playwright-e2e.yml)

# Use matrix strategy for parallel browser testing

# Upload artifacts only on failure
if: failure()
```

### 4. Local Performance Testing
```bash
# Clear cache before test
rm -rf dist/
trunk build --release

# Fresh browser state
rm -rf ~/.cache/ms-playwright

# Run performance test
npm run test:performance
```

### 5. Test Data Management
```typescript
// Use test fixtures for consistent data
test('with test audio file', async ({ page }) => {
  await page.route('**/audio/*', route => {
    route.fulfill({ path: 'test-fixtures/test-audio.wav' });
  });
});
```

## Troubleshooting Common Scenarios

### Scenario: Tests pass locally, fail in CI

**Diagnosis:**
```bash
# Run with CI environment
CI=true npm test

# Check browser versions
npx playwright --version

# Verify WASM build
ls -lah dist/*.wasm
```

**Solution**: Ensure consistent build environment, browser versions

### Scenario: Performance tests are inconsistent

**Diagnosis:**
```bash
# Run multiple times
for i in {1..5}; do
  npm run test:performance
  mv performance-data/performance-metrics.json perf-run-$i.json
done

# Analyze variance
```

**Solution**: Add warm-up period, increase sample size

### Scenario: Audio tests fail intermittently

**Diagnosis:**
```bash
# Check audio device mocking
npx playwright test --headed audio-functionality.spec.ts

# Verify fake audio devices in browser args
```

**Solution**: Ensure `--use-fake-device-for-media-stream` flag

## Conclusion

These examples demonstrate:
- Local development workflows
- CI/CD integration patterns
- Performance monitoring
- Debugging techniques
- Custom test scenarios
- Best practices

For more details, see:
- `tests/README.md` - Full documentation
- `tests/QUICK_START.md` - Quick reference
- `E2E_TEST_SUMMARY.md` - Test suite overview
