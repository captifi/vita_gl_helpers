//! Uniform Variable Management
//!
//! This module provides types for setting shader uniform variables.
//!
//! # Example
//! ```ignore
//! use vita_gl_helpers::uniform_table;
//! use vita_gl_helpers::uniforms::*;
//!
//! // Define a uniform table for your shader
//! uniform_table!(MyUniforms,
//!     mvp: UniformMatrix4fv => "uMVP",
//!     color: Uniform4fv => "uColor",
//!     time: Uniform1fv => "uTime"
//! );
//!
//! // Get uniform locations from a program
//! let uniforms = program.get_uniform_table::<MyUniforms>()?;
//!
//! // Set uniform values
//! uniforms.mvp.set(matrix_data, false);
//! uniforms.color.set([1.0, 0.0, 0.0, 1.0]);
//! uniforms.time.set(elapsed_time);
//! ```

use derive_more::From;

use crate::program::Program;

// ============================================================================
// Uniform type definitions via macro
// ============================================================================

/// Internal macro to define uniform types.
macro_rules! uniform_def {
    // Vector uniforms (non-matrix)
    ($name:ident, $accept:ty, $doc:expr) => {
        #[doc = $doc]
        #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
        pub struct $name(pub gl::types::GLint);

        impl $name {
            /// Create a new uniform location.
            #[inline]
            pub const fn new(location: gl::types::GLint) -> Self {
                $name(location)
            }

            /// Get the raw uniform location.
            #[inline]
            pub const fn location(&self) -> gl::types::GLint {
                self.0
            }

            /// Set the uniform value.
            pub fn set(&self, to: $accept) {
                self.set_multi(&[to])
            }

            /// Set multiple uniform values (for arrays).
            pub fn set_multi(&self, to: &[$accept]) {
                self.set_subrange(0, to)
            }

            /// Set uniform values starting at an offset (for arrays).
            pub fn set_subrange(&self, offset: usize, to: &[$accept]) {
                unsafe {
                    gl::$name(self.0 + offset as i32, to.len() as _, to.as_ptr() as _);
                }
            }
        }
    };

    // Matrix uniforms
    ($name:ident, $accept:ty, mat, $doc:expr) => {
        #[doc = $doc]
        #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
        pub struct $name(pub gl::types::GLint);

        impl $name {
            /// Create a new uniform location.
            #[inline]
            pub const fn new(location: gl::types::GLint) -> Self {
                $name(location)
            }

            /// Get the raw uniform location.
            #[inline]
            pub const fn location(&self) -> gl::types::GLint {
                self.0
            }

            /// Set the matrix uniform value.
            ///
            /// # Arguments
            /// * `to` - The matrix data (column-major order)
            /// * `transpose` - Whether to transpose the matrix
            pub fn set(&self, to: $accept, transpose: bool) {
                self.set_multi(&[to], transpose)
            }

            /// Set multiple matrix uniform values (for arrays).
            pub fn set_multi(&self, to: &[$accept], transpose: bool) {
                self.set_subrange(0, to, transpose)
            }

            /// Set matrix uniform values starting at an offset (for arrays).
            pub fn set_subrange(&self, offset: usize, to: &[$accept], transpose: bool) {
                unsafe {
                    gl::$name(
                        self.0 + offset as i32,
                        to.len() as _,
                        if transpose { gl::TRUE } else { gl::FALSE },
                        to.as_ptr() as _,
                    );
                }
            }
        }
    };
}

// ============================================================================
// Scalar and vector uniforms
// ============================================================================

uniform_def!(Uniform1fv, f32, "A single float uniform (`float` in GLSL).");
uniform_def!(Uniform2fv, [f32; 2], "A 2-component float vector uniform (`vec2` in GLSL).");
uniform_def!(Uniform3fv, [f32; 3], "A 3-component float vector uniform (`vec3` in GLSL).");
uniform_def!(Uniform4fv, [f32; 4], "A 4-component float vector uniform (`vec4` in GLSL).");

uniform_def!(Uniform1iv, i32, "A single integer uniform (`int` in GLSL).");
uniform_def!(Uniform2iv, [i32; 2], "A 2-component integer vector uniform (`ivec2` in GLSL).");
uniform_def!(Uniform3iv, [i32; 3], "A 3-component integer vector uniform (`ivec3` in GLSL).");
uniform_def!(Uniform4iv, [i32; 4], "A 4-component integer vector uniform (`ivec4` in GLSL).");

// ============================================================================
// Matrix uniforms
// ============================================================================

uniform_def!(UniformMatrix2fv, [f32; 4], mat, "A 2x2 matrix uniform (`mat2` in GLSL).");
uniform_def!(UniformMatrix3fv, [f32; 9], mat, "A 3x3 matrix uniform (`mat3` in GLSL).");
uniform_def!(UniformMatrix4fv, [f32; 16], mat, "A 4x4 matrix uniform (`mat4` in GLSL).");

