# Module 11: Zombified Service Worker (PWA/Caching)

## 🧟 Overview

This module demonstrates a **subtle but critical bug** in Service Worker caching logic that can cause a PWA to serve stale assets indefinitely in production builds.

## 🐛 The Bug

The Service Worker implements a common pattern for cache versioning and cleanup, but contains a critical flaw:

### What Happens:

1. **On Installation**: The Service Worker caches assets with a version tag (e.g., `pwa-cache-v1`)
2. **On Activation**: Old caches are properly detected and deleted
3. **On Fetch**: The Service Worker serves assets from cache

### The Problem:

The `CURRENT_CACHE` variable is initialized at **parse time** (when the Service Worker file is first loaded), not at runtime:

```javascript
// BUG: This is set when file is parsed, not when events fire
let CURRENT_CACHE = 'pwa-cache-v1';
```

Even though the `activate` event correctly updates this variable to match the new cache version, the **fetch event handler already has a closure over the old value**. This means:

- ✅ Development seems to work fine (cache names match initially)
- ❌ Production deployments serve old cached assets indefinitely
- 🔄 Only a hard browser reload (Ctrl+Shift+R) forces the Service Worker to re-parse and get the new value

## 🎯 Learning Objectives

Participants debugging this module will learn about:

1. **Service Worker Lifecycle**: Understanding install, activate, and fetch events
2. **JavaScript Closures**: How closures can capture stale values
3. **Cache Invalidation**: Proper patterns for cache versioning
4. **PWA Debugging**: Using DevTools to inspect Service Workers and caches
5. **Variable Scope**: The difference between parse-time and runtime initialization

## 📁 Module Structure

```
Module 11/
├── index.html          # Main HTML file with UI
├── app.js             # Application logic and UI controls
├── styles.css         # Styling for the demo
├── service-worker.js  # The buggy Service Worker
├── manifest.json      # PWA manifest
├── logo.svg           # App icon (SVG format)
└── README.md          # This file
```

## 🚀 How to Use

### Running Locally

1. **Serve the files** using any local web server (Service Workers require HTTPS or localhost):
   ```bash
   # Using Python
   cd "Module 11"
   python3 -m http.server 8000
   
   # Using Node.js (http-server)
   npx http-server -p 8000
   
   # Using PHP
   php -S localhost:8000
   ```

2. **Open browser** at `http://localhost:8000`

3. **Interact with the app**:
   - Click "Check Current Cache" to see cached resources
   - Click "Simulate Version Update" to trigger the bug
   - Click "Clear All Caches" to reset
   - Click "Unregister Service Worker" to start fresh

### Testing the Bug

1. **Initial Load**: 
   - Open the app, Service Worker installs with `pwa-cache-v1`
   - Assets are cached correctly

2. **Simulate Production Deployment**:
   - Modify `CACHE_VERSION` in `service-worker.js` to `v2`
   - The activate event will run and delete `pwa-cache-v1`
   - The activate event will create `pwa-cache-v2`
   - **BUG**: Fetch events still try to open `pwa-cache-v1` (the stale value)

3. **Observe the Problem**:
   - Assets won't be found in the "current" cache
   - Service Worker will fetch from network but cache in wrong location
   - Old assets remain served until hard reload

## 🔧 The Fix

To fix this bug, the cache name should be determined **at fetch time**, not at parse time:

### Option 1: Calculate cache name dynamically
```javascript
self.addEventListener('fetch', (event) => {
  const expectedCacheName = `pwa-cache-${CACHE_VERSION}`;
  event.respondWith(
    caches.open(expectedCacheName).then((cache) => {
      // ... rest of logic
    })
  );
});
```

### Option 2: Use a constant
```javascript
const CACHE_NAME = `pwa-cache-${CACHE_VERSION}`;
// Use CACHE_NAME everywhere instead of CURRENT_CACHE
```

### Option 3: Remove the variable altogether
```javascript
// Reference CACHE_VERSION directly in all event handlers
self.addEventListener('fetch', (event) => {
  event.respondWith(
    caches.open(`pwa-cache-${CACHE_VERSION}`).then((cache) => {
      // ... rest of logic
    })
  );
});
```

## 🎓 Discussion Points

### Why This Bug is Subtle

1. **Works in Development**: When you update the code locally and refresh, the browser re-parses the Service Worker file, getting the new value
2. **Fails in Production**: In production, the Service Worker persists across updates until explicitly updated
3. **Hard to Debug**: Cache operations appear to succeed, but they're operating on the wrong cache
4. **Common Pattern**: Many tutorials show similar patterns without explaining the pitfall

### Real-World Impact

This type of bug can cause:
- Users stuck on old versions of your app
- Confusion about why updates aren't deploying
- Need for manual cache clearing instructions
- Potential data inconsistencies if APIs change

## 🔍 Debugging Tips

1. **Chrome DevTools**:
   - Application tab → Service Workers (see active workers)
   - Application tab → Cache Storage (inspect cache contents)
   - Network tab → Disable cache to test without Service Worker

2. **Console Logs**: The Service Worker includes extensive logging to help track cache operations

3. **Hard Reload**: Use Ctrl+Shift+R (Cmd+Shift+R on Mac) to bypass cache

4. **Incognito Mode**: Test with a fresh browser state

## 📚 Additional Resources

- [Service Worker API - MDN](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API)
- [The Service Worker Lifecycle](https://web.dev/service-worker-lifecycle/)
- [Workbox - Production-Ready Service Worker Libraries](https://developers.google.com/web/tools/workbox)

## ⚠️ Note for Participants

This module intentionally contains a bug. Your goal is to:
1. Understand how the bug manifests
2. Identify the root cause
3. Implement a proper fix
4. Ensure cache invalidation works correctly

Good luck debugging! 🐛🔍
