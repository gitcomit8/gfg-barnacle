# Module 18: Raw Pointer Memory Leak

## 🐛 Bug Description

This module demonstrates a critical memory leak that occurs when passing complex state from Rust WASM to JavaScript using raw pointers without proper deallocation.

### The Problem

The Rust code uses `Box::into_raw()` to transfer ownership of heap-allocated data to JavaScript, returning a raw pointer (`*mut T` as `usize`). However, the JavaScript glue code never calls back into a deallocation function, causing memory to leak indefinitely.

### Why This Is Difficult to Debug

1. **Looks Standard**: The interface appears like normal JS-Rust integration
2. **No Immediate Errors**: No runtime errors or warnings initially
3. **Gradual Degradation**: Memory usage climbs slowly over time
4. **Eventual Crash**: Eventually causes `RuntimeError: memory access out of bounds`
5. **Subtle Missing Piece**: The missing dealloc function is easy to overlook

## 🎯 Expected Behavior

With proper memory management:
- Create state → Use state → Free state
- Memory usage remains stable
- No crashes or errors

## ❌ Actual Behavior

Without deallocation:
- Create state → Use state → Create new state → **OLD STATE LEAKED**
- Memory usage climbs linearly: ~100KB per interaction
- WASM heap fills up over time
- Browser tab crashes with "memory access out of bounds"

## 🔍 Technical Analysis

### Rust Side (`src/lib.rs`)

```rust
#[wasm_bindgen]
pub fn create_app_state(user_id: u32, username: String) -> usize {
    let state = AppState::new(user_id, username);
    let boxed_state = Box::new(state);
    
    // ⚠️ BUG: Returns raw pointer, ownership transferred to JS
    let raw_ptr = Box::into_raw(boxed_state);
    raw_ptr as usize
}

// ❌ MISSING FUNCTION - This should exist but doesn't!
/*
#[wasm_bindgen]
pub fn free_app_state(state_ptr: usize) {
    unsafe {
        let boxed_state = Box::from_raw(state_ptr as *mut AppState);
        drop(boxed_state);
    }
}
*/
```

### JavaScript Side (`index.html`)

```javascript
function createNewUser() {
    // ⚠️ BUG: We never free the old state!
    if (currentStatePtr !== null) {
        leakedPointers.push(currentStatePtr); // Lost forever!
    }
    
    const statePtr = wasm.create_app_state(userId, username);
    currentStatePtr = statePtr;
}
```

## 💾 Memory Leak Details

Each `AppState` object contains:
- `user_id`: 4 bytes
- `username`: Variable (String allocation)
- `session_token`: Variable (String allocation)
- `preferences`: 
  - Strings: ~100 bytes
  - `data`: **102,400 bytes (100KB vector)**
- `activity_log`: Variable (Vec allocation)

**Total per leak**: ~100KB minimum

With 100 user interactions:
- Memory leaked: ~10MB
- Heap pressure increases
- GC cannot reclaim (not JS memory)
- WASM heap eventually exhausted

## 🛠️ How to Fix

### Option 1: Add Deallocation Function (Recommended)

1. Uncomment the `free_app_state` function in `src/lib.rs`:

```rust
#[wasm_bindgen]
pub fn free_app_state(state_ptr: usize) {
    unsafe {
        let boxed_state = Box::from_raw(state_ptr as *mut AppState);
        drop(boxed_state);
    }
}
```

2. Update JavaScript to call it:

```javascript
function createNewUser() {
    // ✅ Free the old state before creating new one
    if (currentStatePtr !== null) {
        wasm.free_app_state(currentStatePtr);
    }
    
    const statePtr = wasm.create_app_state(userId, username);
    currentStatePtr = statePtr;
}

function performAction() {
    if (currentStatePtr === null) return;
    
    // ✅ Free old state when updating
    const oldPtr = currentStatePtr;
    const newPtr = wasm.update_app_state(currentStatePtr, action, details);
    
    if (oldPtr !== newPtr) {
        wasm.free_app_state(oldPtr);
    }
    
    currentStatePtr = newPtr;
}
```

3. Rebuild the module:

```bash
cd "Module 18"
wasm-pack build --target web
```

### Option 2: Use Serializable Objects Instead

Replace raw pointers with serialized data:

```rust
#[wasm_bindgen]
pub fn create_app_state(user_id: u32, username: String) -> JsValue {
    let state = AppState::new(user_id, username);
    // Returns a serialized JS object (no raw pointer)
    serde_wasm_bindgen::to_value(&state).unwrap()
}
```

This eliminates the need for manual deallocation.

## 📊 Reproduction Steps

1. Open `index.html` in a browser
2. Click "Create New User" multiple times
3. Click "Perform Action" repeatedly
4. Watch the "WASM Memory" stat increase
5. Click "Stress Test (100x)" to accelerate the leak
6. Observe:
   - Memory usage climbs steadily
   - "Leaked Objects" counter increases
   - Eventually: Browser becomes unresponsive or crashes

## 🧪 Testing the Fix

After implementing the fix:

1. Rebuild: `wasm-pack build --target web`
2. Refresh the page
3. Perform 100+ interactions
4. Memory usage should remain stable (~2-5 MB)
5. No crashes or errors

## 🎓 Key Lessons

1. **Raw pointers require manual memory management**: Unlike Rust's automatic memory management, raw pointers need explicit deallocation
2. **FFI boundaries need careful attention**: When crossing language boundaries, ownership rules must be explicit
3. **Missing deallocators are silent killers**: No compiler warnings, only runtime memory exhaustion
4. **Always pair allocate with deallocate**: Every `into_raw()` needs a corresponding `from_raw()` + `drop()`

## 📚 Related Topics

- Rust ownership and borrowing
- FFI (Foreign Function Interface) best practices
- WASM memory model
- Manual memory management
- Resource acquisition is initialization (RAII)

## 🔗 References

- [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [Rust FFI Documentation](https://doc.rust-lang.org/nomicon/ffi.html)
- [Box::into_raw documentation](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.into_raw)
- [Box::from_raw documentation](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.from_raw)

## ⚙️ Build Instructions

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Build

```bash
cd "Module 18"
wasm-pack build --target web
```

### Run

```bash
# Option 1: Python
python3 -m http.server 8000

# Option 2: Node.js
npx http-server -p 8000

# Option 3: PHP
php -S localhost:8000
```

Then open: http://localhost:8000

## 🚨 Security Considerations

Raw pointer operations are inherently unsafe:
- **Use-after-free**: Using a pointer after it's been freed
- **Double-free**: Freeing the same pointer twice
- **Null pointer dereference**: Using an invalid pointer
- **Memory corruption**: Writing to freed memory

This module intentionally demonstrates these risks for educational purposes.

## 📝 License

Educational purposes only. Do not use in production code.
