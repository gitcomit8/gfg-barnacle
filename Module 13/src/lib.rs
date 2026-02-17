/*!
# Task Toggle Module - Optimistic UI Update (BUGGY VERSION)

This module implements a "Like" button / "Task Toggle" system with optimistic UI updates.

## ⚠️ WARNING: This module contains a deliberate bug! ⚠️

### The Bug
When a user rapidly clicks the toggle button multiple times (e.g., 3 times in quick succession),
the local state updates optimistically: true -> false -> true. However, due to network jitter,
API responses may arrive out of order (request 1, request 3, request 2).

This causes the UI to "flicker" back to the wrong state after responses arrive, because
the system doesn't track which response is most recent.

### Expected Behavior
- Click 1: Local state = true, API request 1 sent
- Click 2: Local state = false, API request 2 sent  
- Click 3: Local state = true, API request 3 sent

### Actual Behavior (with bug)
If responses arrive as: Response 1, Response 3, Response 2
- Response 1 arrives: state = true ✓
- Response 3 arrives: state = true ✓
- Response 2 arrives: state = false ✗ (WRONG! Should stay true)

The final state becomes false instead of true because response 2 arrives last,
even though request 3 was made after request 2.

### The Fix (Not Implemented)
To fix this bug, you need to implement one of:
1. **Idempotency Keys**: Track request IDs and only apply the most recent request
2. **Request Queue**: Process requests serially, canceling outdated requests
3. **Version Numbers**: Track version/timestamp with each state change

This module deliberately omits these fixes to demonstrate the race condition.
*/

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub id: String,
    pub is_completed: bool,
    pub likes: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ToggleRequest {
    pub request_id: Uuid,
    pub task_id: String,
    pub new_state: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// The main TaskToggleService with the race condition bug
pub struct TaskToggleService {
    // Local state that gets updated optimistically
    local_state: Arc<RwLock<TaskState>>,
    // Simulated API endpoint that introduces random delays
    api_delay_ms: u64,
}

impl TaskToggleService {
    pub fn new(task_id: String, initial_state: bool) -> Self {
        Self {
            local_state: Arc::new(RwLock::new(TaskState {
                id: task_id,
                is_completed: initial_state,
                likes: 0,
                timestamp: chrono::Utc::now(),
            })),
            api_delay_ms: 100,
        }
    }

    /// Get the current local state (what the UI shows)
    pub async fn get_local_state(&self) -> TaskState {
        self.local_state.read().await.clone()
    }

    /// Toggle the task with optimistic update
    /// 
    /// BUG: This function updates local state immediately but doesn't track
    /// which API response corresponds to which request. If responses arrive
    /// out of order, the final state will be wrong.
    pub async fn toggle(&self, task_id: String) -> Result<TaskState, String> {
        let request = ToggleRequest {
            request_id: Uuid::new_v4(),
            task_id: task_id.clone(),
            new_state: !self.local_state.read().await.is_completed,
            timestamp: chrono::Utc::now(),
        };

        // OPTIMISTIC UPDATE: Update local state immediately before API call
        {
            let mut state = self.local_state.write().await;
            state.is_completed = request.new_state;
            state.timestamp = chrono::Utc::now();
        }

        // Spawn async task to simulate API call
        // BUG: We don't track request order or use idempotency keys!
        let local_state_clone = self.local_state.clone();
        let api_delay = self.api_delay_ms;
        
        tokio::spawn(async move {
            // Simulate API call with variable delay (network jitter)
            // Random delay between 50-200ms to simulate real-world conditions
            let jitter = (request.request_id.as_u128() % 150) as u64;
            sleep(Duration::from_millis(api_delay + jitter)).await;

            // BUG: When response arrives, we just apply it without checking
            // if a newer request has already been processed
            let mut state = local_state_clone.write().await;
            state.is_completed = request.new_state;
            state.timestamp = chrono::Utc::now();
        });

        Ok(self.local_state.read().await.clone())
    }

    /// Like button with the same race condition bug
    pub async fn toggle_like(&self) -> Result<TaskState, String> {
        let request_id = Uuid::new_v4();
        let current_likes = self.local_state.read().await.likes;
        let new_likes = current_likes + 1;

        // OPTIMISTIC UPDATE: Increment likes immediately
        {
            let mut state = self.local_state.write().await;
            state.likes = new_likes;
            state.timestamp = chrono::Utc::now();
        }

        // Spawn async task to simulate API call
        let local_state_clone = self.local_state.clone();
        let api_delay = self.api_delay_ms;
        
        tokio::spawn(async move {
            // Simulate API call with jitter
            let jitter = (request_id.as_u128() % 150) as u64;
            sleep(Duration::from_millis(api_delay + jitter)).await;

            // BUG: Overwrite with the response value without checking order
            let mut state = local_state_clone.write().await;
            state.likes = new_likes;
            state.timestamp = chrono::Utc::now();
        });

        Ok(self.local_state.read().await.clone())
    }

    /// Simulate rapid clicks (for testing the bug)
    pub async fn rapid_toggle(&self, task_id: String, count: usize) -> Vec<TaskState> {
        let mut results = Vec::new();
        
        for _ in 0..count {
            match self.toggle(task_id.clone()).await {
                Ok(state) => results.push(state),
                Err(_) => {}
            }
            // Small delay between clicks (like a human rapidly clicking)
            sleep(Duration::from_millis(10)).await;
        }
        
        results
    }
}

/// A "fixed" version would look like this (commented out):
/// 
/// ```rust,ignore
/// pub struct FixedTaskToggleService {
///     local_state: Arc<RwLock<TaskState>>,
///     // Track the most recent request ID
///     latest_request_id: Arc<RwLock<Uuid>>,
/// }
/// 
/// impl FixedTaskToggleService {
///     pub async fn toggle(&self, task_id: String) -> Result<TaskState, String> {
///         let request_id = Uuid::new_v4();
///         
///         // Update the latest request ID
///         *self.latest_request_id.write().await = request_id;
///         
///         // ... optimistic update ...
///         
///         // In the API response handler:
///         tokio::spawn(async move {
///             // ... API call ...
///             
///             // Only apply if this is still the most recent request
///             let latest_id = *latest_request_id_clone.read().await;
///             if request_id == latest_id {
///                 // Apply the state change
///             }
///             // Otherwise, ignore this outdated response
///         });
///     }
/// }
/// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_single_toggle() {
        let service = TaskToggleService::new("task1".to_string(), false);
        
        let state = service.toggle("task1".to_string()).await.unwrap();
        assert_eq!(state.is_completed, true);
    }

    #[tokio::test]
    async fn test_race_condition_bug() {
        // This test demonstrates the bug!
        let service = TaskToggleService::new("task2".to_string(), false);
        
        // Rapidly toggle 3 times
        let _states = service.rapid_toggle("task2".to_string(), 3).await;
        
        // Immediately after toggling, the local state shows "true" (3rd click)
        let immediate_state = service.get_local_state().await;
        assert_eq!(immediate_state.is_completed, true);
        
        // Wait for all API responses to arrive
        sleep(Duration::from_millis(500)).await;
        
        // BUG: The final state might be wrong due to race condition!
        // It should be "true" but might be "false" if response 2 arrived last
        let final_state = service.get_local_state().await;
        
        // This assertion might fail due to the race condition bug
        // In a real scenario, this would manifest as UI flickering
        println!("Final state: {:?}", final_state);
        println!("Expected: true, Got: {}", final_state.is_completed);
    }
}
