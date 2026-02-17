# Module 17: The Shadowed Canvas Context

## 🐛 Bug Description

This module contains an intentional **WebGL state caching bug** that creates non-deterministic rendering glitches in WASM-based image processing applications.

### The Problem

The module is designed for high-performance image processing using WebGL. To optimize performance, it caches the WebGL context state (current shader programs, texture bindings, buffer bindings) on the Rust/WASM side. However:

1. **WASM caches GL state** (shader programs, textures, buffers) for performance
2. **JavaScript UI code modifies GL context** between WASM calls (for rendering overlays, UI elements, etc.)
3. **WASM assumes cache is valid** and skips rebinding operations
4. **GL renders with wrong state** → visual glitches, inverted colors, corrupted output
5. **Bug is non-deterministic** - only appears after specific JS-side UI updates

### The Consequence

> ⚠️ **Critical Issue**: The rendered output will appear **glitched**, **inverted**, or **corrupted**, but only intermittently. The bug manifests after UI interactions like button clicks, overlay rendering, or other canvas operations from JavaScript. This makes it extremely difficult to reproduce and debug.

## 🔧 Technical Details

### Root Cause

The bug stems from aggressive state caching without invalidation:

```rust
// ❌ BUGGY CODE (in this module)
let program_id = program.as_ref() as *const _ as u32;
if self.state_cache.borrow().current_program != Some(program_id) {
    // Only bind if cache says we need to
    self.context.use_program(Some(program));
    self.state_cache.borrow_mut().current_program = Some(program_id);
} else {
    // BUG: Skip binding, assume state is still valid
    // But JS may have changed the current program!
    console::log_1(&JsValue::from_str("Cache hit: skipping bind (POTENTIAL BUG!)"));
}
```

### Why It's Difficult to Debug

1. **Non-deterministic**: Bug only appears after specific sequences of JS/WASM calls
2. **Looks like a driver bug**: Glitched rendering might be blamed on GPU drivers
3. **Works in isolation**: WASM-only tests work fine; bug needs JS interleaving
4. **Performance feature gone wrong**: The cache improves performance, making it seem beneficial
5. **No error messages**: WebGL silently uses wrong state, producing visual artifacts
6. **Timing dependent**: Fast UI updates are more likely to trigger the bug

### State Cache Structure

```rust
struct GLStateCache {
    current_program: Option<u32>,   // Cached shader program
    current_texture: Option<u32>,   // Cached texture binding
    current_buffer: Option<u32>,    // Cached buffer binding
    invert_mode: bool,              // Additional state flag
}
```

### Bug Manifestation Flow

```
Initial State:
  └─> WASM initializes, cache is empty

First WASM Call (apply_grayscale):
  └─> Cache miss → binds grayscale program
  └─> Updates cache: current_program = grayscale_id
  └─> Renders correctly ✓

JavaScript UI Update:
  └─> Renders UI overlay with different shader
  └─> GL state changes: current_program = overlay_shader_id
  └─> WASM cache is now STALE ✗

Second WASM Call (apply_grayscale):
  └─> Cache check: current_program == grayscale_id
  └─> Cache hit! Skip binding (WRONG!)
  └─> Renders with overlay_shader_id → GLITCH ✗
```

## 🏗️ Module Structure

```
Module 17/
├── Cargo.toml              # Rust/WASM package configuration
├── src/
│   └── lib.rs              # Main buggy Rust code with state caching
├── demo.html               # Interactive demo showing the bug
├── demo-exploit.js         # JavaScript that triggers the bug
├── build.sh                # Build script for WASM compilation
├── README.md               # This file
└── .gitignore              # Ignore build artifacts
```

## 🚀 Building the Module

### Prerequisites

- Rust toolchain (1.70+)
- wasm-pack for WebAssembly compilation

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Build Commands

```bash
# Navigate to module directory
cd "Module 17"

# Build the WASM module for web
wasm-pack build --target web

# Or use the build script
chmod +x build.sh
./build.sh

# The compiled WASM will be in pkg/ directory
```

## 🧪 Testing the Bug

### Method 1: Run the Interactive Demo

1. Build the WASM module (see above)
2. Serve the demo.html file with a local web server:
   ```bash
   # Using Python
   python3 -m http.server 8000
   
   # Or using Node.js
   npx http-server
   ```
3. Open `http://localhost:8000/demo.html` in a browser
4. Click the "Apply Filter" button multiple times
5. Click "Render UI Overlay" between filter applications
6. **Observe**: After UI overlay rendering, the next filter will show glitches!

### Method 2: Console Logs

Open browser DevTools console to see cache behavior:
```
Cache miss: binding grayscale program
Cache hit: skipping program bind (POTENTIAL BUG!)  ← Bug indicator
```

### Expected Behavior (Bug)

