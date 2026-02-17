/*!
# Session State Manager Module (BUGGY VERSION with Global Lazy)

This module implements a global session state manager using `once_cell::sync::Lazy`.
It's designed for webapp integration to manage user sessions.

## ⚠️ WARNING: This module contains multiple subtle bugs! ⚠️

### The Bugs

This module has several interconnected concurrency bugs that are particularly difficult to detect:

1. **Lost Updates Bug**: When multiple threads try to update the same session simultaneously,
   updates can be lost due to incorrect read-modify-write operations. The code reads the state,
   modifies it locally, then writes it back without ensuring atomicity.

2. **Stale Data Bug**: The session data is cached in a global HashMap with a 60-second timeout,
   but when data is "refreshed" from a simulated database, the timestamp check has a race condition
   that allows stale data to overwrite fresh data.

3. **Memory Leak Bug**: Failed session cleanups accumulate in a separate "cleanup queue" that
   grows unbounded because the cleanup thread has a logic error that prevents it from actually
   removing failed cleanups.

4. **Phantom Session Bug**: When a session is deleted, there's a window where the session appears
   to be both deleted and active simultaneously due to inconsistent state updates across multiple
   data structures.

### Why These Bugs Are Hard to Find

- The bugs only manifest under concurrent load (multiple users/threads)
- They involve subtle timing windows (race conditions)
- The code appears to use proper synchronization (RwLock) but has logical errors
- Tests might pass when run individually but fail under load
- The symptoms are inconsistent and hard to reproduce

### Integration Context

This module is meant to be used in a web application where:
- Multiple HTTP request handlers access session data concurrently
- Sessions need to be refreshed from a database periodically
- Old sessions need to be cleaned up automatically
- Session data is cached globally for performance

*/

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Session data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub session_id: String,
    pub user_id: String,
    pub username: String,
    pub login_time: DateTime<Utc>,
    pub last_activity: SystemTime,
    pub access_count: u64,
    pub is_authenticated: bool,
    pub metadata: HashMap<String, String>,
}

/// Internal session storage with caching metadata
#[derive(Debug, Clone)]
struct CachedSession {
    data: SessionData,
    cache_time: SystemTime,
    last_db_sync: SystemTime,
    version: u64, // Added but not used correctly - BUG!
}

/// Global session storage using Lazy
/// BUG: The storage uses RwLock but has logical race conditions
static SESSION_STORE: Lazy<Arc<RwLock<HashMap<String, CachedSession>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(HashMap::new()))
});

/// Global cleanup queue that grows unbounded - BUG!
static CLEANUP_QUEUE: Lazy<Arc<RwLock<Vec<String>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(Vec::new()))
});

/// Global statistics counter with race condition - BUG!
static STATS: Lazy<Arc<RwLock<SessionStats>>> = Lazy::new(|| {
    Arc::new(RwLock::new(SessionStats::default()))
});

