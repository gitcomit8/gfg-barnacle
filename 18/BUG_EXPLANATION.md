# Bug Explanation: Raw Pointer Memory Leak

## The Flow of the Bug

```
┌─────────────────────────────────────────────────────────────────┐
│                    RUST SIDE (WASM)                             │
└─────────────────────────────────────────────────────────────────┘

    User calls create_app_state(1, "alice")
                    ↓
    1. AppState allocated on WASM heap (100KB+)
                    ↓
    2. Wrapped in Box (owned pointer)
                    ↓
    3. Box::into_raw() called
        - Ownership transferred OUT of Rust
        - Box destructor will NOT run
        - Memory stays allocated
                    ↓
    4. Raw pointer (0x12345678) returned to JS

┌─────────────────────────────────────────────────────────────────┐
│                  JAVASCRIPT SIDE                                │
└─────────────────────────────────────────────────────────────────┘

    Receives: statePtr = 0x12345678
                    ↓
    5. Pointer stored in variable: currentStatePtr
                    ↓
    User calls create_app_state(2, "bob")
                    ↓
    6. NEW AppState allocated (another 100KB+)
                    ↓
    7. New pointer (0x87654321) returned
                    ↓
    8. currentStatePtr = 0x87654321
        ⚠️ OLD POINTER (0x12345678) IS LOST!
        ⚠️ MEMORY AT 0x12345678 IS NEVER FREED!
                    ↓
    9. 100KB+ LEAKED FOREVER

            REPEAT FOR EACH INTERACTION
                    ↓ ↓ ↓
            MEMORY LEAK ACCUMULATES
                    ↓ ↓ ↓
           WASM HEAP GETS EXHAUSTED
                    ↓ ↓ ↓
     RuntimeError: memory access out of bounds
```

## Memory State Timeline

```
TIME: T0 (Initial)
┌─────────────────────────────────────┐
│  WASM HEAP (2MB initial)            │
│  [free space: 2MB]                  │
│                                     │
└─────────────────────────────────────┘
JS Variables: currentStatePtr = null


TIME: T1 (After create_app_state(1, "alice"))
┌─────────────────────────────────────┐
│  WASM HEAP                          │
│  ┌──────────────┐                   │
│  │ AppState #1  │ ← 0x1000          │
│  │  (100KB)     │                   │
│  └──────────────┘                   │
│  [free space: ~1.9MB]               │
└─────────────────────────────────────┘
JS Variables: currentStatePtr = 0x1000


TIME: T2 (After create_app_state(2, "bob"))
┌─────────────────────────────────────┐
│  WASM HEAP                          │
│  ┌──────────────┐                   │
│  │ AppState #1  │ ← 0x1000 (LEAKED!)│
│  │  (100KB)     │   ↑               │
│  └──────────────┘   │ No reference  │
│  ┌──────────────┐   │ from JS!      │
│  │ AppState #2  │ ← 0x2000          │
│  │  (100KB)     │                   │
│  └──────────────┘                   │
│  [free space: ~1.8MB]               │
└─────────────────────────────────────┘
JS Variables: currentStatePtr = 0x2000
              (0x1000 is lost!)


TIME: T3 (After 20 more interactions)
┌─────────────────────────────────────┐
│  WASM HEAP                          │
│  ┌──────────────┐                   │
│  │ Leaked #1    │ ← 0x1000          │
│  ├──────────────┤                   │
│  │ Leaked #2    │ ← 0x2000          │
│  ├──────────────┤                   │
│  │ Leaked #3-20 │ ← 0x3000-0x14000  │
│  ├──────────────┤                   │
│  │ AppState #22 │ ← 0x15000         │
│  │  (current)   │                   │
│  └──────────────┘                   │
│  [free space: ~0MB] ← EXHAUSTED!    │
└─────────────────────────────────────┘
JS Variables: currentStatePtr = 0x15000
              (21 states leaked = ~2.1MB)

⚠️ NEXT ALLOCATION WILL FAIL! ⚠️
```

## Code Path Analysis

### Rust: Creating State

```rust
#[wasm_bindgen]
pub fn create_app_state(user_id: u32, username: String) -> usize {
    // Step 1: Allocate on heap
    let state = AppState::new(user_id, username);
    // state owns: username (String), session_token (String), 
    //             preferences (with 100KB data Vec), activity_log (Vec)
    
    // Step 2: Wrap in Box (smart pointer)
    let boxed_state = Box::new(state);
    // boxed_state is on stack, points to state on heap
    // When boxed_state goes out of scope normally, 
    // its destructor would free the heap memory
    
    // Step 3: Convert to raw pointer (OWNERSHIP TRANSFER)
    let raw_ptr = Box::into_raw(boxed_state);
    // ⚠️ into_raw() moves ownership OUT of Rust
    // ⚠️ Box destructor will NOT run
    // ⚠️ Memory will NOT be freed automatically
    // ⚠️ Caller (JavaScript) is now responsible for freeing!
    
    // Step 4: Return as usize (pointer value)
    raw_ptr as usize
    // Returns: 0x12345678 (example address)
}
// Function ends, but memory stays allocated!
```

### JavaScript: Using State (BUGGY)

