use wasm_bindgen::prelude::*;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::cell::RefCell;
use std::rc::Rc;

/// A buggy custom Future implementation that bridges a JS Promise
/// 
/// ## THE BUG:
/// This Future's poll() method returns Poll::Pending but NEVER stores
/// or calls the Waker. This means:
/// 
/// 1. When polled, it checks if the internal Promise has resolved
/// 2. If not resolved, it returns Poll::Pending
/// 3. BUT it doesn't save the Waker to wake the task later
/// 4. The async runtime has no way to know when to poll again
/// 5. The Future hangs forever - no error, just eternal waiting
/// 
/// ## Why This Is Difficult:
/// - No error is thrown
/// - CPU usage is 0% (not spinning)
/// - The JS Promise actually completes, but Rust never knows
/// - Looks like the function is just "slow" - no indication of bug
/// - Only shows up when actually awaiting the function
///
/// ## The Fix:
/// The poll() method must call `waker.wake_by_ref()` or store the Waker
/// and call it when the Promise resolves via a callback.

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = Promise, js_name = resolve)]
    fn promise_resolve(value: &JsValue) -> js_sys::Promise;
}

/// Internal state for the BuggyFuture
struct BuggyFutureState {
    result: Option<JsValue>,
}

/// A buggy Future that doesn't properly use the Waker
pub struct BuggyFuture {
    state: Rc<RefCell<BuggyFutureState>>,
}

impl BuggyFuture {
    /// Create a new BuggyFuture wrapping a JavaScript Promise
    pub fn new(_promise: js_sys::Promise) -> Self {
        log("BuggyFuture created");
        BuggyFuture {
            state: Rc::new(RefCell::new(BuggyFutureState {
                result: None,
            })),
        }
    }
}

impl Future for BuggyFuture {
    type Output = Result<JsValue, JsValue>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        log("BuggyFuture::poll() called");
        
        let mut state = self.state.borrow_mut();
        
        // Check if we have a result from the Promise
        if let Some(result) = state.result.take() {
            log("BuggyFuture: Returning Poll::Ready with result");
            return Poll::Ready(Ok(result));
        }
        
        // THE BUG IS HERE:
        // We return Poll::Pending but we DON'T do any of these:
        // 1. Store the waker: cx.waker().clone()
        // 2. Set up a callback on the Promise to wake the task  
        // 3. Call waker.wake() or waker.wake_by_ref()
        //
        // Without waking, the async runtime never knows to poll again!
        //
        // Note: We also don't set up the Promise callback, so even if
        // we had stored the waker, nothing would call it!
        
        log("BuggyFuture: Returning Poll::Pending WITHOUT storing waker! 🐛");
        
        // This is the critical bug - we return Pending but never wake the task
        Poll::Pending
    }
}

/// Creates a Promise that resolves after a delay using setTimeout
/// This is implemented in JavaScript and passed to Rust
#[wasm_bindgen]
pub fn create_delayed_promise(delay_ms: u32, value: JsValue) -> js_sys::Promise {
    log(&format!("Creating Promise that will resolve in {}ms", delay_ms));
    
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let resolve_clone = resolve.clone();
        let value_clone = value.clone();
        
        let callback = Closure::once(move || {
            log("JS Promise resolved! ✓");
            let _ = resolve_clone.call1(&JsValue::NULL, &value_clone);
        });
        
        // Use setTimeout from the global window object
        let window = js_sys::global().unchecked_into::<web_sys::Window>();
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            delay_ms as i32,
        );
        
        callback.forget();
    });
    
    promise
}

/// This function uses BuggyFuture and will hang forever!
/// 
/// ## Usage from JavaScript:
/// ```javascript
/// await fetch_data_buggy(2000); // Hangs forever, no error!
/// ```
/// 
/// The Promise completes after 2 seconds, but the Future never wakes up.
#[wasm_bindgen]
pub async fn fetch_data_buggy(delay_ms: u32) -> Result<JsValue, JsValue> {
    log("fetch_data_buggy() called");
    log("⚠️  This will hang forever!");
    
    let value = JsValue::from_str("Buggy data");
    let promise = create_delayed_promise(delay_ms, value);
    let buggy_future = BuggyFuture::new(promise);
    
    // This await will hang forever because BuggyFuture never wakes!
    buggy_future.await
}

/// A correct implementation for comparison
/// This uses wasm-bindgen-futures' JsFuture which properly handles the Waker
#[wasm_bindgen]
pub async fn fetch_data_correct(delay_ms: u32) -> Result<JsValue, JsValue> {
    log("fetch_data_correct() called");
    log("✓ This will work correctly");
    
    let value = JsValue::from_str("Correct data");
    let promise = create_delayed_promise(delay_ms, value);
    
    // JsFuture properly stores and wakes the Waker
    wasm_bindgen_futures::JsFuture::from(promise).await
}

/// Simulates fetching user data - but hangs forever!
#[wasm_bindgen]
pub async fn get_user_data(user_id: u32) -> Result<JsValue, JsValue> {
    log(&format!("Fetching user data for ID: {}", user_id));
    
    // Create a Promise that simulates an API call
    let user_data = format!(
        r#"{{"id": {}, "name": "User {}", "email": "user{}@example.com"}}"#,
        user_id, user_id, user_id
    );
    
    let promise = create_delayed_promise(1500, JsValue::from_str(&user_data));
    
    // Use the buggy future - this will hang!
    let buggy_future = BuggyFuture::new(promise);
    buggy_future.await
}

/// A more complex example: chained async operations
/// All will hang at the first buggy await
#[wasm_bindgen]
pub async fn complex_async_operation() -> Result<JsValue, JsValue> {
    log("Starting complex async operation...");
    
    // Step 1: Fetch config (will hang here forever)
    log("Step 1: Fetching config...");
    let _config = fetch_data_buggy(1000).await?;
    
    // These steps never execute because we hang at step 1
    log("Step 2: Processing config... (NEVER REACHED)");
    let _processed = fetch_data_buggy(500).await?;
    
    log("Step 3: Finalizing... (NEVER REACHED)");
    let _result = fetch_data_buggy(800).await?;
    
    Ok(JsValue::from_str("All operations complete! (NEVER REACHED)"))
}

/// Demonstrates the bug with a simple countdown
/// The countdown will never complete
#[wasm_bindgen]
pub async fn countdown_buggy(seconds: u32) -> Result<JsValue, JsValue> {
    log(&format!("Starting countdown from {}...", seconds));
    
    for i in (0..=seconds).rev() {
        log(&format!("Countdown: {}", i));
        
        if i > 0 {
            // This await will hang forever on first iteration
            fetch_data_buggy(1000).await?;
        }
    }
    
    log("Countdown complete! (NEVER REACHED)");
    Ok(JsValue::from_str("Done!"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // Basic compilation test
        assert!(true, "Module structure is valid");
    }
    
    // Note: Testing async WASM functions requires a browser environment
    // These tests would need to run with wasm-pack test --headless --chrome
}
