use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::console;

/// Complex application state that holds user data
/// This struct contains nested data and allocated strings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub user_id: u32,
    pub username: String,
    pub session_token: String,
    pub preferences: UserPreferences,
    pub activity_log: Vec<ActivityEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: String,
    pub language: String,
    pub notifications_enabled: bool,
    pub data: Vec<u8>, // Simulates large data blob
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub timestamp: f64,
    pub action: String,
    pub details: String,
}

impl AppState {
    /// Creates a new AppState with substantial allocated memory
    pub fn new(user_id: u32, username: String) -> Self {
        // Allocate a large data blob to make the leak more obvious
        let data = vec![0u8; 1024 * 100]; // 100KB of data per state object
        
        AppState {
            user_id,
            username: username.clone(),
            session_token: format!("token_{}_{}", user_id, js_sys::Date::now()),
            preferences: UserPreferences {
                theme: String::from("dark"),
                language: String::from("en-US"),
                notifications_enabled: true,
                data,
            },
            activity_log: Vec::new(),
        }
    }
    
    /// Adds an activity entry to the log
    pub fn add_activity(&mut self, action: String, details: String) {
        self.activity_log.push(ActivityEntry {
            timestamp: js_sys::Date::now(),
            action,
            details,
        });
    }
}

/// Creates a new application state and returns a raw pointer to it
/// 
/// ⚠️ BUG: This function uses Box::into_raw() to transfer ownership to JavaScript,
/// but there is NO corresponding deallocation function provided!
/// 
/// The JavaScript code receives a raw pointer (usize) but has no way to free the memory.
/// Each call to this function leaks ~100KB+ of memory that will never be reclaimed.
#[wasm_bindgen]
pub fn create_app_state(user_id: u32, username: String) -> usize {
    let state = AppState::new(user_id, username);
    let boxed_state = Box::new(state);
    
    // Convert Box to raw pointer - ownership is transferred to JS
    // BUG: JS cannot and will not deallocate this memory!
    let raw_ptr = Box::into_raw(boxed_state);
    
    console::log_1(&JsValue::from_str(&format!(
        "Created AppState at address: {:p} ({}KB allocated)",
        raw_ptr,
        std::mem::size_of::<AppState>() / 1024
    )));
    
    raw_ptr as usize
}

/// Updates the state by adding an activity entry
/// 
/// ⚠️ BUG: This creates a NEW state object and returns a new pointer,
/// but the OLD pointer is never freed! This compounds the memory leak.
#[wasm_bindgen]
pub fn update_app_state(state_ptr: usize, action: String, details: String) -> usize {
    unsafe {
        // Reconstruct the Box from the raw pointer
        let mut boxed_state = Box::from_raw(state_ptr as *mut AppState);
        
        // Update the state
        boxed_state.add_activity(action, details);
        
        // BUG: We create a NEW Box and return a NEW pointer
        // The old pointer should be freed, but JS is holding onto it!
        let new_boxed_state = Box::new((*boxed_state).clone());
        let new_raw_ptr = Box::into_raw(new_boxed_state);
        
        // We need to prevent the original from being dropped
        // (even though it should be freed)
        std::mem::forget(boxed_state);
        
        console::log_1(&JsValue::from_str(&format!(
            "Updated AppState: new address: {:p}",
            new_raw_ptr
        )));
        
        new_raw_ptr as usize
    }
}

/// Reads data from the state without modifying it
/// 
/// This function is "safe" in that it doesn't create new pointers,
/// but it still requires the pointer to be valid (not freed)
#[wasm_bindgen]
pub fn get_user_info(state_ptr: usize) -> String {
    unsafe {
        let state_ref = &*(state_ptr as *const AppState);
        format!(
            "User: {} (ID: {}), Activities: {}",
            state_ref.username,
            state_ref.user_id,
            state_ref.activity_log.len()
        )
    }
}

/// Gets the current activity count
#[wasm_bindgen]
pub fn get_activity_count(state_ptr: usize) -> usize {
    unsafe {
        let state_ref = &*(state_ptr as *const AppState);
        state_ref.activity_log.len()
    }
}

/// Gets the session token
#[wasm_bindgen]
pub fn get_session_token(state_ptr: usize) -> String {
    unsafe {
        let state_ref = &*(state_ptr as *const AppState);
        state_ref.session_token.clone()
    }
}

/// This is the MISSING function that should exist but doesn't!
/// 
/// ❌ INTENTIONALLY COMMENTED OUT TO DEMONSTRATE THE BUG ❌
/// 
/// If this function existed and was called by JS, the memory leak would be fixed.
/// Uncomment this function and call it from JS to fix the bug.
/*
#[wasm_bindgen]
pub fn free_app_state(state_ptr: usize) {
    unsafe {
        // Reconstruct the Box from the raw pointer
        // When the Box goes out of scope, it will be properly deallocated
        let boxed_state = Box::from_raw(state_ptr as *mut AppState);
        drop(boxed_state);
        
        console::log_1(&JsValue::from_str(&format!(
            "Freed AppState at address: {:p}",
            state_ptr as *const AppState
        )));
    }
}
*/

/// Utility function to get WASM memory stats
/// This can be used to observe the memory leak
#[wasm_bindgen]
pub fn get_memory_info() -> String {
    let memory = wasm_bindgen::memory();
    let buffer = js_sys::Reflect::get(&memory, &JsValue::from_str("buffer"))
        .unwrap_or(JsValue::NULL);
    
    if let Ok(array_buffer) = buffer.dyn_into::<js_sys::ArrayBuffer>() {
        let size_bytes = array_buffer.byte_length() as f64;
        let size_mb = size_bytes / (1024.0 * 1024.0);
        format!("WASM Memory: {:.2} MB ({} bytes)", size_mb, size_bytes)
    } else {
        String::from("Memory info unavailable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_state_creation() {
        let state = AppState::new(1, String::from("testuser"));
        assert_eq!(state.user_id, 1);
        assert_eq!(state.username, "testuser");
        assert_eq!(state.preferences.data.len(), 1024 * 100);
    }
    
    #[test]
    fn test_activity_log() {
        let mut state = AppState::new(1, String::from("testuser"));
        state.add_activity(String::from("login"), String::from("User logged in"));
        assert_eq!(state.activity_log.len(), 1);
        assert_eq!(state.activity_log[0].action, "login");
    }
}