```javascript
let currentStatePtr = null;

function createNewUser() {
    // ❌ BUG: Never free the old pointer!
    if (currentStatePtr !== null) {
        // OLD STATE IS ABANDONED HERE
        // Memory at currentStatePtr is now LEAKED
        // No way to access it anymore
        leakedPointers.push(currentStatePtr); // Just for tracking
    }
    
    // Create new state
    const statePtr = wasm.create_app_state(userId, username);
    // New allocation: 0x87654321
    
    // Overwrite old pointer
    currentStatePtr = statePtr;
    // Old value (0x12345678) is lost forever
    // Memory at 0x12345678 can never be freed
}
```

### JavaScript: Using State (FIXED)

```javascript
let currentStatePtr = null;

function createNewUser() {
    // ✅ FIX: Free the old pointer first!
    if (currentStatePtr !== null) {
        wasm.free_app_state(currentStatePtr);
        // This calls Box::from_raw() + drop()
        // Memory at currentStatePtr is now freed
    }
    
    // Create new state
    const statePtr = wasm.create_app_state(userId, username);
    currentStatePtr = statePtr;
}
```

## Why `update_app_state` Also Leaks

```rust
#[wasm_bindgen]
pub fn update_app_state(state_ptr: usize, action: String, details: String) -> usize {
    unsafe {
        // Step 1: Reconstruct Box from raw pointer
        let mut boxed_state = Box::from_raw(state_ptr as *mut AppState);
        // Now Rust owns the memory again
        
        // Step 2: Modify it
        boxed_state.add_activity(action, details);
        
        // Step 3: Clone and create NEW Box
        let new_boxed_state = Box::new((*boxed_state).clone());
        // Allocates NEW memory for the clone
        
        // Step 4: Convert NEW Box to raw pointer
        let new_raw_ptr = Box::into_raw(new_boxed_state);
        // NEW ownership transferred to JS
        
        // Step 5: Forget original (prevent drop)
        std::mem::forget(boxed_state);
        // ⚠️ Original memory stays allocated
        // ⚠️ JS should free it, but doesn't!
        
        // Return NEW pointer
        new_raw_ptr as usize
    }
}
```

JavaScript calling this:

```javascript
function performAction() {
    const oldPtr = currentStatePtr;       // 0x12345678
    const newPtr = wasm.update_app_state( // Returns 0x87654321
        currentStatePtr,
        'click',
        'User clicked button'
    );
    
    currentStatePtr = newPtr;             // 0x87654321
    // ❌ oldPtr (0x12345678) is now leaked!
    // ❌ Both old AND new state are in memory
}
```

## The Missing Function

This function exists in the code but is COMMENTED OUT:

```rust
/*
#[wasm_bindgen]
pub fn free_app_state(state_ptr: usize) {
    unsafe {
        // Step 1: Reconstruct Box from raw pointer
        let boxed_state = Box::from_raw(state_ptr as *mut AppState);
        // Rust takes ownership back
        
        // Step 2: Drop it
        drop(boxed_state);
        // Box's destructor runs
        // - Frees the AppState
        // - Frees all owned data (Strings, Vecs)
        // - Returns memory to WASM heap
    }
    // When function ends, all memory is freed
}
*/
```

**Uncommenting this function is the key to fixing the bug!**

## Comparison: Correct vs Buggy

### ❌ Buggy Pattern (Current)

```
Allocate → Use → Allocate → Use → Allocate
  100KB     ✓     100KB     ✓     100KB
    ↓              ↓              ↓
  LEAK           LEAK           LEAK
```

Memory: 0KB → 100KB → 200KB → 300KB → CRASH

### ✅ Correct Pattern (Fixed)

```
Allocate → Use → Free → Allocate → Use → Free
  100KB     ✓    (0KB)   100KB     ✓    (0KB)
```

Memory: 0KB → 100KB → 0KB → 100KB → 0KB → Stable

## Real-World Analogy

**Buggy Version:**
- Restaurant gives you a table (pointer)
- You eat, leave, but don't tell them you're done
- Get a new table for next meal
- Old table stays reserved forever (leaked)
- Eventually: "Sorry, no tables available" (out of memory)

**Fixed Version:**
- Restaurant gives you a table (pointer)
- You eat, tell them you're done (free_app_state)
- They clean and free the table
- You get a new table for next meal
- System sustainable forever

## Detection and Diagnosis

### Symptoms:
1. Increasing memory usage over time
2. Browser tab slowdown
3. Eventually: `RuntimeError: memory access out of bounds`
4. No obvious error messages initially

### How to Detect:
1. Monitor `get_memory_info()` output
2. Watch for linear memory growth
3. Use browser DevTools Memory profiler
4. Check WASM memory allocation patterns

### Where to Look:
1. Functions returning raw pointers (`usize`)
2. Absence of corresponding `free_*` functions
3. JavaScript not calling dealloc functions
4. `Box::into_raw()` without matching `Box::from_raw()` + `drop()`

## Prevention Strategies

1. **Prefer safe abstractions**: Use `wasm-bindgen` types like `JsValue`
2. **Document ownership**: Clearly state who owns what
3. **Pair allocate/deallocate**: Always provide both functions
4. **Use RAII wrappers**: Create JS classes that auto-free on destruction
5. **Add memory tracking**: Log allocations and frees
6. **Test memory usage**: Monitor memory in integration tests
