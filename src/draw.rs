//! Draw Call Helpers
//!
//! This module provides utilities for issuing OpenGL draw calls.
//!
//! # Example
//! ```ignore
//! use vita_gl_helpers::draw::*;
//!
//! // Draw arrays (non-indexed)
//! draw_arrays(Mode::Triangles, 0, 3);
//!
//! // Draw with indices from a slice
//! let indices = [0u16, 1, 2, 2, 3, 0];
//! ElementsU16 { indices: &indices }.draw(Mode::Triangles);
//!
//! // Draw with indices from a buffer
//! ElementsBufU16 { indices: index_buffer, len: 6 }.draw(Mode::Triangles);
//!
//! // Instanced drawing
//! ElementsBufU16 { indices: index_buffer, len: 6 }
//!     .draw_instanced(Mode::Triangles, 100);
//! ```

use std::ffi::c_void;

use gl::types::{GLenum, GLint, GLsizei};

use crate::buffer::Buffer;

/// Primitive drawing modes.
///
/// Specifies what kind of primitives to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum Mode {
    /// Draw individual points.
    Points = gl::POINTS,
    /// Draw line segments.
    Lines = gl::LINES,
    /// Draw connected line strips.
    LineStrip = gl::LINE_STRIP,
    /// Draw closed line loops.
    LineLoop = gl::LINE_LOOP,
    /// Draw triangles (3 vertices per triangle).
    Triangles = gl::TRIANGLES,
    /// Draw connected triangle strips.
    TriangleStrip = gl::TRIANGLE_STRIP,
    /// Draw triangle fans.
    TriangleFan = gl::TRIANGLE_FAN,
    /// Draw quads (4 vertices per quad) - vitaGL extension.
    Quads = gl::QUADS,
}

impl From<Mode> for GLenum {
    fn from(mode: Mode) -> Self {
        mode as GLenum
    }
}

/// Draw primitives from array data.
///
/// This is the simplest draw call - it renders primitives using
/// consecutive vertices from the currently bound vertex arrays.
///
/// # Arguments
/// * `mode` - The primitive type to draw
/// * `first` - Index of the first vertex to draw
/// * `count` - Number of vertices to draw
///
/// # Example
/// ```ignore
/// // Draw a single triangle (3 vertices)
/// draw_arrays(Mode::Triangles, 0, 3);
///
/// // Draw a quad strip (4 vertices)
/// draw_arrays(Mode::TriangleStrip, 0, 4);
/// ```
pub fn draw_arrays(mode: Mode, first: GLint, count: GLsizei) {
    unsafe { gl::DrawArrays(mode as GLenum, first, count) }
}

/// Draw multiple instances of primitives from array data.
///
/// # Arguments
/// * `mode` - The primitive type to draw
/// * `first` - Index of the first vertex
/// * `count` - Number of vertices per instance
/// * `primcount` - Number of instances to draw
///
/// # Example
/// ```ignore
/// // Draw 100 instances of a quad
/// draw_arrays_instanced(Mode::TriangleStrip, 0, 4, 100);
/// ```
pub fn draw_arrays_instanced(mode: Mode, first: GLint, count: GLsizei, primcount: GLsizei) {
    unsafe { gl::DrawArraysInstanced(mode as GLenum, first, count, primcount) }
}

/// Parameters for element (indexed) drawing.
///
/// Used internally by the [`Elements`] trait.
pub struct ElementParams {
    /// Number of indices to draw.
    pub count: GLsizei,
    /// Type of index values (e.g., `GL_UNSIGNED_SHORT`).
    pub type_: GLenum,
    /// Pointer to index data or offset into bound element buffer.
    pub indices: *const c_void,
}

/// Trait for types that can be used for indexed drawing.
///
/// Implement this trait to create custom index sources.
pub trait Elements {
    /// Prepare for drawing and return the element parameters.
    fn use_me(&self) -> ElementParams;

    /// Draw indexed primitives.
    ///
    /// # Arguments
    /// * `mode` - The primitive type to draw
    fn draw(&self, mode: Mode) {
        let params = self.use_me();
        unsafe {
            gl::DrawElements(mode as GLenum, params.count, params.type_, params.indices);
        }
    }

    /// Draw multiple instances of indexed primitives.
    ///
    /// # Arguments
    /// * `mode` - The primitive type to draw
    /// * `primcount` - Number of instances to draw
    fn draw_instanced(&self, mode: Mode, primcount: GLsizei) {
        let params = self.use_me();
        unsafe {
            gl::DrawElementsInstanced(
                mode as GLenum,
                params.count,
                params.type_,
                params.indices,
                primcount,
            );
        }
    }
}

