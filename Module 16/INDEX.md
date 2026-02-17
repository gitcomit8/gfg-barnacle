# Module 16: Multi-Step Form State Fragmentation

## Overview

**Module Type**: State Management Bug  
**Difficulty**: Medium to Hard  
**Language**: Rust (compiles to WebAssembly)  
**Integration**: React, Vue, Angular, Vanilla JS  
**Bug Category**: Data Loss, UX Issue, State Fragmentation  

## What You'll Learn

This module teaches developers about:

- State management consistency in multi-step forms
- Component lifecycle and data persistence
- Single source of truth principle
- Debugging backward navigation issues
- Integration patterns for global state

## The Bug

A 5-step checkout process where:
- Steps 1-3: Use Global Store (Redux/Zustand) ✅
- Step 4: Uses local component state (useState) ⚠️
- Step 5: Uses Global Store ✅

**Problem**: When user navigates backward to Step 4, the component remounts with empty local state, losing all data.

## Files in This Module

| File | Purpose |
|------|---------|
| `src/lib.rs` | Rust implementation with buggy state management |
| `Cargo.toml` | Rust package configuration |
| `demo.html` | Interactive HTML demo of the bug |
| `integration-example.jsx` | React/Redux integration example |
| `README.md` | Comprehensive documentation |
| `QUICKSTART.md` | Quick setup guide |
| `SECURITY.md` | Security implications |
| `INDEX.md` | This file |
| `build.sh` | Build script |
| `package.json` | NPM configuration |

## Quick Start

```bash
# View the demo
open demo.html

# Build the WASM module
wasm-pack build --target web

# Run tests
cargo test
```

## Key Concepts

### Global Store
- Persists data across component lifecycles
- Accessible from any component
- Examples: Redux, Zustand, MobX, Vuex

### Local State
- Lives only in the component
- Lost when component unmounts
- Examples: useState, this.state

### The Bug Pattern

```
Global Store: [Step1] [Step2] [Step3] [MISSING!] [Step5]
Local State:  [ -- ]  [ -- ]  [ -- ]  [Step4]    [ -- ]
```

When navigating back to Step 4, the component remounts and local state resets!

## Real-World Impact

- E-commerce: Lost shipping preferences, gift messages
- Surveys: Lost partial answers when reviewing
- Onboarding: Lost user preferences
- Applications: Lost form progress

**Statistics**: This type of bug increases cart abandonment by 15-25%.

## How to Fix

1. **Unify State Management**: Store ALL steps in the global store
2. **Persist Local State**: Use sessionStorage/localStorage
3. **Prevent Unmounting**: Keep all steps mounted, hide with CSS
4. **Use Context**: React Context for form-level state

## Educational Value

This module is perfect for teaching:

- State management architecture
- Component lifecycle understanding
- Multi-step form best practices
- Debugging data loss issues
- Testing backward navigation flows

## Integration Examples

### React + Redux
See `integration-example.jsx`

### React + Zustand
```javascript
const useCheckoutStore = create((set) => ({
  specialInstructions: null,
  setSpecialInstructions: (data) => set({ specialInstructions: data }),
}));
```

### Vue + Pinia
```javascript
export const useCheckoutStore = defineStore('checkout', {
  state: () => ({
    specialInstructions: null,
  }),
});
```

### Vanilla JS
Use the WASM module directly - see `demo.html`

## Testing Strategy

1. **Forward Navigation Test**: Verify data saves correctly
2. **Backward Navigation Test**: Verify data persists when going back
3. **Data Integrity Test**: Ensure all steps are in the store
4. **Component Remount Test**: Check if local state survives

## Related Modules

This bug is similar to:
- Module 12: Hydration Mismatch (SSR issues)
- State management anti-patterns
- Component lifecycle bugs

## Tags

`#state-management` `#multi-step-form` `#data-loss` `#redux` `#react` `#wasm` `#rust` `#checkout` `#ux-bug` `#form-fragmentation`

## Difficulty Ratings

- **Identification**: ⭐⭐⭐ (Medium) - Bug only shows on backward navigation
- **Debugging**: ⭐⭐⭐⭐ (Hard) - Requires understanding component lifecycle
- **Fixing**: ⭐⭐ (Easy) - Simple fix once identified

## Prerequisites

To use this module, you should understand:
- Basic state management concepts
- Component lifecycle (mounting/unmounting)
- Multi-step form patterns
- JavaScript/TypeScript
- Optional: Rust basics for studying the implementation

## Assessment Questions

1. Why does Step 4 data get lost?
2. What's the difference between local and global state?
3. How would you fix this bug?
4. What testing would catch this bug?
5. What are the security implications?

## Success Criteria

You've mastered this module when you can:
- Identify state fragmentation bugs in code reviews
- Design consistent state management architectures
- Explain the component lifecycle to others
- Implement proper multi-step form state
- Debug data loss issues efficiently

---

**Version**: 0.1.0  
**Author**: GFG Barnacle Team  
**License**: Educational Use  
**Status**: Intentionally Buggy 🐛
