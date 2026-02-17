# Module 18: Re-entrancy Deadlock (JS-to-Rust-to-JS)

## 🐛 Bug Description

This module contains an intentional **re-entrancy deadlock bug** that occurs when Rust code calls JavaScript closures that accidentally call back into Rust functions.

### The Problem

The module uses `RefCell` for interior mutability in Rust. When a Rust function borrows the `RefCell` and then calls a JavaScript callback, if that callback tries to call another Rust function that also needs to borrow the same `RefCell`, it triggers a runtime panic.

### The Bug Flow

1. **JavaScript calls** `processor.process_items(callback)`
2. **Rust function** `process_items()` borrows `state` **mutably** with `self.state.borrow_mut()`
3. **Rust calls** the JavaScript `callback` function (while still holding the mutable borrow)
4. **JavaScript callback** calls another Rust method like `processor.get_item_count()`
5. **Rust function** `get_item_count()` tries to borrow `state` **immutably** with `self.state.borrow()`
6. **💥 PANIC**: `RefCell` panics with **"already borrowed: BorrowMutError"**

### The Consequence

> ⚠️ **Critical Runtime Error**: The application crashes with a cryptic panic message that doesn't clearly indicate it's a re-entrancy issue. Developers typically blame their JavaScript code for "calling functions in the wrong order" rather than recognizing the re-entrant borrow conflict.

## 🔧 Technical Details

### Root Cause

**Re-entrant Borrowing of RefCell**:

```rust
// ❌ BUGGY CODE (in this module)
pub fn process_items(&self, callback: &js_sys::Function) -> Result<(), JsValue> {
    // BUG: Borrow state mutably
    let mut state = self.state.borrow_mut();
    
    for (index, item) in state.items.iter().enumerate() {
        // BUG: Call JS callback while holding the borrow
        callback.call2(&this, &item_js, &index_js)?;
        
        // If callback calls get_item_count(), it panics!
    }
    Ok(())
}

pub fn get_item_count(&self) -> usize {
    // BUG: This will panic if called from within the callback above
    let state = self.state.borrow();  // ❌ "already borrowed!"
    state.items.len()
}
```

### The Panic Message

```
RuntimeError: unreachable
    at __rust_start_panic
    at rust_panic
    at std::panicking::rust_panic_with_hook
    at std::panicking::begin_panic_handler
    at core::panicking::panic_fmt
    at core::result::unwrap_failed
    at core::cell::RefCell<T>::borrow
    
thread 'main' panicked at 'already borrowed: BorrowMutError'
```

### Why It's Difficult to Debug

1. **Cryptic error message**: "already borrowed: BorrowMutError" doesn't explain re-entrancy
2. **JavaScript in the middle**: Stack trace shows JS calls, obscuring the Rust-to-Rust connection
3. **Appears to be JS's fault**: Developers blame their callback for "calling at the wrong time"
4. **Works in simple cases**: Bug only appears when callbacks try to query state
5. **No compile-time detection**: Rust's borrow checker can't detect this runtime issue

### Call Stack Visualization

```
JavaScript                 Rust
─────────────────────────────────────────
                          process_items()
                          ├─ borrow_mut() ✓
                          │
  callback()          ◄───┤
  ├─ do something        
  └─ get_item_count() ───►│
                          │
                          get_item_count()
                          └─ borrow() ❌ PANIC!
                             (already borrowed!)
```

## 🏗️ Module Structure

```
Module 18/
├── Cargo.toml              # Rust/WASM package configuration
├── src/
│   └── lib.rs              # Main buggy Rust code with re-entrancy issue
├── demo.html               # Interactive demo showing the bug
├── README.md               # This file
├── QUICKSTART.md           # Quick setup and reproduction guide
├── TECHNICAL.md            # Deep technical analysis
├── SECURITY.md             # Security implications
├── integration-example.jsx # React component example
├── package.json            # NPM package configuration
└── build.sh                # Build script
```

## 🚀 Building the Module

### Prerequisites

- Rust toolchain (1.70+)
- wasm-pack for WebAssembly compilation

### Build Commands

