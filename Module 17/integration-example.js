/**
 * Module 17: Shadowed Canvas Context - Integration Example
 * 
 * This file demonstrates how to integrate the buggy WASM module
 * into a real webapp, showing when and how the bug manifests.
 */

// Import the WASM module
import init, { ImageProcessor, get_webgl_context } from './pkg/shadowed_canvas_context.js';

/**
 * Example 1: Basic Usage (No Bug)
 * When used in isolation, the module works correctly
 */
async function basicUsageExample() {
    await init();
    
    const gl = get_webgl_context('myCanvas');
    const processor = new ImageProcessor(gl);
    
    // Apply filters - works fine
    processor.apply_grayscale();
    processor.apply_blur(800, 600);
    processor.apply_invert();
    
    console.log('Frame count:', processor.get_frame_count());
}

/**
 * Example 2: Bug Manifestation
 * This shows how the bug appears when JS modifies GL state
 */
async function buggyIntegrationExample() {
    await init();
    
    const canvas = document.getElementById('myCanvas');
    const gl = get_webgl_context('myCanvas');
    const processor = new ImageProcessor(gl);
    
    // Apply grayscale - works correctly
    processor.apply_grayscale();
    console.log('✓ First grayscale application - OK');
    
    // Simulate UI overlay rendering (common in webapps)
    // This modifies the WebGL context state!
    renderUIOverlay(canvas);
    console.log('⚠️ JS modified GL context');
    
    // Apply grayscale again - BUG TRIGGERS HERE!
    // WASM thinks its cached state is valid, but it's not
    processor.apply_grayscale();
    console.log('❌ Second grayscale application - GLITCHED!');
    // Output will be wrong because WASM used stale cache
}

/**
 * Example 3: Webapp with Animation Loop (Bug is Non-Deterministic)
 */
async function animationLoopExample() {
    await init();
    
    const canvas = document.getElementById('myCanvas');
    const gl = get_webgl_context('myCanvas');
    const processor = new ImageProcessor(gl);
    
    let frame = 0;
    let showUI = false;
    
    function animate() {
        frame++;
        
        // Every 60 frames, toggle UI overlay
        if (frame % 60 === 0) {
            showUI = !showUI;
        }
        
        // If UI is shown, JS modifies GL state
        if (showUI) {
            renderUIOverlay(canvas);
        }
        
        // Apply image processing
        // BUG: When showUI just turned false, WASM cache is stale!
        if (frame % 30 === 0) {
            processor.apply_blur(canvas.width, canvas.height);
        } else {
            processor.apply_grayscale();
        }
        
        requestAnimationFrame(animate);
    }
    
    animate();
}

/**
 * Example 4: The Fix - Manual Cache Invalidation
 */
async function fixedIntegrationExample() {
    await init();
    
    const canvas = document.getElementById('myCanvas');
    const gl = get_webgl_context('myCanvas');
    const processor = new ImageProcessor(gl);
    
    // Apply grayscale
    processor.apply_grayscale();
    
    // JS modifies GL state
    renderUIOverlay(canvas);
    
    // FIX: Invalidate cache before next WASM call
    processor.invalidate_cache();
    
    // Now this works correctly!
    processor.apply_grayscale();
    console.log('✓ With cache invalidation - WORKS!');
}

/**
 * Example 5: React Component Integration
 */
class ImageProcessorComponent extends React.Component {
    constructor(props) {
        super(props);
        this.canvasRef = React.createRef();
        this.processor = null;
    }
    
    async componentDidMount() {
        await init();
        const gl = get_webgl_context(this.canvasRef.current.id);
        this.processor = new ImageProcessor(gl);
    }
    
    applyFilter = (filterName) => {
        try {
            switch(filterName) {
                case 'grayscale':
                    this.processor.apply_grayscale();
                    break;
                case 'blur':
                    this.processor.apply_blur(800, 600);
                    break;
                case 'invert':
                    this.processor.apply_invert();
                    break;
            }
        } catch (err) {
            console.error('Filter error:', err);
        }
    }
    
    // BUG SCENARIO: If React renders other canvas content between filter calls,
    // the WASM cache becomes stale!
    
