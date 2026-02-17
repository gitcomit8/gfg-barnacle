# Technical Deep Dive: The Shadowed Canvas Context Bug

## Overview

This document provides a detailed technical analysis of the WebGL state caching bug present in Module 17.

## Architecture

### Component Structure

```
┌─────────────────────────────────────────────────────────┐
│                    JavaScript Layer                      │
│  - UI Rendering                                          │
│  - Event Handlers                                        │
│  - Canvas Manipulation                                   │
└────────────────┬────────────────────────────────────────┘
                 │ Shares WebGL Context
                 │
┌────────────────▼────────────────────────────────────────┐
│                   WebGL Context                          │
│  - Current Program (Shader)                              │
│  - Texture Bindings                                      │
│  - Buffer Bindings                                       │
│  - Render State                                          │
└────────────────┬────────────────────────────────────────┘
                 │ Used by
                 │
┌────────────────▼────────────────────────────────────────┐
│                WASM Module (Rust)                        │
│  - ImageProcessor                                        │
│  - GLStateCache (BUGGY!)                                 │
│  - Image Processing Shaders                              │
└─────────────────────────────────────────────────────────┘
```

## The Bug Mechanism

### State Cache Structure

```rust
struct GLStateCache {
    current_program: Option<ProgramType>,  // Cached shader program
    current_texture: Option<u32>,          // Cached texture ID
    current_buffer: Option<u32>,           // Cached buffer ID
    invert_mode: bool,
}
```

### Normal Flow (No Bug)

```
1. WASM: apply_grayscale()
   └─> Cache empty → bind grayscale shader
   └─> Update cache: current_program = Grayscale
   └─> Render with grayscale shader ✓

2. WASM: apply_grayscale() again
   └─> Check cache: current_program == Grayscale
   └─> Cache hit! Skip binding (optimization)
   └─> Render with grayscale shader ✓
```

### Bug Flow (Cache Invalidation Failure)

```
1. WASM: apply_grayscale()
   └─> Cache empty → bind grayscale shader
   └─> Update cache: current_program = Grayscale
   └─> Render ✓

2. JS: renderUIOverlay()
   └─> Get 2D context (internally uses WebGL)
   └─> Draw overlay with different shader
   └─> GL state changes: current_program = OverlayShader
   └─> WASM cache now STALE! ✗

3. WASM: apply_grayscale() again
   └─> Check cache: current_program == Grayscale
   └─> Cache hit! Skip binding (WRONG!)
   └─> GL is actually using OverlayShader
   └─> Render with WRONG shader → GLITCH ✗
```

## Code Analysis

### Buggy Cache Check

**Location**: `src/lib.rs:234-241`

```rust
let program_type = ProgramType::Grayscale;
if self.state_cache.borrow().current_program != Some(program_type) {
    console::log_1(&JsValue::from_str("Cache miss: binding grayscale program"));
    self.context.use_program(Some(program));
    self.state_cache.borrow_mut().current_program = Some(program_type);
} else {
    // BUG: Assumes program is still active - it might not be!
    console::log_1(&JsValue::from_str("Cache hit: skipping program bind (POTENTIAL BUG!)"));
}
```

**Problem**: The cache assumes no external code modifies the WebGL context. In a webapp, JavaScript frequently modifies the context for UI rendering, making this assumption invalid.

### Why Traditional Debugging Fails

1. **Timing Dependent**: Bug only appears when JS modifies GL state between WASM calls
2. **Non-Deterministic**: Depends on user interaction patterns
3. **Silent Failure**: No exceptions thrown, just wrong visual output
4. **Cache Appears Correct**: From WASM's perspective, the cache is consistent
5. **Works in Isolation**: WASM-only tests pass; bug needs JS/WASM interleaving

## WebGL State Machine

### GL Context State (Simplified)

```javascript
{
  CURRENT_PROGRAM: WebGLProgram | null,
  ACTIVE_TEXTURE: GL_TEXTURE0,
  TEXTURE_BINDING_2D: WebGLTexture | null,
  ARRAY_BUFFER_BINDING: WebGLBuffer | null,
  // ... many more state variables
}
```

### State Modification Sources

1. **WASM Module**: Image processing operations
2. **JavaScript**: UI rendering, overlays, charts, etc.
3. **Browser**: Internal operations (rare)
4. **Other Libraries**: Three.js, Babylon.js, etc.

**Key Issue**: Multiple sources modify shared state, but WASM cache only tracks WASM modifications.

## Performance vs Correctness Trade-off

### Why Caching Was Added

WebGL state binding operations have overhead:

```
glUseProgram()     ~0.01ms  (10 microseconds)
glBindTexture()    ~0.005ms (5 microseconds)
glBindBuffer()     ~0.005ms (5 microseconds)
```

For 60 FPS rendering (16.67ms per frame), these seem negligible. However:

