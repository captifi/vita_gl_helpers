//! Shader Compilation
//!
//! This module provides safe wrappers for OpenGL shader objects with
//! automatic cleanup via RAII.
//!
//! # Example
//! ```ignore
//! use vita_gl_helpers::shader::{Shader, ShaderType};
//!
//! let vert = Shader::compile(r#"
//!     void main(float2 aPos, float4 out gl_Position : POSITION) {
//!         gl_Position = float4(aPos, 0.0, 1.0);
//!     }
//! "#, ShaderType::Vertex)?;
//!
//! // Shader is automatically deleted when dropped
//! ```

use gl::types::GLuint;

use crate::handle::{GpuResource, Handle};

/// Shader compilation errors.
#[derive(Debug, Clone)]
pub enum ShaderError {
    /// Failed to create shader object.
    NoShader,
    /// Shader compilation failed with the given error message.
    CompileError(String),
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderError::NoShader => write!(f, "Failed to create shader object"),
            ShaderError::CompileError(s) => write!(f, "Shader compilation failed:\n{}", s),
        }
    }
}

impl std::error::Error for ShaderError {}

/// Type of shader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ShaderType {
    /// Vertex shader.
    Vertex = gl::VERTEX_SHADER,
    /// Fragment shader.
    Fragment = gl::FRAGMENT_SHADER,
}

impl From<ShaderType> for gl::types::GLenum {
    fn from(t: ShaderType) -> Self {
        t as gl::types::GLenum
    }
}

/// Marker type for shader resources.
pub struct ShaderResource;

impl GpuResource for ShaderResource {
    type Id = GLuint;

    unsafe fn delete(id: Self::Id) {
        gl::DeleteShader(id);
    }

    fn is_null(id: Self::Id) -> bool {
        id == 0
    }

    fn resource_name() -> &'static str {
        "Shader"
    }
}

/// A compiled shader object with automatic cleanup.
///
/// When dropped, the shader is automatically deleted via `glDeleteShader`.
///
/// # Example
/// ```ignore
/// let shader = Shader::compile(source, ShaderType::Vertex)?;
/// // Use shader for program linking...
/// // shader is automatically deleted when it goes out of scope
/// ```
pub struct Shader {
    handle: Handle<ShaderResource>,
    shader_type: ShaderType,
}

impl Shader {
    /// Compile a shader from source code.
    ///
    /// # Arguments
    /// * `source` - The shader source code (Cg for vitaGL)
    /// * `shader_type` - The type of shader to compile
    ///
    /// # Returns
    /// The compiled shader, or an error if compilation failed.
    ///
    /// # Example
    /// ```ignore
    /// let vert = Shader::compile(r#"
    ///     void main(float2 aPos, float4 out gl_Position : POSITION) {
    ///         gl_Position = float4(aPos, 0.0, 1.0);
    ///     }
    /// "#, ShaderType::Vertex)?;
    /// ```
    pub fn compile(source: &str, shader_type: ShaderType) -> Result<Self, ShaderError> {
        #[cfg(feature = "logging")]
        log::debug!("Compiling {:?} shader:\n{}", shader_type, source);

        let id = unsafe { gl::CreateShader(shader_type.into()) };
        if id == 0 {
            return Err(ShaderError::NoShader);
        }

        unsafe {
            let source_len = source.len() as i32;
            gl::ShaderSource(id, 1, &(source.as_ptr() as _), &source_len);
            gl::CompileShader(id);

            let mut compiled = 0;
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut compiled);

            if compiled == 0 {
                let info_log = get_shader_info_log(id);
                gl::DeleteShader(id);
                return Err(ShaderError::CompileError(info_log));
            }
        }

        Ok(Shader {
            handle: unsafe { Handle::from_raw(id) },
            shader_type,
        })
    }

    /// Get the shader type.
    #[inline]
    pub fn shader_type(&self) -> ShaderType {
        self.shader_type
    }

    /// Get the raw OpenGL shader ID.
    ///
    /// Returns `None` if the shader is invalid.
    #[inline]
    pub fn id(&self) -> Option<GLuint> {
        self.handle.id()
    }

    /// Get the raw OpenGL shader ID, panicking if invalid.
    ///
    /// # Panics
    /// Panics if the shader is invalid.
    #[inline]
    pub fn id_unwrap(&self) -> GLuint {
        self.handle.id_unwrap()
    }

    /// Check if this shader is valid.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.handle.is_valid()
    }

    /// Take ownership of the raw ID, preventing automatic deletion.
    ///
    /// After calling this, you are responsible for deleting the shader.
    pub fn into_raw(self) -> Option<GLuint> {
        self.handle.into_raw()
    }

    /// Get the shader info log (useful for debugging).
    pub fn info_log(&self) -> String {
        self.handle
            .id()
            .map(|id| unsafe { get_shader_info_log(id) })
            .unwrap_or_default()
    }
}

impl std::fmt::Debug for Shader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Shader")
            .field("id", &self.handle.id())
            .field("type", &self.shader_type)
            .finish()
    }
}

/// Get the info log for a shader.
unsafe fn get_shader_info_log(id: GLuint) -> String {
    let mut info_len = 0;
    gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut info_len);

    if info_len <= 0 {
        return String::new();
    }

    let mut info_log = vec![0u8; info_len as usize];
    gl::GetShaderInfoLog(id, info_len, std::ptr::null_mut(), info_log.as_mut_ptr() as _);

    // Remove null terminator if present
    if let Some(0) = info_log.last() {
        info_log.pop();
    }

    String::from_utf8_lossy(&info_log).into_owned()
}

// ============================================================================
// Legacy API (deprecated, for backwards compatibility)
// ============================================================================

/// Load and compile a shader.
///
/// # Deprecated
/// Use [`Shader::compile`] instead.
#[deprecated(since = "0.2.0", note = "Use Shader::compile() instead")]
#[allow(deprecated)]
pub fn load_shader(source: &str, typ: gl::types::GLenum) -> Result<LegacyShader, ShaderError> {
    let shader_type = match typ {
        gl::VERTEX_SHADER => ShaderType::Vertex,
        gl::FRAGMENT_SHADER => ShaderType::Fragment,
        _ => return Err(ShaderError::NoShader),
    };

    let shader = Shader::compile(source, shader_type)?;
    let id = shader.into_raw().unwrap_or(0);
    Ok(LegacyShader(id))
}

/// Legacy shader wrapper (no RAII).
///
/// # Deprecated
/// Use [`Shader`] instead, which provides RAII cleanup.
#[deprecated(since = "0.2.0", note = "Use Shader instead, which provides RAII cleanup")]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct LegacyShader(pub GLuint);

#[allow(deprecated)]
impl LegacyShader {
    /// Delete the shader manually.
    ///
    /// # Safety
    /// Must only be called once, and the shader must not be used after.
    pub unsafe fn delete(&self) {
        gl::DeleteShader(self.0);
    }

    /// Check if this is a null shader.
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

#[allow(deprecated)]
impl From<LegacyShader> for GLuint {
    fn from(s: LegacyShader) -> Self {
        s.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_type_conversion() {
        assert_eq!(gl::types::GLenum::from(ShaderType::Vertex), gl::VERTEX_SHADER);
        assert_eq!(gl::types::GLenum::from(ShaderType::Fragment), gl::FRAGMENT_SHADER);
    }
}
