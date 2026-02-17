// Main application JavaScript
// Handles Service Worker registration and UI interactions

const APP_VERSION = '1.0.0';

// Log function to add messages to the UI
function log(message, type = 'info') {
  const logDiv = document.getElementById('log');
  const timestamp = new Date().toLocaleTimeString();
  const logEntry = document.createElement('div');
  logEntry.className = `log-entry log-${type}`;
  logEntry.textContent = `[${timestamp}] ${message}`;
  logDiv.insertBefore(logEntry, logDiv.firstChild);
  
  // Keep only last 15 entries
  while (logDiv.children.length > 15) {
    logDiv.removeChild(logDiv.lastChild);
  }
}

// Update Service Worker status
function updateSWStatus(status) {
  const statusElement = document.getElementById('sw-status');
  statusElement.textContent = status;
  statusElement.className = 'status-badge ' + 
    (status.includes('Active') ? 'status-active' : 
     status.includes('Error') ? 'status-error' : 
     'status-inactive');
}

// Register Service Worker
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('/service-worker.js')
      .then((registration) => {
        log('✅ Service Worker registered successfully', 'success');
        updateSWStatus('Active');
        
        // Check for updates
        registration.addEventListener('updatefound', () => {
          const newWorker = registration.installing;
          log('🔄 New Service Worker found, installing...', 'info');
          
          newWorker.addEventListener('statechange', () => {
            if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
              log('⚠️ New version available! Reload to update.', 'warning');
            }
          });
        });
        
        // Listen for controller change
        navigator.serviceWorker.addEventListener('controllerchange', () => {
          log('🔄 Service Worker controller changed', 'info');
        });
      })
      .catch((error) => {
        log('❌ Service Worker registration failed: ' + error, 'error');
        updateSWStatus('Error: Not registered');
        console.error('Service Worker registration failed:', error);
      });
  });
} else {
  log('❌ Service Workers not supported in this browser', 'error');
  updateSWStatus('Not supported');
}

// Check current cache
document.getElementById('checkCache').addEventListener('click', async () => {
  try {
    const cacheNames = await caches.keys();
    log(`📦 Current caches: ${cacheNames.join(', ')}`, 'info');
    
    for (const cacheName of cacheNames) {
      const cache = await caches.open(cacheName);
      const keys = await cache.keys();
      log(`  └─ ${cacheName}: ${keys.length} items`, 'info');
    }
  } catch (error) {
    log('❌ Error checking cache: ' + error.message, 'error');
  }
});

// Simulate version update
document.getElementById('updateVersion').addEventListener('click', async () => {
  log('🚀 Simulating version update...', 'info');
  log('⚠️ In real scenario, you would deploy new assets', 'warning');
  log('🐛 BUG: Old cache will still be used until hard reload!', 'error');
  
  if ('serviceWorker' in navigator && navigator.serviceWorker.controller) {
    try {
      const registration = await navigator.serviceWorker.getRegistration();
      await registration.update();
      log('✅ Update check triggered', 'success');
    } catch (error) {
      log('❌ Update failed: ' + error.message, 'error');
    }
  }
});

// Clear all caches
document.getElementById('clearCache').addEventListener('click', async () => {
  try {
    const cacheNames = await caches.keys();
    await Promise.all(cacheNames.map(name => caches.delete(name)));
    log('🗑️ All caches cleared', 'success');
    log('🔄 Reload page to fetch fresh assets', 'info');
  } catch (error) {
    log('❌ Error clearing cache: ' + error.message, 'error');
  }
});

// Unregister Service Worker
document.getElementById('unregister').addEventListener('click', async () => {
  try {
    if ('serviceWorker' in navigator) {
      const registration = await navigator.serviceWorker.getRegistration();
      if (registration) {
        await registration.unregister();
        log('✅ Service Worker unregistered', 'success');
        updateSWStatus('Unregistered');
        log('🔄 Reload page to see changes', 'info');
      }
    }
  } catch (error) {
    log('❌ Error unregistering: ' + error.message, 'error');
  }
});

// Display app version
document.getElementById('app-version').textContent = APP_VERSION;

// Initial log
log('🎬 Application started', 'info');
