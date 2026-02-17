# Quick Start Guide - Module 12

## Overview
This module demonstrates a **hydration mismatch bug** that occurs in Server-Side Rendered applications.

## The Bug in 30 Seconds
```rust
// This code runs on both server and client
let random = Math::random();  // Gets 0.7234 on server
                              // Gets 0.9876 on client
                              // ❌ MISMATCH! → Broken UI
```

## Files in This Module

| File | Purpose |
|------|---------|
| `src/lib.rs` | Main Rust code with the intentional bug |
| `Cargo.toml` | Rust package configuration |
| `demo.html` | Interactive web demo (open in browser) |
| `integration-example.jsx` | React/Next.js examples (buggy & fixed) |
| `README.md` | Comprehensive documentation |
| `build.sh` | Script to compile Rust → WebAssembly |
| `package.json` | NPM configuration |

## How to Build

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Build the Module
```bash
cd "Module 12"

# Option 1: Use the build script
./build.sh

# Option 2: Manual build
wasm-pack build --target web
```

### Output
The compiled WebAssembly will be in the `pkg/` directory:
- `pkg/hydration_mismatch_module.js` - JavaScript bindings
- `pkg/hydration_mismatch_module_bg.wasm` - WebAssembly binary
- `pkg/hydration_mismatch_module.d.ts` - TypeScript definitions

## How to Use

### In a Web Page
```html
<script type="module">
  import init, { HydrationData } from './pkg/hydration_mismatch_module.js';
  
  await init();
  const data = HydrationData.new();
  console.log(data.session_id);  // Different each time!
</script>
```

### In Next.js
```javascript
import { HydrationData } from '../pkg/hydration_mismatch_module';

// ❌ BUGGY - Don't do this
export default function Page() {
  const data = HydrationData.new();  // Called on server AND client!
  return <div>{data.session_id}</div>;
}

// ✅ CORRECT - Use useEffect
export default function Page() {
  const [data, setData] = useState(null);
  
  useEffect(() => {
    const hydrationData = HydrationData.new();
    setData(hydrationData);
  }, []);
  
  return <div>{data?.session_id ?? 'Loading...'}</div>;
}
```

## Quick Demo

### View the Interactive Demo
Simply open `demo.html` in your web browser:
```bash
# On macOS
open demo.html

# On Linux
xdg-open demo.html

# On Windows
start demo.html
```

Click the "Simulate Hydration Mismatch" button to see the bug in action!

## Testing

### Run Unit Tests
```bash
cargo test
```

### Run WASM Tests
```bash
wasm-pack test --headless --firefox
```

## The Bug Explained Visually

```
┌─────────────────────────────────────────────────────┐
│ Server-Side Render                                  │
├─────────────────────────────────────────────────────┤
│ Math.random() → 0.7234                              │
│ UUID.new() → "550e8400-e29b..."                     │
│ Date.now() → 1708167440000                          │
│                                                     │
│ Generates HTML:                                     │
│ <div>                                               │
│   <span>0.7234</span>                               │
│   <span>550e8400-e29b...</span>                     │
│   <button>Click</button>                            │
│ </div>                                              │
└─────────────────────────────────────────────────────┘
                    ⬇️ HTML sent to browser
┌─────────────────────────────────────────────────────┐
│ Client-Side Hydration                               │
├─────────────────────────────────────────────────────┤
│ Math.random() → 0.9876  ❌ DIFFERENT!               │
│ UUID.new() → "8f7d6c5b-4a3e..."  ❌ DIFFERENT!      │
│ Date.now() → 1708167440157  ❌ DIFFERENT!           │
│                                                     │
│ React expects:                                      │
│ <div>                                               │
│   <span>0.9876</span>        ⚠️ Mismatch!          │
│   <span>8f7d6c5b-4a3e...</span>  ⚠️ Mismatch!      │
│   <button>Click</button>     ⚠️ Events broken!     │
│ </div>                                              │
└─────────────────────────────────────────────────────┘
                    ⬇️
┌─────────────────────────────────────────────────────┐
│ Result: Hydration Error                             │
├─────────────────────────────────────────────────────┤
│ ❌ Event listeners don't attach                     │
│ ❌ Buttons don't work                               │
│ ❌ Forms don't submit                               │
│ ❌ Interactive elements fail silently               │
│ ✅ Visual appearance looks correct (misleading!)    │
└─────────────────────────────────────────────────────┘
```

## Common Symptoms

If you're experiencing this bug, you might see:
- 🔴 Console warnings: "Text content did not match"
- 🔴 Console errors: "Hydration failed"
- 🔴 Buttons that look clickable but do nothing
- 🔴 Forms that won't submit
- 🔴 Event handlers that don't fire
- ✅ Page that looks visually correct (confusing!)

## How to Fix

See `integration-example.jsx` for multiple fix strategies:
1. Use `useEffect` for client-only values
2. Pass stable props from server
3. Conditional rendering with mount check
4. Suppress hydration warnings (not recommended)

## Learn More

- Read the full `README.md` for detailed explanation
- Open `demo.html` for interactive demonstration
- Study `integration-example.jsx` for code examples
- Check `src/lib.rs` for the Rust implementation

## Support

This module is for educational purposes. It intentionally contains bugs to demonstrate hydration mismatch issues.

**Do NOT use in production applications!**

---

Happy bug hunting! 🐛🔍
