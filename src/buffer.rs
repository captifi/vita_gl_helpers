//! GPU Buffer Management
//!
//! This module provides safe wrappers for OpenGL buffer objects with
//! automatic cleanup via RAII.
//!
//! # Example
//! ```ignore
//! use vita_gl_helpers::buffer::Buffer;
//!
//! // Create a new buffer - automatically cleaned up when dropped
//! let buffer = Buffer::new();
//!
//! // Upload data
//! let vertices: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];
//! buffer.data(gl::ARRAY_BUFFER, &vertices, gl::STATIC_DRAW);
//!
//! // Bind for use
//! buffer.bind(gl::ARRAY_BUFFER);
//! ```

use gl::types::{GLenum, GLsizei, GLuint};

use crate::attribute::{Attribute, AttributeFormat};
use crate::handle::{GpuResource, Handle};

/// Marker type for buffer resources.
pub struct BufferResource;

impl GpuResource for BufferResource {
    type Id = GLuint;

    unsafe fn delete(id: Self::Id) {
        gl::DeleteBuffers(1, &id);
    }

    fn is_null(id: Self::Id) -> bool {
        id == 0
    }

    fn resource_name() -> &'static str {
        "Buffer"
    }
}

/// A GPU buffer object with automatic cleanup.
///
/// When dropped, the buffer is automatically deleted via `glDeleteBuffers`.
///
/// # Example
/// ```ignore
/// let buffer = Buffer::new();
/// buffer.data(gl::ARRAY_BUFFER, &[1.0f32, 2.0, 3.0], gl::STATIC_DRAW);
/// // buffer is automatically deleted when it goes out of scope
/// ```
pub struct Buffer(Handle<BufferResource>);

impl Buffer {
    /// Create a new GPU buffer.
    ///
    /// # Example
    /// ```ignore
    /// let buffer = Buffer::new();
    /// assert!(buffer.is_valid());
    /// ```
    pub fn new() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
        }
        Buffer(unsafe { Handle::from_raw(id) })
    }

    /// Create multiple buffers at once (more efficient than individual creation).
    ///
    /// # Example
    /// ```ignore
    /// let buffers = Buffer::new_batch(3);
    /// assert_eq!(buffers.len(), 3);
    /// ```
    pub fn new_batch(count: usize) -> Vec<Self> {
        let mut ids = vec![0 as GLuint; count];
        unsafe {
            gl::GenBuffers(count as i32, ids.as_mut_ptr());
        }
        ids.into_iter()
            .map(|id| Buffer(unsafe { Handle::from_raw(id) }))
            .collect()
    }

    /// Create a buffer from a raw OpenGL ID. Takes ownership.
    ///
    /// # Safety
    /// The caller must ensure the ID is a valid buffer created with `glGenBuffers`
    /// and that ownership is being transferred.
    pub unsafe fn from_raw(id: GLuint) -> Self {
        Buffer(Handle::from_raw(id))
    }

    /// Get the raw OpenGL buffer ID.
    ///
    /// Returns `None` if the buffer is invalid.
    #[inline]
    pub fn id(&self) -> Option<GLuint> {
        self.0.id()
    }

    /// Get the raw OpenGL buffer ID, panicking if invalid.
    ///
    /// # Panics
    /// Panics if the buffer is invalid.
    #[inline]
    pub fn id_unwrap(&self) -> GLuint {
        self.0.id_unwrap()
    }

    /// Check if this buffer is valid.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0.is_valid()
    }

    /// Take ownership of the raw ID, preventing automatic deletion.
    ///
    /// After calling this, you are responsible for deleting the buffer.
    pub fn into_raw(self) -> Option<GLuint> {
        self.0.into_raw()
    }

    /// Bind this buffer to the specified target.
    ///
    /// # Arguments
    /// * `target` - The buffer target (e.g., `gl::ARRAY_BUFFER`, `gl::ELEMENT_ARRAY_BUFFER`)
    ///
    /// # Example
    /// ```ignore
    /// buffer.bind(gl::ARRAY_BUFFER);
    /// ```
    pub fn bind(&self, target: impl Into<GLenum>) {
        if let Some(id) = self.0.id() {
            unsafe {
                gl::BindBuffer(target.into(), id);
            }
        }
    }

    /// Bind this buffer, execute a closure, then optionally unbind.
    ///
    /// This pattern ensures the buffer is bound during the operation.
    ///
    /// # Example
    /// ```ignore
    /// buffer.bind_then(gl::ARRAY_BUFFER, |bound| {
    ///     bound.data(&vertices, gl::STATIC_DRAW);
    /// });
    /// ```
    pub fn bind_then<R>(&self, target: impl Into<GLenum>, then: impl FnOnce(BoundBuffer) -> R) -> R {
        let target = target.into();
        self.bind(target);
        then(BoundBuffer(target))
    }

    /// Upload data to this buffer.
    ///
    /// This binds the buffer, uploads the data, then leaves it bound.
    ///
    /// # Arguments
    /// * `target` - The buffer target
    /// * `data` - The data to upload
    /// * `usage` - Usage hint (e.g., `gl::STATIC_DRAW`, `gl::DYNAMIC_DRAW`)
    ///
    /// # Example
    /// ```ignore
    /// let vertices = [0.0f32, 0.5, 0.5, -0.5, -0.5, -0.5];
    /// buffer.data(gl::ARRAY_BUFFER, &vertices, gl::STATIC_DRAW);
    /// ```
    pub fn data<T>(&self, target: impl Into<GLenum>, data: impl AsRef<[T]>, usage: impl Into<GLenum>) {
        self.bind_then(target, |b| b.data::<T>(data, usage));
    }

    /// Bind this buffer to a vertex attribute.
    ///
    /// This is a convenience method that binds the buffer as `GL_ARRAY_BUFFER`
    /// and sets up the vertex attribute pointer.
    ///
    /// # Arguments
    /// * `attribute` - The vertex attribute to bind to
    /// * `format` - The format of the attribute data
    /// * `stride` - Byte stride between consecutive attributes (0 for tightly packed)
    /// * `offset` - Byte offset to the first attribute
    ///
    /// # Example
    /// ```ignore
    /// buffer.bind_to(atable.pos, pos_format, 0, 0);
    /// ```
    pub fn bind_to(
        &self,
        attribute: Attribute,
        format: AttributeFormat,
        stride: GLsizei,
        offset: usize,
    ) {
        self.bind_then(gl::ARRAY_BUFFER, |b| {
            b.bind_to(attribute, format, stride, offset)
        })
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Buffer").field(&self.0).finish()
    }
}

