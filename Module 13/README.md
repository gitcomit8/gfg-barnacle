# Module 13: Task Toggle with Race Condition Bug

## 🐛 Buggy Module - Not for Production Use!

This module demonstrates a **race condition bug** in optimistic UI updates for a webapp.

## Overview

This Rust module implements a "Like" button or "Task Toggle" feature that updates the UI immediately (optimistically) before API calls complete. However, it contains a deliberate bug that causes UI flickering when users click rapidly.

## The Bug

### Scenario
When a user clicks the toggle button three times rapidly:
1. **Click 1**: Local state → `true`, API Request 1 sent
2. **Click 2**: Local state → `false`, API Request 2 sent
3. **Click 3**: Local state → `true`, API Request 3 sent

### Expected Result
Final state should be `true` (matching the 3rd click).

### Actual Result (BUG)
Due to network jitter, API responses may arrive out of order:
- Response 1 arrives: state = `true` ✓
- Response 3 arrives: state = `true` ✓  
- **Response 2 arrives LAST**: state = `false` ✗ **WRONG!**

The UI flickers back to `false` even though the user's last action was to set it to `true`.

## Why This Happens

The module does **NOT** implement:
- ❌ Idempotency keys to track request order
- ❌ Request queue to process requests serially
- ❌ Version numbers to reject stale responses
- ❌ Request cancellation for outdated requests

## Building and Testing

```bash
cd "Module 13"

# Build the module
cargo build

# Run tests (will demonstrate the bug)
cargo test -- --nocapture

# Run a specific test showing the race condition
cargo test test_race_condition_bug -- --nocapture
```

## Module Structure

```
Module 13/
├── Cargo.toml           # Dependencies and project configuration
├── src/
│   └── lib.rs          # Main implementation with the bug
├── examples/
│   └── webapp_demo.rs  # Example showing webapp integration
└── README.md           # This file
```

## Integration with Webapp

This module can be compiled to WebAssembly (wasm) and integrated into web frontends. See `examples/webapp_demo.rs` for a demonstration.

## The Challenge

**Your task**: Fix this module to prevent the race condition!

### Possible Solutions

1. **Idempotency Keys**: 
   - Track each request with a unique ID
   - Only apply responses from the most recent request
   - Ignore responses from older requests

2. **Request Queue**:
   - Queue requests and process them serially
   - Cancel pending requests when a new one arrives
   - Ensure only the final request's response is applied

3. **Version/Sequence Numbers**:
   - Assign incrementing version numbers to each state change
   - Only apply responses with version ≥ current version
   - Reject responses with older versions

## API Reference

### `TaskToggleService`

Main service for managing task toggle state.

```rust
// Create a new service
let service = TaskToggleService::new("task_id".to_string(), false);

// Toggle the task (optimistic update)
let state = service.toggle("task_id".to_string()).await?;

// Get current local state
let current = service.get_local_state().await;

// Simulate rapid clicking (for testing)
let results = service.rapid_toggle("task_id".to_string(), 3).await;
```

### `TaskState`

Represents the state of a task.

```rust
pub struct TaskState {
    pub id: String,
    pub is_completed: bool,
    pub likes: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

## License

Educational purposes only. This code intentionally contains bugs.

## Credits

Created for demonstration of race conditions in optimistic UI updates.
