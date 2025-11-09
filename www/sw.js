// Service Worker for Rusty Audio PWA
// Provides offline support, caching, and performance optimization

const CACHE_NAME = 'rusty-audio-v1.0.0';
const RUNTIME_CACHE = 'rusty-audio-runtime';
const WASM_CACHE = 'rusty-audio-wasm';
const AUDIO_CACHE = 'rusty-audio-files';

// Files to cache on install
const STATIC_ASSETS = [
    './',
    './index.html',
    './manifest.json',
    './icon-72.png',
    './icon-96.png',
    './icon-128.png',
    './icon-144.png',
    './icon-152.png',
    './icon-192.png',
    './icon-384.png',
    './icon-512.png',
    './favicon-32x32.png',
    './favicon-16x16.png',
];

// WASM files (cached separately for better control)
const WASM_FILES = [
    './rusty_audio.js',
    './rusty_audio_bg.wasm',
];

// Maximum cache sizes (in bytes)
const MAX_AUDIO_CACHE_SIZE = 100 * 1024 * 1024; // 100MB for audio files
const MAX_RUNTIME_CACHE_SIZE = 50 * 1024 * 1024; // 50MB for runtime assets

// Install event - cache static assets
self.addEventListener('install', (event) => {
    console.log('[SW] Installing service worker...');

    event.waitUntil(
        Promise.all([
            // Cache static assets
            caches.open(CACHE_NAME).then((cache) => {
                console.log('[SW] Caching static assets');
                return cache.addAll(STATIC_ASSETS);
            }),
            // Cache WASM files
            caches.open(WASM_CACHE).then((cache) => {
                console.log('[SW] Caching WASM files');
                return cache.addAll(WASM_FILES).catch(err => {
                    console.warn('[SW] Failed to cache WASM files (may not exist yet):', err);
                });
            }),
        ]).then(() => {
            console.log('[SW] Installation complete');
            return self.skipWaiting();
        })
    );
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
    console.log('[SW] Activating service worker...');

    event.waitUntil(
        caches.keys().then((cacheNames) => {
            return Promise.all(
                cacheNames.map((cacheName) => {
                    if (cacheName !== CACHE_NAME &&
                        cacheName !== RUNTIME_CACHE &&
                        cacheName !== WASM_CACHE &&
                        cacheName !== AUDIO_CACHE) {
                        console.log('[SW] Deleting old cache:', cacheName);
                        return caches.delete(cacheName);
                    }
                })
            );
        }).then(() => {
            console.log('[SW] Activation complete');
            return self.clients.claim();
        })
    );
});

// Fetch event - serve from cache, fallback to network
self.addEventListener('fetch', (event) => {
    const { request } = event;
    const url = new URL(request.url);

    // Skip cross-origin requests
    if (url.origin !== location.origin) {
        return;
    }

    // Handle different resource types
    if (isWasmRequest(request)) {
        event.respondWith(handleWasmRequest(request));
    } else if (isAudioRequest(request)) {
        event.respondWith(handleAudioRequest(request));
    } else if (isStaticAsset(request)) {
        event.respondWith(handleStaticRequest(request));
    } else {
        event.respondWith(handleRuntimeRequest(request));
    }
});

// WASM request handler - cache first, then network
async function handleWasmRequest(request) {
    const cache = await caches.open(WASM_CACHE);

    // Try cache first
    const cachedResponse = await cache.match(request);
    if (cachedResponse) {
        console.log('[SW] Serving WASM from cache:', request.url);
        return cachedResponse;
    }

    // Fetch from network and cache
    try {
        console.log('[SW] Fetching WASM from network:', request.url);
        const response = await fetch(request);

        if (response.ok) {
            cache.put(request, response.clone());
        }

        return response;
    } catch (error) {
        console.error('[SW] WASM fetch failed:', error);
        throw error;
    }
}

// Audio file request handler - cache with size limits
async function handleAudioRequest(request) {
    const cache = await caches.open(AUDIO_CACHE);

    // Try cache first
    const cachedResponse = await cache.match(request);
    if (cachedResponse) {
        console.log('[SW] Serving audio from cache:', request.url);
        return cachedResponse;
    }

    // Fetch from network
    try {
        console.log('[SW] Fetching audio from network:', request.url);
        const response = await fetch(request);

        if (response.ok) {
            // Check cache size before adding
            const contentLength = response.headers.get('content-length');
            if (contentLength && parseInt(contentLength) < MAX_AUDIO_CACHE_SIZE) {
                cache.put(request, response.clone());
                await trimCache(AUDIO_CACHE, MAX_AUDIO_CACHE_SIZE);
            }
        }

        return response;
    } catch (error) {
        console.error('[SW] Audio fetch failed:', error);
        return new Response('Audio file not available offline', {
            status: 503,
            statusText: 'Service Unavailable',
        });
    }
}

