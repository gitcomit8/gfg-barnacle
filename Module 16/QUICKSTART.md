# Quick Start Guide - Module 16

## What is this?

This module demonstrates a **state fragmentation bug** in multi-step forms where different steps use inconsistent state management approaches.

## The Bug in 30 Seconds

1. User fills a 5-step checkout form
2. Steps 1-3 and 5 save data to a **Global Store** (Redux/Zustand)
3. Step 4 saves data to **local component state** (useState)
4. When user clicks "Back" to Step 4, the component remounts
5. **Result**: All Step 4 data is LOST! 🐛

## Try It Now

### Option 1: Open the Demo HTML

```bash
cd "Module 16"
open demo.html
# or use a local server:
python3 -m http.server 8000
# Visit: http://localhost:8000/demo.html
```

### Option 2: Build the WASM Module

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build
cd "Module 16"
wasm-pack build --target web

# Or use the build script
chmod +x build.sh
./build.sh
```

### Option 3: Integrate with React

See `integration-example.jsx` for a complete React/Redux implementation.

## How to Reproduce the Bug

1. Open `demo.html` in your browser
2. Fill out **Step 1** (Personal Info) → Click "Next"
3. Fill out **Step 2** (Shipping) → Click "Next"
4. Fill out **Step 3** (Billing) → Click "Next"
5. Fill out **Step 4** (Special Instructions) - Write a meaningful gift message!
6. Click "Next" to go to **Step 5** (Review)
7. **Now click "Back" to return to Step 4**
8. **🐛 BUG**: Your gift message and delivery notes are GONE!

## Why Does This Happen?

```
Step 4 Component Lifecycle:

Mount (1st time)     → useState('') creates empty state → User types "Happy Birthday!"
Unmount (go to Step 5) → Component destroyed, local state lost
Mount (2nd time)     → useState('') creates FRESH empty state → Message is gone!
```

## The Fix

**Move Step 4 data to the Global Store:**

```javascript
// ❌ WRONG: Step 4 uses local state
const [giftMessage, setGiftMessage] = useState('');

// ✅ CORRECT: Step 4 uses Redux
const dispatch = useDispatch();
const giftMessage = useSelector(state => state.checkout.giftMessage);
dispatch(saveGiftMessage(value));
```

## Key Learning Points

1. ✅ **All related data should use the same state management approach**
2. ✅ **Local state (useState) is lost when components unmount**
3. ✅ **Global stores (Redux/Zustand) persist data across component lifecycles**
4. ✅ **Always test backward navigation in multi-step forms**
5. ✅ **Document which state management system each component uses**

## Test the Module

```bash
# Run Rust tests
cargo test

# The tests verify:
# - Global store only has data for steps 1-3 and 5
# - Step 4 is always "incomplete" in the store
# - Local state resets on every component mount
```

## Next Steps

1. Read the full `README.md` for detailed analysis
2. Review `integration-example.jsx` for React implementation
3. Check `SECURITY.md` for security implications
4. Study the Rust code in `src/lib.rs`

## Common Questions

**Q: Is this a real bug?**
A: Yes! This happens frequently in production applications when different developers work on different steps.

**Q: How common is this?**
A: Very common. It's a top cause of cart abandonment and user frustration.

**Q: Can't users just not click "Back"?**
A: They will. Users frequently go back to edit information, especially in checkout flows.

**Q: What if I use sessionStorage?**
A: That's a valid workaround, but using a consistent global store is cleaner and more maintainable.

## Need Help?

- Full documentation: `README.md`
- Security info: `SECURITY.md`
- Module overview: `INDEX.md`
- React integration: `integration-example.jsx`

---

**Remember**: This is an intentionally buggy module for educational purposes! 🐛