- Initial filter application: Works correctly
- After JS modifies GL state: Glitched/inverted output
- Console shows "Cache hit" when it should rebind
- Output is non-deterministic based on interaction sequence

## 🔍 Code Analysis

### Buggy Functions

1. **`apply_grayscale()`**: Checks cache before binding shader
2. **`apply_blur()`**: Same caching bug with blur shader
3. **`apply_invert()`**: Most susceptible - often called after UI updates
4. **`GLStateCache`**: The root cause - assumes external code won't modify GL state

### Key Bug Locations

**lib.rs:223-232** - Cache check that assumes state is valid:
```rust
if self.state_cache.borrow().current_program != Some(program_id) {
    self.context.use_program(Some(program));
    self.state_cache.borrow_mut().current_program = Some(program_id);
} else {
    // BUG: Skips rebinding, assumes cache is accurate
    console::log_1(&JsValue::from_str("Cache hit: skipping bind (POTENTIAL BUG!)"));
}
```

**lib.rs:208** - Buffer cache assumption:
```rust
// BUG: We assume the vertex buffer is still bound from initialization
self.context.enable_vertex_attrib_array(position_location);
```

## ✅ How to Fix This Bug

### Solution 1: Invalidate Cache on Each Call

```rust
// At the start of each public method
pub fn apply_grayscale(&mut self) -> Result<(), JsValue> {
    self.invalidate_cache(); // Force rebind every time
    // ... rest of the function
}
```

**Pros**: Simple, guaranteed to work  
**Cons**: Loses all performance benefits of caching

### Solution 2: Use GL State Queries

```rust
// Check actual GL state instead of trusting cache
let actual_program = self.context.get_parameter(
    WebGlRenderingContext::CURRENT_PROGRAM
)?;

if actual_program != expected_program {
    self.context.use_program(Some(program));
}
```

**Pros**: Accurate, preserves some caching benefits  
**Cons**: State queries can be slow on some GPUs

### Solution 3: Version-Based Cache Invalidation

```rust
// JS calls version bump when it modifies GL state
#[wasm_bindgen]
pub fn notify_gl_state_changed(&mut self) {
    self.state_cache_version += 1;
}

// Check version before trusting cache
if self.last_validated_version != self.state_cache_version {
    self.invalidate_cache();
    self.last_validated_version = self.state_cache_version;
}
```

**Pros**: Efficient, preserves caching when no external changes  
**Cons**: Requires coordination with JS code

### Solution 4: Ownership Pattern (Recommended)

```rust
// Take exclusive ownership of GL context
// Prevent JS from modifying it while WASM holds reference
pub struct ExclusiveGLContext {
    context: WebGlRenderingContext,
    // ... cache fields
}

// Only way to get context is through WASM API
// JS cannot access context directly
```

**Pros**: Architecturally sound, prevents the issue entirely  
**Cons**: Requires restructuring application

## 🎯 Learning Objectives

By studying this buggy module, developers will learn:

1. **Shared state management**: Dangers of caching shared mutable state
2. **WebGL context**: How GL state works and why binding operations matter
3. **WASM/JS interop**: Challenges of shared resource management
4. **Non-deterministic bugs**: How to debug timing-dependent issues
5. **Performance tradeoffs**: When caching optimization causes correctness issues
6. **State invalidation**: Importance of cache invalidation strategies

## 📚 Related Concepts

- **WebGL State Management**
- **Cache Invalidation**
- **WASM Performance Optimization**
- **Shared Resource Synchronization**
- **Graphics Pipeline State**

## 🎨 Visual Symptoms

When the bug occurs, you'll see:

- **Color inversion**: Wrong shader applied
- **Garbled output**: Wrong texture sampled
- **Black screen**: Wrong buffer data used
- **Flickering**: Bug appears/disappears based on timing
- **Partial corruption**: Some regions correct, others wrong

## ⚠️ Security Implications

While primarily a rendering bug, it could potentially:

- Leak image data from one user to another in multi-user apps
- Cause UI elements to render incorrectly, masking security indicators
- Create confusion that could be exploited in phishing attacks

## 🏷️ Tags

`#bug` `#webgl` `#wasm` `#rust` `#state-caching` `#non-deterministic` `#graphics` `#performance` `#cache-invalidation` `#image-processing`

## 📝 License

This module is for educational purposes to demonstrate WebGL state management bugs.

---

**Note**: This module is **intentionally buggy** for educational purposes. Do **NOT** use in production applications!

## 🤔 Challenge

Can you fix this module without removing the cache entirely? The goal is to maintain performance benefits while ensuring correctness even when JS modifies the GL context.

### Hints

1. Consider when the cache might be invalidated
2. Think about communication between JS and WASM
3. Look at WebGL state query APIs
4. Consider architectural patterns that prevent the issue