```bash
# Install wasm-pack if not already installed
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build the WASM module
cd "Module 18"
wasm-pack build --target web

# Or use the provided build script
chmod +x build.sh
./build.sh

# The compiled WASM will be in pkg/ directory
```

## 🧪 Testing the Bug

### Method 1: Run the Interactive Demo

```bash
# Open demo.html in a browser
open demo.html

# Or serve it with a local server
python3 -m http.server 8000
# Then visit http://localhost:8000/demo.html
```

### Method 2: Minimal JavaScript Reproduction

```javascript
import init, { DataProcessor } from './pkg/reentrancy_deadlock_module.js';

await init();

const processor = new DataProcessor();
processor.add_item("item1");
processor.add_item("item2");
processor.add_item("item3");

// This will trigger the bug!
processor.process_items((item, index) => {
    console.log(`Processing: ${item}`);
    
    // BUG: This call back into Rust will cause a panic!
    const count = processor.get_item_count(); // ❌ Panic!
    console.log(`Total items: ${count}`);
});
```

### Method 3: Run Unit Tests

```bash
cargo test
```

## 🔍 Code Analysis

### Vulnerable Functions

1. **`process_items(callback)`**: Primary bug location - borrows mutably while calling JS
2. **`validate_items(validator)`**: Same issue - holds borrow during validation callbacks
3. **`transform_items(transformer)`**: Same issue - holds borrow during transformation
4. **`BatchProcessor::process_all()`**: Similar re-entrancy vulnerability

### Safe Functions

These functions are safe to call from callbacks (though they will still panic if called during the above functions):

- `get_item_count()`
- `get_operation_count()`
- `get_processed_count()`
- `get_item(index)`
- `get_summary()`

### Bug Manifestation Examples

#### Example 1: Counting During Processing

```javascript
// ❌ This WILL panic
processor.process_items((item, index) => {
    console.log(`Item ${index}: ${item}`);
    console.log(`Total: ${processor.get_item_count()}`); // PANIC!
});
```

#### Example 2: Nested Queries

```javascript
// ❌ This WILL panic
processor.validate_items((item, index) => {
    const processedCount = processor.get_processed_count(); // PANIC!
    return processedCount < 10;
});
```

#### Example 3: Transform with State Query

```javascript
// ❌ This WILL panic
processor.transform_items((item) => {
    const summary = processor.get_summary(); // PANIC!
    return item.toUpperCase();
});
```

## ✅ How to Fix This Bug

### Solution 1: Drop the Borrow Before Calling JS (Recommended)

```rust
// ✅ FIXED VERSION
pub fn process_items(&self, callback: &js_sys::Function) -> Result<(), JsValue> {
    // Clone the data we need before calling JS
    let items = {
        let state = self.state.borrow();
        state.items.clone()
    }; // Borrow is dropped here!
    
    for (index, item) in items.iter().enumerate() {
        let item_js = JsValue::from_str(item);
        let index_js = JsValue::from_f64(index as f64);
        
        // Now it's safe to call JS - no active borrows!
        callback.call2(&JsValue::null(), &item_js, &index_js)?;
    }
    
    // Update state after all callbacks complete
    let mut state = self.state.borrow_mut();
    state.processed_count += items.len();
    state.total_operations += 1;
    
    Ok(())
}
```

### Solution 2: Use try_borrow() for Defensive Programming

```rust
// ✅ DEFENSIVE VERSION
pub fn get_item_count(&self) -> Result<usize, JsValue> {
    match self.state.try_borrow() {
        Ok(state) => Ok(state.items.len()),
        Err(_) => Err(JsValue::from_str(
            "Cannot get item count: state is currently borrowed. \
             This might be due to calling this method from within a callback."
        )),
    }
}
```

### Solution 3: Use Separate State for Metadata

```rust
// ✅ ARCHITECTURAL FIX
pub struct DataProcessor {
    items: Rc<RefCell<Vec<String>>>,
    metadata: Rc<RefCell<Metadata>>, // Separate RefCell!
}

// Now you can borrow items and metadata independently
```

### Solution 4: Queue Operations Instead of Immediate Execution

