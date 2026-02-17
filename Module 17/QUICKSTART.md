# Quick Start Guide: Module 17

## What is This?

This module demonstrates a **WebGL state caching bug** in a WASM-based image processing library. The bug causes non-deterministic rendering glitches when JavaScript modifies the WebGL context between WASM calls.

## Prerequisites

- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **wasm-pack**: `curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`
- A web browser (Chrome, Firefox, Safari, or Edge)
- A local web server (Python, Node.js, or any HTTP server)

## 5-Minute Setup

### 1. Build the Module

```bash
cd "Module 17"
chmod +x build.sh
./build.sh
```

This compiles the Rust code to WebAssembly and outputs to the `pkg/` directory.

### 2. Start a Web Server

**Option A - Python:**
```bash
python3 -m http.server 8000
```

**Option B - Node.js:**
```bash
npx http-server -p 8000
```

**Option C - PHP:**
```bash
php -S localhost:8000
```

### 3. Open the Demo

Navigate to: `http://localhost:8000/demo.html`

## Triggering the Bug

Follow these steps in the demo:

1. ✅ Click **"Apply Grayscale"** → Works correctly
2. ⚠️ Click **"Render UI Overlay (JS)"** → Modifies GL state
3. 🐛 Click **"Apply Grayscale"** again → **GLITCH!**

**What happened?**
- Step 1: WASM cached the grayscale shader as "current"
- Step 2: JavaScript changed the GL state (but WASM doesn't know)
- Step 3: WASM trusts its cache and skips rebinding → uses wrong shader!

## Observing the Bug

### In Browser Console

Open DevTools (F12) and watch for these messages:

```
✅ Cache miss: binding grayscale program  ← First call
⚠️ Cache hit: skipping bind              ← After JS modification (BUG!)
```

### Visual Symptoms

Look for:
- Inverted colors
- Garbled output
- Partial rendering
- Flickering
- Inconsistent results

## Understanding the Files

```
Module 17/
├── src/lib.rs              ← Main Rust code (contains the bug)
├── demo.html               ← Interactive demo
├── integration-example.js  ← React/webapp examples
├── build.sh                ← Build script
├── README.md               ← Overview and documentation
├── TECHNICAL.md            ← Deep technical analysis
├── SOLUTION.md             ← Solution strategies
└── QUICKSTART.md           ← This file
```

## Common Questions

### Q: Why doesn't the bug always appear?

**A**: The bug is non-deterministic. It only appears when:
1. JavaScript modifies the GL context
2. WASM makes another call immediately after
3. The cached state differs from actual state

### Q: Can I fix the bug?

**A**: Yes! See `SOLUTION.md` for detailed fix strategies. Quick fix:

```rust
// Before each WASM operation
processor.invalidate_cache();
processor.apply_grayscale();
```

### Q: Is this a real bug pattern?

**A**: Absolutely! This pattern has appeared in:
- Google Maps WebGL
- Unity WebGL exports
- Custom WebGL frameworks
- Mobile browser rendering engines

### Q: How do I verify my fix works?

**A**: Test with this sequence:

```javascript
processor.apply_grayscale();
// Should work ✓

// JS modifies GL state
const gl = canvas.getContext('webgl');
const tempProgram = gl.createProgram();
gl.useProgram(tempProgram);
// Your fix here (e.g., processor.notify_external_gl_modification())

processor.apply_grayscale();
// Should still work ✓
```

## Integration into Your Project

### Step 1: Build the Module

```bash
wasm-pack build --target web
```

### Step 2: Import in JavaScript

```javascript
import init, { ImageProcessor, get_webgl_context } from './pkg/shadowed_canvas_context.js';

async function main() {
    await init();
    
    const gl = get_webgl_context('myCanvas');
    const processor = new ImageProcessor(gl);
    
    // Use the processor
    processor.apply_grayscale();
}

main();
```

### Step 3: Handle JS/WASM Coordination

```javascript
// BUGGY CODE (don't do this)
processor.apply_grayscale();
const gl = canvas.getContext('webgl');
gl.useProgram(someOtherProgram);  // JS modifies GL
processor.apply_grayscale();      // BUG!

// FIXED CODE (do this)
processor.apply_grayscale();
gl.useProgram(someOtherProgram);
processor.invalidate_cache();  // Notify WASM
processor.apply_grayscale();   // Works!
```

## Testing Your Changes

### Manual Testing

1. Apply a filter → Verify it works
2. Perform JS canvas operation
3. Apply another filter → Verify no glitch

### Automated Testing

```javascript
describe('ImageProcessor', () => {
    it('should handle JS GL modifications', async () => {
        const processor = new ImageProcessor(gl);
        
        await processor.apply_grayscale();
        
        // JS modifies state
        canvas.getContext('2d').fillRect(0, 0, 100, 100);
        
        // Should still work after fix
        await processor.apply_grayscale();
        
        // Verify output
        const pixels = getCanvasPixels(canvas);
        expect(pixels).toMatchGrayscale();
    });
});
```

## Performance Notes

| Solution | Overhead | Correctness |
|----------|----------|-------------|
| No cache | +10% | ✅ Always correct |
| Query GL state | +25% | ✅ Always correct |
| Invalidation API | +1% | ✅ With discipline |
| Separate contexts | +0% | ✅ Always correct |

## Next Steps

1. **Read `README.md`** for bug overview
2. **Read `TECHNICAL.md`** for deep dive
3. **Read `SOLUTION.md`** for fix strategies
4. **Try fixing the bug** yourself
5. **Test your solution** thoroughly

## Troubleshooting

### Build Fails

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
wasm-pack build --target web
```

### Demo Doesn't Load

- Check browser console for errors
- Ensure web server is running
- Try a different browser
- Clear browser cache

### Bug Doesn't Appear

- Make sure to click "Render UI Overlay" between filter applications
- Check console logs for cache behavior
- Try multiple rapid clicks
- Refresh and try again

## Learning Resources

- [WebGL Fundamentals](https://webglfundamentals.org/)
- [Rust and WebAssembly Book](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [WebGL State Management](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices)

## Getting Help

If you're stuck:

1. Check the `SOLUTION.md` file for detailed strategies
2. Review the `TECHNICAL.md` file for understanding
3. Look at the console logs for cache behavior
4. Compare with working examples in other modules

## Success Checklist

- [ ] Module builds successfully
- [ ] Demo loads in browser
- [ ] Can trigger the bug reliably
- [ ] Understand why the bug occurs
- [ ] Can explain the cache invalidation problem
- [ ] Know at least 2 ways to fix it

---

**Remember**: This module is intentionally buggy for educational purposes. The goal is to understand the bug pattern and learn how to fix it!

Happy debugging! 🐛🔍
