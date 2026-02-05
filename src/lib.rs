//! # vitaGL Helpers
//!
//! Safe, ergonomic helpers for vitaGL development on PlayStation Vita.
//!
//! This library provides Rust wrappers around vitaGL/OpenGL functionality with:
//! - **RAII resource management** - GPU resources are automatically cleaned up
//! - **Type-safe APIs** - Prevents common OpenGL mistakes at compile time
//! - **Declarative macros** - Reduce boilerplate for shader variables
//! - **Builder patterns** - Fluent APIs for configuration
//!
//! ## Quick Start
//!
//! ```ignore
//! use vita_gl_helpers::prelude::*;
//!
//! fn main() {
//!     // Initialize vitaGL with builder pattern
//!     let ctx = VitaGlBuilder::new()
//!         .msaa(4)
//!         .build()
//!         .expect("Failed to initialize vitaGL");
//!
//!     // Compile shaders (automatically deleted when dropped)
//!     let vert = Shader::compile(VERT_SOURCE, ShaderType::Vertex).unwrap();
//!     let frag = Shader::compile(FRAG_SOURCE, ShaderType::Fragment).unwrap();
//!
//!     // Link program (automatically deleted when dropped)
//!     let program = Program::link(&vert, &frag).unwrap();
//!
//!     // Create buffer (automatically deleted when dropped)
//!     let buffer = Buffer::new();
//!     buffer.data(gl::ARRAY_BUFFER, &vertices, gl::STATIC_DRAW);
//!
//!     // Main loop
//!     loop {
//!         unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }
//!         program.use_program();
//!         // ... draw ...
//!         ctx.swap_buffers();
//!     }
//! }
//! ```
//!
//! ## Modules
//!
//! - [`buffer`] - GPU buffer management with RAII
//! - [`shader`] - Shader compilation with RAII
//! - [`program`] - Shader program linking with RAII
//! - [`texture`] - Texture management with RAII
//! - [`attribute`] - Vertex attribute configuration
//! - [`uniforms`] - Uniform variable management
//! - [`draw`] - Draw call helpers
//! - [`errors`] - GL error handling
//! - [`handle`] - Generic RAII handle wrapper

use std::ffi::CString;

// ============================================================================
// Module declarations
// ============================================================================

pub mod attribute;
pub mod buffer;
pub mod draw;
pub mod errors;
pub mod handle;
pub mod program;
pub mod shader;
pub mod texture;
pub mod uniforms;

// ============================================================================
// Prelude - common imports for convenience
// ============================================================================

/// Prelude module for convenient imports.
///
/// ```ignore
/// use vita_gl_helpers::prelude::*;
/// ```
pub mod prelude {
    pub use crate::attribute::{Attribute, AttributeFormat, AttributeSize, AttributeTable, AttributeType};
    pub use crate::buffer::Buffer;
    pub use crate::draw::{draw_arrays, Elements, Mode};
    pub use crate::errors::{eprintln_errors, get_error, GlError};
    pub use crate::program::Program;
    pub use crate::shader::{Shader, ShaderType};
    pub use crate::texture::Texture;
    pub use crate::uniforms::UniformTable;
    pub use crate::{swap_buffers, VitaGlBuilder, VitaGlContext};

    // Re-export macros
    pub use crate::{attribute_table, uniform_table};
}

// ============================================================================
// FFI declarations for vitaGL
// ============================================================================

extern "C" {
    fn vglSwapBuffers(has_commondialog: u8);
    fn vglSetupRuntimeShaderCompiler(
        opt_level: i32,
        use_fastmath: i32,
        use_fastprecision: i32,
        use_fastint: i32,
    );
    fn vglInitExtended(
        legacy_pool_size: i32,
        width: i32,
        height: i32,
        ram_threshold: i32,
        msaa: u32,
    ) -> u8;
    fn vglGetProcAddress(name: *const u8) -> *const u8;
}

// ============================================================================
// Initialization - Builder Pattern
// ============================================================================

/// Error during vitaGL initialization.
#[derive(Debug, Clone)]
pub enum InitError {
    /// vglInitExtended returned failure.
    InitFailed,
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::InitFailed => write!(f, "vitaGL initialization failed"),
        }
    }
}