#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub failed_cleanups: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// Session Manager - the main API
pub struct SessionManager {
    cache_ttl: Duration,
    cleanup_interval: Duration,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            cache_ttl: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(300),
        }
    }

    /// Create a new session
    /// BUG: The session counter update is not atomic with the insert
    pub fn create_session(&self, user_id: String, username: String) -> Result<String, String> {
        let session_id = Uuid::new_v4().to_string();
        let now = SystemTime::now();

        let session_data = SessionData {
            session_id: session_id.clone(),
            user_id,
            username,
            login_time: Utc::now(),
            last_activity: now,
            access_count: 0,
            is_authenticated: true,
            metadata: HashMap::new(),
        };

        let cached = CachedSession {
            data: session_data,
            cache_time: now,
            last_db_sync: now,
            version: 1,
        };

        // BUG: Non-atomic read-modify-write pattern
        // Another thread could modify stats between these operations
        {
            let mut store = SESSION_STORE.write();
            store.insert(session_id.clone(), cached);
        }
        
        // BUG: Stats update is separate from session insert
        // If this panics or is interrupted, stats will be inconsistent
        {
            let mut stats = STATS.write();
            stats.total_sessions += 1;
            stats.active_sessions += 1;
        }

        Ok(session_id)
    }

    /// Get session data with cache
    /// BUG: The cache validity check has a race condition
    pub fn get_session(&self, session_id: &str) -> Result<SessionData, String> {
        let now = SystemTime::now();
        
        // BUG: Two-phase lock pattern - read then potentially write
        // Another thread could invalidate our assumptions between locks
        let needs_refresh = {
            let store = SESSION_STORE.read();
            match store.get(session_id) {
                Some(cached) => {
                    // BUG: Cache TTL check is racy
                    if let Ok(elapsed) = now.duration_since(cached.cache_time) {
                        if elapsed < self.cache_ttl {
                            // Cache hit - but we drop the lock here!
                            drop(store);
                            
                            // BUG: Update stats without holding session lock
                            let mut stats = STATS.write();
                            stats.cache_hits += 1;
                            drop(stats);
                            
                            // BUG: Re-acquire lock - session might have changed!
                            let store = SESSION_STORE.read();
                            return Ok(store.get(session_id)
                                .ok_or_else(|| "Session disappeared".to_string())?
                                .data.clone());
                        }
                        true // Needs refresh
                    } else {
                        true
                    }
                }
                None => return Err("Session not found".to_string()),
            }
        };

        if needs_refresh {
            // BUG: Update stats outside of transaction
            let mut stats = STATS.write();
            stats.cache_misses += 1;
            drop(stats);
            
            // Simulate database fetch with delay
            self.refresh_from_database(session_id)?;
            
            // BUG: Re-read after refresh - could get stale data if another thread
            // also did a refresh with older data that completed after ours
            let store = SESSION_STORE.read();
            Ok(store.get(session_id)
                .ok_or_else(|| "Session not found after refresh".to_string())?
                .data.clone())
        } else {
            Err("Unreachable code reached".to_string())
        }
    }

    /// Update session data
    /// BUG: Lost update problem - classic read-modify-write race
    pub fn update_session(&self, session_id: &str, metadata_key: String, metadata_value: String) -> Result<(), String> {
        // BUG: Read-modify-write without proper isolation
        let mut session_data = {
            let store = SESSION_STORE.read();
            let cached = store.get(session_id)
                .ok_or_else(|| "Session not found".to_string())?;
            cached.data.clone() // Clone the data
        }; // Lock is released here!
        
        // BUG: Between releasing the read lock and acquiring the write lock,
        // another thread could have modified the session. Our update will
        // overwrite their changes!
        session_data.metadata.insert(metadata_key, metadata_value);
        session_data.access_count += 1;
        session_data.last_activity = SystemTime::now();
        
        // Acquire write lock and update
        {
            let mut store = SESSION_STORE.write();
            if let Some(cached) = store.get_mut(session_id) {
                // BUG: We're overwriting with our locally modified copy,
                // potentially losing updates made by other threads
                cached.data = session_data;
                // BUG: Version is incremented but never actually checked!
                cached.version += 1;
            }
        }

        Ok(())
    }

    /// Increment access count
    /// BUG: Another read-modify-write race condition
    pub fn increment_access(&self, session_id: &str) -> Result<u64, String> {
        let new_count = {
            let store = SESSION_STORE.read();
            let cached = store.get(session_id)
                .ok_or_else(|| "Session not found".to_string())?;
            cached.data.access_count + 1 // Read current value
        }; // Lock released!
        
        // BUG: Another thread could increment between our read and write
        {
            let mut store = SESSION_STORE.write();
            if let Some(cached) = store.get_mut(session_id) {
                // BUG: Overwriting with our calculated value, losing concurrent increments
                cached.data.access_count = new_count;
                cached.data.last_activity = SystemTime::now();
            }
        }
        
        Ok(new_count)
    }

    /// Delete a session
    /// BUG: Phantom session problem - inconsistent state across data structures
    pub fn delete_session(&self, session_id: &str) -> Result<(), String> {
        // BUG: Multi-step deletion with gaps between operations
        
        // Step 1: Remove from main store
        let was_present = {
            let mut store = SESSION_STORE.write();
            store.remove(session_id).is_some()
        }; // Lock released!
        
        if !was_present {
            return Err("Session not found".to_string());
        }
        
        // BUG: Gap here - session is removed but stats not updated
        // If someone calls get_stats() now, they'll see incorrect active count
        
        // Step 2: Update statistics
        {
            let mut stats = STATS.write();
            stats.active_sessions = stats.active_sessions.saturating_sub(1);
        }
        
        // BUG: Gap here - if cleanup fails, the session is gone but cleanup queue grows
        
        // Step 3: Add to cleanup queue (to simulate database cleanup)
        {
            let mut queue = CLEANUP_QUEUE.write();
            queue.push(session_id.to_string());
        }
        
        Ok(())
    }

    /// Simulate refreshing session from database
    /// BUG: Can overwrite newer data with older data due to improper timestamp checking
    fn refresh_from_database(&self, session_id: &str) -> Result<(), String> {
        // Simulate database delay
        std::thread::sleep(Duration::from_millis(10));
        
        let now = SystemTime::now();
        
        // BUG: Read current version without holding lock during "database fetch"
        let current_version = {
            let store = SESSION_STORE.read();
            store.get(session_id)
                .map(|cached| cached.version)
                .unwrap_or(0)
        }; // Lock released during "database fetch"!
        
        // Simulate fetching from database (in reality, this is just reading and re-writing)
        // BUG: Another thread could have updated with newer data while we were "fetching"
        
        let fetched_data = {
            let store = SESSION_STORE.read();
            store.get(session_id)
                .map(|cached| cached.data.clone())
                .ok_or_else(|| "Session not found".to_string())?
        };
        
        // BUG: Update without checking if data is actually newer
        {
            let mut store = SESSION_STORE.write();
            if let Some(cached) = store.get_mut(session_id) {
                // BUG: We compare version but use the wrong logic
                // Should reject if current version > fetched version, but we don't
                if cached.version >= current_version {
                    // BUG: This condition is backwards - we update even when we shouldn't
                    cached.data = fetched_data;
                    cached.cache_time = now;
                    cached.last_db_sync = now;
                }
            }
        }
        
        Ok(())
    }

    /// Run cleanup task
    /// BUG: The cleanup queue grows unbounded because failed items are never removed
    pub async fn run_cleanup(&self) {
        loop {
            tokio::time::sleep(self.cleanup_interval).await;
            
            // BUG: Process cleanup queue but don't remove failed items
            let items_to_clean = {
                let queue = CLEANUP_QUEUE.read();
                queue.clone() // Clone entire queue
            };
            
            for session_id in items_to_clean {
                // Simulate cleanup operation that might fail
                let cleanup_success = session_id.len() % 2 == 0; // Arbitrary condition
                
                if cleanup_success {
                    // Remove from queue only if successful
                    let mut queue = CLEANUP_QUEUE.write();
                    if let Some(pos) = queue.iter().position(|id| id == &session_id) {
                        queue.remove(pos);
                    }
                } else {
                    // BUG: Failed cleanups stay in queue forever!
                    // Queue grows unbounded
                    let mut stats = STATS.write();
                    stats.failed_cleanups += 1;
                }
            }
        }
    }

    /// Get statistics
    /// BUG: Statistics are inconsistent due to race conditions in other methods
    pub fn get_stats(&self) -> SessionStats {
        let stats = STATS.read();
        stats.clone()
    }

    /// Get active session count
    /// BUG: This count may not match actual sessions due to race conditions
    pub fn get_active_count(&self) -> usize {
        let store = SESSION_STORE.read();
        store.len()
    }

    /// Get cleanup queue size (for debugging the memory leak)
    pub fn get_cleanup_queue_size(&self) -> usize {
        let queue = CLEANUP_QUEUE.read();
        queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_session() {
        let manager = SessionManager::new();
        let result = manager.create_session("user123".to_string(), "alice".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_session() {
        let manager = SessionManager::new();
        let session_id = manager.create_session("user456".to_string(), "bob".to_string()).unwrap();
        let result = manager.get_session(&session_id);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_updates_show_bug() {
        // This test demonstrates the lost update bug
        let manager = Arc::new(SessionManager::new());
        let session_id = manager.create_session("user789".to_string(), "charlie".to_string()).unwrap();
        
        // Spawn multiple threads that update the same session
        let mut handles = vec![];
        for i in 0..10 {
            let manager_clone = manager.clone();
            let session_id_clone = session_id.clone();
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    let key = format!("key_{}", i);
                    let value = format!("value_{}_{}", i, j);
                    let _ = manager_clone.update_session(&session_id_clone, key, value);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.await.unwrap();
        }
        
        // BUG: Due to race conditions, some updates will be lost
        // The metadata map should have 10 keys (key_0 through key_9)
        // but might have fewer due to lost updates
        let session = manager.get_session(&session_id).unwrap();
        println!("Metadata keys after concurrent updates: {}", session.metadata.len());
        // This might fail: assert_eq!(session.metadata.len(), 10);
    }

    #[tokio::test]
    async fn test_concurrent_increment_shows_bug() {
        // This test demonstrates the lost increment bug
        let manager = Arc::new(SessionManager::new());
        let session_id = manager.create_session("user999".to_string(), "dave".to_string()).unwrap();
        
        // Spawn multiple threads that increment access count
        let mut handles = vec![];
        for _ in 0..20 {
            let manager_clone = manager.clone();
            let session_id_clone = session_id.clone();
            let handle = tokio::spawn(async move {
                for _ in 0..50 {
                    let _ = manager_clone.increment_access(&session_id_clone);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.await.unwrap();
        }
        
        // BUG: Due to race conditions, the count will be less than expected
        // Expected: 20 threads * 50 increments = 1000
        // Actual: Much less due to lost updates
        let session = manager.get_session(&session_id).unwrap();
        println!("Access count after 1000 concurrent increments: {}", session.access_count);
        println!("Expected: 1000, Got: {}", session.access_count);
        // This will likely fail: assert_eq!(session.access_count, 1000);
    }
}
