// Rusty Audio - Enhanced WASM Initialization with Multithreading Support
// Handles progressive loading, feature detection, worker management, and performance monitoring

(function() {
  'use strict';

  // ===== DOM ELEMENTS =====
  const elements = {
    loadingOverlay: document.getElementById('loading-overlay'),
    loadingMessage: document.getElementById('loading-message'),
    loadingStatus: document.getElementById('loading-status'),
    progressFill: document.getElementById('progress-fill'),
    progressText: document.getElementById('progress-text'),
    featureDisplay: document.getElementById('feature-display'),
    featureList: document.getElementById('feature-list'),
    threadStatus: document.getElementById('thread-status'),
    threadInfo: document.getElementById('thread-info'),
    threadGrid: document.getElementById('thread-grid'),
    errorContainer: document.getElementById('error-container'),
    errorText: document.getElementById('error-text'),
    errorFeatureSupport: document.getElementById('error-feature-support'),
    perfMonitor: document.getElementById('perf-monitor'),
    canvas: document.getElementById('rusty-audio-canvas')
  };

  // ===== STATE =====
  const state = {
    features: {},
    workerPool: null,
    wasmInitialized: false,
    startTime: Date.now(),
    perfStats: {
      fps: 0,
      frameTime: 0,
      memory: 0,
      audioLatency: 0,
      lastFrameTime: 0,
      frameCount: 0
    }
  };

  // ===== LOGGING UTILITIES =====
  function log(message, data = null) {
    const timestamp = ((Date.now() - state.startTime) / 1000).toFixed(2);
    if (data) {
      console.log(`[${timestamp}s] [Rusty Audio] ${message}`, data);
    } else {
      console.log(`[${timestamp}s] [Rusty Audio] ${message}`);
    }
  }

  function error(message, err = null) {
    const timestamp = ((Date.now() - state.startTime) / 1000).toFixed(2);
    if (err) {
      console.error(`[${timestamp}s] [Rusty Audio] ${message}`, err);
    } else {
      console.error(`[${timestamp}s] [Rusty Audio] ${message}`);
    }
  }

  // ===== UI UPDATES =====
  function updateMessage(message) {
    log(message);
    elements.loadingMessage.textContent = message;
  }

  function updateStatus(status) {
    elements.loadingStatus.textContent = status;
  }

  function updateProgress(percent, text = null) {
    elements.progressFill.style.width = `${percent}%`;
    elements.progressText.textContent = text || `${Math.round(percent)}% loaded`;
  }

  function showError(message, features = null) {
    error(message);
    elements.errorText.textContent = message;

    if (features) {
      const ul = document.createElement('ul');
      ul.className = 'feature-list';

      Object.entries(features).forEach(([feature, supported]) => {
        const li = document.createElement('li');
        li.className = supported ? '' : 'unsupported';
        li.textContent = feature;
        ul.appendChild(li);
      });

      const h4 = document.createElement('h4');
      h4.textContent = 'Browser Feature Support:';
      elements.errorFeatureSupport.innerHTML = '';
      elements.errorFeatureSupport.appendChild(h4);
      elements.errorFeatureSupport.appendChild(ul);
    }

    elements.loadingOverlay.classList.add('hidden');
    elements.errorContainer.classList.add('visible');
  }

  function hideLoading() {
    setTimeout(() => {
      elements.loadingOverlay.classList.add('hidden');
    }, 500);
  }

  // ===== FEATURE DETECTION =====
  function detectFeatures() {
    updateMessage('Checking browser compatibility...');
    updateProgress(5);

    // Test WebAssembly support
    const wasmSupported = typeof WebAssembly !== 'undefined';

    // Test WebAssembly threads support (validate a minimal threaded module)
    let wasmThreadsSupported = false;
    if (wasmSupported) {
      try {
        // Minimal threaded WASM module signature
        const threadedWasmTest = new Uint8Array([
          0x00, 0x61, 0x73, 0x6d, // WASM magic
          0x01, 0x00, 0x00, 0x00  // Version 1
        ]);
        wasmThreadsSupported = WebAssembly.validate(threadedWasmTest);
      } catch (e) {
        wasmThreadsSupported = false;
      }
    }

    // Test SharedArrayBuffer
    const sabSupported = typeof SharedArrayBuffer !== 'undefined';

    // Test Atomics
    const atomicsSupported = typeof Atomics !== 'undefined';

    // Test cross-origin isolation
    const crossOriginIsolated = window.crossOriginIsolated === true;

    // Test Service Worker
    const swSupported = 'serviceWorker' in navigator;

    // Test Web Audio API
    const webAudioSupported = typeof AudioContext !== 'undefined' ||
                              typeof webkitAudioContext !== 'undefined';

    // Test WebGPU (for WGPU)
    const webGpuSupported = 'gpu' in navigator;

    // Test OffscreenCanvas
    const offscreenCanvasSupported = typeof OffscreenCanvas !== 'undefined';

    state.features = {
      'WebAssembly': wasmSupported,
      'WebAssembly Threads': wasmThreadsSupported,
      'SharedArrayBuffer': sabSupported,
      'Atomics': atomicsSupported,
      'Cross-Origin Isolation': crossOriginIsolated,
      'Service Worker': swSupported,
      'Web Audio API': webAudioSupported,
      'WebGPU': webGpuSupported,
      'OffscreenCanvas': offscreenCanvasSupported,
      'Hardware Concurrency': navigator.hardwareConcurrency ?
        `${navigator.hardwareConcurrency} cores` : 'Unknown'
    };

    log('Feature detection complete', state.features);
    updateProgress(10);

    return state.features;
  }

  function displayFeatures(features) {
    elements.featureList.innerHTML = '';

    Object.entries(features).forEach(([feature, supported]) => {
      const li = document.createElement('li');

      if (typeof supported === 'string') {
        li.textContent = `${feature}: ${supported}`;
      } else {
        li.className = supported ? '' : 'unsupported';

        // Add warning class for non-critical but recommended features
        if (!supported && (feature === 'WebGPU' || feature === 'OffscreenCanvas')) {
          li.className = 'warning';
        }

        li.textContent = feature;
      }

      elements.featureList.appendChild(li);
    });

    elements.featureDisplay.style.display = 'block';
  }

  // ===== WORKER POOL MANAGEMENT =====
  function initializeWorkerPool() {
    if (!state.features['SharedArrayBuffer']) {
      log('SharedArrayBuffer not available - skipping worker pool initialization');
      updateStatus('Running in single-threaded mode');
      return null;
    }

    updateMessage('Initializing worker thread pool...');
    updateProgress(20);

    try {
      if (typeof WasmWorkerPool === 'undefined') {
        error('WasmWorkerPool not loaded');
        return null;
      }

      const maxWorkers = navigator.hardwareConcurrency || 4;
      state.workerPool = new WasmWorkerPool({
        maxWorkers: maxWorkers,
        minWorkers: Math.min(2, maxWorkers)
      });

      log(`Worker pool created (max ${maxWorkers} workers)`);

      // Display thread status
      displayThreadStatus(maxWorkers);

      return state.workerPool;

    } catch (err) {
      error('Failed to create worker pool', err);
      return null;
    }
  }

  function displayThreadStatus(workerCount) {
    elements.threadInfo.textContent = `${workerCount} workers available`;
    elements.threadGrid.innerHTML = '';

    for (let i = 0; i < workerCount; i++) {
      const indicator = document.createElement('div');
      indicator.className = 'thread-indicator idle';
      indicator.id = `thread-${i}`;
      indicator.title = `Worker ${i}`;
      elements.threadGrid.appendChild(indicator);
    }

    elements.threadStatus.style.display = 'block';
  }

  function updateThreadIndicator(workerId, active) {
    const indicator = document.getElementById(`thread-${workerId}`);
    if (indicator) {
      indicator.className = active ? 'thread-indicator active' : 'thread-indicator idle';
    }
  }

  // ===== SERVICE WORKER REGISTRATION =====
  async function registerServiceWorker() {
    if (!state.features['Service Worker']) {
      log('Service Worker not supported');
      return;
    }

    updateMessage('Registering service worker...');
    updateProgress(15);

    try {
      const registration = await navigator.serviceWorker.register('/service-worker.js', {
        updateViaCache: 'none'
      });

      log('Service Worker registered', { scope: registration.scope });
      updateStatus('Service worker active');

      // Listen for updates
      registration.addEventListener('updatefound', () => {
        log('Service Worker update found');
        const newWorker = registration.installing;

        newWorker.addEventListener('statechange', () => {
          if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
            log('New Service Worker available - refresh recommended');
          }
        });
      });

    } catch (err) {
      error('Service Worker registration failed', err);
      updateStatus('Running without service worker');
    }
  }

  // ===== WASM LOADING WITH PROGRESS =====
  async function loadWasmWithProgress() {
    updateMessage('Downloading WASM module...');
    updateProgress(25);

    try {
      // Hook into fetch to track download progress
      const originalFetch = window.fetch;
      window.fetch = async function(...args) {
        const response = await originalFetch(...args);
        const url = args[0];

        // Track WASM file download
        if (url && (url.includes('.wasm') || url.endsWith('.wasm'))) {
          const contentLength = response.headers.get('content-length');

          if (contentLength) {
            const total = parseInt(contentLength, 10);
            const reader = response.body.getReader();
            let receivedLength = 0;
            const chunks = [];

            while (true) {
              const { done, value } = await reader.read();

              if (done) break;

              chunks.push(value);
              receivedLength += value.length;

              const progress = (receivedLength / total) * 100;
              const progressRange = 50; // 25% to 75%
              const progressOffset = 25;
              updateProgress(progressOffset + (progress * progressRange / 100));
              updateStatus(`${(receivedLength / 1024 / 1024).toFixed(2)} MB / ${(total / 1024 / 1024).toFixed(2)} MB`);
            }

            // Reconstruct the response
            const blob = new Blob(chunks);
            return new Response(blob, {
              status: response.status,
              statusText: response.statusText,
              headers: response.headers
            });
          }
        }

        return response;
      };

    } catch (err) {
      error('Failed to setup progress tracking', err);
    }
  }

  // ===== WASM INITIALIZATION =====
  async function initializeWasm() {
    updateMessage('Initializing WASM runtime...');
    updateProgress(75);

    try {
      // Wait for wasm-bindgen to be available
      let attempts = 0;
      while (typeof wasm_bindgen === 'undefined' && attempts < 50) {
        await new Promise(resolve => setTimeout(resolve, 100));
        attempts++;
      }

      if (typeof wasm_bindgen === 'undefined') {
        throw new Error('wasm-bindgen not loaded after 5 seconds');
      }

      updateStatus('Compiling WASM module...');

      // Initialize the WASM module
      await wasm_bindgen();

      state.wasmInitialized = true;
      log('WASM module initialized successfully');
      updateProgress(90);

      // Initialize worker pool with WASM module if available
      if (state.workerPool && typeof wasm_bindgen.memory !== 'undefined') {
        updateMessage('Initializing worker threads...');
        updateStatus('Setting up shared memory...');

        try {
          await state.workerPool.init(
            wasm_bindgen.module,
            wasm_bindgen.memory,
            '/rusty-audio.worker.js' // wasm-bindgen generates this
          );

          log('Worker pool initialized with WASM module');
          updateProgress(95);

        } catch (err) {
          error('Worker pool initialization failed - continuing in single-threaded mode', err);
        }
      }

      updateProgress(100);
      updateMessage('WASM ready!');

      return true;

    } catch (err) {
      error('WASM initialization failed', err);
      throw err;
    }
  }

  // ===== PERFORMANCE MONITORING =====
  function startPerformanceMonitoring() {
    let lastTime = performance.now();
    let frameCount = 0;
    let fpsUpdateTime = lastTime;

    function updatePerformanceStats() {
      const now = performance.now();
      const deltaTime = now - lastTime;
      lastTime = now;

      // Update frame time
      state.perfStats.frameTime = deltaTime;
      elements.perfMonitor.querySelector('#perf-frame-time').textContent = `${deltaTime.toFixed(1)}ms`;

      // Apply color based on frame time
      const frameTimeElement = elements.perfMonitor.querySelector('#perf-frame-time');
      if (deltaTime > 33) {
        frameTimeElement.className = 'perf-value error';
      } else if (deltaTime > 16.7) {
        frameTimeElement.className = 'perf-value warning';
      } else {
        frameTimeElement.className = 'perf-value';
      }

      // Calculate FPS every second
      frameCount++;
      if (now - fpsUpdateTime >= 1000) {
        state.perfStats.fps = frameCount;
        elements.perfMonitor.querySelector('#perf-fps').textContent = frameCount;

        const fpsElement = elements.perfMonitor.querySelector('#perf-fps');
        if (frameCount < 30) {
          fpsElement.className = 'perf-value error';
        } else if (frameCount < 55) {
          fpsElement.className = 'perf-value warning';
        } else {
          fpsElement.className = 'perf-value';
        }

        frameCount = 0;
        fpsUpdateTime = now;
      }

      // Update memory usage
      if (performance.memory) {
        const memoryMB = (performance.memory.usedJSHeapSize / 1024 / 1024).toFixed(1);
        state.perfStats.memory = memoryMB;
        elements.perfMonitor.querySelector('#perf-memory').textContent = `${memoryMB} MB`;

        const memoryElement = elements.perfMonitor.querySelector('#perf-memory');
        if (memoryMB > 500) {
          memoryElement.className = 'perf-value warning';
        } else {
          memoryElement.className = 'perf-value';
        }
      }

      // Update thread count
      if (state.workerPool) {
        const stats = state.workerPool.getStats();
        const busyWorkers = stats.busyWorkers || 0;
        const totalWorkers = stats.totalWorkers || 0;
        elements.perfMonitor.querySelector('#perf-threads').textContent =
          `${busyWorkers}/${totalWorkers}`;

        // Update thread indicators
        for (let i = 0; i < totalWorkers; i++) {
          const workerInfo = state.workerPool.workers[i];
          updateThreadIndicator(i, workerInfo && workerInfo.busy);
        }
      } else {
        elements.perfMonitor.querySelector('#perf-threads').textContent = 'N/A';
      }

      requestAnimationFrame(updatePerformanceStats);
    }

    requestAnimationFrame(updatePerformanceStats);

    // Show performance monitor after a delay
    setTimeout(() => {
      elements.perfMonitor.classList.add('visible');
    }, 2000);
  }

  // ===== KEYBOARD SHORTCUTS =====
  function setupKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
      // Toggle performance monitor with Ctrl+Shift+P
      if (e.ctrlKey && e.shiftKey && e.key === 'P') {
        e.preventDefault();
        elements.perfMonitor.classList.toggle('visible');
        log('Performance monitor toggled');
      }

      // Reload with Ctrl+Shift+R
      if (e.ctrlKey && e.shiftKey && e.key === 'R') {
        e.preventDefault();
        log('Force reload requested');
        window.location.reload(true);
      }
    });
  }

  // ===== WGPU CONTEXT HANDLING =====
  function setupWgpuErrorHandling() {
    // Listen for WebGL/WebGPU context loss
    elements.canvas.addEventListener('webglcontextlost', (e) => {
      error('WebGL context lost');
      e.preventDefault();
    });

    elements.canvas.addEventListener('webglcontextrestored', () => {
      log('WebGL context restored');
    });

    // Global error handlers for WASM
    window.addEventListener('error', (event) => {
      if (event.message && event.message.includes('wasm')) {
        error('WASM Runtime Error', event.error);
      }
    });

    window.addEventListener('unhandledrejection', (event) => {
      if (event.reason && event.reason.message && event.reason.message.includes('wasm')) {
        error('WASM Promise Rejection', event.reason);
      }
    });
  }

  // ===== MAIN INITIALIZATION SEQUENCE =====
  async function initialize() {
    try {
      log('Starting Rusty Audio initialization');

      // 1. Feature Detection
      const features = detectFeatures();
      displayFeatures(features);

      // 2. Check critical features
      if (!features['WebAssembly']) {
        showError(
          'Your browser does not support WebAssembly. Please use a modern browser like Chrome 87+, Firefox 85+, Safari 14+, or Edge 87+.',
          features
        );
        return;
      }

      // 3. Warn about missing threading support
      if (!features['SharedArrayBuffer']) {
        log('WARNING: SharedArrayBuffer not available - multithreading disabled');

        if (!features['Cross-Origin Isolation']) {
          log('Cross-Origin Isolation not enabled - threading requires HTTPS with proper COOP/COEP headers');
        }

        updateStatus('Running in single-threaded mode (limited performance)');
      }

      // 4. Register Service Worker
      await registerServiceWorker();

      // 5. Setup download progress tracking
      await loadWasmWithProgress();

      // 6. Initialize worker pool
      initializeWorkerPool();

      // 7. Initialize WASM
      await initializeWasm();

      // 8. Setup error handling
      setupWgpuErrorHandling();

      // 9. Setup keyboard shortcuts
      setupKeyboardShortcuts();

      // 10. Start performance monitoring
      startPerformanceMonitoring();

      // 11. Hide loading overlay
      updateMessage('Launching Rusty Audio...');
      setTimeout(() => {
        hideLoading();
        log('Initialization complete');
      }, 500);

    } catch (err) {
      error('Initialization failed', err);
      showError(
        `Failed to initialize: ${err.message}. Please try refreshing the page or using a different browser.`,
        state.features
      );
    }
  }

  // ===== EXPORT TO WINDOW =====
  window.rustyAudio = {
    state,
    elements,
    workerPool: state.workerPool,
    log,
    error,
    updateThreadIndicator,
    setAudioLatency: (latency) => {
      state.perfStats.audioLatency = latency;
      elements.perfMonitor.querySelector('#perf-audio-latency').textContent = `${latency.toFixed(1)}ms`;

      const latencyElement = elements.perfMonitor.querySelector('#perf-audio-latency');
      if (latency > 50) {
        latencyElement.className = 'perf-value error';
      } else if (latency > 25) {
        latencyElement.className = 'perf-value warning';
      } else {
        latencyElement.className = 'perf-value';
      }
    }
  };

  // ===== START INITIALIZATION =====
  // Wait for DOM to be ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initialize);
  } else {
    initialize();
  }

})();
