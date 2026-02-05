//! GL Error Handling
//!
//! This module provides utilities for checking and reporting OpenGL errors.
//!
//! # Example
//! ```ignore
//! use vita_gl_helpers::errors::*;
//!
//! // Check for errors after GL operations
//! unsafe { gl::DrawArrays(gl::TRIANGLES, 0, 3); }
//! eprintln_errors(); // Prints any errors to stderr
//!
//! // Or check programmatically
//! let error = get_error();
//! if error != GlError::NoError {
//!     println!("GL Error: {}", error);
//! }
//!
//! // Iterate over all pending errors
//! for error in Errors {
//!     println!("Error: {:?}", error);
//! }
//! ```

use derive_more::TryFrom;

/// OpenGL error codes.
///
/// These correspond to the values returned by `glGetError()`.
#[derive(TryFrom, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[try_from(repr)]
#[repr(u32)]
pub enum GlError {
    /// No error (GL_NO_ERROR)
    NoError = gl::NO_ERROR,
    /// Invalid enum parameter (GL_INVALID_ENUM)
    InvalidEnum = gl::INVALID_ENUM,
    /// Invalid value parameter (GL_INVALID_VALUE)
    InvalidValue = gl::INVALID_VALUE,
    /// Invalid operation for current state (GL_INVALID_OPERATION)
    InvalidOperation = gl::INVALID_OPERATION,
    /// Invalid framebuffer operation (GL_INVALID_FRAMEBUFFER_OPERATION)
    InvalidFramebufferOperation = gl::INVALID_FRAMEBUFFER_OPERATION,
    /// Out of memory (GL_OUT_OF_MEMORY)
    OutOfMemory = gl::OUT_OF_MEMORY,
    /// Stack underflow (GL_STACK_UNDERFLOW)
    StackUnderflow = gl::STACK_UNDERFLOW,
    /// Stack overflow (GL_STACK_OVERFLOW)
    StackOverflow = gl::STACK_OVERFLOW,
}

impl std::fmt::Display for GlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            GlError::NoError => "GL_NO_ERROR",
            GlError::InvalidEnum => "GL_INVALID_ENUM",
            GlError::InvalidValue => "GL_INVALID_VALUE",
            GlError::InvalidOperation => "GL_INVALID_OPERATION",
            GlError::InvalidFramebufferOperation => "GL_INVALID_FRAMEBUFFER_OPERATION",
            GlError::OutOfMemory => "GL_OUT_OF_MEMORY",
            GlError::StackUnderflow => "GL_STACK_UNDERFLOW",
            GlError::StackOverflow => "GL_STACK_OVERFLOW",
        };
        write!(f, "{}", name)
    }
}

impl std::error::Error for GlError {}

impl GlError {
    /// Check if this is an actual error (not NoError).
    #[inline]
    pub fn is_error(&self) -> bool {
        *self != GlError::NoError
    }

    /// Get a human-readable description of the error.
    pub fn description(&self) -> &'static str {
        match self {
            GlError::NoError => "No error has been recorded",
            GlError::InvalidEnum => "An unacceptable value was specified for an enumerated argument",
            GlError::InvalidValue => "A numeric argument was out of range",
            GlError::InvalidOperation => "The specified operation is not allowed in the current state",
            GlError::InvalidFramebufferOperation => "The framebuffer object is not complete",
            GlError::OutOfMemory => "There is not enough memory left to execute the command",
            GlError::StackUnderflow => "An attempt was made to perform an operation that would cause an internal stack to underflow",
            GlError::StackOverflow => "An attempt was made to perform an operation that would cause an internal stack to overflow",
        }
    }
}

/// Get the current GL error.
///
/// Calls `glGetError()` and converts the result to a [`GlError`].
/// Note that `glGetError()` clears the error flag, so this will return
/// the oldest error and remove it from the error queue.
///
/// # Example
/// ```ignore
/// let error = get_error();
/// if error.is_error() {
///     println!("GL Error: {} - {}", error, error.description());
/// }
/// ```
pub fn get_error() -> GlError {
    let error_code = unsafe { gl::GetError() };
    error_code
        .try_into()
        .expect("Unexpected error code from glGetError")
}

/// Iterator over all pending GL errors.
///
/// Repeatedly calls `glGetError()` until `GL_NO_ERROR` is returned.
///
/// # Example
/// ```ignore
/// for error in Errors {
///     eprintln!("GL Error: {}", error);
/// }
/// ```
pub struct Errors;

impl Iterator for Errors {
    type Item = GlError;

    fn next(&mut self) -> Option<Self::Item> {
        let next_error = get_error();
        if next_error == GlError::NoError {
            None
        } else {
            Some(next_error)
        }
    }
}

/// Print all pending GL errors to stderr.
///
/// This is a convenience function for debugging. It iterates over all
/// pending errors and prints them to stderr.
///
/// # Example
/// ```ignore
/// // After some GL operations
/// eprintln_errors();
/// ```
pub fn eprintln_errors() {
    for error in Errors {
        eprintln!("GL ERROR: {} - {}", error, error.description());
    }
}

/// Check for GL errors and return the first one if any.
///
/// Unlike [`get_error`], this returns `Ok(())` if there are no errors,
/// making it suitable for use with the `?` operator.
///
/// # Example
/// ```ignore
/// fn render() -> Result<(), GlError> {
///     unsafe { gl::DrawArrays(gl::TRIANGLES, 0, 3); }
///     check_error()?;
///     Ok(())
/// }
/// ```
pub fn check_error() -> Result<(), GlError> {
    let error = get_error();
    if error.is_error() {
        Err(error)
    } else {
        Ok(())
    }
}

/// Check for GL errors and panic if any are found.
///
/// This is useful for debugging when you want to immediately catch errors.
///
/// # Panics
/// Panics if any GL error is pending.
///
/// # Example
/// ```ignore
/// unsafe { gl::DrawArrays(gl::TRIANGLES, 0, 3); }
/// assert_no_error(); // Panics if there was an error
/// ```
pub fn assert_no_error() {
    let error = get_error();
    if error.is_error() {
        panic!("GL Error: {} - {}", error, error.description());
    }
}

/// Conditional error checking based on the `debug-gl` feature.
///
/// When the `debug-gl` feature is enabled, this checks for errors.
/// When disabled, this is a no-op for performance.
#[cfg(feature = "debug-gl")]
pub fn debug_check_error(operation: &str) {
    let error = get_error();
    if error.is_error() {
        eprintln!("GL ERROR during '{}': {} - {}", operation, error, error.description());
    }
}

#[cfg(not(feature = "debug-gl"))]
#[inline(always)]
pub fn debug_check_error(_operation: &str) {
    // No-op when debug-gl feature is disabled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gl_error_display() {
        assert_eq!(format!("{}", GlError::NoError), "GL_NO_ERROR");
        assert_eq!(format!("{}", GlError::InvalidEnum), "GL_INVALID_ENUM");
        assert_eq!(format!("{}", GlError::OutOfMemory), "GL_OUT_OF_MEMORY");
    }

    #[test]
    fn test_gl_error_is_error() {
        assert!(!GlError::NoError.is_error());
        assert!(GlError::InvalidEnum.is_error());
        assert!(GlError::OutOfMemory.is_error());
    }
}
