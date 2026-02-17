# Module 15: Buggy Real-Time Notification System (SSE)

## 🐛 Overview

This module demonstrates a **critical bug** in a real-time notification system using Server-Sent Events (SSE). The bug is intentionally designed to be difficult to diagnose because the application works perfectly fine for the first few minutes.

## The Bug: Connection Leak

### What Happens?
Every time a user navigates away from the dashboard and returns, a new SSE connection is opened **without closing the old one**. This creates a connection leak that accumulates over time.

### Why Is It Hard to Diagnose?

1. **Works Fine Initially**: For the first 5 navigations, everything works perfectly
2. **No Error Messages**: The browser doesn't throw any console errors
3. **Silent Failure**: After ~6 connections (browser's max connections per domain limit), all subsequent API calls just hang indefinitely
4. **Delayed Symptoms**: The problem only manifests after several minutes of usage
5. **Not Obvious**: Developers might think it's a backend issue, network problem, or deployment issue

### The Technical Issue

Browsers limit the number of HTTP/1.1 connections per domain (typically 6 connections). When SSE connections aren't properly closed:

- **Connection 1-5**: Everything works fine ✅
- **Connection 6**: Still works, but at the limit ⚠️
- **Connection 7+**: Browser queue is full, all new requests (including regular API calls) hang indefinitely 🔴

## Architecture

### Backend (Rust + Actix-Web)
- **Framework**: Actix-Web 4.4 with Server-Sent Events
- **Bug Location**: `src/main.rs` - `sse_notifications()` function and frontend JavaScript
- **Connection Tracking**: Global counter tracks connections but never decrements

### Frontend (HTML + JavaScript)
- **Bug Location**: `connectSSE()` function in the embedded HTML
- **Issue**: Commented-out code that should close the old EventSource before creating a new one

```javascript
// THE BUG: We never close the old EventSource before creating a new one
function connectSSE() {
    // BUG: Should close existing connection first, but we don't!
    // if (eventSource) {
    //     eventSource.close();  // <-- THIS IS COMMENTED OUT!
    // }

    eventSource = new EventSource('/api/notifications');
    // ...
}
```

## How to Run

### Prerequisites
- Rust 1.70+ installed
- Cargo package manager

### Steps

1. **Navigate to Module 15**:
   ```bash
   cd "Module 15"
   ```

2. **Build and Run**:
   ```bash
   cargo run
   ```

3. **Open in Browser**:
   ```
   http://localhost:8080
   ```

## Reproducing the Bug

### Method 1: Simulated Navigation (Quick)
1. Open the dashboard at http://localhost:8080
2. Click "Navigate Away (Simulated)" button 6 times
3. Click "Check Connection Status" to see all 6+ connections active
4. Try making any API call - it will hang indefinitely

### Method 2: Real Navigation (More Realistic)
1. Open the dashboard
2. Navigate to another page (or open a new tab to google.com)
3. Come back to the dashboard
4. Repeat 6 times
5. Watch as the browser stops responding to new requests

### What You'll See

**After 1-5 navigations**:
```
Active Connections: 3
Status: OK
✅ Everything works fine
```

**After 6+ navigations**:
```
Active Connections: 7
Status: ⚠️ DANGER: Approaching browser connection limit! App will freeze soon!
🔴 All API calls hang
🔴 No error messages
🔴 Browser appears frozen
```

## The Fix (Don't Implement!)

For educational purposes only. The proper fix would be:

### Frontend Fix
```javascript
function connectSSE() {
    // FIXED: Close existing connection before creating new one
    if (eventSource) {
        eventSource.close();
    }

    eventSource = new EventSource('/api/notifications');
    // ...
}
```

### Backend Fix
```rust
// Properly track and clean up connections
// Use connection IDs to remove from active list when connection closes
// Implement proper connection lifecycle management
```

## Testing Checklist

- [ ] Server starts successfully on port 8080
- [ ] Dashboard loads and displays notifications
- [ ] "Navigate Away" button creates new connections
- [ ] Connection count increases with each navigation
- [ ] Status API shows active connection count
- [ ] After 6 connections, warning message appears
- [ ] Subsequent navigations cause the app to hang (bug confirmed)

## Integration Points

This module is designed to be integrated into a larger web application where:

1. **React/Vue/Angular Router**: Navigation events would trigger the bug naturally
2. **Dashboard Components**: Users frequently navigate between dashboard views
3. **Multi-Tab Usage**: Opening multiple tabs compounds the issue
4. **Mobile Apps**: Background/foreground transitions trigger reconnections

## Educational Value

This bug teaches:

1. **Resource Management**: Importance of cleaning up connections
2. **Browser Limits**: Understanding HTTP connection pooling
3. **Silent Failures**: How bugs can appear long after the root cause
4. **Memory Leaks**: Connection leaks are a form of resource leak
5. **Production Debugging**: Symptoms that appear only under load/time

## Security Considerations

⚠️ **This is intentionally buggy code for educational purposes**

In production, connection leaks can:
- Lead to Denial of Service (DoS)
- Exhaust server resources
- Degrade user experience
- Make the application appear broken

## Dependencies

See `Cargo.toml` for full dependency list:
- actix-web: Web framework
- tokio: Async runtime
- tokio-stream: Stream utilities
- serde/serde_json: JSON serialization
- futures: Async utilities
- chrono: Time/date handling

## License

Educational/demonstration purposes only.
