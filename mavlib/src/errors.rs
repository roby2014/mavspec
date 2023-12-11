//! # Errors
//!
//! All errors within `mavlib` are either defined or re-exported into this module namespace.
//!
//! The root error is [`Error`]. In most cases high-level abstractions will return an instance of
//! [`Error`] usually wrapping another underlying error.

use mavlib_core::errors::FrameError;
use mavlib_spec::errors::MessageError;

/// This is a common result returned by `mavlib` functions and methods.
pub type Result<T> = core::result::Result<T, Error>;

/// The mother of all `mavlib` errors.
///
/// In most cases high-level abstractions will return an instance of [`Error`] usually wrapping
/// another underlying errors.
#[derive(Debug)]
pub enum Error {
    /// Wraps [`std::io::Error`].
    ///
    /// Available only when `std` feature is enabled.
    #[cfg(feature = "std")]
    Io(std::io::Error),
    /// Wraps errors related to frame encoding/decoding.
    Frame(FrameError),
    /// Wraps errors related to message encoding/decoding
    Message(MessageError),
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    /// Wraps [`std::io::Error`] with `mavlib` [`Error`].
    ///
    /// Available only when `std` feature is enabled.
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