```rust
// ✅ QUEUE-BASED FIX
pub struct DataProcessor {
    state: Rc<RefCell<DataState>>,
    pending_queries: Rc<RefCell<Vec<Query>>>,
}

pub fn get_item_count(&self) -> usize {
    // Check if we're in a callback
    if self.is_processing() {
        // Queue the query for later
        self.queue_query(Query::GetItemCount);
        return 0; // Return placeholder
    }
    
    // Normal path
    self.state.borrow().items.len()
}
```

## 🎯 Learning Objectives

By studying this buggy module, developers will learn:

1. **RefCell semantics**: How Rust's runtime borrow checking works
2. **Re-entrancy issues**: Recognizing re-entrant call patterns
3. **FFI boundary challenges**: Issues when crossing language boundaries
4. **Defensive programming**: Using `try_borrow()` to handle potential conflicts
5. **Architectural patterns**: Designing APIs that avoid re-entrancy traps

## 📊 Real-World Impact

This bug pattern commonly occurs in:

- **WASM modules with callbacks**: Any Rust WASM code that calls JS callbacks
- **Plugin systems**: When host code calls plugins that call back to host
- **Event handlers**: UI frameworks where events trigger other events
- **Async callbacks**: Completion handlers that query state
- **Observer patterns**: Observers that modify or query observed state

### Common Scenarios

1. **Progress callbacks**: Querying progress during a processing loop
2. **Validation with state access**: Validators that check global state
3. **Transform with logging**: Transformers that log to a shared logger
4. **Event handlers**: Events that trigger other events on the same object
5. **Middleware chains**: Middleware that calls other middleware

## 📚 Related Concepts

- **RefCell and Interior Mutability** in Rust
- **Borrow Checking** (compile-time vs runtime)
- **Re-entrancy** in concurrent/callback systems
- **Foreign Function Interface (FFI)** boundary issues
- **WebAssembly** integration with JavaScript
- **Call Stack Management** across language boundaries

## ⚠️ Security Implications

While this is primarily a correctness bug, it has security implications:

1. **Denial of Service**: Application crashes when bug is triggered
2. **Data Inconsistency**: Incomplete operations leave state inconsistent
3. **Error Information Leakage**: Panic messages might reveal internal structure
4. **Reliability Impact**: Users lose trust in the application
5. **Availability**: Service becomes unavailable during crashes

## 🎓 Educational Value

This module teaches developers to:

- **Recognize re-entrancy patterns**: Identify when callbacks might call back
- **Design safe APIs**: Create APIs that prevent re-entrant borrows
- **Debug FFI issues**: Trace problems across language boundaries
- **Handle runtime errors**: Use `try_borrow()` and proper error handling
- **Think about call flows**: Visualize complex call chains

## 🔗 Integration with Web Applications

This Rust/WASM module can be integrated into:

- **React applications**: Components that process data with callbacks
- **Vue.js applications**: Reactive systems with computed properties
- **Angular applications**: Services with observable streams
- **Vanilla JavaScript**: Any JS code that needs data processing
- **Node.js backends**: Server-side data processing with WASM

## 🏷️ Tags

`#bug` `#re-entrancy` `#deadlock` `#refcell` `#wasm` `#rust` `#javascript` `#callback` `#ffi` `#borrow-checker` `#runtime-error` `#panic` `#webassembly`

## 📝 License

This module is for educational purposes to demonstrate re-entrancy bugs in Rust/JS integration.

---

**⚠️ IMPORTANT**: This module is **intentionally buggy** for educational purposes. **DO NOT use in production applications without fixing the re-entrancy issue!**

## 🤔 Challenge

Can you identify ALL the vulnerable functions in this module? Here are some hints:

1. Find all functions that call `borrow_mut()` and then invoke JS callbacks
2. Locate all functions that call `borrow()` (which will panic if called from a callback)
3. Trace the call chain: Rust → JS → Rust to see the re-entrant path
4. Consider what happens with `BatchProcessor::process_all()`

**Bonus Challenge**: Try to fix the bug without breaking the API! Can you make it safe while keeping the same function signatures?

Good luck! 🎯
