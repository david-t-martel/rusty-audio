// Offline-first service worker for Rusty Audio with WASM threading support (REFACTORED)
//
// P0-6 FIX: Corrected Cross-Origin-Resource-Policy header
//
// BEFORE (TOO PERMISSIVE - SECURITY RISK):
// 'Cross-Origin-Resource-Policy': 'cross-origin'  // ❌ Allows any origin!
//
// AFTER (SECURE):
// 'Cross-Origin-Resource-Policy': 'same-site'     // ✅ Only same-site access

const CACHE_NAME = "rusty-audio-v4-refactored";
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
//
// P0-6 FIX: Tightened security while maintaining functionality
//
// RATIONALE:
// - COOP: 'same-origin' prevents popup windows from accessing our window object
// - COEP: 'require-corp' requires all resources to explicitly opt-in via CORP
// - CORP: 'same-site' allows same-site subresources but blocks cross-origin
//
// SECURITY IMPACT:
// - Before: Any cross-origin site could embed our resources
// - After: Only same-site origins can access resources
// - Maintains: SharedArrayBuffer functionality for WASM threading
const COOP_COEP_HEADERS = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
  'Cross-Origin-Resource-Policy': 'same-site'  // P0-6 FIX: Changed from 'cross-origin'
};

// Cache control policies by resource type
const CACHE_POLICIES = {
  wasm: 'public, max-age=31536000, immutable',  // 1 year - WASM files are immutable
  js: 'public, max-age=31536000, immutable',    // 1 year - JS files are immutable
  html: 'no-cache, must-revalidate',             // Always check for updates
  static: 'public, max-age=86400',               // 1 day - icons, etc.
};

// Maximum cache size (entries)
const MAX_CACHE_SIZE = 100;

// Statistics tracking
let stats = {
  requests: 0,
  cacheHits: 0,
  cacheMisses: 0,
  errors: 0
};

/**
 * Add cross-origin headers to response
 *
 * P0-6 FIX: Updated to use 'same-site' CORP policy
 *
 * @param {Response} response - Original response
 * @returns {Response} Response with security headers
 */
function addCrossOriginHeaders(response) {
  const headers = new Headers(response.headers);

  // Add cross-origin isolation headers
  Object.entries(COOP_COEP_HEADERS).forEach(([key, value]) => {
    headers.set(key, value);
  });

  // Apply cache control based on resource type
  if (!headers.has('Cache-Control')) {
    const url = new URL(response.url);
    const ext = url.pathname.split('.').pop().toLowerCase();

    if (ext === 'wasm') {
      headers.set('Cache-Control', CACHE_POLICIES.wasm);
    } else if (ext === 'js') {
      headers.set('Cache-Control', CACHE_POLICIES.js);
    } else if (ext === 'html') {
      headers.set('Cache-Control', CACHE_POLICIES.html);
    } else {
      headers.set('Cache-Control', CACHE_POLICIES.static);
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

/**
 * Check if cache is over size limit and evict old entries
 *
 * P0-6 FIX: Added cache size limiting to prevent unbounded growth
 */
async function enforceCacheSizeLimit(cacheName) {
  const cache = await caches.open(cacheName);
  const keys = await cache.keys();

  if (keys.length > MAX_CACHE_SIZE) {
    const keysToDelete = keys.slice(0, keys.length - MAX_CACHE_SIZE);
    console.log(`[Service Worker] Evicting ${keysToDelete.length} old cache entries`);

    await Promise.all(keysToDelete.map(key => cache.delete(key)));
  }
}

self.addEventListener("install", event => {
  console.log('[Service Worker] Installing (refactored with P0-6 fix)...');
  self.skipWaiting();
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => {
        console.log('[Service Worker] Caching core assets');
        return cache.addAll(CORE_ASSETS);
      })
      .catch(err => {
        console.error('[Service Worker] Cache installation failed:', err);
        stats.errors++;
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

  stats.requests++;

  // Network-first for HTML (ensures updates), fallback to cache
  if (req.mode === "navigate" || (req.headers.get("accept") || "").includes("text/html")) {
    event.respondWith(
      fetch(req)
        .then(resp => {
          stats.cacheMisses++;

          // Add cross-origin headers to enable SharedArrayBuffer
          const newResp = addCrossOriginHeaders(resp);

          // Cache the response
          const copy = newResp.clone();
          caches.open(CACHE_NAME)
            .then(c => c.put("/index.html", copy))
            .then(() => enforceCacheSizeLimit(CACHE_NAME))
            .catch(err => {
              console.error('[Service Worker] Cache put failed:', err);
              stats.errors++;
            });

          return newResp;
        })
        .catch(() => {
          stats.cacheHits++;
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
            stats.cacheHits++;
            // Add headers to cached response
            return addCrossOriginHeaders(cached);
          }

          stats.cacheMisses++;

          // Not in cache, fetch from network
          return fetch(req)
            .then(resp => {
              // Add cross-origin headers
              const newResp = addCrossOriginHeaders(resp);

              // Cache for future use
              const copy = newResp.clone();
              caches.open(CACHE_NAME)
                .then(c => c.put(req, copy))
                .then(() => enforceCacheSizeLimit(CACHE_NAME))
                .catch(err => {
                  console.error('[Service Worker] Cache put failed:', err);
                  stats.errors++;
                });

              return newResp;
            })
            .catch(err => {
              console.error('[Service Worker] Fetch failed:', err);
              stats.errors++;
              throw err;
            });
        })
    );
    return;
  }

  // Default: try network then cache (with headers)
  event.respondWith(
    fetch(req)
      .then(resp => {
        stats.cacheMisses++;
        return addCrossOriginHeaders(resp);
      })
      .catch(() => {
        stats.cacheHits++;
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
          stats.errors++;
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
          stats.errors++;
          event.ports[0]?.postMessage({ error: err.message });
        });
      break;

    case 'GET_STATS':
      console.log('[Service Worker] Sending statistics');
      event.ports[0]?.postMessage({
        stats: {
          ...stats,
          cacheHitRate: stats.requests > 0 ? (stats.cacheHits / stats.requests * 100).toFixed(1) : 0
        }
      });
      break;

    default:
      console.log('[Service Worker] Unknown message type:', event.data.type);
  }
});

// Performance monitoring
setInterval(() => {
  if (stats.requests > 0) {
    const hitRate = ((stats.cacheHits / stats.requests) * 100).toFixed(1);
    console.log(
      `[Service Worker] Performance: ${stats.requests} requests, ` +
      `${hitRate}% cache hit rate, ${stats.errors} errors`
    );

    // Reset stats
    stats = {
      requests: 0,
      cacheHits: 0,
      cacheMisses: 0,
      errors: 0
    };
  }
}, 60000); // Log every minute

console.log('[Service Worker] Script loaded (refactored with P0-6 security fix)');
console.log('[Service Worker] CORS Policy: same-site (secure)');
