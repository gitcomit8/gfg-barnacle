# Solution Guide: Fixing the Shadowed Canvas Context Bug

## Problem Summary

The WASM module caches WebGL context state (shader programs, textures, buffers) for performance optimization. However, JavaScript code can modify this state between WASM calls, causing the cache to become stale and resulting in rendering glitches.

## Understanding the Root Cause

### The Flawed Assumption

The cache assumes:
```rust
// WRONG ASSUMPTION: "I am the only code modifying this GL context"
if cache.current_program == Some(ProgramType::Grayscale) {
    // Skip binding - grayscale shader is already active
}
```

**Reality**: JavaScript, other WASM modules, or browser internals may also modify the GL context.

## Solution Approaches

### Level 1: Beginner Solution (Remove Cache)

**Difficulty**: ⭐☆☆☆☆

Simply remove the cache and always bind state:

```rust
pub fn apply_grayscale(&mut self) -> Result<(), JsValue> {
    let program = self.grayscale_program.as_ref().unwrap();
    
    // Always bind, no cache check
    self.context.use_program(Some(program));
    
    self.render_effect(program)?;
    Ok(())
}
```

**Pros**:
- Simple and correct
- Easy to implement
- No coordination needed

**Cons**:
- Loses all performance benefits
- May be too slow for complex scenes
- Not learning the real solution

**Grade**: Works, but defeats the optimization purpose. **C+**

---

### Level 2: Intermediate Solution (State Queries)

**Difficulty**: ⭐⭐⭐☆☆

Query actual GL state before trusting cache:

```rust
pub fn apply_grayscale(&mut self) -> Result<(), JsValue> {
    let program = self.grayscale_program.as_ref().unwrap();
    
    // Query actual GL state
    let current_program = self.context
        .get_parameter(WebGlRenderingContext::CURRENT_PROGRAM)?;
    
    // Compare with expected program
    if !is_same_program(&current_program, program) {
        self.context.use_program(Some(program));
        self.state_cache.borrow_mut().current_program = Some(ProgramType::Grayscale);
    }
    
    self.render_effect(program)?;
    Ok(())
}

fn is_same_program(current: &JsValue, expected: &WebGlProgram) -> bool {
    // Implementation to compare programs
    // ...
}
```

**Pros**:
- Always correct
- Preserves some cache benefits
- No JS coordination needed

**Cons**:
- GL state queries are expensive (~50μs each)
- May be slower than no cache on some hardware
- Not using the cache efficiently

**Grade**: Correct but inefficient. **B**

---

### Level 3: Advanced Solution (Invalidation API)

**Difficulty**: ⭐⭐⭐⭐☆

Provide an API for JS to notify WASM when GL state changes:

```rust
impl ImageProcessor {
    /// Call this before any WASM GL operation after JS has modified the context
    #[wasm_bindgen]
    pub fn notify_external_gl_modification(&mut self) {
        self.invalidate_cache();
    }
    
    pub fn apply_grayscale(&mut self) -> Result<(), JsValue> {
        // Cache is trustworthy because JS notifies us of changes
        let program = self.grayscale_program.as_ref().unwrap();
        let program_type = ProgramType::Grayscale;
        
        if self.state_cache.borrow().current_program != Some(program_type) {
            self.context.use_program(Some(program));
            self.state_cache.borrow_mut().current_program = Some(program_type);
        }
        
        self.render_effect(program)?;
        Ok(())
    }
}
```

**JavaScript usage**:
```javascript
// After JS modifies GL state
const ctx = canvas.getContext('2d');
ctx.fillRect(0, 0, 100, 100);

// Notify WASM
processor.notify_external_gl_modification();

// Now safe to use WASM
processor.apply_grayscale();
```

**Pros**:
- Efficient - no performance overhead
- Preserves cache benefits
- Correct when used properly

**Cons**:
- Requires JS cooperation
- Easy to forget notifications
- Manual coordination needed

**Grade**: Efficient and correct with discipline. **A-**

---

### Level 4: Expert Solution (Automatic Invalidation)

**Difficulty**: ⭐⭐⭐⭐⭐

Wrap the canvas context to intercept modifications:

```javascript
class ManagedWebGLContext {
    constructor(canvas, wasmProcessor) {
        this.gl = canvas.getContext('webgl');
        this.wasm = wasmProcessor;
        this.intercepted = false;
    }
    
    // Wrap all GL state-modifying methods
    useProgram(program) {
        this.gl.useProgram(program);
        if (!this.intercepted) {
            this.wasm.notify_external_gl_modification();
        }
    }
    
    bindBuffer(target, buffer) {
        this.gl.bindBuffer(target, buffer);
        if (!this.intercepted) {
            this.wasm.notify_external_gl_modification();
        }
    }
    
    // Mark when WASM is using the context
    beginWASMOperation() {
        this.intercepted = true;
    }
    
    endWASMOperation() {
        this.intercepted = false;
    }
    
    // ... wrap all state-modifying methods
}

// Usage
const managedCtx = new ManagedWebGLContext(canvas, processor);
managedCtx.useProgram(someProgram);  // Auto-notifies WASM
```

**Pros**:
- Automatic invalidation
- No manual coordination
- Efficient
- Developer-friendly

**Cons**:
- Complex implementation
- Wrapping overhead
- Must wrap all GL methods
- Fragile to API changes

