# Module 18 - File Index

## 📂 Module Structure

```
Module 18/
├── 📄 INDEX.md                    ← You are here
├── 📘 QUICKSTART.md               ← Start here for quick setup
├── 📕 README.md                   ← Full documentation
├── 🔬 TECHNICAL.md                ← Deep technical analysis
├── 🔒 SECURITY.md                 ← Security analysis
│
├── 🦀 Rust Source Code
│   ├── Cargo.toml                 ← Rust package config
│   ├── Cargo.lock                 ← Dependency lock file
│   └── src/
│       └── lib.rs                 ← Main buggy Rust code with RefCell
│
├── 🌐 Web Integration
│   ├── demo.html                  ← Interactive browser demo
│   ├── integration-example.jsx    ← React component examples
│   └── package.json               ← NPM configuration
│
├── 🔧 Build Tools
│   ├── build.sh                   ← Build script (Rust → WASM)
│   └── .gitignore                 ← Git ignore rules
│
└── 📦 Build Output (generated)
    ├── pkg/                       ← WASM output (after build)
    └── target/                    ← Cargo build artifacts
```

## 🚀 Getting Started

### For First-Time Users
1. Read **QUICKSTART.md** - Get up and running in 5 minutes
2. Open **demo.html** in browser - See the bug in action
3. Read **README.md** - Understand the re-entrancy bug deeply

### For Developers
1. Build the module: `./build.sh`
2. Study **src/lib.rs** - See the buggy RefCell usage
3. Review **integration-example.jsx** - Learn safe patterns

### For Security Reviewers
1. Read **SECURITY.md** - Understand security implications
2. Review **src/lib.rs** - Analyze the re-entrancy vulnerability
3. Check **TECHNICAL.md** - Deep dive into RefCell mechanics

### For Rust Learners
1. Read **TECHNICAL.md** - Learn about RefCell and borrow checking
2. Study **src/lib.rs** - See real-world FFI patterns
3. Experiment with the demo - Trigger the panic yourself

## 📖 Documentation Quick Reference

| Document | Purpose | Read Time |
|----------|---------|-----------|
| **QUICKSTART.md** | Quick setup and bug reproduction | 3-5 min |
| **README.md** | Comprehensive bug documentation | 15-20 min |
| **TECHNICAL.md** | Deep dive into RefCell and re-entrancy | 20-25 min |
| **SECURITY.md** | Security analysis and DoS implications | 10-15 min |
| **demo.html** | Interactive panic demonstration | 5 min |
| **integration-example.jsx** | React code examples | 10-15 min |

## 🎯 Key Files by Use Case

### "I want to understand the re-entrancy bug"
→ Start with **README.md** (Section: "The Bug Flow")

### "I want to see it crash"
→ Open **demo.html** in your browser and click "Trigger Bug"

### "I want to understand RefCell"
→ Read **TECHNICAL.md** (Section: "Runtime Borrow Checking")

### "I want to build the module"
→ Run **./build.sh** or follow **QUICKSTART.md**

### "I want to integrate it in React"
→ Read **integration-example.jsx** (see safe vs unsafe patterns)

### "I want to know the security impact"
→ Read **SECURITY.md** (Section: "Availability Impact")

### "I want to fix similar bugs in my code"
→ Read **README.md** (Section: "How to Fix This Bug")

### "I want to understand Rust FFI"
→ Read **TECHNICAL.md** (Section: "The JavaScript-Rust Boundary")

## 🐛 What This Module Does

This module demonstrates a **re-entrancy deadlock bug** in Rust WASM modules:

```
JavaScript calls process_items()
        ↓
Rust borrows state mutably (RefCell)
        ↓
Rust calls JavaScript callback
        ↓
JavaScript calls get_item_count()
        ↓
Rust tries to borrow state again
        ↓
    ❌ PANIC!
"already borrowed: BorrowMutError"
        ↓
Application crashes
```

## 📝 Important Notes

- ⚠️ **This module is intentionally buggy**
- 🎓 **For educational purposes only**
- ❌ **Do NOT use in production**
- ✅ **Safe for learning and testing**
- 💥 **Will crash on purpose to demonstrate the bug**

## 🔗 File Relationships

```
Cargo.toml ──builds──> src/lib.rs ──compiles to──> pkg/*.wasm
                                                       │
                                                       └──> used by demo.html
                                                       └──> used by integration-example.jsx

README.md ──explains──> The Bug ──shown in──> demo.html
                                     │
                                     └──> demonstrated in integration-example.jsx

TECHNICAL.md ──explains──> RefCell Mechanics ──used in──> src/lib.rs

SECURITY.md ──analyzes──> DoS Risk ──from──> src/lib.rs panics
```

## 🎓 Learning Path

