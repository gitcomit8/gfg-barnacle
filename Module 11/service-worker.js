// Zombified Service Worker - PWA Caching Module
// BUG: The activate event clears cache, but fetch uses a stale cache key

// This cache name is set at Service Worker installation time
// BUG: This variable is initialized when the file is first parsed and never updates
let CURRENT_CACHE = `pwa-cache-${CACHE_VERSION}`;

// Version that should be updated with each deployment
const CACHE_VERSION = 'v1';

// List of assets to cache
const ASSETS_TO_CACHE = [
  '/',
  '/index.html',
  '/app.js',
  '/styles.css',
  '/logo.svg'
];

// Install event - cache all assets
self.addEventListener('install', (event) => {
  console.log('[Service Worker] Installing...');
  
  event.waitUntil(
    caches.open(`pwa-cache-${CACHE_VERSION}`).then((cache) => {
      console.log('[Service Worker] Caching assets with version:', CACHE_VERSION);
      return cache.addAll(ASSETS_TO_CACHE);
    })
  );
  
  // Skip waiting to activate immediately
  self.skipWaiting();
});

// Activate event - clear old caches
self.addEventListener('activate', (event) => {
  console.log('[Service Worker] Activating...');
  
  const expectedCacheName = `pwa-cache-${CACHE_VERSION}`;
  
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheName !== expectedCacheName) {
            console.log('[Service Worker] Deleting old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    }).then(() => {
      console.log('[Service Worker] Claiming clients...');
      // BUG: Even though we update the cache name here, CURRENT_CACHE was already set
      CURRENT_CACHE = expectedCacheName;
      return self.clients.claim();
    })
  );
});

// Fetch event - serve from cache, fallback to network
// BUG: This uses the CURRENT_CACHE variable which was set at parse time
// and never updates until the browser is hard-reloaded
self.addEventListener('fetch', (event) => {
  event.respondWith(
    caches.open(CURRENT_CACHE).then((cache) => {
      return cache.match(event.request).then((response) => {
        if (response) {
          console.log('[Service Worker] Serving from cache:', event.request.url);
          return response;
        }
        
        console.log('[Service Worker] Fetching from network:', event.request.url);
        return fetch(event.request).then((networkResponse) => {
          // Cache the new response
          cache.put(event.request, networkResponse.clone());
          return networkResponse;
        });
      });
    })
  );
});

// Listen for messages to force update
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'SKIP_WAITING') {
    self.skipWaiting();
  }
});