**Grade**: Professional solution. **A+**

---

### Level 5: Architectural Solution (Separate Contexts)

**Difficulty**: ⭐⭐⭐☆☆

Use separate canvases for WASM and JS:

```html
<div style="position: relative;">
    <canvas id="wasm-canvas" style="position: absolute; z-index: 1;"></canvas>
    <canvas id="ui-canvas" style="position: absolute; z-index: 2;"></canvas>
</div>
```

```javascript
// WASM uses wasm-canvas exclusively
const wasmCanvas = document.getElementById('wasm-canvas');
const wasmGl = wasmCanvas.getContext('webgl');
const processor = new ImageProcessor(wasmGl);

// JS uses ui-canvas exclusively
const uiCanvas = document.getElementById('ui-canvas');
const uiCtx = uiCanvas.getContext('2d');
uiCtx.fillText("UI Overlay", 10, 10);
```

**Pros**:
- Complete isolation
- No coordination needed
- Simple and robust
- Performance excellent

**Cons**:
- More memory usage (two canvases)
- Compositing complexity
- May not work for all use cases
- Requires architectural changes

**Grade**: Best when applicable. **A**

---

## Implementation Guide

### Step 1: Identify All Cache Access Points

Find every location where `state_cache` is checked:

```bash
grep -n "state_cache.borrow()" src/lib.rs
```

### Step 2: Add Invalidation Method

```rust
#[wasm_bindgen]
pub fn invalidate_cache(&mut self) {
    console::log_1(&JsValue::from_str("Invalidating GL state cache"));
    let mut cache = self.state_cache.borrow_mut();
    cache.current_program = None;
    cache.current_texture = None;
    cache.current_buffer = None;
}
```

### Step 3: Choose Your Solution

Pick one of the solutions above based on:
- Performance requirements
- Code complexity tolerance
- JS/WASM coordination feasibility
- Architectural constraints

### Step 4: Test Thoroughly

Create integration tests that interleave JS and WASM:

```javascript
test('handles interleaved JS/WASM operations', async () => {
    await processor.apply_grayscale();
    
    // JS modifies GL
    const ctx = canvas.getContext('2d');
    ctx.fillRect(0, 0, 100, 100);
    
    // Apply solution (e.g., call notify)
    processor.notify_external_gl_modification();
    
    // Verify WASM works correctly
    await processor.apply_grayscale();
    expect(output).toMatchSnapshot();
});
```

## Verification Checklist

- [ ] Module compiles without errors
- [ ] Basic WASM operations work
- [ ] JS operations don't break WASM
- [ ] Interleaved operations work correctly
- [ ] Performance is acceptable
- [ ] No visual glitches
- [ ] Console logs show correct cache behavior
- [ ] Integration tests pass

## Common Pitfalls

### Pitfall 1: Forgetting to Call Invalidation

```javascript
// BAD
ctx.fillRect(0, 0, 100, 100);
processor.apply_grayscale();  // Forgot to notify!

// GOOD
ctx.fillRect(0, 0, 100, 100);
processor.notify_external_gl_modification();
processor.apply_grayscale();
```

### Pitfall 2: Over-Invalidating

```javascript
// BAD - invalidating too often loses cache benefits
processor.notify_external_gl_modification();
processor.apply_grayscale();
processor.notify_external_gl_modification();
processor.apply_blur();

// GOOD - only invalidate when JS actually modified state
ctx.fillRect(0, 0, 100, 100);  // JS modified state
processor.notify_external_gl_modification();
processor.apply_grayscale();
processor.apply_blur();  // No JS between, no notification needed
```

### Pitfall 3: Partial Implementation

```rust
// BAD - only fixed one method
pub fn apply_grayscale(&mut self) {
    self.invalidate_cache();  // Fixed
    // ...
}

pub fn apply_blur(&mut self) {
    // BUG STILL HERE!
}

// GOOD - fix all methods
```

## Bonus: Advanced Optimization

Track invalidation with a version number:

```rust
struct GLStateCache {
    current_program: Option<ProgramType>,
    cache_version: u64,
}

impl ImageProcessor {
    fn is_cache_valid(&self) -> bool {
        self.last_validated_version == self.state_cache.borrow().cache_version
    }
    
    #[wasm_bindgen]
    pub fn notify_external_gl_modification(&mut self) {
        self.state_cache.borrow_mut().cache_version += 1;
    }
    
    pub fn apply_grayscale(&mut self) -> Result<(), JsValue> {
        if !self.is_cache_valid() {
            self.invalidate_cache();
            self.last_validated_version = self.state_cache.borrow().cache_version;
        }
        
        // Now cache is trustworthy
        // ... rest of implementation
    }
}
```

This allows WASM to make multiple calls without revalidating if JS hasn't modified state.

## Success Criteria

Your solution should:

1. ✅ Produce correct visual output consistently
2. ✅ Work after JS modifies GL state
3. ✅ Maintain reasonable performance
4. ✅ Be maintainable and understandable
5. ✅ Pass all integration tests

## Reflection Questions

After implementing your solution, consider:

1. What are the trade-offs of your approach?
2. How would this scale to a real application?
3. What happens if a third library also uses the context?
4. How would you test this thoroughly?
5. What patterns could prevent this class of bug?

---

**Remember**: The "best" solution depends on your specific constraints. A simple solution that works is better than a complex solution that's hard to maintain!
