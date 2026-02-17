/*!
 * WebApp Demo - Task Toggle Integration
 * 
 * This example demonstrates how the buggy TaskToggleService would be
 * integrated into a real webapp. It shows the race condition bug in action.
 * 
 * Run with: cargo run --example webapp_demo
 */

use task_toggle_module::TaskToggleService;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("=== Task Toggle WebApp Demo ===\n");
    println!("This demo simulates a webapp with a task toggle button.");
    println!("Watch for the UI flickering bug!\n");

    // Initialize the service (like initializing state in React/Vue)
    let service = TaskToggleService::new("demo-task-1".to_string(), false);
    
    println!("Initial state: {:?}\n", service.get_local_state().await);

    // Scenario 1: Normal usage (works fine)
    println!("--- Scenario 1: Single Click (Works Fine) ---");
    single_click_demo(&service).await;
    
    println!("\n");
    
    // Scenario 2: Rapid clicks (demonstrates the bug)
    println!("--- Scenario 2: Rapid Triple Click (SHOWS BUG) ---");
    rapid_click_demo(&service).await;
}

async fn single_click_demo(service: &TaskToggleService) {
    println!("User clicks toggle button once...");
    
    let state = service.toggle("demo-task-1".to_string()).await.unwrap();
    println!("UI immediately shows: is_completed = {}", state.is_completed);
    
    println!("Waiting for API response...");
    sleep(Duration::from_millis(300)).await;
    
    let final_state = service.get_local_state().await;
    println!("Final state after API: is_completed = {}", final_state.is_completed);
    println!("✓ State is correct!");
}

async fn rapid_click_demo(service: &TaskToggleService) {
    // Reset to known state
    let mut current_state = service.get_local_state().await;
    println!("Starting state: is_completed = {}", current_state.is_completed);
    
    println!("\nUser rapidly clicks 3 times (like spam-clicking)...");
    
    // Click 1
    println!("  [Time 0ms] Click 1 → Sending request 1");
    service.toggle("demo-task-1".to_string()).await.unwrap();
    current_state = service.get_local_state().await;
    println!("    UI shows: is_completed = {}", current_state.is_completed);
    sleep(Duration::from_millis(10)).await;
    
    // Click 2
    println!("  [Time 10ms] Click 2 → Sending request 2");
    service.toggle("demo-task-1".to_string()).await.unwrap();
    current_state = service.get_local_state().await;
    println!("    UI shows: is_completed = {}", current_state.is_completed);
    sleep(Duration::from_millis(10)).await;
    
    // Click 3
    println!("  [Time 20ms] Click 3 → Sending request 3");
    service.toggle("demo-task-1".to_string()).await.unwrap();
    current_state = service.get_local_state().await;
    println!("    UI shows: is_completed = {} (this is what user expects!)", current_state.is_completed);
    
    println!("\nWaiting for API responses to arrive...");
    
    // Check state at intervals to see it change
    for i in 1..=5 {
        sleep(Duration::from_millis(100)).await;
        let state = service.get_local_state().await;
        println!("  [Time {}ms] Current state: is_completed = {}", 20 + (i * 100), state.is_completed);
    }
    
    let final_state = service.get_local_state().await;
    println!("\n🐛 BUG DETECTED!");
    println!("Expected final state: is_completed = true (from click 3)");
    println!("Actual final state: is_completed = {}", final_state.is_completed);
    
    if final_state.is_completed {
        println!("⚠️  In this run, responses happened to arrive in order.");
        println!("   Try running again - the bug is non-deterministic!");
    } else {
        println!("❌ The UI flickered to the WRONG state!");
        println!("   This is because response 2 arrived after response 3.");
    }
}

/// Example of how this would be exposed to JavaScript in a web frontend
#[cfg(target_arch = "wasm32")]
mod wasm_bindings {
    use wasm_bindgen::prelude::*;
    use super::*;

    #[wasm_bindgen]
    pub struct WebTaskToggle {
        service: TaskToggleService,
    }

    #[wasm_bindgen]
    impl WebTaskToggle {
        #[wasm_bindgen(constructor)]
        pub fn new(task_id: String) -> Self {
            Self {
                service: TaskToggleService::new(task_id, false),
            }
        }

        #[wasm_bindgen]
        pub async fn toggle(&self, task_id: String) -> JsValue {
            match self.service.toggle(task_id).await {
                Ok(state) => serde_wasm_bindgen::to_value(&state).unwrap(),
                Err(e) => JsValue::from_str(&format!("Error: {}", e)),
            }
        }

        #[wasm_bindgen]
        pub async fn get_state(&self) -> JsValue {
            let state = self.service.get_local_state().await;
            serde_wasm_bindgen::to_value(&state).unwrap()
        }
    }
}

/*
Example JavaScript usage (after compiling to WASM):

```javascript
import { WebTaskToggle } from './task_toggle_module';

// Initialize
const taskToggle = new WebTaskToggle('task-123');

// Attach to button
document.getElementById('toggle-btn').addEventListener('click', async () => {
  // This will have the race condition bug!
  const state = await taskToggle.toggle('task-123');
  updateUI(state); // UI might flicker back to wrong state
});

// Get current state
const currentState = await taskToggle.get_state();
console.log('Current state:', currentState);
```

Typical framework integration (React):

```jsx
function TaskItem({ taskId }) {
  const [isCompleted, setIsCompleted] = useState(false);
  const taskToggle = useRef(new WebTaskToggle(taskId));
  
  const handleToggle = async () => {
    // BUG: If user clicks rapidly, state will flicker
    const state = await taskToggle.current.toggle(taskId);
    setIsCompleted(state.is_completed);
  };
  
  return (
    <div>
      <button onClick={handleToggle}>
        {isCompleted ? '✓ Completed' : '○ Not Completed'}
      </button>
    </div>
  );
}
```
*/
