# Module 19: Custom Future with Missing Waker Bug

## 🐛 Bug Description

This module contains an intentional **missing Waker bug** in a custom Future implementation that bridges JavaScript Promises using wasm-bindgen-futures.

### The Problem

The module implements a custom `Future` that wraps a JavaScript Promise. However, the `poll()` method has a critical bug:

1. **The `poll()` method returns `Poll::Pending`** when the Promise hasn't resolved yet
2. **BUT it never stores or calls the `Waker`** from the Context
3. **The async runtime has no way to know when to poll again**
4. **The Future hangs forever** - no error, no CPU usage, just eternal waiting

### The Consequence

> ⚠️ **Critical Issue**: When JavaScript code calls the async Rust function, it will `await` forever. The Promise on the JS side actually completes successfully, but the Rust side never knows about it. There's:
> - **No error thrown**
> - **0% CPU usage** (not a busy-wait loop)
> - **No timeout** (waits indefinitely)
> - **No indication of what went wrong**

This is one of the most difficult bugs to diagnose because everything *looks* fine - the JS Promise resolves, the browser is responsive, but the await never completes.

## 🔧 Technical Details

### Root Cause

The bug is in the `BuggyFuture::poll()` implementation:

```rust
// ❌ BUGGY CODE (in this module)
impl Future for BuggyFuture {
    type Output = Result<JsValue, JsValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.result.is_none() {
            // THE BUG: Returns Pending without storing/waking!
            return Poll::Pending;  // ⚠️ Waker is lost!
        }
        
        Poll::Ready(Ok(self.result.take().unwrap()))
    }
}
```

### What Should Happen

When implementing a custom Future, you **must** do one of these when returning `Poll::Pending`:

1. **Store the Waker** and call it later when ready:
   ```rust
   let waker = cx.waker().clone();
   // Store waker and call waker.wake() when Promise resolves
   ```

2. **Set up a callback** that wakes the task:
   ```rust
   let waker = cx.waker().clone();
   promise.then(&Closure::once(move || {
       waker.wake();  // Wake the task when Promise resolves
   }));
   ```

3. **Use an existing implementation** like `JsFuture` that handles this correctly

### Why It's Difficult to Debug

1. **Silent failure**: No error messages anywhere
2. **Looks like it's working**: The JS Promise does complete successfully
3. **No CPU spikes**: Not a busy-wait, so performance monitoring shows nothing
4. **Browser stays responsive**: The main thread isn't blocked
5. **DevTools show nothing**: Promise is resolved, but Rust side just waits
6. **Can't timeout**: Standard timeout mechanisms don't help because the future is just "pending"

### The Async Runtime's Perspective

When the async runtime polls the Future:

```
1. Runtime: "Is the Future ready?"
2. Future: "No, I'm Poll::Pending"
3. Runtime: "Okay, I'll poll you when someone wakes you up"
4. [Time passes...]
5. JS Promise resolves successfully
6. [More time passes...]
7. Runtime: [Never polls again because no waker was called]
8. [Forever...]
```

## 🏗️ Module Structure

```
Module 19/
├── Cargo.toml                 # Rust/WASM package configuration
├── src/
│   └── lib.rs                # Main buggy implementation
├── demo.html                 # Interactive demo showing the bug
├── integration-example.js    # JavaScript usage example
├── README.md                 # This file
├── QUICKSTART.md            # Build and test instructions
└── TECHNICAL.md             # Deep dive into Future/Waker mechanism
```

## 🚀 Quick Start

### Prerequisites

- Rust toolchain (1.70+)
- wasm-pack for WebAssembly compilation
- A web browser for testing

### Build Commands

```bash
# Install wasm-pack if not already installed
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build the WASM module
cd "Module 19"
wasm-pack build --target web

# The compiled WASM will be in pkg/ directory
```

### Test the Bug

Open `demo.html` in a web browser and click the "Fetch Data (Buggy)" button. Watch the console - you'll see:
- The Promise is created
- The Future is polled once
- The Promise resolves after 2 seconds
- But the await never completes!

## 🧪 Demonstration

### JavaScript Usage

```javascript
import init, { fetch_data_buggy, fetch_data_correct } from './pkg/waker_bug_module.js';

await init();

// This will hang forever! ⚠️
console.log('Calling buggy function...');
const result = await fetch_data_buggy(2000);  // Hangs here forever
console.log('Result:', result);  // NEVER EXECUTED

// Compare with correct implementation:
console.log('Calling correct function...');
const result2 = await fetch_data_correct(2000);  // Returns after 2 seconds
console.log('Result:', result2);  // ✓ Works!
```

