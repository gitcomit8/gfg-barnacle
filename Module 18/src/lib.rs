use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::console;

/// Re-entrancy Deadlock Module (JS-to-Rust-to-JS)
/// 
/// This module demonstrates a critical bug where Rust calls a JS closure that
/// accidentally calls back into another Rust function, causing a RefCell borrow panic.
///
/// ## The Bug Explained:
/// 1. JS calls `process_items()` which borrows STATE mutably
/// 2. `process_items()` calls a JS callback for each item
/// 3. The JS callback (provided by participant) calls `get_item_count()` 
/// 4. `get_item_count()` tries to borrow STATE immutably
/// 5. **BUG**: RefCell panics with "already borrowed: BorrowMutError"
///
/// ## Why This Is Difficult:
/// - Participants blame their JS code for "calling things in the wrong order"
/// - The panic message doesn't clearly explain it's a re-entrancy issue
/// - The bug only appears when the JS callback tries to call Rust functions
/// - Stack trace shows JS in the middle, obscuring the root cause

/// Internal state that holds our data
#[derive(Debug, Clone)]
struct DataState {
    items: Vec<String>,
    processed_count: usize,
    total_operations: usize,
}

impl DataState {
    fn new() -> Self {
        DataState {
            items: vec![],
            processed_count: 0,
            total_operations: 0,
        }
    }
}

/// The DataProcessor with the re-entrancy bug
/// 
/// BUG: Uses RefCell to allow interior mutability, but doesn't protect
/// against re-entrant calls from JS callbacks
#[wasm_bindgen]
pub struct DataProcessor {
    state: Rc<RefCell<DataState>>,
}

#[wasm_bindgen]
impl DataProcessor {
    /// Creates a new DataProcessor
    #[wasm_bindgen(constructor)]
    pub fn new() -> DataProcessor {
        console::log_1(&JsValue::from_str("DataProcessor created"));
        DataProcessor {
            state: Rc::new(RefCell::new(DataState::new())),
        }
    }

    /// Add an item to the internal list
    /// This function is safe - no re-entrancy here
    #[wasm_bindgen]
    pub fn add_item(&self, item: String) {
        let mut state = self.state.borrow_mut();
        state.items.push(item.clone());
        console::log_1(&JsValue::from_str(&format!("Added item: {}", item)));
    }

    /// Get the current item count
    /// 
    /// BUG: This function will panic if called while state is already borrowed!
    /// When a JS callback during process_items() calls this, it triggers:
    /// "already borrowed: BorrowMutError"
    #[wasm_bindgen]
    pub fn get_item_count(&self) -> usize {
        // BUG: This borrow_mut() will panic if state is already borrowed
        // Participants usually call this from JS callbacks during processing
        let state = self.state.borrow();
        console::log_1(&JsValue::from_str(&format!("Getting item count: {}", state.items.len())));
        state.items.len()
    }

    /// Get the total number of operations performed
    /// 
    /// BUG: Same issue - will panic if called during process_items()
    #[wasm_bindgen]
    pub fn get_operation_count(&self) -> usize {
        let state = self.state.borrow();
        state.total_operations
    }

    /// Get the number of processed items
    /// 
    /// BUG: Same issue - will panic if called during process_items()
    #[wasm_bindgen]
    pub fn get_processed_count(&self) -> usize {
        let state = self.state.borrow();
        state.processed_count
    }

    /// Get a specific item by index
    /// 
    /// BUG: Will panic if called during process_items()
    #[wasm_bindgen]
    pub fn get_item(&self, index: usize) -> Option<String> {
        let state = self.state.borrow();
        state.items.get(index).cloned()
    }

    /// Process all items with a callback
    /// 
    /// **THIS IS WHERE THE BUG MANIFESTS!**
    /// 
    /// BUG: This function borrows state mutably, then calls a JS closure.
    /// If that JS closure calls ANY other method on this object (like get_item_count),
    /// it will trigger a panic because we're trying to borrow while already borrowed.
    /// 
    /// The panic message will be: "already borrowed: BorrowMutError"
    /// 
    /// Participants usually think they're calling functions "in the wrong order"
    /// in their JS code, not realizing they've created a re-entrant call chain.
    #[wasm_bindgen]
    pub fn process_items(&self, callback: &js_sys::Function) -> Result<(), JsValue> {
        console::log_1(&JsValue::from_str("Starting to process items..."));
        
        // BUG: We borrow state mutably here and hold it for the entire loop
        let mut state = self.state.borrow_mut();
        
        console::log_1(&JsValue::from_str(&format!(
            "Processing {} items...", state.items.len()
        )));

        // Clone items to avoid borrowing issues with iteration
        let items = state.items.clone();
        
        // Now iterate and call the JS callback for each item
        for (index, item) in items.iter().enumerate() {
            console::log_1(&JsValue::from_str(&format!(
                "Processing item {}: {}", index, item
            )));
            
            // BUG: We call the JS callback while holding a mutable borrow of state!
            // If the callback tries to call ANY method on this object, it will panic
            let this = JsValue::null();
            let item_js = JsValue::from_str(item);
            let index_js = JsValue::from_f64(index as f64);
            
            callback.call2(&this, &item_js, &index_js)?;
            
            // Update processing stats
            state.processed_count += 1;
            state.total_operations += 1;
        }
        
        console::log_1(&JsValue::from_str("Finished processing all items"));
        Ok(())
    }

