//! Texture Management
//!
//! This module provides safe wrappers for OpenGL texture objects with
//! automatic cleanup via RAII.
//!
//! # Example
//! ```ignore
//! use vita_gl_helpers::texture::Texture;
//!
//! // Create a new texture - automatically cleaned up when dropped
//! let texture = Texture::new();
//!
//! // Bind and configure
//! texture.bind_then(gl::TEXTURE_2D, |bound| {
//!     bound.image_2d(0, gl::RGBA as i32, 256, 256, gl::RGBA, gl::UNSIGNED_BYTE, pixels);
//!     bound.parameter_i(gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
//!     bound.parameter_i(gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
//! });
//! ```

use std::ffi::c_void;

use gl::types::{GLenum, GLint, GLuint};

use crate::handle::{GpuResource, Handle};

/// Marker type for texture resources.
pub struct TextureResource;

impl GpuResource for TextureResource {
    type Id = GLuint;

    unsafe fn delete(id: Self::Id) {
        gl::DeleteTextures(1, &id);
    }

    fn is_null(id: Self::Id) -> bool {
        id == 0
    }

    fn resource_name() -> &'static str {
        "Texture"
    }
}

/// A GPU texture object with automatic cleanup.
///
/// When dropped, the texture is automatically deleted via `glDeleteTextures`.
///
/// # Example
/// ```ignore
/// let texture = Texture::new();
/// texture.bind(gl::TEXTURE_2D);
/// // texture is automatically deleted when it goes out of scope
/// ```
pub struct Texture(Handle<TextureResource>);

impl Texture {
    /// Create a new GPU texture.
    ///
    /// # Example
    /// ```ignore
    /// let texture = Texture::new();
    /// assert!(texture.is_valid());
    /// ```
    pub fn new() -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
        }
        Texture(unsafe { Handle::from_raw(id) })
    }

    /// Create multiple textures at once (more efficient than individual creation).
    ///
    /// # Example
    /// ```ignore
    /// let textures = Texture::new_batch(5);
    /// assert_eq!(textures.len(), 5);
    /// ```
    pub fn new_batch(count: usize) -> Vec<Self> {
        let mut ids = vec![0 as GLuint; count];
        unsafe {
            gl::GenTextures(count as i32, ids.as_mut_ptr());
        }
        ids.into_iter()
            .map(|id| Texture(unsafe { Handle::from_raw(id) }))
            .collect()
    }

    /// Create a texture from a raw OpenGL ID. Takes ownership.
    ///
    /// # Safety
    /// The caller must ensure the ID is a valid texture created with `glGenTextures`
    /// and that ownership is being transferred.
    pub unsafe fn from_raw(id: GLuint) -> Self {
        Texture(Handle::from_raw(id))
    }

    /// Get the raw OpenGL texture ID.
    ///
    /// Returns `None` if the texture is invalid.
    #[inline]
    pub fn id(&self) -> Option<GLuint> {
        self.0.id()
    }

    /// Get the raw OpenGL texture ID, panicking if invalid.
    ///
    /// # Panics
    /// Panics if the texture is invalid.
    #[inline]
    pub fn id_unwrap(&self) -> GLuint {
        self.0.id_unwrap()
    }

    /// Check if this texture is valid.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0.is_valid()
    }

    /// Take ownership of the raw ID, preventing automatic deletion.
    ///
    /// After calling this, you are responsible for deleting the texture.
    pub fn into_raw(self) -> Option<GLuint> {
        self.0.into_raw()
    }

    /// Bind this texture to the specified target.
    ///
    /// # Arguments
    /// * `target` - The texture target (e.g., `gl::TEXTURE_2D`)
    ///
    /// # Example
    /// ```ignore
    /// texture.bind(gl::TEXTURE_2D);
    /// ```
    pub fn bind(&self, target: GLenum) {
        if let Some(id) = self.0.id() {
            unsafe {
                gl::BindTexture(target, id);
            }
        }
    }

    /// Bind this texture, execute a closure, then optionally unbind.
    ///
    /// This pattern ensures the texture is bound during the operation.
    ///
    /// # Example
    /// ```ignore
    /// texture.bind_then(gl::TEXTURE_2D, |bound| {
    ///     bound.parameter_i(gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    /// });
    /// ```
    pub fn bind_then<R>(&self, target: GLenum, then: impl FnOnce(BoundTexture) -> R) -> R {
        self.bind(target);
        then(BoundTexture(target))
    }

    /// Activate a texture unit and bind this texture.
    ///
    /// # Arguments
    /// * `unit` - The texture unit (0, 1, 2, etc.)
    /// * `target` - The texture target
    ///
    /// # Example
    /// ```ignore
    /// texture.bind_to_unit(0, gl::TEXTURE_2D);
    /// ```
    pub fn bind_to_unit(&self, unit: u32, target: GLenum) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + unit);
        }
        self.bind(target);
    }
}