impl std::error::Error for InitError {}

/// Builder for vitaGL initialization.
///
/// Provides a fluent API for configuring vitaGL before initialization.
///
/// # Example
/// ```ignore
/// let ctx = VitaGlBuilder::new()
///     .resolution(960, 544)
///     .msaa(4)
///     .shader_optimization(2)
///     .build()?;
/// ```
#[derive(Debug, Clone)]
pub struct VitaGlBuilder {
    legacy_pool_size: i32,
    width: i32,
    height: i32,
    ram_threshold: i32,
    msaa: u32,
    shader_opt_level: i32,
    use_fastmath: bool,
    use_fastprecision: bool,
    use_fastint: bool,
}

impl Default for VitaGlBuilder {
    fn default() -> Self {
        VitaGlBuilder {
            legacy_pool_size: 0,
            width: 960,
            height: 544,
            ram_threshold: 65 * 1024 * 1024, // 65 MB
            msaa: 0,
            shader_opt_level: 2,
            use_fastmath: true,
            use_fastprecision: false,
            use_fastint: true,
        }
    }
}

impl VitaGlBuilder {
    /// Create a new builder with default settings.
    ///
    /// Default settings:
    /// - Resolution: 960x544 (Vita native)
    /// - MSAA: 0 (disabled)
    /// - RAM threshold: 65 MB
    /// - Shader optimization: level 2
    /// - Fast math: enabled
    /// - Fast int: enabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the display resolution.
    ///
    /// Default: 960x544 (Vita native resolution)
    pub fn resolution(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the MSAA (Multi-Sample Anti-Aliasing) level.
    ///
    /// Common values: 0 (disabled), 2, 4
    /// Default: 0
    pub fn msaa(mut self, samples: u32) -> Self {
        self.msaa = samples;
        self
    }

    /// Set the RAM threshold for texture memory.
    ///
    /// Default: 65 MB
    pub fn ram_threshold(mut self, bytes: i32) -> Self {
        self.ram_threshold = bytes;
        self
    }

    /// Set the legacy pool size.
    ///
    /// Default: 0
    pub fn legacy_pool_size(mut self, size: i32) -> Self {
        self.legacy_pool_size = size;
        self
    }

    /// Set the shader compiler optimization level.
    ///
    /// Levels: 0-3 (higher = more optimization)
    /// Default: 2
    pub fn shader_optimization(mut self, level: i32) -> Self {
        self.shader_opt_level = level;
        self
    }

    /// Enable or disable fast math in shader compilation.
    ///
    /// Default: true
    pub fn fast_math(mut self, enabled: bool) -> Self {
        self.use_fastmath = enabled;
        self
    }

    /// Enable or disable fast precision in shader compilation.
    ///
    /// Default: false
    pub fn fast_precision(mut self, enabled: bool) -> Self {
        self.use_fastprecision = enabled;
        self
    }

    /// Enable or disable fast int in shader compilation.
    ///
    /// Default: true
    pub fn fast_int(mut self, enabled: bool) -> Self {
        self.use_fastint = enabled;
        self
    }

    /// Initialize vitaGL with the configured settings.
    ///
    /// # Returns
    /// A [`VitaGlContext`] on success, or an [`InitError`] on failure.
    ///
    /// # Example
    /// ```ignore
    /// let ctx = VitaGlBuilder::new()
    ///     .msaa(4)
    ///     .build()?;
    /// ```
    pub fn build(self) -> Result<VitaGlContext, InitError> {
        #[cfg(feature = "logging")]
        log::info!(
            "Initializing vitaGL: {}x{}, MSAA={}, RAM={}MB",
            self.width,
            self.height,
            self.msaa,
            self.ram_threshold / (1024 * 1024)
        );

        unsafe {
            vglSetupRuntimeShaderCompiler(
                self.shader_opt_level,
                self.use_fastmath as i32,
                self.use_fastprecision as i32,
                self.use_fastint as i32,
            );

            let result = vglInitExtended(
                self.legacy_pool_size,
                self.width,
                self.height,
                self.ram_threshold,
                self.msaa,
            );

            if result == 0 {
                return Err(InitError::InitFailed);
            }
        }

        // Load GL function pointers
        gl::load_with(|name| {
            let name = CString::new(name).unwrap();
            unsafe { vglGetProcAddress(name.as_ptr() as _) as _ }
        });

        #[cfg(feature = "logging")]
        log::info!("vitaGL initialized successfully");

        Ok(VitaGlContext {
            width: self.width,
            height: self.height,
        })
    }
}

/// A vitaGL context representing an initialized graphics system.
///
/// Created by [`VitaGlBuilder::build`].
#[derive(Debug)]
pub struct VitaGlContext {
    width: i32,
    height: i32,
}

impl VitaGlContext {
    /// Swap the front and back buffers.
    ///
    /// Call this at the end of each frame to display the rendered content.
    pub fn swap_buffers(&self) {
        unsafe {
            vglSwapBuffers(0);
        }
    }

