import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright Configuration for Rusty Audio WASM Testing
 *
 * Comprehensive test configuration for validating WASM audio application
 * across multiple browsers with SharedArrayBuffer, Web Audio API, and WGPU support.
 */
export default defineConfig({
  // Test directory structure
  testDir: './tests/e2e',
  testMatch: '**/*.spec.ts',

  // Parallel execution settings
  fullyParallel: false, // Sequential for audio tests to avoid device conflicts
  workers: process.env.CI ? 1 : 2, // Limit workers to avoid audio device conflicts

  // Retry configuration
  retries: process.env.CI ? 2 : 1,

  // Test timeout (WASM compilation can take time)
  timeout: 60000, // 60 seconds per test
  expect: {
    timeout: 10000, // 10 seconds for assertions
  },

  // Global setup/teardown
  globalSetup: './tests/helpers/global-setup.ts',
  globalTeardown: './tests/helpers/global-teardown.ts',

  // Reporter configuration
  reporter: [
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
    ['json', { outputFile: 'playwright-report/results.json' }],
    ['junit', { outputFile: 'playwright-report/junit.xml' }],
    ['list'],
    // Custom reporter for performance metrics
    ['./tests/helpers/performance-reporter.ts'],
  ],

  // Shared settings for all projects
  use: {
    // Base URL for tests
    baseURL: process.env.TEST_URL || 'http://localhost:8080',

    // Trace collection
    trace: process.env.CI ? 'on-first-retry' : 'retain-on-failure',

    // Screenshot on failure
    screenshot: 'only-on-failure',

    // Video recording
    video: process.env.CI ? 'retain-on-failure' : 'off',

    // Navigation timeout (WASM download can be slow)
    navigationTimeout: 30000,

    // Action timeout
    actionTimeout: 10000,

    // Viewport size
    viewport: { width: 1920, height: 1080 },

    // Ignore HTTPS errors for local testing
    ignoreHTTPSErrors: true,

    // Additional context options
    permissions: ['microphone', 'audio-capture'],

    // Headers for cross-origin isolation (SharedArrayBuffer requirement)
    extraHTTPHeaders: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },

  // Browser configurations
  projects: [
    // ===== Chromium (Primary Target) =====
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        launchOptions: {
          args: [
            '--enable-features=SharedArrayBuffer',
            '--enable-unsafe-webgpu', // For WGPU testing
            '--disable-web-security', // For local CORS testing
            '--use-fake-ui-for-media-stream', // Auto-allow audio permissions
            '--use-fake-device-for-media-stream', // Use fake audio devices
            '--autoplay-policy=no-user-gesture-required',
          ],
        },
        contextOptions: {
          permissions: ['microphone', 'audio-capture'],
        },
      },
    },

    // ===== Chromium with Threading Disabled (Fallback Test) =====
    {
      name: 'chromium-no-threading',
      use: {
        ...devices['Desktop Chrome'],
        launchOptions: {
          args: [
            '--disable-features=SharedArrayBuffer',
            '--use-fake-ui-for-media-stream',
            '--use-fake-device-for-media-stream',
            '--autoplay-policy=no-user-gesture-required',
          ],
        },
      },
    },

    // ===== Firefox =====
    {
      name: 'firefox',
      use: {
        ...devices['Desktop Firefox'],
        launchOptions: {
          firefoxUserPrefs: {
            'dom.postMessage.sharedArrayBuffer.bypassCOOP_COEP.insecure.enabled': true,
            'media.navigator.permission.disabled': true,
            'media.navigator.streams.fake': true,
            'media.autoplay.default': 0,
          },
        },
        contextOptions: {
          permissions: ['microphone'],
        },
      },
    },

    // ===== WebKit (Safari) =====
    {
      name: 'webkit',
      use: {
        ...devices['Desktop Safari'],
        launchOptions: {
          // Safari has limited SharedArrayBuffer support
          // Testing graceful fallback
        },
        contextOptions: {
          permissions: ['microphone'],
        },
      },
    },

    // ===== Mobile Chrome (Responsive Testing) =====
    {
      name: 'mobile-chrome',
      use: {
        ...devices['Pixel 5'],
        launchOptions: {
          args: [
            '--enable-features=SharedArrayBuffer',
            '--use-fake-ui-for-media-stream',
            '--use-fake-device-for-media-stream',
            '--autoplay-policy=no-user-gesture-required',
          ],
        },
      },
    },

    // ===== Performance Profiling (Chromium with DevTools) =====
    {
      name: 'chromium-profiling',
      use: {
        ...devices['Desktop Chrome'],
        launchOptions: {
          args: [
            '--enable-features=SharedArrayBuffer',
            '--enable-unsafe-webgpu',
            '--use-fake-ui-for-media-stream',
            '--use-fake-device-for-media-stream',
            '--autoplay-policy=no-user-gesture-required',
            '--enable-precise-memory-info', // For memory profiling
          ],
        },
        // Enable DevTools protocol for performance metrics
        devtools: false,
      },
      testMatch: '**/performance.spec.ts',
    },
  ],

  // Web server configuration
  webServer: {
    command: 'npx http-server dist -p 8080 --cors -c-1 --gzip -P http://localhost:8080? -H "Cross-Origin-Opener-Policy: same-origin" -H "Cross-Origin-Embedder-Policy: require-corp"',
    port: 8080,
    reuseExistingServer: !process.env.CI,
    timeout: 120000, // 2 minutes for server startup
    stdout: 'pipe',
    stderr: 'pipe',
  },

  // Output directory
  outputDir: 'test-results',

  // Preserve output on failure
  preserveOutput: 'failures-only',

  // Maximum failures before stopping
  maxFailures: process.env.CI ? undefined : 5,
});