- Mobile GPUs have higher overhead
- Many objects require many bind calls
- Desktop apps target 144+ FPS
- VR requires 90+ FPS per eye

**Optimization Goal**: Skip redundant bind operations when state hasn't changed.

**The Flaw**: Only tracking WASM-side changes, not JS-side changes.

## Manifestation Scenarios

### Scenario 1: UI Overlay Rendering

```javascript
// User clicks button → render tooltip
function showTooltip() {
    const ctx = canvas.getContext('2d');  // Gets/uses GL context
    ctx.fillText("Tooltip", x, y);        // Modifies GL state
}

// Next WASM call uses stale cache → glitch
processor.apply_grayscale();  // BUG!
```

### Scenario 2: Animation Loop

```javascript
function animate() {
    // Process image
    processor.apply_blur(800, 600);
    
    // Render UI (modifies GL state)
    if (showUI) {
        drawUIElements();  // Changes GL context
    }
    
    // Next frame - cache is stale if showUI was true
    requestAnimationFrame(animate);
}
```

### Scenario 3: Multiple Libraries

```javascript
// WASM image processing
processor.apply_grayscale();

// Three.js rendering (uses same canvas)
threeRenderer.render(scene, camera);  // Modifies GL state

// WASM again - wrong shader bound!
processor.apply_invert();  // BUG!
```

## Detection Strategies

### 1. Console Logging

Enable verbose logging to see cache hits:

```
Cache miss: binding grayscale program    ← OK
Cache hit: skipping program bind         ← Potential bug location
```

If you see "Cache hit" after JS operations, the bug may manifest.

### 2. Frame Analysis

Use browser DevTools to capture GL calls:

```
Frame N:
  useProgram(grayscaleProgram)  ← WASM
  drawArrays()                  ← WASM
  
Frame N+1:
  useProgram(uiShader)          ← JS
  drawElements()                ← JS
  drawArrays()                  ← WASM (wrong shader!)
```

### 3. Visual Inspection

Look for:
- Color inversions
- Partial rendering
- Flickering
- Corrupted pixels
- Inconsistent output

## Performance Impact of Fixes

### Solution 1: Remove Cache (Simple)

```rust
// Always bind, no cache
self.context.use_program(Some(program));
```

**Impact**: +5-10% frame time on complex scenes

### Solution 2: Query GL State (Accurate)

```rust
let current = self.context.get_parameter(GL::CURRENT_PROGRAM)?;
if current != expected {
    self.context.use_program(Some(program));
}
```

**Impact**: +20-30% frame time (queries are expensive!)

### Solution 3: Invalidation API (Efficient)

```rust
// JS notifies WASM when it modifies GL state
#[wasm_bindgen]
pub fn notify_gl_modified() {
    self.state_cache.invalidate();
}
```

**Impact**: +0-2% frame time (optimal)

### Solution 4: Separate Contexts (Architectural)

```rust
// WASM uses its own canvas/context
let wasmCanvas = document.getElementById('wasm-canvas');
let uiCanvas = document.getElementById('ui-canvas');
```

**Impact**: No performance loss, but more memory

## Testing Recommendations

### Unit Tests (Won't Catch Bug)

```rust
#[test]
fn test_apply_grayscale() {
    let processor = ImageProcessor::new(context);
    processor.apply_grayscale();  // Works fine in isolation
}
```

**Problem**: No JS/WASM interleaving in unit tests.

### Integration Tests (Will Catch Bug)

```javascript
it('should handle JS GL modifications', async () => {
    await processor.apply_grayscale();
    
    // Simulate JS modifying GL state
    const ctx = canvas.getContext('2d');
    ctx.fillRect(0, 0, 100, 100);
    
    // This should still work
    await processor.apply_grayscale();
    
    // Verify output matches expected
    expect(getPixel(50, 50)).toMatchColor(expectedGrayscale);
});
```

## Debugging Tips

1. **Add verbose logging** to all GL bind operations
2. **Use WebGL inspector** browser extension
3. **Test with UI interactions** that modify the canvas
4. **Compare with reference implementation** (no cache)
5. **Use frame capture tools** to see actual GL calls
6. **Profile on mobile devices** where bug is more common

## Related Bugs in the Wild

This pattern has appeared in real applications:

1. **Google Maps WebGL**: Early versions had tile rendering glitches
2. **Unity WebGL**: Cache invalidation issues with browser UI
3. **WebGL frameworks**: Many have struggled with state management
4. **Mobile browsers**: More aggressive state changes cause issues

## References

- [WebGL Specification](https://www.khronos.org/registry/webgl/specs/latest/)
- [WebGL Best Practices](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices)
- [State Management in Graphics APIs](https://www.khronos.org/opengl/wiki/State_Management)

---

**Key Takeaway**: When caching external resource state, always have a mechanism to detect or be notified of external modifications. Assumptions about exclusive access rarely hold in shared environments.