// Static asset handler - cache first, fallback to network
async function handleStaticRequest(request) {
    const cache = await caches.open(CACHE_NAME);

    // Try cache first
    const cachedResponse = await cache.match(request);
    if (cachedResponse) {
        return cachedResponse;
    }

    // Fetch from network and cache
    try {
        const response = await fetch(request);

        if (response.ok) {
            cache.put(request, response.clone());
        }

        return response;
    } catch (error) {
        console.error('[SW] Static asset fetch failed:', error);
        // Return offline page if available
        return cache.match('./index.html');
    }
}

// Runtime request handler - network first, fallback to cache
async function handleRuntimeRequest(request) {
    const cache = await caches.open(RUNTIME_CACHE);

    try {
        const response = await fetch(request);

        if (response.ok) {
            cache.put(request, response.clone());
            await trimCache(RUNTIME_CACHE, MAX_RUNTIME_CACHE_SIZE);
        }

        return response;
    } catch (error) {
        const cachedResponse = await cache.match(request);
        if (cachedResponse) {
            return cachedResponse;
        }

        throw error;
    }
}

// Helper functions
function isWasmRequest(request) {
    const url = new URL(request.url);
    return url.pathname.endsWith('.wasm') || url.pathname.endsWith('_bg.js');
}

function isAudioRequest(request) {
    const url = new URL(request.url);
    const audioExtensions = ['.mp3', '.wav', '.flac', '.ogg', '.m4a', '.aac'];
    return audioExtensions.some(ext => url.pathname.endsWith(ext));
}

function isStaticAsset(request) {
    const url = new URL(request.url);
    const staticExtensions = ['.html', '.css', '.js', '.png', '.jpg', '.svg', '.ico', '.json'];
    return staticExtensions.some(ext => url.pathname.endsWith(ext));
}

// Trim cache to stay under size limit (LRU eviction)
async function trimCache(cacheName, maxSize) {
    const cache = await caches.open(cacheName);
    const keys = await cache.keys();

    let totalSize = 0;
    const entries = [];

    // Calculate total size
    for (const request of keys) {
        const response = await cache.match(request);
        const size = parseInt(response.headers.get('content-length') || '0');
        entries.push({ request, size, timestamp: Date.now() });
        totalSize += size;
    }

    // Remove oldest entries if over limit
    if (totalSize > maxSize) {
        entries.sort((a, b) => a.timestamp - b.timestamp);

        for (const entry of entries) {
            if (totalSize <= maxSize) break;

            await cache.delete(entry.request);
            totalSize -= entry.size;
            console.log('[SW] Evicted from cache:', entry.request.url);
        }
    }
}

// Message handler for cache management
self.addEventListener('message', (event) => {
    if (event.data.type === 'SKIP_WAITING') {
        self.skipWaiting();
    } else if (event.data.type === 'CLEAR_CACHE') {
        event.waitUntil(
            caches.keys().then((cacheNames) => {
                return Promise.all(
                    cacheNames.map((cacheName) => caches.delete(cacheName))
                );
            }).then(() => {
                console.log('[SW] All caches cleared');
                event.ports[0].postMessage({ success: true });
            })
        );
    } else if (event.data.type === 'CACHE_SIZE') {
        event.waitUntil(
            getCacheSize().then((size) => {
                event.ports[0].postMessage({ size });
            })
        );
    }
});

// Get total cache size
async function getCacheSize() {
    const cacheNames = await caches.keys();
    let totalSize = 0;

    for (const cacheName of cacheNames) {
        const cache = await caches.open(cacheName);
        const keys = await cache.keys();

        for (const request of keys) {
            const response = await cache.match(request);
            const size = parseInt(response.headers.get('content-length') || '0');
            totalSize += size;
        }
    }

    return totalSize;
}

// Background sync for offline actions
self.addEventListener('sync', (event) => {
    if (event.tag === 'sync-audio-metadata') {
        event.waitUntil(syncAudioMetadata());
    }
});

async function syncAudioMetadata() {
    console.log('[SW] Syncing audio metadata...');
    // Implement metadata sync logic here
}

// Push notifications (future feature)
self.addEventListener('push', (event) => {
    const options = {
        body: event.data ? event.data.text() : 'New notification',
        icon: './icon-192.png',
        badge: './icon-96.png',
        vibrate: [200, 100, 200],
    };

    event.waitUntil(
        self.registration.showNotification('Rusty Audio', options)
    );
});

console.log('[SW] Service worker script loaded');