### What You'll See

**Buggy version:**
```
fetch_data_buggy() called
⚠️  This will hang forever!
Created Promise that will resolve in 2000ms
BuggyFuture created
BuggyFuture::poll() called
BuggyFuture: Returning Poll::Pending WITHOUT storing waker! 🐛
[2 seconds pass]
JS Promise resolved! ✓
[Silence forever...]
```

**Correct version:**
```
fetch_data_correct() called
✓ This will work correctly
Created Promise that will resolve in 2000ms
[2 seconds pass]
JS Promise resolved! ✓
fetch_data_correct completed successfully!
```

## 🔍 Functions in the Module

### `fetch_data_buggy(delay_ms: u32)`
Demonstrates the basic bug - awaits a Promise that resolves but never wakes the Future.

### `fetch_data_correct(delay_ms: u32)`
Correct implementation using `JsFuture` for comparison.

### `get_user_data(user_id: u32)`
Simulates a real-world API call that hangs forever.

### `complex_async_operation()`
Shows how the bug affects chained async operations - everything stops at the first buggy await.

### `countdown_buggy(seconds: u32)`
A countdown that never completes, hanging on the first delay.

## ✅ How to Fix This Bug

### Solution 1: Use JsFuture (Recommended)

```rust
use wasm_bindgen_futures::JsFuture;

pub async fn fetch_data_fixed(delay_ms: u32) -> Result<JsValue, JsValue> {
    let promise = create_delayed_promise(delay_ms);
    JsFuture::from(promise).await  // ✓ JsFuture handles waking correctly
}
```

### Solution 2: Manually Store and Wake

```rust
impl Future for FixedFuture {
    type Output = Result<JsValue, JsValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.result.is_none() {
            let waker = cx.waker().clone();
            
            // Set up callback to wake when Promise resolves
            let callback = Closure::once(move |value: JsValue| {
                // Store result and wake the task
                waker.wake();  // ✓ This wakes the Future!
            });
            
            self.promise.then(&callback);
            callback.forget();
            
            return Poll::Pending;
        }
        
        Poll::Ready(Ok(self.result.take().unwrap()))
    }
}
```

### Solution 3: Use spawn_local with Channels

```rust
use futures::channel::oneshot;

pub async fn fetch_data_fixed(delay_ms: u32) -> Result<JsValue, JsValue> {
    let (tx, rx) = oneshot::channel();
    
    let promise = create_delayed_promise(delay_ms);
    
    wasm_bindgen_futures::spawn_local(async move {
        let result = JsFuture::from(promise).await;
        tx.send(result).unwrap();
    });
    
    rx.await.unwrap()
}
```

## 🎯 Learning Objectives

By studying this buggy module, developers will learn:

1. **How Futures work**: Understanding the polling mechanism and the role of Wakers
2. **Async runtime mechanics**: How executors schedule and wake tasks
3. **JS/Rust async bridge**: How wasm-bindgen-futures connects Promises and Futures
4. **Common pitfalls**: Why custom Future implementations are tricky
5. **Debugging techniques**: How to identify hung async operations

## 📚 Related Concepts

- **Rust Futures and async/await**
- **The Poll enum and Context**
- **The Waker mechanism**
- **JavaScript Promises**
- **wasm-bindgen and wasm-bindgen-futures**
- **WebAssembly async operations**

## ⚠️ Security and Performance Implications

While this bug doesn't directly expose security vulnerabilities, it can:

- **Cause denial of service**: Hung operations consume resources
- **Block critical operations**: If a timeout or health check uses this, it fails silently
- **Create memory leaks**: Pending futures that never complete accumulate
- **Break error handling**: Errors can't propagate if the future never completes

## 🏷️ Tags

`#bug` `#future` `#waker` `#async` `#rust` `#wasm` `#webassembly` `#wasm-bindgen` `#promise` `#hanging` `#poll-pending`

## 📝 License

This module is for educational purposes to demonstrate custom Future bugs.

---

**⚠️ WARNING**: This module is intentionally buggy for educational purposes. Do NOT use in production applications!

**Challenge**: Can you fix the `BuggyFuture` implementation to properly handle the Waker?
