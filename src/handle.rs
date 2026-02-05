//! RAII Handle wrapper for GPU resources
//!
//! This module provides a generic handle wrapper that automatically
//! cleans up GPU resources when they go out of scope.
//!
//! # Example
//! ```ignore
//! // Resources are automatically cleaned up when dropped
//! let buffer = Buffer::new();
//! // ... use buffer ...
//! // buffer is deleted automatically here
//! ```

use std::marker::PhantomData;

/// Trait defining how a GPU resource is deleted.
///
/// Implement this trait for each type of GPU resource to enable
/// automatic cleanup via the [`Handle`] wrapper.
pub trait GpuResource {
    /// The type of the raw OpenGL ID (usually `GLuint`)
    type Id: Copy + Default + Eq;

    /// Delete the resource. Called automatically on [`Handle::drop`].
    ///
    /// # Safety
    /// This function calls OpenGL delete functions which require
    /// a valid GL context and valid resource ID.
    unsafe fn delete(id: Self::Id);

    /// Check if the ID represents a null/invalid resource.
    fn is_null(id: Self::Id) -> bool;

    /// Get the name of this resource type for debugging.
    fn resource_name() -> &'static str;
}

/// RAII wrapper for GPU resources.
///
/// This wrapper ensures that GPU resources are properly deleted when
/// they go out of scope, preventing memory leaks.
///
/// # Ownership
/// - `Handle` has exclusive ownership of the resource
/// - Use `Rc<Handle<T>>` or `Arc<Handle<T>>` for shared ownership
/// - Use [`Handle::into_raw`] to transfer ownership out of the handle
///
/// # Example
/// ```ignore
/// let buffer = Buffer::new();
/// assert!(buffer.is_valid());
/// // buffer is automatically deleted when it goes out of scope
/// ```
pub struct Handle<T: GpuResource> {
    id: Option<T::Id>,
    _marker: PhantomData<T>,
}

impl<T: GpuResource> Handle<T> {
    /// Create a handle from a raw OpenGL ID. Takes ownership.
    ///
    /// # Safety
    /// The caller must ensure:
    /// - `id` was created by the corresponding `glGen*` function
    /// - `id` has not been deleted
    /// - Ownership of `id` is transferred to this handle
    /// - No other code will delete this resource
    ///
    /// # Example
    /// ```ignore
    /// let mut id = 0;
    /// unsafe { gl::GenBuffers(1, &mut id); }
    /// let buffer = unsafe { Handle::<BufferResource>::from_raw(id) };
    /// ```
    #[inline]
    pub unsafe fn from_raw(id: T::Id) -> Self {
        Handle {
            id: if T::is_null(id) { None } else { Some(id) },
            _marker: PhantomData,
        }
    }

    /// Create a null/invalid handle.
    ///
    /// This is useful as a placeholder or default value.
    #[inline]
    pub const fn null() -> Self {
        Handle {
            id: None,
            _marker: PhantomData,
        }
    }

    /// Get the raw ID without transferring ownership.
    ///
    /// Returns `None` if the handle is null/invalid.
    #[inline]
    pub fn id(&self) -> Option<T::Id> {
        self.id
    }

    /// Get the raw ID, panicking if null.
    ///
    /// # Panics
    /// Panics if the handle is null/invalid.
    #[inline]
    pub fn id_unwrap(&self) -> T::Id {
        self.id.expect("Attempted to use null GPU handle")
    }

    /// Take ownership of the raw ID, preventing automatic deletion.
    ///
    /// After calling this, the handle becomes null and the caller
    /// is responsible for deleting the resource.
    ///
    /// # Example
    /// ```ignore
    /// let buffer = Buffer::new();
    /// let raw_id = buffer.into_raw();
    /// // Now you must manually delete the buffer
    /// unsafe { gl::DeleteBuffers(1, &raw_id.unwrap()); }
    /// ```
    #[inline]
    pub fn into_raw(mut self) -> Option<T::Id> {
        self.id.take()
    }

    /// Check if this handle is valid (non-null).
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.id.is_some()
    }

    /// Check if this handle is null/invalid.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.id.is_none()
    }
}

impl<T: GpuResource> Drop for Handle<T> {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            #[cfg(feature = "logging")]
            log::trace!("Deleting {} with id {:?}", T::resource_name(), id);

            unsafe {
                T::delete(id);
            }
        }
    }
}

impl<T: GpuResource> Default for Handle<T> {
    fn default() -> Self {
        Self::null()
    }
}

// Handles cannot be cloned - use Rc/Arc for shared ownership
// This is intentional to prevent double-free

impl<T: GpuResource> std::fmt::Debug for Handle<T>
where
    T::Id: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("type", &T::resource_name())
            .field("id", &self.id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mock resource for testing
    struct MockResource;

    static DELETED_IDS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

    impl GpuResource for MockResource {
        type Id = u32;

        unsafe fn delete(id: Self::Id) {
            DELETED_IDS.lock().unwrap().push(id);
        }

        fn is_null(id: Self::Id) -> bool {
            id == 0
        }

        fn resource_name() -> &'static str {
            "MockResource"
        }
    }

    #[test]
    fn test_handle_creation() {
        let handle = unsafe { Handle::<MockResource>::from_raw(42) };
        assert!(handle.is_valid());
        assert_eq!(handle.id(), Some(42));
    }

    #[test]
    fn test_null_handle() {
        let handle = Handle::<MockResource>::null();
        assert!(handle.is_null());
        assert_eq!(handle.id(), None);
    }

    #[test]
    fn test_into_raw_prevents_deletion() {
        DELETED_IDS.lock().unwrap().clear();

        let handle = unsafe { Handle::<MockResource>::from_raw(123) };
        let raw = handle.into_raw();

        assert_eq!(raw, Some(123));
        assert!(DELETED_IDS.lock().unwrap().is_empty());
    }
}
