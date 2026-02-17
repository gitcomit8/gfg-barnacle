// integration-example.jsx
// This file demonstrates how the buggy Rust/WASM module would be used
// in a Next.js application, causing hydration mismatch errors.

import React, { useEffect, useState } from 'react';

// ❌ BUGGY COMPONENT - Causes Hydration Mismatch
// This component uses the Rust WASM module that generates random values
export function BuggyHydrationComponent() {
  // This is called on both server and client, generating different values!
  const hydrationData = useMemo(() => {
    // In a real implementation, this would call the WASM module:
    // const wasmModule = await import('../pkg/hydration_mismatch_module');
    // return wasmModule.HydrationData.new();
    
    // For demonstration, we simulate the WASM module's behavior:
    return {
      session_id: generateUUID(),           // ❌ Different on server vs client
      random_number: Math.random(),         // ❌ Different on server vs client
      timestamp: Date.now(),                // ❌ Different on server vs client
      component_key: `comp-${Math.random()}-${Date.now()}`, // ❌ Different
    };
  }, []); // Empty deps means this runs once per render context (server + client)
  
  return (
    <div 
      className="hydration-component" 
      data-session={hydrationData.session_id}
      data-key={hydrationData.component_key}
    >
      <h2>Hydration Test Component</h2>
      <p>Session ID: <span id="session-display">{hydrationData.session_id}</span></p>
      <p>Random Number: <span id="random-display">{hydrationData.random_number.toFixed(10)}</span></p>
      <p>Timestamp: <span id="timestamp-display">{hydrationData.timestamp}</span></p>
      <button onClick={handleClick}>Click Me!</button>
      <div id="click-count">Clicks: 0</div>
    </div>
  );
}

// ❌ ANOTHER BUGGY PATTERN - Random ID in render
export function BuggyButtonComponent() {
  // This generates a different ID on server vs client!
  const buttonId = `btn-${Math.random().toString(36).substr(2, 9)}`;
  
  return (
    <button id={buttonId} onClick={() => alert('Clicked!')}>
      Button with ID: {buttonId}
    </button>
  );
}

// ❌ BUGGY PATTERN - Timestamp in render
export function BuggyTimestampComponent() {
  const renderTime = new Date().toISOString();
  
  return (
    <div>
      <p>Rendered at: {renderTime}</p>
      {/* Server and client will show different times! */}
    </div>
  );
}

// ❌ EXTREMELY BUGGY - Multiple sources of randomness
export function ExtremelyBuggyComponent() {
  // All of these cause hydration mismatches!
  const uuid = generateUUID();
  const random1 = Math.random();
  const random2 = Math.random() * 1000;
  const timestamp = Date.now();
  const randomColor = `#${Math.floor(Math.random()*16777215).toString(16)}`;
  
  return (
    <div style={{ backgroundColor: randomColor }}>
      <h3>ID: {uuid}</h3>
      <p>Random 1: {random1}</p>
      <p>Random 2: {random2}</p>
      <p>Timestamp: {timestamp}</p>
      <button onClick={() => console.log('Click')}>
        This button won't work!
      </button>
    </div>
  );
}

// ✅ CORRECT IMPLEMENTATION - Fix #1: useEffect for client-only values
export function FixedComponentWithUseEffect() {
  const [hydrationData, setHydrationData] = useState(null);
  
  useEffect(() => {
    // Only runs on client after hydration is complete
    setHydrationData({
      session_id: generateUUID(),
      random_number: Math.random(),
      timestamp: Date.now(),
      component_key: `comp-${Math.random()}-${Date.now()}`,
    });
  }, []);
  
  if (!hydrationData) {
    return <div>Loading...</div>; // Server renders this
  }
  
  return (
    <div className="hydration-component">
      <h2>Fixed Component</h2>
      <p>Session ID: {hydrationData.session_id}</p>
      <p>Random Number: {hydrationData.random_number.toFixed(10)}</p>
      <p>Timestamp: {hydrationData.timestamp}</p>
      <button onClick={() => alert('This works!')}>Click Me!</button>
    </div>
  );
}

// ✅ CORRECT IMPLEMENTATION - Fix #2: Server-side props
export async function getServerSideProps() {
  // Generate stable values on the server
  const stableData = {
    session_id: generateUUID(),
    random_number: Math.random(),
    timestamp: Date.now(),
  };
  
  return {
    props: {
      hydrationData: stableData,
    },
  };
}