### Beginner Path (New to Rust/WASM)
1. Open **demo.html** → See visual demonstration of the panic
2. Read **QUICKSTART.md** → Understand the basics
3. Read **README.md** introduction → Learn what re-entrancy means

### Intermediate Path (Know Rust basics)
1. Read **README.md** fully → Deep understanding of the bug
2. Study **integration-example.jsx** → See unsafe callback patterns
3. Read **TECHNICAL.md** (RefCell section) → Understand runtime borrowing
4. Build the module with **build.sh** → Hands-on experience

### Advanced Path (Experienced Rustacean)
1. Study **src/lib.rs** → Analyze the vulnerable code
2. Read **TECHNICAL.md** fully → Deep dive into FFI boundaries
3. Experiment with fixes → Try multiple solution approaches
4. Read **SECURITY.md** → Understand real-world impact
5. Create variations → Test edge cases

## 🏆 Key Takeaways

After exploring this module, you'll understand:

✅ How RefCell provides runtime borrow checking  
✅ What re-entrancy means in FFI contexts  
✅ Why callbacks across language boundaries are dangerous  
✅ How to detect and prevent re-entrancy bugs  
✅ When to use RefCell vs Mutex vs RwLock  
✅ Security implications of panic-based DoS  
✅ Proper patterns for Rust/JS integration  

## 🔬 Technical Highlights

### Core Concepts Demonstrated
- **RefCell**: Runtime borrow checking
- **Re-entrancy**: Recursive calls across FFI boundary
- **FFI**: Foreign Function Interface (Rust ↔ JS)
- **wasm-bindgen**: JS/Rust interop
- **Interior Mutability**: Mutating through shared references
- **Panic Handling**: Dealing with unrecoverable errors

### Bug Categories
- Runtime borrow conflict
- Re-entrant function calls
- State management across language boundaries
- Callback safety in FFI

## 📊 Comparison with Other Modules

| Module | Bug Type | Difficulty | Language Boundary |
|--------|----------|------------|-------------------|
| Module 12 | Hydration Mismatch | Medium | Rust/JS (SSR) |
| Module 13 | Race Condition | Medium | Async |
| **Module 18** | **Re-entrancy Deadlock** | **Hard** | **Rust ↔ JS ↔ Rust** |

What makes Module 18 unique:
- Only module with **RefCell panic** demonstration
- Only module showing **triple-language-crossing** (Rust→JS→Rust)
- Only module with **runtime borrow checking** as the bug source
- Most **conceptually challenging** for non-Rust developers

## 🔍 Common Misconceptions

Participants often think:
1. ❌ "I'm calling JS functions in the wrong order" → Actually a re-entrancy issue
2. ❌ "There's a bug in my callback" → Actually Rust's borrow checker doing its job
3. ❌ "The Rust code is broken" → Actually working as designed (preventing UB)
4. ❌ "I need to fix my JavaScript" → Actually need to fix Rust's borrow scopes
5. ❌ "This is a threading issue" → Actually a single-threaded re-entrancy issue

## 🛠️ Debugging Tips

When you encounter this bug:
1. Check if you're calling Rust functions from within callbacks
2. Look for `borrow()` or `borrow_mut()` calls
3. Trace the call stack to see re-entrancy
4. Consider using `try_borrow()` for defensive code
5. Add logging to track borrow acquisition/release

## 📧 Support

This is an educational module. For questions:
- Review the documentation files
- Check the detailed code comments in src/lib.rs
- Examine the examples in integration-example.jsx
- Study the call flow diagrams in TECHNICAL.md

## 🏷️ Module Metadata

- **Type:** Educational Bug Demonstration
- **Language:** Rust (compiles to WebAssembly)
- **Framework Compatibility:** Any JS framework (React, Vue, Angular, vanilla)
- **Difficulty:** Hard to solve (intentionally)
- **Bug Category:** Re-entrancy Deadlock / RefCell Panic
- **Rust Concepts:** RefCell, Interior Mutability, FFI, wasm-bindgen
- **Status:** Complete and tested

## 🎯 Challenge Questions

Test your understanding:

1. Why does RefCell panic instead of just returning an error?
2. How is this different from a threading issue?
3. What would happen with `Mutex` instead of `RefCell`?
4. Can you identify all 4 vulnerable functions?
5. How would you fix this without changing the API?
6. What's the difference between `borrow()` and `try_borrow()`?

Answers in **TECHNICAL.md**!

---

**Ready to explore?** 

- **Quick start:** QUICKSTART.md → demo.html → crash it!
- **Deep dive:** README.md → TECHNICAL.md → understand RefCell
- **Fix it:** Study src/lib.rs → try to fix without breaking the API

Happy debugging! 🐛💥
