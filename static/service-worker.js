// Simple offline-first service worker for Rusty Audio
const CACHE_NAME = "rusty-audio-v1";
const CORE_ASSETS = [
  "/",               // index.html
  "/index.html",
  "/manifest.webmanifest",
  "/service-worker.js",
  // Trunk outputs (filehash disabled for stability):
  "/rusty-audio.js",
  "/rusty-audio_bg.wasm",
  // Assets and icons:
  "/icons/icon-192.png",
  "/icons/icon-512.png"
];

self.addEventListener("install", event => {
  self.skipWaiting();
  event.waitUntil(
    caches.open(CACHE_NAME).then(cache => cache.addAll(CORE_ASSETS))
  );
});

self.addEventListener("activate", event => {
  event.waitUntil(
    caches.keys().then(keys => Promise.all(
      keys.filter(k => k !== CACHE_NAME).map(k => caches.delete(k))
    )).then(() => self.clients.claim())
  );
});

self.addEventListener("fetch", event => {
  const req = event.request;
  const url = new URL(req.url);

  // Network-first for HTML (ensures updates), fallback to cache
  if (req.mode === "navigate" || (req.headers.get("accept") || "").includes("text/html")) {
    event.respondWith(
      fetch(req).then(resp => {
        const copy = resp.clone();
        caches.open(CACHE_NAME).then(c => c.put("/index.html", copy)).catch(()=>{});
        return resp;
      }).catch(() => caches.match("/index.html"))
    );
    return;
  }

  // Cache-first for WASM/JS/CSS and static assets
  if (/\.(wasm|js|css|png|jpg|jpeg|svg|woff2?)$/i.test(url.pathname)) {
    event.respondWith(
      caches.match(req).then(cached => cached || fetch(req).then(resp => {
        const copy = resp.clone();
        caches.open(CACHE_NAME).then(c => c.put(req, copy)).catch(()=>{});
        return resp;
      }))
    );
    return;
  }

  // Default: try network then cache
  event.respondWith(
    fetch(req).catch(() => caches.match(req))
  );
});