// Note: Non-square matrix types are not available in OpenGL ES 2.0 / vitaGL
// uniform_def!(UniformMatrix2x3fv, [f32; 6], mat);
// uniform_def!(UniformMatrix3x2fv, [f32; 6], mat);
// uniform_def!(UniformMatrix2x4fv, [f32; 8], mat);
// uniform_def!(UniformMatrix4x2fv, [f32; 8], mat);
// uniform_def!(UniformMatrix3x4fv, [f32; 12], mat);
// uniform_def!(UniformMatrix4x3fv, [f32; 12], mat);

// ============================================================================
// Sampler uniform (for textures)
// ============================================================================

/// A sampler uniform for binding textures.
///
/// This is essentially an integer uniform used to specify which texture unit
/// a sampler should read from.
///
/// # Example
/// ```ignore
/// uniform_table!(MyUniforms,
///     tex: Sampler2D => "uTexture"
/// );
///
/// // Bind texture to unit 0
/// texture.bind_to_unit(0, gl::TEXTURE_2D);
/// uniforms.tex.set(0); // Tell shader to use texture unit 0
/// ```
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sampler2D(pub gl::types::GLint);

impl Sampler2D {
    /// Create a new sampler uniform location.
    #[inline]
    pub const fn new(location: gl::types::GLint) -> Self {
        Sampler2D(location)
    }

    /// Get the raw uniform location.
    #[inline]
    pub const fn location(&self) -> gl::types::GLint {
        self.0
    }

    /// Set the texture unit for this sampler.
    ///
    /// # Arguments
    /// * `unit` - The texture unit index (0, 1, 2, etc.)
    pub fn set(&self, unit: i32) {
        unsafe {
            gl::Uniform1i(self.0, unit);
        }
    }
}

// ============================================================================
// Error type
// ============================================================================

/// Error when required uniforms are missing from a shader program.
#[derive(Debug, From)]
pub struct MissingUniforms(pub Vec<&'static str>);

impl std::fmt::Display for MissingUniforms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing uniforms: [{}]", self.0.join(", "))
    }
}

impl std::error::Error for MissingUniforms {}

// ============================================================================
// UniformTable trait
// ============================================================================

/// Trait for uniform table structs generated by [`uniform_table!`].
///
/// Implement this trait to create a collection of related uniforms
/// that can be retrieved from a shader program at once.
pub trait UniformTable: Sized {
    /// Retrieve uniform locations from a shader program.
    ///
    /// # Errors
    /// Returns `MissingUniforms` if any required uniforms are not found.
    fn with_locations_from(p: &Program) -> Result<Self, MissingUniforms>;
}

// ============================================================================
// uniform_table! macro
// ============================================================================

/// Macro to define a shader uniform table struct.
///
/// This generates a struct containing uniform locations and implements
/// the [`UniformTable`] trait for retrieving them from a shader program.
///
/// # Supported Types
/// - `Uniform1fv` - float
/// - `Uniform2fv` - vec2
/// - `Uniform3fv` - vec3
/// - `Uniform4fv` - vec4
/// - `Uniform1iv` - int
/// - `Uniform2iv` - ivec2
/// - `Uniform3iv` - ivec3
/// - `Uniform4iv` - ivec4
/// - `UniformMatrix2fv` - mat2
/// - `UniformMatrix3fv` - mat3
/// - `UniformMatrix4fv` - mat4
/// - `Sampler2D` - sampler2D
///
/// # Example
/// ```ignore
/// use vita_gl_helpers::uniform_table;
///
/// uniform_table!(MyUniforms,
///     mvp: UniformMatrix4fv => "uMVP",
///     color: Uniform4fv => "uColor",
///     time: Uniform1fv => "uTime",
///     texture: Sampler2D => "uTexture"
/// );
///
/// // Usage:
/// let uniforms = program.get_uniform_table::<MyUniforms>()?;
/// uniforms.mvp.set(matrix, false);
/// uniforms.color.set([1.0, 0.5, 0.0, 1.0]);
/// uniforms.time.set(elapsed);
/// uniforms.texture.set(0); // texture unit 0
/// ```
#[macro_export]
macro_rules! uniform_table {
    ($sname:ident, $($lname:ident : $t:ident => $lstr:expr),* $(,)?) => {
        /// Auto-generated uniform table struct.
        #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
        pub struct $sname {
            $(
                /// Uniform location for the shader variable.
                pub $lname: $crate::uniforms::$t
            ),*
        }

        impl $crate::uniforms::UniformTable for $sname {
            fn with_locations_from(
                p: &$crate::program::Program
            ) -> Result<Self, $crate::uniforms::MissingUniforms> {
                let to_check = [$($lstr),*];
                let locations = [$(p.get_uniform_location($lstr)),*];
                let errors: Vec<&'static str> = to_check
                    .into_iter()
                    .zip(locations.iter())
                    .filter_map(|(n, &l)| if l == -1 { Some(n) } else { None })
                    .collect();

                if !errors.is_empty() {
                    return Err($crate::uniforms::MissingUniforms(errors));
                }

                let mut locations_iter = locations.into_iter();
                Ok($sname {
                    $($lname: $crate::uniforms::$t(locations_iter.next().unwrap())),*
                })
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_types_exist() {
        // Just verify the types compile
        let _ = Uniform1fv::new(0);
        let _ = Uniform4fv::new(0);
        let _ = UniformMatrix4fv::new(0);
        let _ = Sampler2D::new(0);
    }
}
