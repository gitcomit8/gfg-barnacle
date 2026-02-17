# Technical Deep Dive: Re-entrancy Deadlock

## Overview

This document provides a deep technical analysis of the re-entrancy deadlock bug in Module 18.

## The Rust Type System and Borrowing

### Compile-Time Borrow Checking

Rust's borrow checker enforces these rules at **compile time**:

```rust
let mut x = 5;
let r1 = &x;      // OK: immutable borrow
let r2 = &x;      // OK: multiple immutable borrows
let r3 = &mut x;  // ❌ ERROR: can't borrow mutably while immutably borrowed
```

### Runtime Borrow Checking with RefCell

`RefCell<T>` moves borrow checking to **runtime**:

```rust
use std::cell::RefCell;

let x = RefCell::new(5);
let r1 = x.borrow();      // OK: immutable borrow
let r2 = x.borrow();      // OK: multiple immutable borrows
let r3 = x.borrow_mut();  // ❌ PANIC: can't borrow mutably while immutably borrowed
```

## The Bug Architecture

### Data Structure

```rust
pub struct DataProcessor {
    state: Rc<RefCell<DataState>>,
}

struct DataState {
    items: Vec<String>,
    processed_count: usize,
    total_operations: usize,
}
```

- `Rc<T>`: Reference counted pointer (enables multiple owners)
- `RefCell<T>`: Interior mutability with runtime borrow checking
- `DataState`: The actual data being protected

### The Problematic Pattern

```rust
pub fn process_items(&self, callback: &js_sys::Function) -> Result<(), JsValue> {
    // 1. Acquire mutable borrow
    let mut state = self.state.borrow_mut();  // ✓ OK
    
    // 2. Clone data to iterate
    let items = state.items.clone();
    
    // 3. Call JavaScript callback
    for (index, item) in items.iter().enumerate() {
        // 4. JavaScript is called while holding the borrow!
        callback.call2(&this, &item_js, &index_js)?;
        
        // 5. If callback calls get_item_count()...
        state.processed_count += 1;  // Still borrowed here!
    }
    
    Ok(())
}

pub fn get_item_count(&self) -> usize {
    // 6. Try to borrow (immutably) while already borrowed mutably
    let state = self.state.borrow();  // ❌ PANIC!
    state.items.len()
}
```

## Call Flow Analysis

### Normal Flow (No Re-entrancy)

```
JavaScript                 Rust                          RefCell State
─────────────────────────────────────────────────────────────────────────
                          process_items()
                          ├─ borrow_mut()                ✓ [Borrowed Mut]
                          │
  callback()          ◄───┤
  ├─ console.log()                                       ✓ [Still Borrowed Mut]
  └─ return              
                          │                              ✓ [Still Borrowed Mut]
                          └─ state updated
                                                         ✓ [Released]
```

### Buggy Flow (With Re-entrancy)

```
JavaScript                 Rust                          RefCell State
─────────────────────────────────────────────────────────────────────────
                          process_items()
                          ├─ borrow_mut()                ✓ [Borrowed Mut]
                          │
  callback()          ◄───┤
  ├─ console.log()                                       ✓ [Still Borrowed Mut]
  └─ get_item_count() ───►│
                          │
                          get_item_count()
                          └─ borrow()                    ❌ [PANIC!]
                                                            [Already Borrowed Mut]
```

## Why RefCell Panics

RefCell maintains runtime state tracking:

```rust
pub struct RefCell<T> {
    borrow: Cell<BorrowFlag>,  // Tracks borrow state
    value: UnsafeCell<T>,       // The actual data
}

// BorrowFlag can be:
// - 0: Not borrowed
// - Positive: Number of immutable borrows
// - -1: Mutably borrowed

impl<T> RefCell<T> {
    pub fn borrow(&self) -> Ref<T> {
        if self.borrow.get() == -1 {
            panic!("already borrowed: BorrowMutError");
        }
        // ... rest of implementation
    }
}
```