export function FixedComponentWithProps({ hydrationData }) {
  // Uses the same values passed from server
  return (
    <div className="hydration-component">
      <h2>Fixed Component with Props</h2>
      <p>Session ID: {hydrationData.session_id}</p>
      <p>Random Number: {hydrationData.random_number.toFixed(10)}</p>
      <p>Timestamp: {hydrationData.timestamp}</p>
      <button onClick={() => alert('This works too!')}>Click Me!</button>
    </div>
  );
}

// ✅ CORRECT IMPLEMENTATION - Fix #3: Conditional client-side rendering
export function FixedComponentWithMountCheck() {
  const [isMounted, setIsMounted] = useState(false);
  
  useEffect(() => {
    setIsMounted(true);
  }, []);
  
  // Don't render random content until client-side mount
  if (!isMounted) {
    return <div>Preparing component...</div>;
  }
  
  const hydrationData = {
    session_id: generateUUID(),
    random_number: Math.random(),
    timestamp: Date.now(),
  };
  
  return (
    <div className="hydration-component">
      <h2>Fixed Component (Client Only)</h2>
      <p>Session ID: {hydrationData.session_id}</p>
      <p>Random Number: {hydrationData.random_number.toFixed(10)}</p>
      <p>Timestamp: {hydrationData.timestamp}</p>
      <button onClick={() => alert('This works perfectly!')}>Click Me!</button>
    </div>
  );
}

// Utility function used in examples
function generateUUID() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    const r = Math.random() * 16 | 0;
    const v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
}

function handleClick() {
  console.log('Button clicked!');
  const countElement = document.getElementById('click-count');
  if (countElement) {
    const currentCount = parseInt(countElement.textContent.split(': ')[1] || '0');
    countElement.textContent = `Clicks: ${currentCount + 1}`;
  }
}

// ═══════════════════════════════════════════════════════════════════
// Example Next.js Page Component using the buggy module
// ═══════════════════════════════════════════════════════════════════

export default function HydrationMismatchDemoPage() {
  return (
    <div style={{ padding: '20px', maxWidth: '800px', margin: '0 auto' }}>
      <h1>🐛 Hydration Mismatch Examples</h1>
      
      <section>
        <h2>❌ Buggy Components (Will Cause Hydration Errors)</h2>
        <BuggyHydrationComponent />
        <BuggyButtonComponent />
        <BuggyTimestampComponent />
        <ExtremelyBuggyComponent />
      </section>
      
      <section>
        <h2>✅ Fixed Components (Properly Handle Random Values)</h2>
        <FixedComponentWithUseEffect />
        <FixedComponentWithMountCheck />
      </section>
      
      <section>
        <h2>📝 Expected Console Errors from Buggy Components:</h2>
        <pre style={{ 
          backgroundColor: '#ffe6e6', 
          padding: '15px', 
          borderRadius: '5px',
          overflow: 'auto'
        }}>
{`Warning: Text content did not match. Server: "550e8400-e29b..." Client: "8f7d6c5b-4a3e..."
Warning: Prop \`data-session\` did not match. Server: "550e8400..." Client: "8f7d6c5b..."
Uncaught Error: Hydration failed because the initial UI does not match what was rendered on the server.
Uncaught Error: There was an error while hydrating. Because the error happened outside of a Suspense boundary, the entire root will switch to client rendering.`}
        </pre>
      </section>
      
      <section>
        <h2>🔍 How to Verify the Bug:</h2>
        <ol>
          <li>Open browser DevTools Console</li>
          <li>Look for hydration warnings (in development mode)</li>
          <li>Try clicking buttons in buggy components - they may not work</li>
          <li>Compare with fixed components - their buttons should work</li>
          <li>Check the HTML source vs rendered DOM - values will differ</li>
        </ol>
      </section>
    </div>
  );
}

// ═══════════════════════════════════════════════════════════════════
// Explanation Comments
// ═══════════════════════════════════════════════════════════════════

/*
 * WHY THIS BUG HAPPENS:
 * 
 * 1. Next.js renders the component on the server
 *    - Calls Math.random(), gets 0.7234
 *    - Generates HTML: <span>0.7234</span>
 * 
 * 2. HTML is sent to the browser
 * 
 * 3. React hydrates the component on the client
 *    - Calls Math.random() again, gets 0.9876
 *    - Expects to find: <span>0.9876</span>
 *    - Actually finds: <span>0.7234</span>
 * 
 * 4. React detects mismatch and throws error
 *    - Tries to recover by re-rendering
 *    - Event handlers may not attach correctly
 *    - Interactive elements become non-functional
 * 
 * WHY IT'S HARD TO DEBUG:
 * 
 * - The page visually appears correct
 * - No obvious errors in production builds (only warnings in dev)
 * - Buttons look clickable but do nothing
 * - Intermittent failures make it hard to reproduce
 * - The real issue is in the React reconciliation layer
 */
