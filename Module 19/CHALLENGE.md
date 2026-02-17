# Module 19: Challenge Summary

## Challenge: Fix the Hanging Future

### Difficulty: Advanced ⭐⭐⭐⭐⭐

This module presents a **subtle and difficult-to-diagnose bug** in a custom Future implementation for WebAssembly.

## The Symptom

When you call any of the async functions from JavaScript:

```javascript
await fetch_data_buggy(2000);
```

The function will:
- ✓ Start executing
- ✓ Create a JavaScript Promise
- ✓ The Promise will resolve successfully after 2 seconds
- ❌ **BUT the await will hang forever**

No error. No timeout. No CPU usage. Just... nothing.

## Your Mission

Find and fix the bug that causes the Future to hang.

## Hints

<details>
<summary>Hint 1 (Click to reveal)</summary>

The bug is in the `poll()` method of `BuggyFuture`. Look at what happens when it returns `Poll::Pending`.
</details>

<details>
<summary>Hint 2 (Click to reveal)</summary>

The async runtime needs a way to know when to poll the Future again. What mechanism does Rust's async system use for this?
</details>

<details>
<summary>Hint 3 (Click to reveal)</summary>

Look at the `Context` parameter in the `poll()` method. What's in it that you're not using?
</details>

<details>
<summary>Hint 4 (Click to reveal)</summary>

The key is the `Waker`. You need to:
1. Get it from the Context
2. Store it somewhere
3. Call it when the Promise resolves
</details>

<details>
<summary>Solution (Click to reveal)</summary>

The bug: The `poll()` method returns `Poll::Pending` but never stores or calls the `Waker` from the Context.

To fix it:
1. Clone the waker: `let waker = cx.waker().clone()`
2. Set up a Promise callback that stores the result and calls `waker.wake()`
3. Return `Poll::Pending` only after setting up the callback

Or better yet, use `wasm_bindgen_futures::JsFuture` which handles this correctly!

See `fetch_data_correct()` for a working implementation.
</details>

## Testing Your Fix

1. Modify the `BuggyFuture` implementation in `src/lib.rs`
2. Rebuild: `cargo build --target wasm32-unknown-unknown`
3. Or use wasm-pack: `wasm-pack build --target web`
4. Open `demo.html` in a browser
5. Try the buggy button - it should now work!

## Why This is Hard

1. **No error messages** - The code compiles fine
2. **No runtime error** - Nothing panics or throws
3. **The Promise works** - The JS side completes successfully
4. **No obvious clue** - Everything looks correct until you understand Wakers

## Learning Outcomes

By solving this challenge, you'll deeply understand:

- How Rust's Future trait works
- The role of the Waker in async execution
- How async runtimes schedule tasks
- Common pitfalls when bridging Rust and JavaScript async code
- Why you should usually use existing abstractions like `JsFuture`

## Bonus Challenges

1. **Make it work without JsFuture**: Implement the Waker mechanism yourself
2. **Add timeouts**: Modify the code to timeout after 5 seconds
3. **Handle errors**: Make the buggy version properly handle Promise rejections
4. **Add cancellation**: Implement a way to cancel the pending Future

## Files to Examine

- `src/lib.rs` - The buggy implementation
- `README.md` - Detailed explanation of the bug
- `TECHNICAL.md` - Deep dive into Futures and Wakers
- `demo.html` - Interactive demonstration

## Scoring

- **Find the bug**: 30 points
- **Explain why it hangs**: 30 points
- **Fix it correctly**: 40 points

**Total: 100 points**

Good luck! 🚀
