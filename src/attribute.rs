//! Vertex Attribute Configuration
//!
//! This module provides types and utilities for configuring vertex attributes,
//! which define how vertex data is interpreted by shaders.
//!
//! # Example
//! ```ignore
//! use vita_gl_helpers::attribute::*;
//! use vita_gl_helpers::attribute_table;
//!
//! // Define an attribute table for your shader
//! attribute_table!(MyAttributes,
//!     pos => "aPosition",
//!     color => "aColor",
//!     texcoord => "aTexCoord"
//! );
//!
//! // Get attribute locations from a program
//! let attrs = program.get_attribute_table::<MyAttributes>()?;
//!
//! // Enable and configure attributes
//! attrs.enable_all();
//! buffer.bind_to(attrs.pos, pos_format, 0, 0);
//! ```

use std::ffi::c_void;

use derive_more::From;
use gl::types::{GLsizei, GLuint};

use crate::program::Program;

/// A vertex attribute location.
///
/// Represents the location of a vertex attribute in a shader program,
/// obtained via `glGetAttribLocation`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct Attribute(pub GLuint);

impl Attribute {
    /// Create a new attribute with the given location.
    #[inline]
    pub const fn new(location: GLuint) -> Self {
        Attribute(location)
    }

    /// Get the raw attribute location.
    #[inline]
    pub const fn location(&self) -> GLuint {
        self.0
    }

    /// Set the vertex attribute divisor for instanced rendering.
    ///
    /// # Arguments
    /// * `divisor` - The divisor value:
    ///   - 0: Attribute advances once per vertex (default)
    ///   - 1: Attribute advances once per instance
    ///
    /// # Note
    /// On vitaGL, only divisors 0 and 1 are recognized.
    pub fn divisor(&self, divisor: GLuint) {
        unsafe {
            gl::VertexAttribDivisor(self.0, divisor);
        }
    }

    /// Enable this vertex attribute array.
    ///
    /// Must be called before rendering with this attribute.
    pub fn enable(&self) {
        unsafe {
            gl::EnableVertexAttribArray(self.0);
        }
    }

    /// Disable this vertex attribute array.
    pub fn disable(&self) {
        unsafe {
            gl::DisableVertexAttribArray(self.0);
        }
    }

    /// Set up the vertex attribute pointer.
    ///
    /// # Arguments
    /// * `format` - The format of the attribute data
    /// * `stride` - Byte stride between consecutive attributes (0 for tightly packed)
    /// * `pointer` - Byte offset to the first attribute in the buffer
    ///
    /// # Safety
    /// A buffer must be bound to `GL_ARRAY_BUFFER` before calling this.
    pub unsafe fn pointer(&self, format: AttributeFormat, stride: GLsizei, pointer: *const c_void) {
        gl::VertexAttribPointer(
            self.0,
            format.size as i32,
            format.type_ as u32,
            if format.normalized { gl::TRUE } else { gl::FALSE },
            stride,
            pointer,
        );
    }
}

/// Number of components in a vertex attribute.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(i32)]
pub enum AttributeSize {
    /// 1 component (scalar)
    One = 1,
    /// 2 components (vec2)
    Two = 2,
    /// 3 components (vec3)
    Three = 3,
    /// 4 components (vec4)
    Four = 4,
}

/// Data type of vertex attribute components.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u32)]
pub enum AttributeType {
    /// Signed byte (-128 to 127)
    Byte = gl::BYTE,
    /// Unsigned byte (0 to 255)
    UnsignedByte = gl::UNSIGNED_BYTE,
    /// Signed short (-32768 to 32767)
    Short = gl::SHORT,
    /// Unsigned short (0 to 65535)
    UnsignedShort = gl::UNSIGNED_SHORT,
    /// Fixed-point (16.16)
    Fixed = gl::FIXED,
    /// 32-bit float
    Float = gl::FLOAT,
}

/// Format specification for a vertex attribute.
///
/// Describes how vertex data should be interpreted.
///
/// # Example
/// ```ignore
/// let pos_format = AttributeFormat {
///     size: AttributeSize::Three,
///     type_: AttributeType::Float,
///     normalized: false,
/// };
///
/// let color_format = AttributeFormat {
///     size: AttributeSize::Four,
///     type_: AttributeType::UnsignedByte,
///     normalized: true, // Convert 0-255 to 0.0-1.0
/// };
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AttributeFormat {
    /// Number of components (1-4)
    pub size: AttributeSize,
    /// Data type of each component
    pub type_: AttributeType,
    /// Whether to normalize integer values to [0,1] or [-1,1]
    pub normalized: bool,
}