    /// Swap buffers with common dialog support.
    ///
    /// Use this if your application uses Vita system dialogs.
    pub fn swap_buffers_with_dialog(&self) {
        unsafe {
            vglSwapBuffers(1);
        }
    }

    /// Get the display width.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Get the display height.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Get the display resolution as (width, height).
    pub fn resolution(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    /// Get the aspect ratio (width / height).
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

// ============================================================================
// Legacy API (deprecated, for backwards compatibility)
// ============================================================================

/// Swap buffers (standalone function).
///
/// Prefer using [`VitaGlContext::swap_buffers`] instead.
pub fn swap_buffers() {
    unsafe {
        vglSwapBuffers(0);
    }
}

/// Runtime shader compiler settings.
///
/// # Deprecated
/// Use [`VitaGlBuilder`] instead.
#[deprecated(since = "0.2.0", note = "Use VitaGlBuilder instead")]
pub struct RuntimeShaderCompilerSettings {
    pub opt_level: i32,
    pub use_fastmath: i32,
    pub use_fastprecision: i32,
    pub use_fastint: i32,
}

#[allow(deprecated)]
impl Default for RuntimeShaderCompilerSettings {
    fn default() -> Self {
        RuntimeShaderCompilerSettings {
            opt_level: 2,
            use_fastmath: 1,
            use_fastprecision: 0,
            use_fastint: 1,
        }
    }
}

/// VGL initialization settings.
///
/// # Deprecated
/// Use [`VitaGlBuilder`] instead.
#[deprecated(since = "0.2.0", note = "Use VitaGlBuilder instead")]
pub struct VglInitSettings {
    pub legacy_pool_size: i32,
    pub ram_threshold: i32,
    pub msaa: u32,
}

#[allow(deprecated)]
impl Default for VglInitSettings {
    fn default() -> Self {
        VglInitSettings {
            legacy_pool_size: 0,
            ram_threshold: 65 * 1024 * 1024,
            msaa: 0,
        }
    }
}

/// Initialize vitaGL with extended settings.
///
/// # Deprecated
/// Use [`VitaGlBuilder`] instead.
#[deprecated(since = "0.2.0", note = "Use VitaGlBuilder::new().build() instead")]
#[allow(deprecated)]
pub fn initialise_extended(rscs: RuntimeShaderCompilerSettings, vis: VglInitSettings) {
    unsafe {
        vglSetupRuntimeShaderCompiler(
            rscs.opt_level,
            rscs.use_fastmath,
            rscs.use_fastprecision,
            rscs.use_fastint,
        );
        vglInitExtended(vis.legacy_pool_size, 960, 544, vis.ram_threshold, vis.msaa);
    }
    gl::load_with(|name| {
        let name = CString::new(name).unwrap();
        unsafe { vglGetProcAddress(name.as_ptr() as _) as _ }
    });
}

/// Initialize vitaGL with default settings.
///
/// # Deprecated
/// Use [`VitaGlBuilder::new().build()`] instead.
#[deprecated(since = "0.2.0", note = "Use VitaGlBuilder::new().build() instead")]
#[allow(deprecated)]
pub fn initialise_default() {
    initialise_extended(Default::default(), Default::default());
}
