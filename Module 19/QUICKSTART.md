# Quick Start Guide - Module 19

## 🚀 Getting Started with the Waker Bug Module

This guide will help you quickly build and test the buggy custom Future implementation.

## Prerequisites

1. **Rust** (version 1.70 or higher)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **wasm-pack** (for building WebAssembly)
   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

3. **A modern web browser** (Chrome, Firefox, Safari, or Edge)

4. **A local web server** (optional but recommended)
   ```bash
   # Option 1: Python
   python3 -m http.server 8000
   
   # Option 2: Node.js
   npx http-server -p 8000
   
   # Option 3: Rust
   cargo install miniserve
   miniserve . -p 8000
   ```

## Build Steps

### 1. Navigate to Module 19

```bash
cd "Module 19"
```

### 2. Build the WebAssembly Module

```bash
wasm-pack build --target web
```

This will:
- Compile the Rust code to WebAssembly
- Generate JavaScript bindings
- Create a `pkg/` directory with the compiled output

**Expected output:**
```
[INFO]: Checking for the Wasm target...
[INFO]: Compiling to Wasm...
   Compiling waker-bug-module v0.1.0
    Finished release [optimized] target(s) in X.XXs
[INFO]: Installing wasm-bindgen...
[INFO]: Done in X.XXs
[INFO]: Your wasm pkg is ready to publish at ./pkg.
```

### 3. Verify the Build

Check that the `pkg/` directory was created:

```bash
ls -la pkg/
```

You should see:
- `waker_bug_module.js` - JavaScript bindings
- `waker_bug_module_bg.wasm` - Compiled WebAssembly
- `waker_bug_module.d.ts` - TypeScript definitions
- `package.json` - NPM package metadata

## Testing the Bug

### Option 1: Use the Demo HTML File

1. Start a local web server in the Module 19 directory:
   ```bash
   python3 -m http.server 8000
   ```

2. Open your browser to `http://localhost:8000/demo.html`

3. Open the browser's Developer Console (F12 or Cmd+Option+I)

4. Click the **"Fetch Data (Buggy)"** button

5. Watch the console output:
   - You'll see "BuggyFuture::poll() called"
   - You'll see "JS Promise resolved!" after 2 seconds
   - But the button stays disabled forever! The await never completes.

6. Refresh the page and click **"Fetch Data (Correct)"** for comparison
   - This button will re-enable after 2 seconds

### Option 2: Test in Browser Console

1. Open `demo.html` in your browser
2. Open Developer Console
3. Try these commands:

```javascript
// This hangs forever
console.log('Starting buggy call...');
fetch_data_buggy(2000).then(result => {
    console.log('Completed:', result);  // NEVER EXECUTED
});
console.log('After fetch_data_buggy (but before it completes)');

// Wait a few seconds and notice nothing happens
// The Promise never resolves on the JS side!

// Compare with correct implementation:
console.log('Starting correct call...');
fetch_data_correct(2000).then(result => {
    console.log('Completed:', result);  // ✓ Executes after 2 seconds
});
```

### Option 3: Integration Testing

Create a simple HTML file to test:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Waker Bug Test</title>
</head>
<body>
    <h1>Testing Waker Bug</h1>
    <div id="status">Loading...</div>
    
    <script type="module">
        import init, { 
            fetch_data_buggy, 
            fetch_data_correct,
            get_user_data,
            countdown_buggy
        } from './pkg/waker_bug_module.js';

        async function runTests() {
            await init();
            
            const status = document.getElementById('status');
            
            // Test 1: Buggy version (will hang)
            status.textContent = 'Testing buggy version...';
            console.log('Test 1: This will hang forever...');
            
            // This await never completes!
            await fetch_data_buggy(2000);
            
            // Everything below is never executed
            status.textContent = 'Bug test completed (NEVER REACHED)';
            console.log('After buggy call (NEVER REACHED)');
        }

        runTests().catch(err => {
            console.error('Error:', err);
            document.getElementById('status').textContent = 'Error: ' + err;
        });
    </script>