impl AttributeFormat {
    /// Float scalar (1 component)
    pub const FLOAT1: Self = Self {
        size: AttributeSize::One,
        type_: AttributeType::Float,
        normalized: false,
    };

    /// Float vec2 (2 components)
    pub const FLOAT2: Self = Self {
        size: AttributeSize::Two,
        type_: AttributeType::Float,
        normalized: false,
    };

    /// Float vec3 (3 components)
    pub const FLOAT3: Self = Self {
        size: AttributeSize::Three,
        type_: AttributeType::Float,
        normalized: false,
    };

    /// Float vec4 (4 components)
    pub const FLOAT4: Self = Self {
        size: AttributeSize::Four,
        type_: AttributeType::Float,
        normalized: false,
    };

    /// Unsigned byte vec4, normalized (e.g., for RGBA colors as u32)
    pub const UBYTE4_NORM: Self = Self {
        size: AttributeSize::Four,
        type_: AttributeType::UnsignedByte,
        normalized: true,
    };

    /// Unsigned short vec2, normalized (e.g., for texture coordinates)
    pub const USHORT2_NORM: Self = Self {
        size: AttributeSize::Two,
        type_: AttributeType::UnsignedShort,
        normalized: true,
    };
}

/// Error when required attributes are missing from a shader program.
#[derive(Debug, From)]
pub struct MissingAttributes(pub Vec<&'static str>);

impl std::fmt::Display for MissingAttributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing attributes: [{}]", self.0.join(", "))
    }
}

impl std::error::Error for MissingAttributes {}

/// Trait for attribute table structs generated by [`attribute_table!`].
///
/// Implement this trait to create a collection of related vertex attributes
/// that can be retrieved from a shader program at once.
pub trait AttributeTable: Sized {
    /// Retrieve attribute locations from a shader program.
    ///
    /// # Errors
    /// Returns `MissingAttributes` if any required attributes are not found.
    fn with_locations_from(p: &Program) -> Result<Self, MissingAttributes>;

    /// Iterate over all attributes in this table.
    fn attributes(&self) -> impl Iterator<Item = &Attribute>;

    /// Enable all vertex attribute arrays in this table.
    fn enable_all(&self) {
        self.attributes().for_each(Attribute::enable);
    }

    /// Disable all vertex attribute arrays in this table.
    fn disable_all(&self) {
        self.attributes().for_each(Attribute::disable);
    }
}

/// Macro to define a vertex attribute table struct.
///
/// This generates a struct containing attribute locations and implements
/// the [`AttributeTable`] trait for retrieving them from a shader program.
///
/// # Example
/// ```ignore
/// use vita_gl_helpers::attribute_table;
///
/// attribute_table!(MyAttributes,
///     position => "aPosition",
///     color => "aColor",
///     texcoord => "aTexCoord"
/// );
///
/// // Usage:
/// let attrs = program.get_attribute_table::<MyAttributes>()?;
/// attrs.enable_all();
/// buffer.bind_to(attrs.position, AttributeFormat::FLOAT3, 0, 0);
/// ```
#[macro_export]
macro_rules! attribute_table {
    ($sname:ident, $($lname:ident => $lstr:expr),* $(,)?) => {
        /// Auto-generated attribute table struct.
        #[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
        pub struct $sname {
            $(
                /// Attribute location for the shader variable.
                pub $lname: $crate::attribute::Attribute
            ),*
        }

        impl $crate::attribute::AttributeTable for $sname {
            fn with_locations_from(
                p: &$crate::program::Program
            ) -> Result<Self, $crate::attribute::MissingAttributes> {
                let to_check = [$($lstr),*];
                let locations = [$(p.get_attrib_location($lstr)),*];
                let errors: Vec<&'static str> = to_check
                    .into_iter()
                    .zip(locations.iter())
                    .filter_map(|(n, &l)| if l < 0 { Some(n) } else { None })
                    .collect();

                if !errors.is_empty() {
                    return Err($crate::attribute::MissingAttributes(errors));
                }

                let mut locations_iter = locations.into_iter();
                Ok($sname {
                    $($lname: $crate::attribute::Attribute(
                        locations_iter.next().unwrap() as u32
                    )),*
                })
            }

            fn attributes(&self) -> impl Iterator<Item = &$crate::attribute::Attribute> {
                [$(&self.$lname),*].into_iter()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_format_constants() {
        assert_eq!(AttributeFormat::FLOAT3.size, AttributeSize::Three);
        assert_eq!(AttributeFormat::FLOAT3.type_, AttributeType::Float);
        assert!(!AttributeFormat::FLOAT3.normalized);

        assert_eq!(AttributeFormat::UBYTE4_NORM.size, AttributeSize::Four);
        assert!(AttributeFormat::UBYTE4_NORM.normalized);
    }
}
