# Technical Deep Dive - Module 19: Waker Bug

## Understanding Rust Futures and the Waker Mechanism

This document provides a comprehensive technical explanation of how Rust's Future trait works, why the Waker is essential, and exactly how this module's bug manifests.

## The Future Trait

### Basic Definition

```rust
pub trait Future {
    type Output;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

### How Async/Await Works

When you write:

```rust
let result = some_async_function().await;
```

The compiler transforms it into something like:

```rust
let mut future = some_async_function();
loop {
    match future.poll(&mut context) {
        Poll::Ready(result) => break result,
        Poll::Pending => {
            // Wait to be woken up
            // When woken, loop continues and polls again
        }
    }
}
```

## The Waker: The Heart of Async

### What is a Waker?

A `Waker` is a handle that allows you to notify the async runtime that a Future is ready to be polled again.

```rust
pub struct Context<'a> {
    waker: &'a Waker,
    // ...
}

impl Waker {
    pub fn wake(self) { /* ... */ }
    pub fn wake_by_ref(&self) { /* ... */ }
}
```

### The Async Runtime's Perspective

An async runtime (like Tokio, async-std, or wasm-bindgen-futures) works like this:

```rust
// Simplified async runtime
struct Runtime {
    tasks: Vec<Pin<Box<dyn Future>>>,
}

impl Runtime {
    fn run(&mut self) {
        for task in &mut self.tasks {
            let waker = create_waker_for_task(task);
            let mut context = Context::from_waker(&waker);
            
            match task.poll(&mut context) {
                Poll::Ready(result) => {
                    // Task is done, remove it
                    task.complete(result);
                }
                Poll::Pending => {
                    // Task isn't ready, it MUST have stored the waker
                    // We'll poll it again when waker.wake() is called
                }
            }
        }
    }
}
```

### The Contract

When a Future returns `Poll::Pending`, it **MUST** ensure that the Waker will be called when it becomes ready. This is a contract:

> "I'm returning Pending, but I promise to call waker.wake() when I'm ready to be polled again."

If you break this contract, the Future will never be polled again.

## The Bug in Detail

### The Buggy Implementation

```rust
impl Future for BuggyFuture {
    type Output = Result<JsValue, JsValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check if we have a result
        if self.result.is_none() {
            // BUG: We return Pending but never use cx.waker()!
            return Poll::Pending;
        }
        
        Poll::Ready(Ok(self.result.take().unwrap()))
    }
}
```

### What Happens

1. **First poll**: Runtime calls `poll()`
2. **BuggyFuture**: "I don't have a result yet"
3. **BuggyFuture**: Returns `Poll::Pending`
4. **BuggyFuture**: **FORGETS TO STORE OR CALL THE WAKER**
5. **Runtime**: "Okay, I'll wait for you to wake me"
6. **JS Promise**: Resolves successfully after delay
7. **BuggyFuture**: *Has no way to notify the runtime*
8. **Runtime**: *Never polls again because it was never woken*
9. **Result**: Hang forever

### Memory Layout

```
Stack:
  ┌─────────────────────┐
  │   Runtime           │
  │  - Task queue       │
  │  - Event loop       │
  └─────────────────────┘
         ↓ calls poll()
  ┌─────────────────────┐
  │  BuggyFuture        │
  │  - promise: Promise │  ← JS Promise (will resolve)
  │  - result: None     │
  │  - started: false   │
  └─────────────────────┘
         ↓ returns Pending (no waker stored!)
  ┌─────────────────────┐
  │   Runtime           │
  │  ⏸ Suspended...     │  ← Waiting forever for wake()
  └─────────────────────┘

Meanwhile in JavaScript:
  ┌─────────────────────┐
  │  JS Promise         │
  │  State: Resolved ✓  │  ← Actually completed!
  │  Value: "data"      │
  └─────────────────────┘
```

## Correct Implementations

### Method 1: Use wasm-bindgen-futures::JsFuture

```rust
use wasm_bindgen_futures::JsFuture;

pub async fn correct_implementation(delay_ms: u32) -> Result<JsValue, JsValue> {
    let promise = create_delayed_promise(delay_ms);
    
    // JsFuture properly handles the waker
    JsFuture::from(promise).await
}
```

**How JsFuture works internally:**

```rust
// Simplified version of what JsFuture does
impl Future for JsFuture {
    type Output = Result<JsValue, JsValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.result.take() {
            return Poll::Ready(result);
        }
        
        // Store the waker for later use
        let waker = cx.waker().clone();
        
        // Set up callbacks on the Promise
        let resolve_callback = Closure::once(move |value: JsValue| {
            // Store result and wake the task
            waker.wake();  // ✓ This is the key!
        });
        
        self.promise.then(&resolve_callback);
        resolve_callback.forget();
        
        Poll::Pending  // Now it's safe to return Pending
    }
}
```

### Method 2: Manual Waker Storage

```rust
use std::cell::RefCell;
use std::rc::Rc;

pub struct CorrectFuture {
    promise: Promise,
    state: Rc<RefCell<FutureState>>,
}

struct FutureState {
    result: Option<Result<JsValue, JsValue>>,
    waker: Option<Waker>,
}

impl Future for CorrectFuture {
    type Output = Result<JsValue, JsValue>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();
        
        // Check if we have a result from a previous callback
        if let Some(result) = state.result.take() {
            return Poll::Ready(result);
        }
        
        // Store the current waker
        state.waker = Some(cx.waker().clone());
        
