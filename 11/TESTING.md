# Testing the Zombified Service Worker Bug

## Test Scenario: Reproducing the Cache Invalidation Bug

### Prerequisites
- A local web server (Python, Node.js, or any HTTP server)
- A modern browser with DevTools (Chrome/Edge recommended)
- The Module 11 files

### Step-by-Step Bug Reproduction

#### Phase 1: Initial Setup (Everything Works)

1. **Start the web server**
   ```bash
   cd "Module 11"
   python3 -m http.server 8000
   ```

2. **Open in browser**
   - Navigate to `http://localhost:8000`
   - Open DevTools (F12)
   - Go to Application tab → Service Workers

3. **Verify initial state**
   - Service Worker should register successfully
   - Status shows "Active"
   - Console shows: `Service Worker registered successfully`

4. **Check the cache**
   - In DevTools: Application → Cache Storage
   - You should see `pwa-cache-v1` with 5 cached items
   - Click "Check Current Cache" button in UI
   - Log shows: `Current caches: pwa-cache-v1`

#### Phase 2: Simulating a Production Deployment (Bug Manifests)

5. **Update the cache version**
   - Stop the server (Ctrl+C)
   - Edit `service-worker.js`
   - Change line 9: `const CACHE_VERSION = 'v1';` → `const CACHE_VERSION = 'v2';`
   - Save the file
   - Restart the server: `python3 -m http.server 8000`

6. **Trigger Service Worker update**
   - In the browser, click "Simulate Version Update" button
   - OR: In DevTools → Application → Service Workers, click "Update"
   - Watch the console logs

7. **Observe the bug**
   ```
   Expected behavior:
   - Old cache (pwa-cache-v1) should be deleted
   - New cache (pwa-cache-v2) should be created
   - Fetch should serve from pwa-cache-v2
   
   Actual behavior (BUG):
   - Old cache (pwa-cache-v1) IS deleted ✓
   - New cache (pwa-cache-v2) IS created ✓
   - Fetch STILL tries to open pwa-cache-v1 ✗
   - Result: Assets not found, always fetching from network
   ```

8. **Verify in DevTools**
   - Application → Cache Storage: Shows `pwa-cache-v2`
   - Console shows Service Worker trying to access wrong cache
   - Network tab shows repeated network requests (no cache hits)

9. **The "zombified" effect**
   - The Service Worker is "alive" but using dead cache reference
   - New assets are fetched but stored in wrong cache or not cached
   - Users stuck with this broken state

#### Phase 3: Confirming the Hard Reload Fix

10. **Hard reload the page**
    - Press Ctrl+Shift+R (Windows/Linux) or Cmd+Shift+R (Mac)
    - OR: DevTools → Right-click refresh → Hard Reload

11. **Bug is "fixed"**
    - Service Worker file is re-parsed
    - `CURRENT_CACHE` gets new value at parse time
    - Fetch now correctly uses `pwa-cache-v2`
    - Caching works again

### Understanding the Bug with DevTools

#### Console Logs to Watch For

**When bug is active:**
```
[Service Worker] Installing...
[Service Worker] Caching assets with version: v2
[Service Worker] Activating...
[Service Worker] Deleting old cache: pwa-cache-v1
[Service Worker] Claiming clients...
[Service Worker] Fetching from network: http://localhost:8000/styles.css
[Service Worker] Fetching from network: http://localhost:8000/app.js
// Notice: Always "Fetching from network", never "Serving from cache"
```

**After hard reload (bug "fixed"):**
```
[Service Worker] Serving from cache: http://localhost:8000/styles.css
[Service Worker] Serving from cache: http://localhost:8000/app.js
// Now it serves from cache correctly
```

#### Cache Storage Inspection

1. Application → Cache Storage
2. Expand each cache name
3. Compare cache name with what Service Worker is trying to use
4. You'll see:
   - Cache exists: `pwa-cache-v2`
   - Service Worker tries: `pwa-cache-v1` (stale!)
   - Mismatch = Bug confirmed

### Debugging Techniques

#### Add Debug Logging

Add this to the fetch event (after line 64) to see what's happening:

```javascript
self.addEventListener('fetch', (event) => {
  console.log('[DEBUG] CURRENT_CACHE value:', CURRENT_CACHE);
  console.log('[DEBUG] CACHE_VERSION value:', CACHE_VERSION);
  console.log('[DEBUG] Expected cache:', `pwa-cache-${CACHE_VERSION}`);
  // ... rest of code
});
```

This will clearly show the mismatch.

#### Use Service Worker Lifecycle Events

Monitor the lifecycle:

```javascript
self.addEventListener('install', (event) => {
  console.log('[DEBUG] Install - CURRENT_CACHE:', CURRENT_CACHE);
  console.log('[DEBUG] Install - CACHE_VERSION:', CACHE_VERSION);
});

self.addEventListener('activate', (event) => {
  console.log('[DEBUG] Activate - CURRENT_CACHE (before):', CURRENT_CACHE);
  // ... activation code
  console.log('[DEBUG] Activate - CURRENT_CACHE (after):', CURRENT_CACHE);
});
```

### The Root Cause

The bug occurs because:

1. **JavaScript closures capture variables by reference** when the Service Worker file is first parsed
2. **`let CURRENT_CACHE = 'pwa-cache-v1'`** is evaluated at parse time
3. **Event listeners are registered with closures** over this initial value
4. **Updating the variable later doesn't update the closure**
5. **Service Workers persist across page reloads** so the old closure remains

### The Fix

Change the fetch event to calculate the cache name dynamically:

```javascript
// BEFORE (buggy):
self.addEventListener('fetch', (event) => {
  event.respondWith(
    caches.open(CURRENT_CACHE).then((cache) => {
      // ...
    })
  );
});

// AFTER (fixed):
self.addEventListener('fetch', (event) => {
  const expectedCacheName = `pwa-cache-${CACHE_VERSION}`;
  event.respondWith(
    caches.open(expectedCacheName).then((cache) => {
      // ...
    })
  );
});
```

Or better yet, use a constant:

```javascript
const CACHE_NAME = `pwa-cache-${CACHE_VERSION}`;
// Use CACHE_NAME everywhere, remove CURRENT_CACHE entirely
```

### Success Criteria

After fixing the bug, you should be able to:

1. Update `CACHE_VERSION` to `v3`
2. Trigger Service Worker update
3. See old cache deleted, new cache created
4. **WITHOUT hard reload**: Assets are served from new cache
5. Click "Check Current Cache" shows correct cache name
6. No repeated network requests in Network tab

### Additional Testing

- Test with slow network to see cache impact
- Test offline mode (DevTools → Network → Offline)
- Test with multiple versions (v1 → v2 → v3)
- Verify no memory leaks from old cache references

---

This test document provides a complete guide for understanding, reproducing, and fixing the Zombified Service Worker bug.
