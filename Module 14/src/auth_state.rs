use std::sync::Mutex;
use std::collections::HashMap;

pub struct AppState {
    pub sessions: Mutex<HashMap<String, String>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            sessions: Mutex::new(HashMap::new()),
        }
    }
}
