//! Shader Program Linking
//!
//! This module provides safe wrappers for OpenGL program objects with
//! automatic cleanup via RAII.
//!
//! # Example
//! ```ignore
//! use vita_gl_helpers::program::Program;
//! use vita_gl_helpers::shader::{Shader, ShaderType};
//!
//! let vert = Shader::compile(vert_source, ShaderType::Vertex)?;
//! let frag = Shader::compile(frag_source, ShaderType::Fragment)?;
//! let program = Program::link(&vert, &frag)?;
//!
//! program.use_program();
//! // Program is automatically deleted when dropped
//! ```

use std::ffi::CString;

use gl::types::GLuint;

use crate::attribute::{AttributeTable, MissingAttributes};
use crate::handle::{GpuResource, Handle};
use crate::shader::Shader;
use crate::uniforms::{MissingUniforms, UniformTable};

/// Program linking errors.
#[derive(Debug, Clone)]
pub enum ProgramError {
    /// Failed to create program object.
    NoProgram,
    /// Program linking failed with the given error message.
    LinkError(String),
}

impl std::fmt::Display for ProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProgramError::NoProgram => write!(f, "Failed to create program object"),
            ProgramError::LinkError(s) => write!(f, "Program linking failed:\n{}", s),
        }
    }
}

impl std::error::Error for ProgramError {}

/// Marker type for program resources.
pub struct ProgramResource;

impl GpuResource for ProgramResource {
    type Id = GLuint;

    unsafe fn delete(id: Self::Id) {
        gl::DeleteProgram(id);
    }

    fn is_null(id: Self::Id) -> bool {
        id == 0
    }

    fn resource_name() -> &'static str {
        "Program"
    }
}

/// A linked shader program with automatic cleanup.
///
/// When dropped, the program is automatically deleted via `glDeleteProgram`.
///
/// # Example
/// ```ignore
/// let program = Program::link(&vert_shader, &frag_shader)?;
/// program.use_program();
/// // program is automatically deleted when it goes out of scope
/// ```
pub struct Program(Handle<ProgramResource>);

impl Program {
    /// Link a program from vertex and fragment shaders.
    ///
    /// # Arguments
    /// * `vert` - The compiled vertex shader
    /// * `frag` - The compiled fragment shader
    ///
    /// # Returns
    /// The linked program, or an error if linking failed.
    ///
    /// # Example
    /// ```ignore
    /// let program = Program::link(&vert_shader, &frag_shader)?;
    /// ```
    pub fn link(vert: &Shader, frag: &Shader) -> Result<Self, ProgramError> {
        #[cfg(feature = "logging")]
        log::debug!("Linking program...");

        let id = unsafe { gl::CreateProgram() };
        if id == 0 {
            return Err(ProgramError::NoProgram);
        }

        unsafe {
            gl::AttachShader(id, vert.id_unwrap());
            gl::AttachShader(id, frag.id_unwrap());
            gl::LinkProgram(id);

            let mut linked = 0;
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut linked);

            if linked == 0 {
                let info_log = get_program_info_log(id);
                gl::DeleteProgram(id);
                return Err(ProgramError::LinkError(info_log));
            }
        }

        #[cfg(feature = "logging")]
        log::debug!("Program linked successfully (id={})", id);

        Ok(Program(unsafe { Handle::from_raw(id) }))
    }

    /// Create a program from a raw OpenGL ID. Takes ownership.
    ///
    /// # Safety
    /// The caller must ensure the ID is a valid program and that
    /// ownership is being transferred.
    pub unsafe fn from_raw(id: GLuint) -> Self {
        Program(Handle::from_raw(id))
    }

    /// Get the raw OpenGL program ID.
    ///
    /// Returns `None` if the program is invalid.
    #[inline]
    pub fn id(&self) -> Option<GLuint> {
        self.0.id()
    }

    /// Get the raw OpenGL program ID, panicking if invalid.
    ///
    /// # Panics
    /// Panics if the program is invalid.
    #[inline]
    pub fn id_unwrap(&self) -> GLuint {
        self.0.id_unwrap()
    }

    /// Check if this program is valid.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0.is_valid()
    }

    /// Take ownership of the raw ID, preventing automatic deletion.
    ///
    /// After calling this, you are responsible for deleting the program.
    pub fn into_raw(self) -> Option<GLuint> {
        self.0.into_raw()
    }

    /// Use this program for rendering.
    ///
    /// Equivalent to `glUseProgram(id)`.
    pub fn use_program(&self) {
        if let Some(id) = self.0.id() {
            unsafe {
                gl::UseProgram(id);
            }
        }
    }

    /// Alias for `use_program()` to match the old API.
    #[inline]
    pub fn use_me(&self) {
        self.use_program();
    }

    /// Get the location of a vertex attribute.
    ///
    /// # Arguments
    /// * `name` - The name of the attribute in the shader
    ///
    /// # Returns
    /// The attribute location, or -1 if not found.
    pub fn get_attrib_location(&self, name: &str) -> i32 {
        let id = match self.0.id() {
            Some(id) => id,
            None => return -1,
        };

        let c_name = match CString::new(name) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        unsafe { gl::GetAttribLocation(id, c_name.as_ptr()) }
    }

    /// Get the location of a uniform variable.
    ///
    /// # Arguments
    /// * `name` - The name of the uniform in the shader
    ///
    /// # Returns
    /// The uniform location, or -1 if not found.
    pub fn get_uniform_location(&self, name: &str) -> i32 {
        let id = match self.0.id() {
            Some(id) => id,
            None => return -1,
        };

        let c_name = match CString::new(name) {
            Ok(s) => s,
            Err(_) => return -1,
        };

        unsafe { gl::GetUniformLocation(id, c_name.as_ptr()) }
    }

    /// Get a uniform table from this program.
    ///
    /// This creates a struct containing all the uniform locations
    /// defined by the `uniform_table!` macro.
    ///
    /// # Example
    /// ```ignore
    /// uniform_table!(MyUniforms,
    ///     mvp: UniformMatrix4fv => "uMVP"
    /// );
    ///
    /// let uniforms: MyUniforms = program.get_uniform_table()?;
    /// ```
    pub fn get_uniform_table<T: UniformTable>(&self) -> Result<T, MissingUniforms> {
        T::with_locations_from(self)
    }

    /// Get an attribute table from this program.
    ///
    /// This creates a struct containing all the attribute locations
    /// defined by the `attribute_table!` macro.
    ///
    /// # Example
    /// ```ignore
    /// attribute_table!(MyAttributes,
    ///     pos => "aPos",
    ///     color => "aColor"
    /// );
    ///
    /// let attrs: MyAttributes = program.get_attribute_table()?;
    /// ```
    pub fn get_attribute_table<T: AttributeTable>(&self) -> Result<T, MissingAttributes> {
        T::with_locations_from(self)
    }

    /// Get the program info log (useful for debugging).
    pub fn info_log(&self) -> String {
        self.0
            .id()
            .map(|id| unsafe { get_program_info_log(id) })
            .unwrap_or_default()
    }
}