/// Draw indexed primitives using 16-bit indices from a slice.
///
/// The indices are passed directly to OpenGL without buffering.
///
/// # Example
/// ```ignore
/// let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];
/// ElementsU16 { indices: &indices }.draw(Mode::Triangles);
/// ```
pub struct ElementsU16<'a> {
    /// The index data.
    pub indices: &'a [u16],
}

impl<'a> Elements for ElementsU16<'a> {
    fn use_me(&self) -> ElementParams {
        // Unbind any element buffer to use client-side indices
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0) };
        ElementParams {
            count: self.indices.len() as GLsizei,
            type_: gl::UNSIGNED_SHORT,
            indices: self.indices.as_ptr() as *const c_void,
        }
    }
}

/// Draw indexed primitives using 32-bit indices from a slice.
///
/// # Example
/// ```ignore
/// let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
/// ElementsU32 { indices: &indices }.draw(Mode::Triangles);
/// ```
pub struct ElementsU32<'a> {
    /// The index data.
    pub indices: &'a [u32],
}

impl<'a> Elements for ElementsU32<'a> {
    fn use_me(&self) -> ElementParams {
        // Unbind any element buffer to use client-side indices
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0) };
        ElementParams {
            count: self.indices.len() as GLsizei,
            type_: gl::UNSIGNED_INT,
            indices: self.indices.as_ptr() as *const c_void,
        }
    }
}

/// Draw indexed primitives using 16-bit indices from a GPU buffer.
///
/// This is more efficient than slice-based indices for static geometry.
///
/// # Example
/// ```ignore
/// let index_buffer = Buffer::new();
/// index_buffer.data(gl::ELEMENT_ARRAY_BUFFER, &[0u16, 1, 2, 2, 3, 0], gl::STATIC_DRAW);
///
/// ElementsBufU16 { indices: index_buffer, len: 6 }.draw(Mode::Triangles);
/// ```
pub struct ElementsBufU16 {
    /// The buffer containing index data.
    pub indices: Buffer,
    /// Number of indices in the buffer.
    pub len: u32,
}

impl Elements for ElementsBufU16 {
    fn use_me(&self) -> ElementParams {
        self.indices.bind(gl::ELEMENT_ARRAY_BUFFER);
        ElementParams {
            count: self.len as GLsizei,
            type_: gl::UNSIGNED_SHORT,
            indices: std::ptr::null(), // Offset 0 into the bound buffer
        }
    }
}

/// Draw indexed primitives using 32-bit indices from a GPU buffer.
///
/// # Example
/// ```ignore
/// let index_buffer = Buffer::new();
/// index_buffer.data(gl::ELEMENT_ARRAY_BUFFER, &[0u32, 1, 2, 2, 3, 0], gl::STATIC_DRAW);
///
/// ElementsBufU32 { indices: index_buffer, len: 6 }.draw(Mode::Triangles);
/// ```
pub struct ElementsBufU32 {
    /// The buffer containing index data.
    pub indices: Buffer,
    /// Number of indices in the buffer.
    pub len: u32,
}

impl Elements for ElementsBufU32 {
    fn use_me(&self) -> ElementParams {
        self.indices.bind(gl::ELEMENT_ARRAY_BUFFER);
        ElementParams {
            count: self.len as GLsizei,
            type_: gl::UNSIGNED_INT,
            indices: std::ptr::null(), // Offset 0 into the bound buffer
        }
    }
}

/// Draw indexed primitives using a raw buffer ID (for legacy code).
///
/// # Example
/// ```ignore
/// ElementsBufIdU16 { buffer_id: raw_buffer, len: 6 }.draw(Mode::Triangles);
/// ```
pub struct ElementsBufIdU16 {
    /// Raw OpenGL buffer ID.
    pub buffer_id: gl::types::GLuint,
    /// Number of indices.
    pub len: u32,
}

impl Elements for ElementsBufIdU16 {
    fn use_me(&self) -> ElementParams {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.buffer_id) };
        ElementParams {
            count: self.len as GLsizei,
            type_: gl::UNSIGNED_SHORT,
            indices: std::ptr::null(),
        }
    }
}

/// Draw indexed primitives using a raw buffer ID with 32-bit indices.
pub struct ElementsBufIdU32 {
    /// Raw OpenGL buffer ID.
    pub buffer_id: gl::types::GLuint,
    /// Number of indices.
    pub len: u32,
}

impl Elements for ElementsBufIdU32 {
    fn use_me(&self) -> ElementParams {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.buffer_id) };
        ElementParams {
            count: self.len as GLsizei,
            type_: gl::UNSIGNED_INT,
            indices: std::ptr::null(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_conversion() {
        assert_eq!(GLenum::from(Mode::Triangles), gl::TRIANGLES);
        assert_eq!(GLenum::from(Mode::Points), gl::POINTS);
        assert_eq!(GLenum::from(Mode::Quads), gl::QUADS);
    }
}
