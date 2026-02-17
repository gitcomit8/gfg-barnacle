# Quick Start Guide - Module 18

## 🚀 Running the Module

### Prerequisites

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Install wasm-pack**:
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

### Build the WASM Module

```bash
cd "Module 18"
wasm-pack build --target web
```

This creates a `pkg/` directory with the compiled WASM module.

### Start a Local Server

Choose one of these options:

**Option 1: Python** (Recommended)
```bash
python3 -m http.server 8000
```

**Option 2: Node.js**
```bash
npx http-server -p 8000
```

**Option 3: PHP**
```bash
php -S localhost:8000
```

### Open in Browser

Navigate to: http://localhost:8000

## 🎯 What You'll See

- **Interactive Demo**: Buttons to create users and perform actions
- **Live Stats**: Real-time tracking of:
  - States created
  - Actions performed
  - WASM memory usage
  - Leaked objects count
- **Activity Log**: Console showing all operations
- **Memory Warning**: Alert when memory usage is high

## 🐛 Experiencing the Bug

### Step 1: Normal Usage
1. Click **"Create New User"** a few times
2. Click **"Perform Action"** multiple times
3. Watch the memory stats

You'll see:
- Memory usage gradually increasing
- Leaked objects counter going up
- No immediate errors

### Step 2: Accelerate the Leak
1. Click **"Simulate Activity (10x)"**
   - Creates 10 interactions rapidly
   - Memory jumps ~1MB

2. Click **"Stress Test (100x)"**
   - Creates 100 users/actions
   - Memory increases ~10MB
   - May trigger memory warning

### Step 3: Observe the Crash (Optional)
- Keep clicking stress test
- Memory will continue climbing
- Eventually: Browser becomes unresponsive or crashes
- Error message: `RuntimeError: memory access out of bounds`

## 🔍 Understanding What's Happening

### Behind the Scenes

Each time you click "Create New User":
```javascript
// JavaScript creates a new state
const statePtr = wasm.create_app_state(userId, username);

// Old pointer is overwritten
currentStatePtr = statePtr;  // ❌ Old memory leaked!
```

In Rust:
```rust
// Allocates ~100KB on WASM heap
let state = AppState::new(user_id, username);

// Transfers ownership to JavaScript
let raw_ptr = Box::into_raw(Box::new(state));
```

**Problem**: JavaScript receives a pointer but never frees it!

### Memory Growth Pattern

```
Interaction 1:   ~2 MB (initial)
Interaction 10:  ~3 MB (+1 MB leaked)
Interaction 50:  ~7 MB (+5 MB leaked)
Interaction 100: ~12 MB (+10 MB leaked)
```

The memory never goes down because:
- JavaScript can't free WASM memory
- Rust gave up ownership (via `into_raw()`)
- No deallocation function is called

## 🛠️ Fixing the Bug

### Quick Fix Steps

1. **Edit `src/lib.rs`**:
   - Find the commented-out `free_app_state` function
   - Uncomment it (remove `/*` and `*/`)

2. **Rebuild**:
   ```bash
   wasm-pack build --target web
   ```

3. **Edit `index.html`**:
   - Find the `createNewUser` function
   - Add this before creating a new user:
   ```javascript
   if (currentStatePtr !== null) {
       wasm.free_app_state(currentStatePtr);
   }
   ```
   
   - Find the `performAction` function
   - Add this after getting the new pointer:
   ```javascript
   if (oldPtr !== newPtr) {
       wasm.free_app_state(oldPtr);
   }
   ```

4. **Test**:
   - Refresh the page
   - Repeat the stress test
   - Memory should stay stable (~2-5 MB)

## 📊 Verifying the Fix

After implementing the fix:

1. Click "Create New User" 100 times
2. Check memory stats:
   - Should remain around 2-5 MB
   - Leaked objects: 0
3. No memory warnings
4. No crashes

## 🎓 Learning Objectives

By working through this module, you'll understand:

1. **Raw Pointer Ownership**: How Rust's ownership transfers to foreign code
2. **Memory Management**: The need for explicit deallocation in FFI
3. **WASM Limitations**: JavaScript can't free WASM memory directly
4. **Debugging Leaks**: How to identify and fix memory leaks
5. **Best Practices**: Proper patterns for Rust-JavaScript integration

## 💡 Key Takeaways

### The Bug
- ❌ Returning raw pointers without deallocation functions
- ❌ JavaScript holding pointers it can't free
- ❌ Silent memory leak accumulation

### The Fix
- ✅ Always provide `free_*` functions for `*_new` functions
- ✅ Call deallocation before reassigning pointers
- ✅ Match every `Box::into_raw()` with `Box::from_raw()` + `drop()`

### Alternative Approaches
1. **Use `wasm-bindgen` types**: Return `JsValue` instead of raw pointers
2. **Use `serde`**: Serialize to JavaScript objects
3. **Create RAII wrappers**: JavaScript classes with destructors
4. **Reference counting**: Use `Rc` or `Arc` with manual deallocation

## 🔗 Additional Resources

- **README.md**: Comprehensive module documentation
- **BUG_EXPLANATION.md**: Detailed technical analysis
- **src/lib.rs**: Annotated source code with comments
- **index.html**: Demo page with inline documentation

## ⚠️ Common Issues

### Build Errors

**Error**: `wasm-pack: command not found`
- **Fix**: Install wasm-pack (see Prerequisites)

**Error**: `cargo: command not found`
- **Fix**: Install Rust (see Prerequisites)

### Runtime Errors

**Error**: `Cannot find module './pkg/raw_pointer_leak_module.js'`
- **Fix**: Run `wasm-pack build --target web` first

**Error**: Page shows but nothing happens
- **Fix**: 
  1. Check browser console for errors
  2. Ensure you're using a local server (not `file://`)
  3. Try a different browser (Chrome/Firefox recommended)

### Testing Issues

**Issue**: Memory doesn't increase
- **Check**: Are you clicking the buttons?
- **Check**: Is the WASM module loaded? (See activity log)
- **Check**: Browser DevTools console for errors

**Issue**: No crash after stress test
- **Note**: Modern browsers have large memory limits
- **Tip**: Use "Stress Test" multiple times or increase iterations

## 📝 Next Steps

1. Try fixing the bug yourself
2. Experiment with different approaches
3. Compare memory usage before/after fix
4. Read through the commented code
5. Understand the memory model

## 🤝 Need Help?

- Check the browser console for error messages
- Review the BUG_EXPLANATION.md for detailed analysis
- Examine the commented code in src/lib.rs
- Use browser DevTools Memory profiler

---

**Happy debugging!** 🐛🔍
