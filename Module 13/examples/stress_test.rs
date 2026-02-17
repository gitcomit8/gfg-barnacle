/*!
 * Stress Test - Demonstrates the race condition bug reliably
 * 
 * This test runs multiple scenarios to catch the race condition bug.
 * 
 * Run with: cargo run --example stress_test
 */

use task_toggle_module::TaskToggleService;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("=== Race Condition Stress Test ===\n");
    println!("Running multiple test scenarios to demonstrate the bug...\n");

    let mut bug_detected = 0;
    let total_runs = 10;

    for run in 1..=total_runs {
        println!("--- Run {}/{} ---", run, total_runs);
        
        let service = TaskToggleService::new(format!("task-{}", run), false);
        
        // Perform rapid toggles
        println!("Executing 5 rapid toggles...");
        for i in 1..=5 {
            service.toggle(format!("task-{}", run)).await.unwrap();
            sleep(Duration::from_millis(5)).await; // Very short delay between clicks
            let state = service.get_local_state().await;
            println!("  After click {}: is_completed = {}", i, state.is_completed);
        }
        
        // Expected: is_completed should be true (odd number of toggles)
        let immediate_state = service.get_local_state().await.is_completed;
        println!("Immediate state (before API responses): {}", immediate_state);
        
        // Wait for all API responses
        println!("Waiting for all API responses...");
        sleep(Duration::from_millis(400)).await;
        
        let final_state = service.get_local_state().await.is_completed;
        println!("Final state (after all API responses): {}", final_state);
        
        // Check if bug occurred
        if immediate_state != final_state {
            println!("🐛 BUG DETECTED! State changed from {} to {}", immediate_state, final_state);
            bug_detected += 1;
        } else {
            println!("✓ State remained consistent (bug did not occur this run)");
        }
        
        println!();
        sleep(Duration::from_millis(100)).await; // Small pause between runs
    }

    println!("\n=== Results ===");
    println!("Total runs: {}", total_runs);
    println!("Bugs detected: {}", bug_detected);
    println!("Success rate: {}%", (total_runs - bug_detected) * 100 / total_runs);
    
    if bug_detected > 0 {
        println!("\n❌ The race condition bug was detected!");
        println!("This demonstrates that responses arrive out of order,");
        println!("causing the UI to flicker to an incorrect state.");
    } else {
        println!("\n⚠️  Bug not detected in this run.");
        println!("The bug is non-deterministic - try running again!");
        println!("With higher network latency, the bug occurs more frequently.");
    }
}
