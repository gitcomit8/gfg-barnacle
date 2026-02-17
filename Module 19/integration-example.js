/**
 * Integration Example: Using the Waker Bug Module in a Web Application
 * 
 * This file demonstrates how the buggy module would be used in a real webapp
 * and how the bug manifests in production code.
 */

import init, { 
    fetch_data_buggy, 
    fetch_data_correct,
    get_user_data,
    complex_async_operation,
    countdown_buggy
} from './pkg/waker_bug_module.js';

// ============================================================================
// EXAMPLE 1: Simple API Call (Will Hang Forever)
// ============================================================================

async function loadUserDashboard(userId) {
    console.log('Loading dashboard for user:', userId);
    
    try {
        // This looks like a normal async API call
        // But it will hang forever!
        const userData = await get_user_data(userId);
        
        // Everything below never executes
        console.log('User data loaded:', userData);
        
        // Update UI with user data
        document.getElementById('user-name').textContent = userData.name;
        document.getElementById('user-email').textContent = userData.email;
        
        return userData;
    } catch (error) {
        console.error('Failed to load user:', error);
        // This catch block is never reached either!
        // The function just hangs at the await
    }
}

// ============================================================================
// EXAMPLE 2: Loading Spinner That Never Stops
// ============================================================================

async function fetchDataWithSpinner() {
    const spinner = document.getElementById('loading-spinner');
    const content = document.getElementById('content');
    
    // Show spinner
    spinner.style.display = 'block';
    content.style.display = 'none';
    
    try {
        // This await hangs forever
        const data = await fetch_data_buggy(2000);
        
        // Spinner never gets hidden!
        spinner.style.display = 'none';
        content.style.display = 'block';
        content.textContent = data;
    } catch (error) {
        spinner.style.display = 'none';
        console.error('Error:', error);
    }
    
    // User sees: spinning wheel forever, no error, no timeout
}

// ============================================================================
// EXAMPLE 3: Form Submission That Looks Successful But Hangs
// ============================================================================

async function handleFormSubmit(event) {
    event.preventDefault();
    
    const submitButton = event.target.querySelector('button[type="submit"]');
    submitButton.disabled = true;
    submitButton.textContent = 'Submitting...';
    
    const formData = new FormData(event.target);
    const data = Object.fromEntries(formData);
    
    try {
        // Send data to server (hangs forever)
        await fetch_data_buggy(1000);
        
        // Success message never shows
        showNotification('Form submitted successfully!', 'success');
        submitButton.disabled = false;
        submitButton.textContent = 'Submit';
        event.target.reset();
    } catch (error) {
        // Error handling never reached
        showNotification('Submission failed', 'error');
        submitButton.disabled = false;
        submitButton.textContent = 'Submit';
    }
    
    // Result: Button stays disabled with "Submitting..." text forever
}

// ============================================================================
// EXAMPLE 4: Chained Operations (Hang at First Step)
// ============================================================================

async function initializeApplication() {
    console.log('Step 1: Loading configuration...');
    const config = await fetch_data_buggy(500);  // HANGS HERE
    
    console.log('Step 2: Authenticating user...');
    const auth = await fetch_data_buggy(800);
    
    console.log('Step 3: Loading initial data...');
    const data = await fetch_data_buggy(1000);
    
    console.log('Step 4: Rendering UI...');
    renderUI(config, auth, data);
    
    console.log('Application ready!');
    
    // None of the steps after "Loading configuration..." execute
    // The entire app initialization hangs at step 1
}

// ============================================================================
// EXAMPLE 5: Timeout Doesn't Help
// ============================================================================

async function fetchWithAttemptedTimeout() {
    console.log('Attempting to fetch with timeout...');
    
    // This is a common pattern developers try
    const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Timeout')), 5000);
    });
    
    try {
        // But this doesn't work because fetch_data_buggy returns
        // a "Promise" that never resolves OR rejects
        const result = await Promise.race([
            fetch_data_buggy(2000),
            timeoutPromise
        ]);
        
        console.log('Got result:', result);
    } catch (error) {
        // The timeout will trigger, but that's misleading
        // because the actual bug is that the Future isn't waking
        console.error('Timed out:', error);
    }
    
    // This will print "Timed out" after 5 seconds, but the
    // underlying Future is still hung and consuming resources
}

// ============================================================================
// EXAMPLE 6: Correct Implementation for Comparison
// ============================================================================

async function correctImplementation() {
    console.log('Using correct implementation...');
    
    const spinner = document.getElementById('loading-spinner');
    spinner.style.display = 'block';
    
    try {
        // This works correctly!
        const result = await fetch_data_correct(2000);
        
        console.log('Result:', result);
        spinner.style.display = 'none';
        
        return result;
    } catch (error) {
        console.error('Error:', error);
        spinner.style.display = 'none';
        throw error;
    }
    
    // This completes after 2 seconds as expected
}

