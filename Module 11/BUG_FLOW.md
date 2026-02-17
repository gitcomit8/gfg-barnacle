# Bug Flow Diagram

## Timeline of Events

```
PARSE TIME (Service Worker file first loaded)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
│
├─ let CURRENT_CACHE = 'pwa-cache-v1'  ← Variable initialized
├─ const CACHE_VERSION = 'v1'
├─ Event listeners registered with closures over CURRENT_CACHE
│
└─ Service Worker ready

RUNTIME - FIRST INSTALL (v1 deployment)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
│
├─ INSTALL event fires
│  └─ Creates cache: pwa-cache-v1 ✓
│
├─ ACTIVATE event fires
│  └─ CURRENT_CACHE = 'pwa-cache-v1' (no change)
│
├─ FETCH events fire
│  └─ Opens cache: CURRENT_CACHE ('pwa-cache-v1') ✓
│
└─ Everything works! ✓

RUNTIME - VERSION UPDATE (v2 deployment)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
│
├─ Developer changes: CACHE_VERSION = 'v2'
│  (but CURRENT_CACHE still 'pwa-cache-v1' at parse time)
│
├─ Browser detects Service Worker change
│
├─ INSTALL event fires (new worker)
│  └─ Creates cache: pwa-cache-v2 ✓
│
├─ ACTIVATE event fires
│  ├─ Deletes old cache: pwa-cache-v1 ✓
│  ├─ Sets: CURRENT_CACHE = 'pwa-cache-v2'
│  └─ Claims clients ✓
│
├─ FETCH events fire  ⚠️ BUG OCCURS HERE
│  ├─ Closure still references OLD CURRENT_CACHE value!
│  ├─ Opens cache: CURRENT_CACHE ('pwa-cache-v1') ✗
│  ├─ Cache not found (was deleted in activate)
│  ├─ Falls back to network every time
│  └─ Assets never served from cache ✗
│
└─ Users experience broken caching! ✗

AFTER HARD RELOAD (Ctrl+Shift+R)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
│
├─ Service Worker file RE-PARSED
│  └─ CURRENT_CACHE = 'pwa-cache-v1' (but v2 exists)
│
├─ New closures created with updated values
│
├─ FETCH events fire
│  └─ Opens cache: CURRENT_CACHE ('pwa-cache-v1') 
│     (but pwa-cache-v2 exists, so still broken)
│
└─ Still needs proper fix! ✗
```

## Variable State Tracking

```
┌─────────────────────┬───────────────────┬────────────────────┬─────────────────┐
│      Phase          │  CACHE_VERSION    │   CURRENT_CACHE    │  Actual Cache   │
├─────────────────────┼───────────────────┼────────────────────┼─────────────────┤
│ Parse (v1)          │       'v1'        │   'pwa-cache-v1'   │      none       │
│ Install (v1)        │       'v1'        │   'pwa-cache-v1'   │  pwa-cache-v1   │
│ Activate (v1)       │       'v1'        │   'pwa-cache-v1'   │  pwa-cache-v1   │
│ Fetch (v1)          │       'v1'        │   'pwa-cache-v1'   │  pwa-cache-v1   │ ✓ Match
├─────────────────────┼───────────────────┼────────────────────┼─────────────────┤
│ Code Change → v2    │       'v2'        │   'pwa-cache-v1'   │  pwa-cache-v1   │
│ Install (v2)        │       'v2'        │   'pwa-cache-v1'   │  pwa-cache-v2   │
│ Activate (v2)       │       'v2'        │   'pwa-cache-v2'*  │  pwa-cache-v2   │
│ Fetch (v2)          │       'v2'        │   'pwa-cache-v1'** │  pwa-cache-v2   │ ✗ Mismatch!
└─────────────────────┴───────────────────┴────────────────────┴─────────────────┘

* Variable updated in activate
** But fetch closure still has old value!
```

## Code Flow with Closure Problem

```javascript
// AT PARSE TIME:
const CACHE_VERSION = 'v1';
let CURRENT_CACHE = `pwa-cache-${CACHE_VERSION}`;  // ← Evaluates to 'pwa-cache-v1', captured in closure

self.addEventListener('fetch', (event) => {
  //                           ↓ Closure captures 'pwa-cache-v1'
  event.respondWith(
    caches.open(CURRENT_CACHE).then((cache) => {
      // ... serve from cache
    })
  );
});

// WHEN CACHE_VERSION CHANGES TO 'v2' AND SERVICE WORKER UPDATES:
// CURRENT_CACHE variable still evaluates to 'pwa-cache-v1' until file is re-parsed

// LATER IN ACTIVATE:
CURRENT_CACHE = 'pwa-cache-v2';  // ← Variable updated

// BUT THE FETCH LISTENER CLOSURE STILL HAS 'pwa-cache-v1'
// The closure was created at parse time with the old value!
```

## The Fix

```javascript
// OPTION 1: Calculate dynamically (no closure capture)
self.addEventListener('fetch', (event) => {
  const cacheName = `pwa-cache-${CACHE_VERSION}`;  // ← Fresh calculation
  event.respondWith(
    caches.open(cacheName).then((cache) => {
      // ... serve from cache
    })
  );
});

// OPTION 2: Use a constant (immutable, always correct)
const CACHE_NAME = `pwa-cache-${CACHE_VERSION}`;  // ← Set once, use everywhere

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {  // ← Uses constant
      return cache.addAll(ASSETS_TO_CACHE);
    })
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheName !== CACHE_NAME) {  // ← Uses constant
            return caches.delete(cacheName);
          }
        })
      );
    })
  );
});

self.addEventListener('fetch', (event) => {
  event.respondWith(
    caches.open(CACHE_NAME).then((cache) => {  // ← Uses constant
      // ... serve from cache
    })
  );
});
```

## Key Insights

1. **Closures capture variables at creation time**, not at execution time
2. **Service Workers persist** across page reloads, so closures persist
3. **Mutable variables** (`let`) can be updated but closures keep old reference
4. **Constants** (`const`) or **dynamic calculation** solve the problem
5. **The bug is subtle** because it works initially and breaks only after updates

## Real-World Analogy

It's like giving someone a key to "Room 101" in a hotel:
- The room number on the key is written in ink (closure)
- Later, you tell them "Actually, your room is 102" (variable update)
- But they still use the key that says "101" (old closure)
- They can't get into their room because it doesn't match (cache mismatch)

The fix: Give them a new key (recalculate) or give them a master key formula (constant).