    render() {
        return (
            <div>
                <canvas ref={this.canvasRef} id="processingCanvas" />
                <button onClick={() => this.applyFilter('grayscale')}>Grayscale</button>
                <button onClick={() => this.applyFilter('blur')}>Blur</button>
                <button onClick={() => this.applyFilter('invert')}>Invert</button>
            </div>
        );
    }
}

/**
 * Helper function that simulates JS modifying GL state
 * This is what causes the cache to become stale
 */
function renderUIOverlay(canvas) {
    // Get the WebGL context (the same one WASM is using)
    const gl = canvas.getContext('webgl');
    
    // Create a simple UI shader that JS might use
    const vertexShader = gl.createShader(gl.VERTEX_SHADER);
    gl.shaderSource(vertexShader, `
        attribute vec2 position;
        void main() { gl_Position = vec4(position, 0.0, 1.0); }
    `);
    gl.compileShader(vertexShader);
    
    const fragmentShader = gl.createShader(gl.FRAGMENT_SHADER);
    gl.shaderSource(fragmentShader, `
        precision mediump float;
        void main() { gl_FragColor = vec4(1.0, 1.0, 1.0, 0.5); }
    `);
    gl.compileShader(fragmentShader);
    
    const uiProgram = gl.createProgram();
    gl.attachShader(uiProgram, vertexShader);
    gl.attachShader(uiProgram, fragmentShader);
    gl.linkProgram(uiProgram);
    
    // BUG TRIGGER: Use this different shader program
    gl.useProgram(uiProgram);
    
    // This modified the GL context!
    // WASM's cached state is now invalid
}

/**
 * Example 6: Multi-Canvas Application (Bug Amplified)
 */
async function multiCanvasExample() {
    await init();
    
    // Multiple canvases sharing operations
    const canvas1 = document.getElementById('canvas1');
    const canvas2 = document.getElementById('canvas2');
    
    const gl1 = get_webgl_context('canvas1');
    const gl2 = get_webgl_context('canvas2');
    
    const processor1 = new ImageProcessor(gl1);
    const processor2 = new ImageProcessor(gl2);
    
    // Process canvas 1
    processor1.apply_grayscale();
    
    // JS modifies canvas 1's GL state
    // Simulate a UI library that uses the same canvas/context
    const tempProgram = gl1.createProgram();
    gl1.useProgram(tempProgram);  // Changes GL state on canvas1
    
    // Process canvas 1 again - might use stale cache!
    processor1.apply_invert();  // BUG RISK!
}

/**
 * Production Workaround Pattern
 */
class SafeImageProcessor {
    constructor(canvas) {
        this.canvas = canvas;
        this.processor = null;
        this.jsModifiedState = false;
    }
    
    async init() {
        await init();
        const gl = get_webgl_context(this.canvas.id);
        this.processor = new ImageProcessor(gl);
    }
    
    // Wrap all filter methods to handle cache invalidation
    applyGrayscale() {
        if (this.jsModifiedState) {
            this.processor.invalidate_cache();
            this.jsModifiedState = false;
        }
        this.processor.apply_grayscale();
    }
    
    applyBlur(width, height) {
        if (this.jsModifiedState) {
            this.processor.invalidate_cache();
            this.jsModifiedState = false;
        }
        this.processor.apply_blur(width, height);
    }
    
    applyInvert() {
        if (this.jsModifiedState) {
            this.processor.invalidate_cache();
            this.jsModifiedState = false;
        }
        this.processor.apply_invert();
    }
    
    // Call this whenever JS modifies the canvas
    notifyJSModification() {
        this.jsModifiedState = true;
    }
}

// Usage:
// const safeProcessor = new SafeImageProcessor(canvas);
// await safeProcessor.init();
// 
// safeProcessor.applyGrayscale();  // Works
// renderUIOverlay(canvas);
// safeProcessor.notifyJSModification();  // Signal state change
// safeProcessor.applyGrayscale();  // Still works!

export {
    basicUsageExample,
    buggyIntegrationExample,
    animationLoopExample,
    fixedIntegrationExample,
    multiCanvasExample,
    SafeImageProcessor,
    ImageProcessorComponent
};