// ============================================================================
// EXAMPLE 7: Real-World Scenario - User Profile Page
// ============================================================================

class UserProfileComponent {
    constructor(userId) {
        this.userId = userId;
        this.isLoading = false;
    }
    
    async loadProfile() {
        if (this.isLoading) return;
        
        this.isLoading = true;
        this.showLoadingState();
        
        try {
            // The bug: this await never completes
            const userData = await get_user_data(this.userId);
            
            // Never reached
            this.renderProfile(userData);
            this.isLoading = false;
        } catch (error) {
            // Never reached
            this.showError(error);
            this.isLoading = false;
        }
        
        // Result: Page stuck in loading state forever
        // No error messages, no console output
        // Just a spinning loader that never stops
    }
    
    showLoadingState() {
        const container = document.getElementById('profile-container');
        container.innerHTML = `
            <div class="loading">
                <div class="spinner"></div>
                <p>Loading profile...</p>
            </div>
        `;
    }
    
    renderProfile(userData) {
        const container = document.getElementById('profile-container');
        container.innerHTML = `
            <div class="profile">
                <h2>${userData.name}</h2>
                <p>${userData.email}</p>
            </div>
        `;
    }
    
    showError(error) {
        const container = document.getElementById('profile-container');
        container.innerHTML = `
            <div class="error">
                <p>Failed to load profile: ${error.message}</p>
            </div>
        `;
    }
}

// ============================================================================
// EXAMPLE 8: Debugging Tips
// ============================================================================

async function debuggingExample() {
    console.log('=== DEBUGGING THE HANG ===');
    
    // Add detailed logging
    console.log('[1] About to call fetch_data_buggy');
    console.time('fetch_data_buggy');
    
    const promise = fetch_data_buggy(2000);
    console.log('[2] Promise created:', promise);
    
    // Add a marker to check if we're past the await
    let awaitCompleted = false;
    
    // Set up a check timer
    const checkInterval = setInterval(() => {
        console.log(`[Check] awaitCompleted = ${awaitCompleted}, elapsed = ${performance.now()}`);
    }, 1000);
    
    try {
        const result = await promise;
        awaitCompleted = true;
        console.timeEnd('fetch_data_buggy');
        console.log('[3] Result received:', result);
    } catch (error) {
        console.log('[3] Error caught:', error);
    } finally {
        clearInterval(checkInterval);
    }
    
    console.log('[4] Function completed, awaitCompleted =', awaitCompleted);
    
    // What you'll see:
    // [1] About to call fetch_data_buggy
    // [2] Promise created: Promise {<pending>}
    // [Check] awaitCompleted = false, elapsed = 1234.5
    // [Check] awaitCompleted = false, elapsed = 2234.5
    // [Check] awaitCompleted = false, elapsed = 3234.5
    // ... forever, steps [3] and [4] never execute
}

// ============================================================================
// HOW TO FIX: Replace all buggy calls with correct implementations
// ============================================================================

async function fixedLoadUserDashboard(userId) {
    console.log('Loading dashboard for user:', userId);
    
    try {
        // Use the correct implementation
        const userData = await fetch_data_correct(1500);
        
        // Now this executes!
        console.log('User data loaded:', userData);
        
        return userData;
    } catch (error) {
        console.error('Failed to load user:', error);
        throw error;
    }
}

// ============================================================================
// INITIALIZATION
// ============================================================================

async function main() {
    // Initialize the WASM module
    await init();
    
    console.log('Module loaded. Try the examples:');
    console.log('1. loadUserDashboard(123) - Will hang forever');
    console.log('2. correctImplementation() - Works correctly');
    console.log('3. debuggingExample() - Shows how to debug');
    
    // Expose functions globally for testing
    window.WakerBugDemo = {
        loadUserDashboard,
        fetchDataWithSpinner,
        initializeApplication,
        fetchWithAttemptedTimeout,
        correctImplementation,
        debuggingExample,
        fixedLoadUserDashboard
    };
}

// Auto-initialize when script loads
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', main);
} else {
    main();
}

// Helper function
function showNotification(message, type) {
    console.log(`[${type.toUpperCase()}] ${message}`);
}

export {
    loadUserDashboard,
    fetchDataWithSpinner,
    handleFormSubmit,
    initializeApplication,
    fetchWithAttemptedTimeout,
    correctImplementation,
    debuggingExample,
    fixedLoadUserDashboard,
    UserProfileComponent
};
