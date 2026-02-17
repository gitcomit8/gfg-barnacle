# Module 12: Hydration Mismatch Module (SSR/Next.js)

## 🐛 Bug Description

This module contains an intentional **hydration mismatch bug** that occurs in Server-Side Rendered (SSR) applications, particularly those using React, Next.js, or similar frameworks.

### The Problem

The module generates random values (UUIDs, random numbers, timestamps) during component rendering. When used in an SSR context:

1. **Server renders** the component with random values (e.g., `Math.random()` returns `0.7234`)
2. **HTML is sent to client** with these values embedded
3. **Client-side JavaScript hydrates** the same component
4. **New random values are generated** during hydration (e.g., `Math.random()` returns `0.9876`)
5. **React detects mismatch** between server HTML and client virtual DOM
6. **Event listeners fail to attach** properly

### The Consequence

> ⚠️ **Critical Issue**: The UI might look perfectly normal, but **interactive elements don't work**. Buttons won't respond to clicks, forms won't submit, and other event handlers fail silently because React's reconciliation process breaks.

## 🔧 Technical Details

### Root Cause

The bug stems from non-deterministic functions being called during render:

```rust
// ❌ BUGGY CODE (in this module)
pub fn new() -> HydrationData {
    let session_id = Uuid::new_v4().to_string();      // Different every time!
    let random_number = Math::random();                // Different every time!
    let timestamp = js_sys::Date::now() as i64;        // Different every time!
    // ...
}
```

### Why It's Difficult to Debug

1. **Visual appearance is correct**: The page renders and looks normal
2. **Silent failure**: No JavaScript errors in production
3. **Event handlers don't work**: Buttons appear clickable but do nothing
4. **Inconsistent behavior**: May work in development but fail in production
5. **Only warns in dev mode**: React shows warnings only in development builds

### Hydration Error Example

```
Warning: Text content did not match. 
Server: "session-550e8400-e29b-41d4-a716-446655440000" 
Client: "session-8f7d6c5b-4a3e-2b1c-0d9f-8e7a6b5c4d3e"
```

## 🏗️ Module Structure

```
Module 12/
├── Cargo.toml              # Rust/WASM package configuration
├── src/
│   └── lib.rs              # Main buggy Rust code
├── demo.html               # Interactive demo showing the bug
├── README.md               # This file
└── integration-example.jsx # React component example
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
cd "Module 12"
wasm-pack build --target web

# The compiled WASM will be in pkg/ directory
```

## 🧪 Testing the Bug

### Method 1: Run the Demo

Open `demo.html` in a browser to see an interactive demonstration of the bug.

### Method 2: Integration Test

```bash
# Run unit tests
cargo test

# Build and test in a Next.js app (example)
# Copy pkg/ to your Next.js project
# Import and use in a server component
```

## 🔍 Code Analysis

### Buggy Functions

1. **`HydrationData::new()`**: Creates random data on every instantiation
2. **`generate_random_id()`**: Uses `Math.random()` and `Uuid::new_v4()`
3. **`get_timestamp_value()`**: Returns current time, always different
4. **`get_hydration_value()`**: Core bug - `Math.random() * 10000.0`
5. **`create_component_id()`**: Generates random component IDs

### Bug Manifestation Flow

```
Server Side:
  └─> HydrationData::new() 
       ├─> session_id: "550e8400-..."
       ├─> random_number: 0.7234
       └─> timestamp: 1708167440000

Client Side (Hydration):
  └─> HydrationData::new()  // Called again!
       ├─> session_id: "8f7d6c5b-..."  ❌ MISMATCH
       ├─> random_number: 0.9876       ❌ MISMATCH
       └─> timestamp: 1708167440157    ❌ MISMATCH

Result: React Hydration Error → Event Handlers Don't Attach
```

## ✅ How to Fix This Bug

### Solution 1: Use `useEffect` for Client-Only Random Values

```javascript
function MyComponent() {
  const [sessionId, setSessionId] = useState(null);
  
  useEffect(() => {
    // Only runs on client after hydration
    setSessionId(generateUUID());
  }, []);
  
  return <div>{sessionId || 'Loading...'}</div>;
}
```

### Solution 2: Pass Stable Values from Server

```javascript
// Server passes a stable value
export async function getServerSideProps() {
  return {
    props: {
      sessionId: generateUUID(), // Generated once on server
    },
  };
}

function MyComponent({ sessionId }) {
  return <div>{sessionId}</div>; // Uses the same value
}
```

### Solution 3: Suppress Hydration Warning (Not Recommended)

```javascript
<div suppressHydrationWarning>
  {Math.random()}
</div>
```

### Solution 4: Conditional Rendering

```javascript
const [isMounted, setIsMounted] = useState(false);

useEffect(() => {
  setIsMounted(true);
}, []);

if (!isMounted) return null; // Don't render on server

return <div>{Math.random()}</div>; // Only client-side
```

## 🎯 Learning Objectives

By studying this buggy module, developers will learn:

1. **Hydration process**: How SSR frameworks synchronize server and client
2. **Non-deterministic functions**: Why `Math.random()`, `Date.now()`, and `UUID` cause issues
3. **React reconciliation**: How React matches server HTML with virtual DOM
4. **Event handler attachment**: Why mismatches break interactivity
5. **Debugging techniques**: How to identify and fix hydration errors

## 📚 Related Concepts

- **Server-Side Rendering (SSR)**
- **Client-Side Hydration**
- **React Reconciliation**
- **Next.js getServerSideProps**
- **WebAssembly Integration**

## ⚠️ Security Implications

While this bug doesn't directly expose security vulnerabilities, it can:

- Make the application appear broken to users
- Cause data inconsistencies if random IDs are used for tracking
- Break security features that rely on event handlers (e.g., CSRF token submission)

## 🏷️ Tags

`#bug` `#hydration` `#ssr` `#nextjs` `#react` `#wasm` `#rust` `#webassembly` `#hydration-mismatch` `#server-side-rendering`

## 📝 License

This module is for educational purposes to demonstrate common SSR bugs.

---

**Note**: This module is intentionally buggy for educational purposes. Do NOT use in production applications!
