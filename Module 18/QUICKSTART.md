# Module 18: Quick Start Guide

## Getting Started in 5 Minutes

### Prerequisites

- Rust toolchain (1.70+)
- wasm-pack

### Installation

```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Navigate to Module 18
cd "Module 18"
```

### Build the Module

```bash
# Build for web
wasm-pack build --target web

# Or build for Node.js
wasm-pack build --target nodejs

# Or build for bundlers (webpack, rollup, etc.)
wasm-pack build --target bundler
```

### Run the Demo

```bash
# Option 1: Open the HTML file directly
open demo.html

# Option 2: Use a local server
python3 -m http.server 8000
# Then visit http://localhost:8000/demo.html
```

## Reproducing the Bug

### Minimal Example

```javascript
import init, { DataProcessor } from './pkg/reentrancy_deadlock_module.js';

// Initialize WASM
await init();

// Create processor
const processor = new DataProcessor();

// Add some items
processor.add_item("Item 1");
processor.add_item("Item 2");
processor.add_item("Item 3");

// THIS WILL CAUSE A PANIC! ⚠️
try {
    processor.process_items((item, index) => {
        console.log(`Processing: ${item}`);
        
        // BUG: Calling back into Rust from the callback
        const count = processor.get_item_count(); // ❌ PANIC!
        console.log(`Total: ${count}`);
    });
} catch (error) {
    console.error("Panic! Re-entrancy error:", error);
    // Error: "already borrowed: BorrowMutError"
}
```

### Safe Version (No Bug)

```javascript
// This is SAFE - callback doesn't call back into Rust
processor.process_items((item, index) => {
    console.log(`Processing: ${item}`);
    // Only use JavaScript here, don't call processor methods
});
```

## Understanding the Error

When you trigger the bug, you'll see:

```
RuntimeError: unreachable
thread 'main' panicked at 'already borrowed: BorrowMutError'
```

This happens because:
1. `process_items()` borrows the internal state mutably
2. It calls your JavaScript callback
3. Your callback calls `get_item_count()`
4. `get_item_count()` tries to borrow the same state
5. **Panic!** The state is already borrowed

## Key Functions

### Safe Functions (when not in a callback)
- `add_item(item)` - Add an item
- `get_item_count()` - Get count
- `get_summary()` - Get summary
- `clear()` - Clear all items

### Dangerous Functions (use callbacks)
- `process_items(callback)` - Process with callback (re-entrancy risk!)
- `validate_items(validator)` - Validate with callback (re-entrancy risk!)
- `transform_items(transformer)` - Transform with callback (re-entrancy risk!)

## Testing

```bash
# Run Rust tests
cargo test

# Build and test in browser
wasm-pack build --target web
open demo.html
```

## What's Next?

1. Read the full [README.md](README.md) for detailed explanation
2. Check [TECHNICAL.md](TECHNICAL.md) for deep technical analysis
3. Review [SECURITY.md](SECURITY.md) for security implications
4. Try the interactive demo to see the bug in action
5. Attempt to fix the bug yourself!

## Common Questions

**Q: Why does this happen?**
A: Rust's `RefCell` enforces borrowing rules at runtime. When you borrow mutably and try to borrow again (even immutably), it panics.

**Q: How do I avoid this?**
A: Don't call back into Rust methods from within callbacks, or fix the Rust code to release borrows before calling callbacks.

**Q: Is this a Rust bug?**
A: No, this is by design. RefCell is working as intended to prevent undefined behavior.

**Q: Can I use this in production?**
A: No! This module is intentionally buggy for educational purposes.

## Need Help?

- Check the full README.md
- Review the code comments in src/lib.rs
- Look at the working examples
- Study the bug manifestation patterns

Happy debugging! 🐛