impl Default for Texture {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Texture").field(&self.0).finish()
    }
}

/// A temporarily bound texture for operations.
///
/// This type is returned by [`Texture::bind_then`] and represents
/// a texture that is currently bound to a target.
#[non_exhaustive]
pub struct BoundTexture(GLenum);

impl BoundTexture {
    /// Upload 2D image data to the bound texture.
    ///
    /// # Arguments
    /// * `level` - Mipmap level (0 for base level)
    /// * `internalformat` - Internal format (e.g., `gl::RGBA`)
    /// * `width` - Image width
    /// * `height` - Image height
    /// * `format` - Pixel format (e.g., `gl::RGBA`)
    /// * `type_` - Pixel type (e.g., `gl::UNSIGNED_BYTE`)
    /// * `pixels` - Pointer to pixel data (can be null for allocation only)
    pub fn image_2d(
        &self,
        level: impl Into<GLint>,
        internalformat: impl Into<GLint>,
        width: impl Into<GLint>,
        height: impl Into<GLint>,
        format: impl Into<GLenum>,
        type_: impl Into<GLenum>,
        pixels: *const c_void,
    ) {
        unsafe {
            gl::TexImage2D(
                self.0,
                level.into(),
                internalformat.into(),
                width.into(),
                height.into(),
                0, // border (must be 0)
                format.into(),
                type_.into(),
                pixels,
            );
        }
    }

    /// Upload 2D image data from a slice.
    ///
    /// # Arguments
    /// * `level` - Mipmap level
    /// * `internalformat` - Internal format
    /// * `width` - Image width
    /// * `height` - Image height
    /// * `format` - Pixel format
    /// * `type_` - Pixel type
    /// * `data` - Pixel data slice
    pub fn image_2d_data<T>(
        &self,
        level: impl Into<GLint>,
        internalformat: impl Into<GLint>,
        width: impl Into<GLint>,
        height: impl Into<GLint>,
        format: impl Into<GLenum>,
        type_: impl Into<GLenum>,
        data: &[T],
    ) {
        self.image_2d(
            level,
            internalformat,
            width,
            height,
            format,
            type_,
            data.as_ptr() as *const c_void,
        );
    }

    /// Generate mipmaps for the bound texture.
    pub fn gen_mipmap(&self) {
        unsafe {
            gl::GenerateMipmap(self.0);
        }
    }

    /// Set a texture parameter (integer).
    ///
    /// # Arguments
    /// * `pname` - Parameter name (e.g., `gl::TEXTURE_MIN_FILTER`)
    /// * `param` - Parameter value
    pub fn parameter_i(&self, pname: impl Into<GLenum>, param: impl Into<GLint>) {
        unsafe {
            gl::TexParameteri(self.0, pname.into(), param.into());
        }
    }

    /// Set a texture parameter (float).
    ///
    /// # Arguments
    /// * `pname` - Parameter name
    /// * `param` - Parameter value
    pub fn parameter_f(&self, pname: impl Into<GLenum>, param: f32) {
        unsafe {
            gl::TexParameterf(self.0, pname.into(), param);
        }
    }

    /// Set common texture parameters for linear filtering.
    pub fn set_linear_filtering(&self) {
        self.parameter_i(gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        self.parameter_i(gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    }

    /// Set common texture parameters for nearest (pixel) filtering.
    pub fn set_nearest_filtering(&self) {
        self.parameter_i(gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        self.parameter_i(gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    }

    /// Set texture wrap mode for both S and T coordinates.
    pub fn set_wrap(&self, wrap_mode: impl Into<GLint>) {
        let mode = wrap_mode.into();
        self.parameter_i(gl::TEXTURE_WRAP_S, mode);
        self.parameter_i(gl::TEXTURE_WRAP_T, mode);
    }
}

// ============================================================================
// Legacy API (deprecated, for backwards compatibility)
// ============================================================================

/// Legacy trait for generating and deleting textures.
///
/// # Deprecated
/// Use [`Texture::new`] and [`Texture::new_batch`] instead.
/// Textures are now automatically deleted when dropped.
#[deprecated(since = "0.2.0", note = "Use Texture::new() instead - textures auto-delete on drop")]
pub trait GenDelTexturesExt {
    /// Generate texture IDs.
    fn gen_textures(&mut self);
    /// Delete texture IDs.
    fn delete_textures(&mut self);
}

#[allow(deprecated)]
impl GenDelTexturesExt for [GLuint] {
    fn gen_textures(&mut self) {
        unsafe { gl::GenTextures(self.len() as i32, self.as_mut_ptr()) }
    }

    fn delete_textures(&mut self) {
        unsafe { gl::DeleteTextures(self.len() as i32, self.as_mut_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_batch_creation() {
        // Note: This test won't actually create GL textures without a context,
        // but it verifies the API compiles correctly
    }
}
