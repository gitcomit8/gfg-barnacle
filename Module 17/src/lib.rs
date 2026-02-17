use wasm_bindgen::prelude::*;
use web_sys::{console, WebGlRenderingContext, WebGlProgram, WebGlShader, WebGlBuffer};
use js_sys::Float32Array;
use std::cell::RefCell;
use std::rc::Rc;

/// The Shadowed Canvas Context Module
/// 
/// This module demonstrates a subtle and difficult-to-debug bug in WebGL image processing:
/// The Rust side caches GL context state (shader programs, texture bindings) for performance,
/// but the JavaScript side can modify the GL state between WASM calls, causing the cache
/// to become stale and produce non-deterministic rendering glitches.
///
/// ## The Bug Explained:
/// 1. WASM caches the "current" shader program ID and texture bindings
/// 2. JS-side UI updates may change the GL context state (for rendering UI overlays, etc.)
/// 3. WASM assumes its cached state is still valid and skips rebinding
/// 4. GL renders with wrong shaders/textures, causing visual glitches
/// 5. Bug only appears after specific JS UI updates, making it non-deterministic

/// Enum representing which shader program is active
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ProgramType {
    Grayscale,
    Blur,
    Invert,
}

/// Cached WebGL state - THIS IS THE SOURCE OF THE BUG
/// The cache assumes the GL state hasn't been modified externally
#[derive(Clone)]
struct GLStateCache {
    current_program: Option<ProgramType>,  // Cached program type
    current_texture: Option<u32>,          // Cached texture binding
    current_buffer: Option<u32>,           // Cached buffer binding
    invert_mode: bool,                     // Whether to invert colors
}

impl GLStateCache {
    fn new() -> Self {
        GLStateCache {
            current_program: None,
            current_texture: None,
            current_buffer: None,
            invert_mode: false,
        }
    }
}

#[wasm_bindgen]
pub struct ImageProcessor {
    context: WebGlRenderingContext,
    state_cache: Rc<RefCell<GLStateCache>>,
    grayscale_program: Option<WebGlProgram>,
    blur_program: Option<WebGlProgram>,
    invert_program: Option<WebGlProgram>,
    vertex_buffer: Option<WebGlBuffer>,
    frame_count: u32,
}

#[wasm_bindgen]
impl ImageProcessor {
    /// Creates a new ImageProcessor with the given WebGL context
    /// BUG: Initializes the state cache, which will become stale
    #[wasm_bindgen(constructor)]
    pub fn new(context: WebGlRenderingContext) -> Result<ImageProcessor, JsValue> {
        console::log_1(&JsValue::from_str("Initializing ImageProcessor with state caching..."));
        
        let state_cache = Rc::new(RefCell::new(GLStateCache::new()));
        
        let mut processor = ImageProcessor {
            context,
            state_cache,
            grayscale_program: None,
            blur_program: None,
            invert_program: None,
            vertex_buffer: None,
            frame_count: 0,
        };
        
        processor.initialize_shaders()?;
        processor.initialize_buffers()?;
        
        Ok(processor)
    }
    
    /// Initialize shader programs
    /// BUG: Stores references to programs but doesn't handle external GL state changes
    fn initialize_shaders(&mut self) -> Result<(), JsValue> {
        // Vertex shader used by all effects
        let vertex_shader_source = r#"
            attribute vec2 a_position;
            attribute vec2 a_texCoord;
            varying vec2 v_texCoord;
            void main() {
                gl_Position = vec4(a_position, 0.0, 1.0);
                v_texCoord = a_texCoord;
            }
        "#;
        
        // Grayscale fragment shader
        let grayscale_fragment_source = r#"
            precision mediump float;
            varying vec2 v_texCoord;
            uniform sampler2D u_image;
            void main() {
                vec4 color = texture2D(u_image, v_texCoord);
                float gray = dot(color.rgb, vec3(0.299, 0.587, 0.114));
                gl_FragColor = vec4(vec3(gray), color.a);
            }
        "#;
        
        // Blur fragment shader
        let blur_fragment_source = r#"
            precision mediump float;
            varying vec2 v_texCoord;
            uniform sampler2D u_image;
            uniform vec2 u_resolution;
            void main() {
                vec2 pixel = 1.0 / u_resolution;
                vec4 color = vec4(0.0);
                color += texture2D(u_image, v_texCoord + vec2(-pixel.x, -pixel.y)) * 0.0625;
                color += texture2D(u_image, v_texCoord + vec2(0.0, -pixel.y)) * 0.125;
                color += texture2D(u_image, v_texCoord + vec2(pixel.x, -pixel.y)) * 0.0625;
                color += texture2D(u_image, v_texCoord + vec2(-pixel.x, 0.0)) * 0.125;
                color += texture2D(u_image, v_texCoord) * 0.25;
                color += texture2D(u_image, v_texCoord + vec2(pixel.x, 0.0)) * 0.125;
                color += texture2D(u_image, v_texCoord + vec2(-pixel.x, pixel.y)) * 0.0625;
                color += texture2D(u_image, v_texCoord + vec2(0.0, pixel.y)) * 0.125;
                color += texture2D(u_image, v_texCoord + vec2(pixel.x, pixel.y)) * 0.0625;
                gl_FragColor = color;
            }
        "#;
        
        // Invert fragment shader
        let invert_fragment_source = r#"
            precision mediump float;
            varying vec2 v_texCoord;
            uniform sampler2D u_image;
            void main() {
                vec4 color = texture2D(u_image, v_texCoord);
                gl_FragColor = vec4(1.0 - color.rgb, color.a);
            }
        "#;
        
        self.grayscale_program = Some(self.create_program(vertex_shader_source, grayscale_fragment_source)?);
        self.blur_program = Some(self.create_program(vertex_shader_source, blur_fragment_source)?);
        self.invert_program = Some(self.create_program(vertex_shader_source, invert_fragment_source)?);
        
        console::log_1(&JsValue::from_str("Shaders initialized"));
        Ok(())
    }
    