impl std::fmt::Debug for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Program").field(&self.0).finish()
    }
}

/// Get the info log for a program.
unsafe fn get_program_info_log(id: GLuint) -> String {
    let mut info_len = 0;
    gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut info_len);

    if info_len <= 0 {
        return String::new();
    }

    let mut info_log = vec![0u8; info_len as usize];
    gl::GetProgramInfoLog(id, info_len, std::ptr::null_mut(), info_log.as_mut_ptr() as _);

    // Remove null terminator if present
    if let Some(0) = info_log.last() {
        info_log.pop();
    }

    String::from_utf8_lossy(&info_log).into_owned()
}

// ============================================================================
// Legacy API (deprecated, for backwards compatibility)
// ============================================================================

/// Link a program from vertex and fragment shaders.
///
/// # Deprecated
/// Use [`Program::link`] instead.
#[deprecated(since = "0.2.0", note = "Use Program::link() instead")]
#[allow(deprecated)]
pub fn link_program(
    vert: crate::shader::LegacyShader,
    frag: crate::shader::LegacyShader,
) -> Result<LegacyProgram, ProgramError> {
    #[cfg(feature = "logging")]
    log::debug!("Linking program (legacy)...");

    let id = unsafe { gl::CreateProgram() };
    if id == 0 {
        return Err(ProgramError::NoProgram);
    }

    unsafe {
        gl::AttachShader(id, vert.0);
        gl::AttachShader(id, frag.0);
        gl::LinkProgram(id);

        let mut linked = 0;
        gl::GetProgramiv(id, gl::LINK_STATUS, &mut linked);

        if linked == 0 {
            let info_log = get_program_info_log(id);
            gl::DeleteProgram(id);
            return Err(ProgramError::LinkError(info_log));
        }
    }

    Ok(LegacyProgram(id))
}

/// Legacy program wrapper (no RAII).
///
/// # Deprecated
/// Use [`Program`] instead, which provides RAII cleanup.
#[deprecated(since = "0.2.0", note = "Use Program instead, which provides RAII cleanup")]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct LegacyProgram(pub GLuint);

#[allow(deprecated)]
impl LegacyProgram {
    /// Use this program.
    pub fn use_me(&self) {
        unsafe {
            gl::UseProgram(self.0);
        }
    }

    /// Delete the program manually.
    ///
    /// # Safety
    /// Must only be called once, and the program must not be used after.
    pub unsafe fn delete(&self) {
        gl::DeleteProgram(self.0);
    }

    /// Get attribute location.
    pub fn get_attrib_location(&self, attrib: &str) -> i32 {
        let c_name = CString::new(attrib).expect("Invalid attribute name");
        unsafe { gl::GetAttribLocation(self.0, c_name.as_ptr()) }
    }

    /// Get uniform location.
    pub fn get_uniform_location(&self, uniform: &str) -> i32 {
        let c_name = CString::new(uniform).expect("Invalid uniform name");
        unsafe { gl::GetUniformLocation(self.0, c_name.as_ptr()) }
    }

    /// Get a uniform table.
    pub fn get_uniform_table<T: UniformTable>(&self) -> Result<T, MissingUniforms> {
        // Create a temporary Program wrapper without RAII
        let program = Program(unsafe { Handle::from_raw(self.0) });
        let result = T::with_locations_from(&program);
        // Prevent the temporary from deleting the program
        program.into_raw();
        result
    }

    /// Get an attribute table.
    pub fn get_attribute_table<T: AttributeTable>(&self) -> Result<T, MissingAttributes> {
        // Create a temporary Program wrapper without RAII
        let program = Program(unsafe { Handle::from_raw(self.0) });
        let result = T::with_locations_from(&program);
        // Prevent the temporary from deleting the program
        program.into_raw();
        result
    }
}

#[allow(deprecated)]
impl From<LegacyProgram> for GLuint {
    fn from(p: LegacyProgram) -> Self {
        p.0
    }
}
