# Quick Start Guide - Module 11

## 🚀 Running the Module

### Option 1: Python (Recommended)
```bash
cd "Module 11"
python3 -m http.server 8000
```
Then open: http://localhost:8000

### Option 2: Node.js
```bash
cd "Module 11"
npx http-server -p 8000
```
Then open: http://localhost:8000

### Option 3: PHP
```bash
cd "Module 11"
php -S localhost:8000
```
Then open: http://localhost:8000

## 🎯 What You'll See

- A dark-themed PWA interface
- Service Worker registration status
- Interactive buttons to test caching
- Real-time activity log
- Service Worker status indicator

## 🐛 The Bug

This module contains a **deliberate bug** in the Service Worker caching logic.

**The Problem:** The Service Worker's fetch event uses a cache name that was captured at parse time, not runtime. When you update the cache version and deploy new code:

- ✅ Old caches are properly deleted
- ✅ New caches are properly created  
- ❌ Fetch still tries to use the OLD cache name
- ❌ Users get stuck serving stale assets

**Your Mission:** Find and fix the bug so cache updates work correctly without requiring a hard reload.

## 📚 Documentation

- **README.md** - Comprehensive module documentation
- **TESTING.md** - Step-by-step bug reproduction guide
- **This file** - Quick start instructions

## 🔧 Browser Requirements

- Chrome 45+
- Firefox 44+
- Safari 11.1+
- Edge 17+

Service Workers require either:
- HTTPS connection, OR
- localhost (for development)

## 💡 Hints

1. Look at where `CURRENT_CACHE` is defined
2. Understand when that value is set vs. when it's used
3. Consider JavaScript closures and scope
4. Think about the Service Worker lifecycle

## ✅ Testing Your Fix

After you think you've fixed it:

1. Change `CACHE_VERSION` from `v1` to `v2` in service-worker.js
2. Reload the page (normal reload, NOT hard reload)
3. Click "Check Current Cache" button
4. Check DevTools Network tab - should show cache hits
5. Verify assets are served from new cache (v2)

If it works without a hard reload, you've fixed it! 🎉

Good luck debugging! 🐛🔍