## The JavaScript-Rust Boundary

### WASM Binding Layer

When you call a Rust function from JavaScript:

```
JavaScript Side          wasm-bindgen          Rust Side
─────────────────────────────────────────────────────────
processor.process_items  ──────►  Marshalling  ──────►  process_items()
                                  - Convert args
                                  - Call Rust fn
                                  
In Rust:
  callback.call2()       ◄──────  Conversion   ◄──────  Return to JS
                                  
Back to JavaScript:
  processor.get_item_count() ───►  Marshalling  ──────►  get_item_count()
                                                            ❌ PANIC!
```

The key issue: **Rust function frames are still on the stack** when JavaScript calls back.

## Memory Safety Implications

### Why RefCell Exists

Without RefCell's runtime checking:

```rust
// If this were allowed:
let mut state = /* ... */;
let r1 = &state;           // Immutable reference
let r2 = &mut state;       // Mutable reference

*r2 = something;           // Modify through r2
println!("{}", r1.field);  // r1 now points to invalid data!
                           // Undefined behavior! 💥
```

RefCell prevents this by panicking instead of allowing undefined behavior.

### The Trade-off

- **Compile-time borrow checking**: Catches errors at compile time, zero runtime cost
- **Runtime borrow checking (RefCell)**: Flexible, but can panic at runtime

## Advanced Patterns

### Pattern 1: Borrow Scope Limiting

```rust
// ❌ BAD: Hold borrow across callback
pub fn process(&self, callback: &Function) -> Result<(), JsValue> {
    let mut state = self.state.borrow_mut();
    for item in &state.items {
        callback.call1(&JsValue::null(), &JsValue::from(item))?;
    }
    Ok(())
}

// ✅ GOOD: Release borrow before callback
pub fn process(&self, callback: &Function) -> Result<(), JsValue> {
    let items = {
        let state = self.state.borrow();
        state.items.clone()
    }; // Borrow dropped here!
    
    for item in &items {
        callback.call1(&JsValue::null(), &JsValue::from(item))?;
    }
    
    // Update state after all callbacks
    self.state.borrow_mut().processed_count += items.len();
    Ok(())
}
```

### Pattern 2: Try-Borrow for Defensive Code

```rust
pub fn get_item_count(&self) -> Result<usize, JsValue> {
    match self.state.try_borrow() {
        Ok(state) => Ok(state.items.len()),
        Err(_) => Err(JsValue::from_str(
            "State is currently borrowed. Are you calling this from a callback?"
        ))
    }
}
```

### Pattern 3: Separate RefCells

```rust
pub struct DataProcessor {
    items: Rc<RefCell<Vec<String>>>,      // Can be borrowed independently
    metadata: Rc<RefCell<Metadata>>,       // Can be borrowed independently
}

pub fn process_items(&self, callback: &Function) -> Result<(), JsValue> {
    let mut items = self.items.borrow_mut();  // Only borrow items
    
    for item in items.iter() {
        callback.call1(&JsValue::null(), &JsValue::from(item))?;
        
        // This is now safe! metadata is a separate RefCell
        let mut meta = self.metadata.borrow_mut();
        meta.processed_count += 1;
    }
    
    Ok(())
}
```

### Pattern 4: Message Passing

```rust
use std::sync::mpsc;

pub struct DataProcessor {
    state: Rc<RefCell<DataState>>,
    query_queue: Rc<RefCell<Vec<Query>>>,
}

pub fn get_item_count(&self) -> usize {
    // If we're processing, queue the query
    if self.is_processing() {
        self.queue_query(Query::GetItemCount);
        return 0;  // Return placeholder
    }
    
    // Otherwise, execute immediately
    self.state.borrow().items.len()
}
```

## Performance Considerations

### RefCell Overhead

