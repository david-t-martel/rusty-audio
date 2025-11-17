// Offline-first service worker for Rusty Audio with WASM threading support
const CACHE_NAME = "rusty-audio-v3";
const CORE_ASSETS = [
  "/",               // index.html
  "/index.html",
  "/manifest.webmanifest",
  "/service-worker.js",
  // Trunk outputs (filehash disabled for stability):
  "/rusty-audio.js",
  "/rusty-audio_bg.wasm",
  // Static scripts
  "/static/wasm-worker-init.js",
  "/static/rusty-audio-init.js",
  // Assets and icons:
  "/icons/icon-192.png",
  "/icons/icon-512.png"
];

// Cross-origin isolation headers for SharedArrayBuffer support
const COOP_COEP_HEADERS = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
  'Cross-Origin-Resource-Policy': 'cross-origin'
};

// Add COOP/COEP headers to response
function addCrossOriginHeaders(response) {
  const headers = new Headers(response.headers);

  // Add cross-origin isolation headers
  Object.entries(COOP_COEP_HEADERS).forEach(([key, value]) => {
    headers.set(key, value);
  });

  // Preserve other important headers
  if (!headers.has('Cache-Control')) {
    if (response.url.includes('.wasm') || response.url.includes('.js')) {
      headers.set('Cache-Control', 'public, max-age=31536000');
    }
  }

  // Set proper content types
  if (response.url.endsWith('.wasm')) {
    headers.set('Content-Type', 'application/wasm');
  } else if (response.url.endsWith('.js')) {
    headers.set('Content-Type', 'application/javascript');
  } else if (response.url.endsWith('.json') || response.url.endsWith('.webmanifest')) {
    headers.set('Content-Type', 'application/json');
  }

  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers
  });
}

self.addEventListener("install", event => {
  console.log('[Service Worker] Installing...');
  self.skipWaiting();
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => {
        console.log('[Service Worker] Caching core assets');
        return cache.addAll(CORE_ASSETS);
      })
      .catch(err => {
        console.error('[Service Worker] Cache installation failed:', err);
      })
  );
});

self.addEventListener("activate", event => {
  console.log('[Service Worker] Activating...');
  event.waitUntil(
    caches.keys().then(keys => {
      console.log('[Service Worker] Cleaning old caches');
      return Promise.all(
        keys.filter(k => k !== CACHE_NAME).map(k => {
          console.log(`[Service Worker] Deleting cache: ${k}`);
          return caches.delete(k);
        })
      );
    }).then(() => {
      console.log('[Service Worker] Claiming clients');
      return self.clients.claim();
    })
  );
});

self.addEventListener("fetch", event => {
  const req = event.request;
  const url = new URL(req.url);

  // Skip non-GET requests
  if (req.method !== 'GET') {
    return;
  }

  // Skip chrome-extension and other non-http(s) requests
  if (!url.protocol.startsWith('http')) {
    return;
  }

  // Network-first for HTML (ensures updates), fallback to cache
  if (req.mode === "navigate" || (req.headers.get("accept") || "").includes("text/html")) {
    event.respondWith(
      fetch(req)
        .then(resp => {
          // Add cross-origin headers to enable SharedArrayBuffer
          const newResp = addCrossOriginHeaders(resp);
          // Cache the response
          const copy = newResp.clone();
          caches.open(CACHE_NAME)
            .then(c => c.put("/index.html", copy))
            .catch(err => console.error('[Service Worker] Cache put failed:', err));
          return newResp;
        })
        .catch(() => {
          console.log('[Service Worker] Network failed, serving cached HTML');
          return caches.match("/index.html")
            .then(cached => cached ? addCrossOriginHeaders(cached) : cached);
        })
    );
    return;
  }

  // Cache-first for WASM/JS and static assets with proper headers
  if (/\.(wasm|js|css|png|jpg|jpeg|svg|woff2?|webmanifest)$/i.test(url.pathname)) {
    event.respondWith(
      caches.match(req)
        .then(cached => {
          if (cached) {
            // Add headers to cached response
            return addCrossOriginHeaders(cached);
          }

          // Not in cache, fetch from network
          return fetch(req)
            .then(resp => {
              // Add cross-origin headers
              const newResp = addCrossOriginHeaders(resp);
              // Cache for future use
              const copy = newResp.clone();
              caches.open(CACHE_NAME)
                .then(c => c.put(req, copy))
                .catch(err => console.error('[Service Worker] Cache put failed:', err));
              return newResp;
            });
        })
    );
    return;
  }

  // Default: try network then cache (with headers)
  event.respondWith(
    fetch(req)
      .then(resp => addCrossOriginHeaders(resp))
      .catch(() => {
        return caches.match(req)
          .then(cached => cached ? addCrossOriginHeaders(cached) : cached);
      })
  );
});

// Handle messages from the main thread
self.addEventListener('message', event => {
  if (!event.data) return;

  switch (event.data.type) {
    case 'SKIP_WAITING':
      console.log('[Service Worker] Received SKIP_WAITING message');
      self.skipWaiting();
      break;

    case 'CLEAR_CACHE':
      console.log('[Service Worker] Clearing cache');
      caches.delete(CACHE_NAME)
        .then(() => {
          console.log('[Service Worker] Cache cleared');
          event.ports[0]?.postMessage({ success: true });
        })
        .catch(err => {
          console.error('[Service Worker] Failed to clear cache:', err);
          event.ports[0]?.postMessage({ success: false, error: err.message });
        });
      break;

    case 'GET_CACHE_SIZE':
      caches.open(CACHE_NAME)
        .then(cache => cache.keys())
        .then(keys => {
          console.log(`[Service Worker] Cache contains ${keys.length} items`);
          event.ports[0]?.postMessage({ size: keys.length, items: keys.map(k => k.url) });
        })
        .catch(err => {
          console.error('[Service Worker] Failed to get cache size:', err);
          event.ports[0]?.postMessage({ error: err.message });
        });
      break;

    default:
      console.log('[Service Worker] Unknown message type:', event.data.type);
  }
});

// Performance monitoring
let requestCount = 0;
let cacheHits = 0;
let cacheMisses = 0;

setInterval(() => {
  if (requestCount > 0) {
    const hitRate = ((cacheHits / requestCount) * 100).toFixed(1);
    console.log(`[Service Worker] Performance: ${requestCount} requests, ${hitRate}% cache hit rate`);
    requestCount = 0;
    cacheHits = 0;
    cacheMisses = 0;
  }
}, 60000); // Log every minute

// Track cache performance
function trackCacheHit() {
  requestCount++;
  cacheHits++;
}

function trackCacheMiss() {
  requestCount++;
  cacheMisses++;
}

// Update fetch handlers to track performance
const originalFetchHandler = self.addEventListener;

console.log('[Service Worker] Script loaded with performance monitoring');