</body>
</html>
```

## Understanding the Console Output

### Buggy Version

```
fetch_data_buggy() called
⚠️  This will hang forever!
Created Promise that will resolve in 2000ms
BuggyFuture created
BuggyFuture::poll() called
BuggyFuture: Returning Poll::Pending WITHOUT storing waker! 🐛
[After 2 seconds]
JS Promise resolved! ✓
[Eternal silence - the Future never wakes up]
```

Notice:
- The JS Promise DOES resolve (you see "JS Promise resolved!")
- But the Rust Future never completes
- No errors, no exceptions
- Just waiting forever

### Correct Version

```
fetch_data_correct() called
✓ This will work correctly
Created Promise that will resolve in 2000ms
[After 2 seconds]
JS Promise resolved! ✓
[Function returns successfully]
```

## Running Tests

The module includes basic compilation tests:

```bash
cd "Module 19"
cargo test
```

**Note**: Full async tests require a browser environment and would use:

```bash
wasm-pack test --headless --chrome
```

## Troubleshooting

### Issue: `wasm-pack: command not found`

**Solution**: Install wasm-pack:
```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Issue: Build fails with "wasm32-unknown-unknown target not found"

**Solution**: Add the WebAssembly target:
```bash
rustup target add wasm32-unknown-unknown
```

### Issue: Demo HTML doesn't load

**Solution**: You need to serve it over HTTP, not file://
```bash
python3 -m http.server 8000
# Then visit http://localhost:8000/demo.html
```

### Issue: CORS errors in browser

**Solution**: Make sure you're using a local web server, not opening the file directly.

### Issue: "Memory access out of bounds" error

**Solution**: This shouldn't happen with this module, but if it does:
1. Clear browser cache
2. Rebuild with `wasm-pack build --target web`
3. Restart web server

## Next Steps

1. **Read the README.md** for detailed bug explanation
2. **Read TECHNICAL.md** for deep dive into Future/Waker mechanism
3. **Try to fix the bug** - modify `src/lib.rs` to make it work
4. **Compare with correct implementations** in the codebase

## Quick Reference

### Build Commands
```bash
# Clean build
rm -rf pkg/ target/
wasm-pack build --target web

# Development build (faster, larger)
wasm-pack build --target web --dev

# Release build (slower, optimized)
wasm-pack build --target web --release
```

### File Structure
```
Module 19/
├── pkg/                      # Generated (don't commit)
│   ├── waker_bug_module.js
│   └── waker_bug_module_bg.wasm
├── src/
│   └── lib.rs               # The buggy code
├── Cargo.toml
├── demo.html
└── README.md
```

## Tips for Testing

1. **Use console.time/timeEnd** to see how long things take:
   ```javascript
   console.time('buggy');
   await fetch_data_buggy(2000);
   console.timeEnd('buggy');  // Never prints
   ```

2. **Set up a timeout** to detect the hang:
   ```javascript
   const timeout = setTimeout(() => {
       console.error('Still waiting after 5 seconds - probably hung!');
   }, 5000);
   
   await fetch_data_buggy(2000);
   clearTimeout(timeout);  // Never reached
   ```

3. **Compare side-by-side**:
   ```javascript
   Promise.race([
       fetch_data_buggy(2000),
       new Promise(resolve => setTimeout(() => resolve('timeout'), 3000))
   ]).then(result => {
       console.log('Result:', result);  // Will be 'timeout'
   });
   ```

## Resources

- [Rust async book](https://rust-lang.github.io/async-book/)
- [wasm-bindgen guide](https://rustwasm.github.io/wasm-bindgen/)
- [The Waker API](https://doc.rust-lang.org/std/task/struct.Waker.html)

---

**Happy bug hunting! 🐛🔍**