    /// Create a WebGL program from vertex and fragment shader sources
    fn create_program(&self, vertex_source: &str, fragment_source: &str) -> Result<WebGlProgram, JsValue> {
        let vertex_shader = self.compile_shader(WebGlRenderingContext::VERTEX_SHADER, vertex_source)?;
        let fragment_shader = self.compile_shader(WebGlRenderingContext::FRAGMENT_SHADER, fragment_source)?;
        
        let program = self.context.create_program()
            .ok_or_else(|| JsValue::from_str("Failed to create program"))?;
        
        self.context.attach_shader(&program, &vertex_shader);
        self.context.attach_shader(&program, &fragment_shader);
        self.context.link_program(&program);
        
        if !self.context.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS).as_bool().unwrap_or(false) {
            let info = self.context.get_program_info_log(&program)
                .unwrap_or_else(|| String::from("Unknown error"));
            return Err(JsValue::from_str(&format!("Program link error: {}", info)));
        }
        
        Ok(program)
    }
    
    /// Compile a shader
    fn compile_shader(&self, shader_type: u32, source: &str) -> Result<WebGlShader, JsValue> {
        let shader = self.context.create_shader(shader_type)
            .ok_or_else(|| JsValue::from_str("Failed to create shader"))?;
        
        self.context.shader_source(&shader, source);
        self.context.compile_shader(&shader);
        
        if !self.context.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS).as_bool().unwrap_or(false) {
            let info = self.context.get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unknown error"));
            return Err(JsValue::from_str(&format!("Shader compile error: {}", info)));
        }
        
        Ok(shader)
    }
    
    /// Initialize vertex buffer for rendering
    fn initialize_buffers(&mut self) -> Result<(), JsValue> {
        let vertices: [f32; 16] = [
            -1.0, -1.0,  0.0, 0.0,
             1.0, -1.0,  1.0, 0.0,
            -1.0,  1.0,  0.0, 1.0,
             1.0,  1.0,  1.0, 1.0,
        ];
        
        let buffer = self.context.create_buffer()
            .ok_or_else(|| JsValue::from_str("Failed to create buffer"))?;
        
        self.context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        
        // BUG: We're caching this buffer binding
        // Note: In a real implementation, we'd store a proper buffer ID
        // For this demo, we just mark that *a* buffer is bound
        self.state_cache.borrow_mut().current_buffer = Some(1);
        
        unsafe {
            let vertex_array = Float32Array::view(&vertices);
            self.context.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vertex_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }
        
        self.vertex_buffer = Some(buffer);
        Ok(())
    }
    
    /// Apply grayscale effect to the canvas
    /// BUG: Uses cached GL state that may be stale
    #[wasm_bindgen]
    pub fn apply_grayscale(&mut self) -> Result<(), JsValue> {
        self.frame_count += 1;
        let program = self.grayscale_program.as_ref()
            .ok_or_else(|| JsValue::from_str("Grayscale program not initialized"))?;
        
        // BUG: Check cache instead of always binding
        // If JS has changed the current program, we won't detect it!
        let program_type = ProgramType::Grayscale;
        if self.state_cache.borrow().current_program != Some(program_type) {
            console::log_1(&JsValue::from_str("Cache miss: binding grayscale program"));
            self.context.use_program(Some(program));
            self.state_cache.borrow_mut().current_program = Some(program_type);
        } else {
            // BUG: Assumes program is still active - it might not be!
            console::log_1(&JsValue::from_str("Cache hit: skipping program bind (POTENTIAL BUG!)"));
        }
        
        self.render_effect(program)?;
        Ok(())
    }
    
    /// Apply blur effect to the canvas
    /// BUG: Uses cached GL state that may be stale
    #[wasm_bindgen]
    pub fn apply_blur(&mut self, width: f32, height: f32) -> Result<(), JsValue> {
        self.frame_count += 1;
        let program = self.blur_program.as_ref()
            .ok_or_else(|| JsValue::from_str("Blur program not initialized"))?;
        
        // BUG: Same caching issue
        let program_type = ProgramType::Blur;
        if self.state_cache.borrow().current_program != Some(program_type) {
            console::log_1(&JsValue::from_str("Cache miss: binding blur program"));
            self.context.use_program(Some(program));
            self.state_cache.borrow_mut().current_program = Some(program_type);
        } else {
            console::log_1(&JsValue::from_str("Cache hit: skipping program bind (POTENTIAL BUG!)"));
        }
        
        // Set resolution uniform
        let resolution_location = self.context.get_uniform_location(program, "u_resolution");
        self.context.uniform2f(resolution_location.as_ref(), width, height);
        
        self.render_effect(program)?;
        Ok(())
    }
    
    /// Apply color inversion effect
    /// BUG: Most susceptible to the cache bug - often called after JS rendering
    #[wasm_bindgen]
    pub fn apply_invert(&mut self) -> Result<(), JsValue> {
        self.frame_count += 1;
        let program = self.invert_program.as_ref()
            .ok_or_else(|| JsValue::from_str("Invert program not initialized"))?;
        
        // BUG: Cache check - this is where the bug manifests most often
        let program_type = ProgramType::Invert;
        let cache = self.state_cache.borrow();
        
        // CRITICAL BUG: If JS modified the GL state for UI rendering,
        // our cache will be wrong but we'll think it's correct!
        if cache.current_program != Some(program_type) {
            drop(cache); // Release borrow
            console::log_1(&JsValue::from_str("Cache miss: binding invert program"));
            self.context.use_program(Some(program));
            self.state_cache.borrow_mut().current_program = Some(program_type);
        } else {
            drop(cache); // Release borrow
            // BUG: We think the program is active, but JS may have changed it!
            console::log_1(&JsValue::from_str("Cache hit: skipping bind (HIGH BUG RISK!)"));
        }
        
        self.render_effect(program)?;
        Ok(())
    }
    
    /// Common rendering logic
    /// BUG: Assumes all cached state is valid
    fn render_effect(&self, program: &WebGlProgram) -> Result<(), JsValue> {
        // Setup attributes
        let position_location = self.context.get_attrib_location(program, "a_position") as u32;
        let texcoord_location = self.context.get_attrib_location(program, "a_texCoord") as u32;
        
        // BUG: We assume the vertex buffer is still bound from initialization
        // If JS has bound a different buffer, this will use the wrong data!
        self.context.enable_vertex_attrib_array(position_location);
        self.context.vertex_attrib_pointer_with_i32(
            position_location,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            16,
            0,
        );
        
        self.context.enable_vertex_attrib_array(texcoord_location);
        self.context.vertex_attrib_pointer_with_i32(
            texcoord_location,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            16,
            8,
        );
        
        // Draw
        self.context.draw_arrays(WebGlRenderingContext::TRIANGLE_STRIP, 0, 4);
        
        Ok(())
    }
    
    /// Clear the canvas
    #[wasm_bindgen]
    pub fn clear(&self) {
        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    }
    
    /// Get frame count (for debugging)
    #[wasm_bindgen]
    pub fn get_frame_count(&self) -> u32 {
        self.frame_count
    }
    
    /// Force cache invalidation (this would be the fix, but it's not called automatically)
    #[wasm_bindgen]
    pub fn invalidate_cache(&mut self) {
        console::log_1(&JsValue::from_str("Manually invalidating cache..."));
        let mut cache = self.state_cache.borrow_mut();
        cache.current_program = None;
        cache.current_texture = None;
        cache.current_buffer = None;
    }
}

/// Utility function to create a WebGL context from a canvas
#[wasm_bindgen]
pub fn get_webgl_context(canvas_id: &str) -> Result<WebGlRenderingContext, JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let canvas = document.get_element_by_id(canvas_id)
        .ok_or("Canvas not found")?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    
    let context = canvas
        .get_context("webgl")?
        .ok_or("Failed to get WebGL context")?
        .dyn_into::<WebGlRenderingContext>()?;
    
    Ok(context)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: These tests require a browser environment with WebGL support
    // Run with: wasm-pack test --headless --firefox
    
    #[test]
    fn test_state_cache_creation() {
        let cache = GLStateCache::new();
        assert_eq!(cache.current_program, None);
        assert_eq!(cache.current_texture, None);
    }
}