```rust
// RefCell adds:
// - Size: 1 extra usize (for BorrowFlag)
// - Cost: 1 check per borrow (cheap!)
// - Panic: Expensive when it happens (stack unwinding)

// Comparison:
struct WithMutex {
    data: Mutex<DataState>,  // Larger, thread-safe, can block
}

struct WithRefCell {
    data: RefCell<DataState>,  // Smaller, single-thread, can panic
}

struct WithRwLock {
    data: RwLock<DataState>,  // Larger, thread-safe, can block
}
```

### When to Use RefCell

✅ **Good Use Cases:**
- Single-threaded scenarios
- Known borrowing patterns
- Interior mutability needed
- Performance-critical single-threaded code

❌ **Bad Use Cases:**
- Multi-threaded code (use `Mutex` or `RwLock`)
- Unpredictable borrowing patterns
- When panics are unacceptable
- When callbacks can re-enter

## Debugging Techniques

### Technique 1: Stack Traces

```rust
use std::backtrace::Backtrace;

pub fn get_item_count(&self) -> usize {
    match self.state.try_borrow() {
        Ok(state) => state.items.len(),
        Err(_) => {
            let bt = Backtrace::capture();
            eprintln!("Borrow failed! Backtrace:\n{}", bt);
            panic!("already borrowed");
        }
    }
}
```

### Technique 2: Logging

```rust
pub fn process_items(&self, callback: &Function) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str("process_items: acquiring borrow"));
    let mut state = self.state.borrow_mut();
    console::log_1(&JsValue::from_str("process_items: borrow acquired"));
    
    for item in &state.items {
        console::log_1(&JsValue::from_str("process_items: calling callback"));
        callback.call1(&JsValue::null(), item)?;
        console::log_1(&JsValue::from_str("process_items: callback returned"));
    }
    
    console::log_1(&JsValue::from_str("process_items: releasing borrow"));
    Ok(())
}
```

### Technique 3: Borrow State Tracking

```rust
pub struct DataProcessor {
    state: Rc<RefCell<DataState>>,
    borrow_count: Rc<RefCell<usize>>,  // Track manual borrow count
}

pub fn process_items(&self, callback: &Function) -> Result<(), JsValue> {
    *self.borrow_count.borrow_mut() += 1;
    
    // ... processing ...
    
    *self.borrow_count.borrow_mut() -= 1;
    Ok(())
}

pub fn get_item_count(&self) -> Result<usize, JsValue> {
    if *self.borrow_count.borrow() > 0 {
        return Err(JsValue::from_str("Cannot query while processing!"));
    }
    
    Ok(self.state.borrow().items.len())
}
```

## Comparison with Other Languages

### JavaScript (No Issue)

```javascript
// JavaScript has no concept of borrowing
const processor = {
    items: [],
    processItems(callback) {
        for (let item of this.items) {
            callback(item);
            // Can freely access this.items here
            console.log(this.items.length); // Always works
        }
    }
};
```

### C++ (Undefined Behavior)

```cpp
// C++ can have similar issues with iterators
std::vector<int> items = {1, 2, 3};
for (auto& item : items) {
    callback(item);
    items.push_back(4);  // ⚠️  Iterator invalidation! Undefined behavior!
}
```

### Rust's Advantage

Rust prevents undefined behavior by **panicking** instead of continuing with invalid state.

## Conclusion

The re-entrancy bug demonstrates:

1. **Runtime borrow checking** with RefCell
2. **FFI boundary challenges** between Rust and JavaScript
3. **The importance of borrow scope management**
4. **Trade-offs between flexibility and safety**

Understanding this bug helps developers:
- Design better APIs
- Recognize re-entrancy patterns
- Use RefCell appropriately
- Debug complex borrow conflicts

## Further Reading

- [Rust Book: RefCell and Interior Mutability](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [Rust RFC on Cell and RefCell](https://github.com/rust-lang/rfcs/blob/master/text/0528-interior-mutability.md)