    /// Validate items with a callback
    /// 
    /// BUG: Same re-entrancy issue as process_items()
    /// Holds a borrow while calling JS, which might call back into Rust
    #[wasm_bindgen]
    pub fn validate_items(&self, validator: &js_sys::Function) -> Result<bool, JsValue> {
        console::log_1(&JsValue::from_str("Starting validation..."));
        
        // BUG: Mutable borrow held while calling JS callback
        let mut state = self.state.borrow_mut();
        state.total_operations += 1;
        
        let items = state.items.clone();
        
        for (index, item) in items.iter().enumerate() {
            let this = JsValue::null();
            let item_js = JsValue::from_str(item);
            let index_js = JsValue::from_f64(index as f64);
            
            // BUG: If validator callback calls get_item_count() or any other
            // method, it will trigger a panic
            let result = validator.call2(&this, &item_js, &index_js)?;
            
            if !result.as_bool().unwrap_or(false) {
                console::log_1(&JsValue::from_str(&format!(
                    "Validation failed at item {}", index
                )));
                return Ok(false);
            }
        }
        
        console::log_1(&JsValue::from_str("Validation passed"));
        Ok(true)
    }

    /// Transform items with a callback
    /// 
    /// BUG: Yet another re-entrancy vulnerability
    #[wasm_bindgen]
    pub fn transform_items(&self, transformer: &js_sys::Function) -> Result<(), JsValue> {
        console::log_1(&JsValue::from_str("Starting transformation..."));
        
        // BUG: Mutable borrow held during JS callback execution
        let mut state = self.state.borrow_mut();
        state.total_operations += 1;
        
        let mut transformed = Vec::new();
        
        for item in state.items.iter() {
            let this = JsValue::null();
            let item_js = JsValue::from_str(item);
            
            // BUG: transformer might call back into Rust, causing panic
            let result = transformer.call1(&this, &item_js)?;
            
            if let Some(s) = result.as_string() {
                transformed.push(s);
            }
        }
        
        state.items = transformed;
        console::log_1(&JsValue::from_str("Transformation complete"));
        Ok(())
    }

    /// Clear all items
    /// Safe method - no callbacks involved
    #[wasm_bindgen]
    pub fn clear(&self) {
        let mut state = self.state.borrow_mut();
        state.items.clear();
        state.processed_count = 0;
        console::log_1(&JsValue::from_str("All items cleared"));
    }

    /// Get a summary of the current state
    /// 
    /// BUG: Will panic if called during process_items() or other callback methods
    #[wasm_bindgen]
    pub fn get_summary(&self) -> String {
        let state = self.state.borrow();
        format!(
            "Items: {}, Processed: {}, Operations: {}",
            state.items.len(),
            state.processed_count,
            state.total_operations
        )
    }
}

/// A helper struct for batch operations
/// This also has the re-entrancy bug
#[wasm_bindgen]
pub struct BatchProcessor {
    processors: Vec<DataProcessor>,
    active_index: Rc<RefCell<usize>>,
}

#[wasm_bindgen]
impl BatchProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> BatchProcessor {
        BatchProcessor {
            processors: vec![],
            active_index: Rc::new(RefCell::new(0)),
        }
    }

    /// Add a processor to the batch
    #[wasm_bindgen]
    pub fn add_processor(&mut self, processor: DataProcessor) {
        self.processors.push(processor);
    }

    /// Process all processors with a callback
    /// 
    /// BUG: Similar re-entrancy issue - borrows active_index while calling JS
    #[wasm_bindgen]
    pub fn process_all(&self, callback: &js_sys::Function) -> Result<(), JsValue> {
        // BUG: Hold mutable borrow while calling JS callbacks
        let mut index = self.active_index.borrow_mut();
        
        for (i, _processor) in self.processors.iter().enumerate() {
            *index = i;
            
            let this = JsValue::null();
            let index_js = JsValue::from_f64(i as f64);
            
            // BUG: If callback tries to call get_active_index(), it panics
            callback.call1(&this, &index_js)?;
        }
        
        Ok(())
    }

    /// Get the current active processor index
    /// 
    /// BUG: Will panic if called during process_all()
    #[wasm_bindgen]
    pub fn get_active_index(&self) -> usize {
        let index = self.active_index.borrow();
        *index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // This test verifies the module structure compiles correctly
        // Full testing requires a WASM environment with JavaScript
        assert!(true, "Module structure is valid");
    }

    #[test]
    fn test_data_state_creation() {
        // Test the internal DataState structure
        let state = DataState::new();
        assert_eq!(state.items.len(), 0);
        assert_eq!(state.processed_count, 0);
        assert_eq!(state.total_operations, 0);
    }

    #[test]
    fn test_data_state_mutations() {
        // Test that we can mutate DataState
        let mut state = DataState::new();
        state.items.push("test".to_string());
        state.processed_count = 5;
        state.total_operations = 10;
        
        assert_eq!(state.items.len(), 1);
        assert_eq!(state.processed_count, 5);
        assert_eq!(state.total_operations, 10);
    }

    // Note: The re-entrancy bug can only be fully demonstrated in a WASM
    // environment where JS can actually call back into Rust.
    // Run: wasm-pack test --headless --firefox
    // Or use the interactive demo.html
}