        // If we haven't set up the Promise callback yet, do it now
        if !self.callback_set {
            let state_clone = self.state.clone();
            
            let callback = Closure::once(move |value: JsValue| {
                let mut state = state_clone.borrow_mut();
                state.result = Some(Ok(value));
                
                // Wake the task!
                if let Some(waker) = state.waker.take() {
                    waker.wake();
                }
            });
            
            self.promise.then(&callback);
            callback.forget();
        }
        
        Poll::Pending
    }
}
```

### Method 3: Using Channels

```rust
use futures::channel::oneshot;

pub async fn correct_with_channel(delay_ms: u32) -> Result<JsValue, JsValue> {
    let (tx, rx) = oneshot::channel();
    
    let promise = create_delayed_promise(delay_ms);
    
    // Spawn a task that uses JsFuture
    wasm_bindgen_futures::spawn_local(async move {
        let result = JsFuture::from(promise).await;
        let _ = tx.send(result);
    });
    
    // This channel receiver properly implements Future with waking
    rx.await.unwrap()
}
```

## Why This Bug is Subtle

### 1. No Compiler Error

The Rust compiler can't detect this bug because:
- The function signature is correct
- The trait is implemented correctly
- There's no lifetime or borrow checker issue
- It only manifests at runtime

### 2. No Runtime Error

The bug causes a hang, not a panic:
- No unwrap() on None
- No array out of bounds
- No null pointer dereference
- Just infinite waiting

### 3. The Promise Actually Works

The JavaScript side works perfectly:
```javascript
const promise = create_delayed_promise(2000);
promise.then(value => {
    console.log('Promise resolved!', value);  // This DOES execute
});
```

### 4. No CPU Usage

Unlike a busy-wait loop:
```rust
// This would use 100% CPU (bad, but detectable)
while !ready {
    // Spin
}
```

Our bug uses 0% CPU because the runtime is properly suspended, waiting for a wake() that never comes.

## Performance and Resource Implications

### Memory Leaks

Hung futures accumulate in memory:

```
Time 0:  [Future1: Pending]
Time 1:  [Future1: Pending][Future2: Pending]  
Time 2:  [Future1: Pending][Future2: Pending][Future3: Pending]
         ↑ All hung forever, accumulating
```

### Resource Exhaustion

If used in a server or long-running application:
- Futures pile up in the task queue
- Associated resources (Promise callbacks, closures) are never freed
- Eventually leads to out-of-memory

### Cascading Failures

In a chain of async operations:

```rust
async fn complex_operation() -> Result<Data, Error> {
    let config = fetch_config().await?;      // Hangs here
    let user = fetch_user(config).await?;    // Never reached
    let data = process(user).await?;         // Never reached
    Ok(data)                                 // Never reached
}
```

Everything waits for the first hung future.

## Debugging Techniques

### 1. Add Logging

```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    console::log_1(&JsValue::from_str("poll() called"));
    
    if self.result.is_none() {
        console::log_1(&JsValue::from_str("Returning Pending"));
        return Poll::Pending;
    }
    
    console::log_1(&JsValue::from_str("Returning Ready"));
    Poll::Ready(Ok(self.result.take().unwrap()))
}
```

If you see "poll() called" once, then nothing, the waker isn't being called.

### 2. Timeout Pattern

```javascript
async function withTimeout(promise, timeout_ms) {
    return Promise.race([
        promise,
        new Promise((_, reject) => 
            setTimeout(() => reject(new Error('Timeout')), timeout_ms)
        )
    ]);
}

// Use it:
try {
    await withTimeout(fetch_data_buggy(2000), 5000);
} catch (e) {
    console.error('Hung!', e);  // Will catch after 5 seconds
}
```

### 3. Compare with Working Version

Run both side-by-side:

```javascript
console.time('buggy');
fetch_data_buggy(2000).then(() => console.timeEnd('buggy'));

console.time('correct');  
fetch_data_correct(2000).then(() => console.timeEnd('correct'));

// After 2 seconds:
// correct: 2000ms
// buggy: [never prints]
```

### 4. Inspect Browser Task Queue

In Chrome DevTools:
1. Performance tab → Record
2. Call the buggy function
3. Stop recording
4. Look for suspended tasks that never wake

## The Fix Checklist

To fix a custom Future implementation:

- [ ] Clone the Waker from Context: `cx.waker().clone()`
- [ ] Store the Waker in a place accessible to callbacks
- [ ] Set up a callback that calls `waker.wake()` when ready
- [ ] Ensure the callback actually runs (use `.forget()` on Closures)
- [ ] Update stored result when callback runs
- [ ] Return `Poll::Pending` only after setting up wake mechanism
- [ ] On next poll, check for result and return `Poll::Ready` if available

## References

- [The Rust async book](https://rust-lang.github.io/async-book/)
- [Waker documentation](https://doc.rust-lang.org/std/task/struct.Waker.html)
- [Pin and Unpin](https://doc.rust-lang.org/std/pin/index.html)
- [wasm-bindgen-futures source](https://github.com/rustwasm/wasm-bindgen/tree/main/crates/futures)

## Related Bugs in the Wild

This type of bug has appeared in real projects:

1. **Custom async I/O wrappers** that forget to wake on event
2. **Manual async state machines** that skip wake calls
3. **Async bridge code** between Rust and other languages
4. **Poll-based APIs** that don't properly signal readiness

---

**Understanding this bug deeply will make you a better async Rust programmer!**
