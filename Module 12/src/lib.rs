use wasm_bindgen::prelude::*;
use uuid::Uuid;
use js_sys::Math;
use web_sys::console;

/// The Hydration Mismatch Module
/// 
/// This module demonstrates a critical bug in SSR (Server-Side Rendering) applications.
/// The bug: Server generates random values that are regenerated on the client,
/// causing hydration mismatches that break React's event system.
///
/// ## The Bug Explained:
/// 1. Server-side rendering generates a random value (UUID, timestamp, or number)
/// 2. This value is embedded in the initial HTML
/// 3. When the client hydrates, it generates a NEW random value
/// 4. React detects a mismatch between server HTML and client virtual DOM
/// 5. Event listeners fail to attach properly, making the UI non-interactive

#[wasm_bindgen]
pub struct HydrationData {
    session_id: String,
    random_number: f64,
    timestamp: i64,
    component_key: String,
}

#[wasm_bindgen]
impl HydrationData {
    /// Creates new hydration data with random values
    /// BUG: This function is called both on server and client,
    /// generating different values each time!
    #[wasm_bindgen(constructor)]
    pub fn new() -> HydrationData {
        // Generate a random UUID - different every time!
        let session_id = Uuid::new_v4().to_string();
        
        // Generate a random number using Math.random()
        // This is the PRIMARY BUG - Math.random() will return different values
        // on server vs client, causing hydration mismatch
        let random_number = Math::random();
        
        // Get current timestamp - also different on server vs client
        let timestamp = js_sys::Date::now() as i64;
        
        // Generate another random component key
        let component_key = format!("comp-{}-{}", random_number, timestamp);
        
        console::log_1(&JsValue::from_str(&format!(
            "Generated HydrationData: session={}, random={}, timestamp={}", 
            session_id, random_number, timestamp
        )));
        
        HydrationData {
            session_id,
            random_number,
            timestamp,
            component_key,
        }
    }
    
    /// Returns the session ID (randomly generated UUID)
    /// BUG: This will be different on server vs client!
    #[wasm_bindgen(getter)]
    pub fn session_id(&self) -> String {
        self.session_id.clone()
    }
    
    /// Returns the random number
    /// BUG: This is generated via Math.random() and will differ!
    #[wasm_bindgen(getter)]
    pub fn random_number(&self) -> f64 {
        self.random_number
    }
    
    /// Returns the timestamp
    /// BUG: Server and client render at different times!
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }
    
    /// Returns the component key
    /// BUG: Derived from random data, will mismatch!
    #[wasm_bindgen(getter)]
    pub fn component_key(&self) -> String {
        self.component_key.clone()
    }
    
    /// Generates HTML with the random data embedded
    /// This is what would be rendered on the server
    #[wasm_bindgen]
    pub fn render_html(&self) -> String {
        format!(
            r#"<div class="hydration-component" data-session="{}" data-key="{}">
    <h2>Hydration Test Component</h2>
    <p>Session ID: <span id="session-display">{}</span></p>
    <p>Random Number: <span id="random-display">{:.10}</span></p>
    <p>Timestamp: <span id="timestamp-display">{}</span></p>
    <button id="test-button" onclick="handleClick()">Click Me!</button>
    <div id="click-count">Clicks: 0</div>
</div>"#,
            self.session_id,
            self.component_key,
            self.session_id,
            self.random_number,
            self.timestamp
        )
    }
}

/// Utility function to generate a random ID
/// BUG: Called on both server and client, generates different values
#[wasm_bindgen]
pub fn generate_random_id() -> String {
    let uuid = Uuid::new_v4();
    let random = Math::random();
    format!("id-{}-{:.0}", uuid, random * 1000000.0)
}

/// Utility function to get current time-based value
/// BUG: Will be different on server vs client
#[wasm_bindgen]
pub fn get_timestamp_value() -> f64 {
    js_sys::Date::now()
}

/// Function to demonstrate the hydration mismatch
/// Returns a value that should be the same but will differ
#[wasm_bindgen]
pub fn get_hydration_value() -> f64 {
    // This Math.random() call is the core of the bug
    // It returns different values on server vs client
    Math::random() * 10000.0
}

/// Creates a "stable" ID that's actually unstable
/// BUG: Uses random data that changes between renders
#[wasm_bindgen]
pub fn create_component_id(prefix: &str) -> String {
    let random_suffix = (Math::random() * 1000000.0) as u32;
    format!("{}-{}", prefix, random_suffix)
}

#[cfg(test)]
mod tests {
    #[cfg(target_arch = "wasm32")]
    use super::*;
    
    // Note: These tests only work in a WASM environment
    // Run with: wasm-pack test --headless --firefox
    
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_hydration_data_creation() {
        let data1 = HydrationData::new();
        let data2 = HydrationData::new();
        
        // These should be different - demonstrating the bug!
        assert_ne!(data1.session_id, data2.session_id);
        assert_ne!(data1.random_number, data2.random_number);
        assert_ne!(data1.component_key, data2.component_key);
    }
    
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_random_id_generation() {
        let id1 = generate_random_id();
        let id2 = generate_random_id();
        
        // IDs should be different each time
        assert_ne!(id1, id2);
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_module_structure() {
        // This test runs on non-WASM targets
        // It just verifies the module structure compiles
        assert!(true, "Module structure is valid");
    }
}