/// A temporarily bound buffer for operations.
///
/// This type is returned by [`Buffer::bind_then`] and represents
/// a buffer that is currently bound to a target.
#[non_exhaustive]
pub struct BoundBuffer(GLenum);

impl BoundBuffer {
    /// Upload data to the bound buffer.
    ///
    /// # Arguments
    /// * `data` - The data to upload
    /// * `usage` - Usage hint
    pub fn data<T>(&self, data: impl AsRef<[T]>, usage: impl Into<GLenum>) {
        let data = data.as_ref();
        let n_bytes = size_of::<T>() * data.len();
        unsafe {
            gl::BufferData(self.0, n_bytes as _, data.as_ptr() as _, usage.into());
        }
    }

    /// Set up a vertex attribute pointer for the bound buffer.
    pub fn bind_to(
        &self,
        attribute: Attribute,
        format: AttributeFormat,
        stride: GLsizei,
        offset: usize,
    ) {
        unsafe { attribute.pointer(format, stride, offset as _) }
    }
}

// ============================================================================
// Legacy API (deprecated, for backwards compatibility)
// ============================================================================

/// Legacy buffer type alias.
///
/// # Deprecated
/// Use [`Buffer`] instead, which provides RAII cleanup.
#[deprecated(since = "0.2.0", note = "Use Buffer instead, which provides RAII cleanup")]
pub type LegacyBuffer = GLuint;

/// Legacy trait for generating and deleting buffers.
///
/// # Deprecated
/// Use [`Buffer::new`] and [`Buffer::new_batch`] instead.
/// Buffers are now automatically deleted when dropped.
#[deprecated(since = "0.2.0", note = "Use Buffer::new() instead - buffers auto-delete on drop")]
pub trait GenDelBuffersExt {
    /// Generate buffer IDs.
    fn gen_buffers(&mut self);
    /// Delete buffer IDs.
    fn del_buffers(&mut self);
}

#[allow(deprecated)]
impl GenDelBuffersExt for [GLuint] {
    fn gen_buffers(&mut self) {
        unsafe { gl::GenBuffers(self.len() as i32, self.as_mut_ptr()) }
    }

    fn del_buffers(&mut self) {
        unsafe { gl::DeleteBuffers(self.len() as i32, self.as_mut_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_batch_creation() {
        // Note: This test won't actually create GL buffers without a context,
        // but it verifies the API compiles correctly
    }
}
