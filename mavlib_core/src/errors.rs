//! # Errors
//!
//! This errors used in `mavlib_core`.
//!
//! The top-level error is [`CoreError`]. Library API returns versions of this error possibly wrapping other types of
//! errors.

use tbytes::errors::TBytesError;

use mavlib_spec::MessageError;
#[cfg(feature = "std")]
use thiserror::Error;

/// Common result type returned by `mavlib_core` functions and methods.
pub type Result<T> = core::result::Result<T, CoreError>;

/// `mavlib_core` top-level error.
///
/// [`CoreError`] is returned by most of the functions and methods across `mavlib_core`. Other errors are either
/// converted to [`CoreError`] or wrapped by its variants.
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum CoreError {
    /// [`std::io::Error`] wrapper.
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "std", error("I/O error"))]
    Io(#[from] std::io::Error),

    /// `no_std` I/O error.
    ///
    /// Wraps [`IoError`](crate::io::no_std::IoError).
    #[cfg(not(feature = "std"))]
    Io(crate::io::no_std::IoError),

    /// Frame encoding/decoding error.
    #[cfg_attr(feature = "std", error("frame decoding/encoding error"))]
    Frame(FrameError),

    /// Message encoding/decoding and specification discovery error.
    #[cfg_attr(feature = "std", error("frame decoding/encoding error"))]
    Message(MessageError),
}

/// Errors related to MAVLink frame encoding/decoding.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum FrameError {
    /// MAVLink header is too small.
    #[cfg_attr(feature = "std", error("header is too small"))]
    HeaderIsTooSmall,
    /// `MAVLink 1` header is too small.
    #[cfg_attr(feature = "std", error("MAVLink 1 header is too small"))]
    V1HeaderIsTooSmall,
    /// `MAVLink 2` header is too small.
    #[cfg_attr(feature = "std", error("MAVLink 2 header is too small"))]
    V2HeaderIsTooSmall,
    /// Incorrect MAVLink version.
    #[cfg_attr(feature = "std", error("invalid MAVLink version"))]
    InvalidMavLinkVersion,
    /// Inconsistent `MAVLink 1` header: `MAVLink 2` fields are defined.
    #[cfg_attr(feature = "std", error("inconsistent MAVLink 1 header"))]
    InconsistentV1Header,
    /// Inconsistent `MAVLink 2` header: `MAVLink 2` fields are not defined.
    #[cfg_attr(feature = "std", error("inconsistent MAVLink 2 header"))]
    InconsistentV2Header,
    /// `MAVLink 1` packet body is too small.
    #[cfg_attr(feature = "std", error("MAVLink 1 packet body is too small"))]
    V1PacketBodyIsTooSmall,
    /// `MAVLink 2` packet body is too small.
    #[cfg_attr(feature = "std", error("MAVLink 2 packet body is too small"))]
    V2PacketBodyIsTooSmall,
    /// `MAVLink 2` signature is too small.
    #[cfg_attr(feature = "std", error("MAVLink 2 signature is too small"))]
    V2SignatureIsTooSmall,
    /// Buffer error.
    #[cfg_attr(feature = "std", error("MAVLink 2 signature is too small"))]
    Buffer(TBytesError),
    /// Upon calculation CRC does not match received [MavLinkFrame::checksum](crate::Frame::checksum).
    #[cfg_attr(feature = "std", error("checksum validation failed"))]
    InvalidChecksum,
}

impl From<TBytesError> for FrameError {
    /// Wraps [`TBytesError`] with [`FrameError`].
    fn from(value: TBytesError) -> Self {
        FrameError::Buffer(value)
    }
}

impl From<TBytesError> for CoreError {
    /// Converts [`TBytesError`] into [`CoreError`].
    ///
    /// [`TBytesError`] be wrapped internally with [`FrameError`] and then passed to
    /// [`CoreError::Frame`].
    fn from(value: TBytesError) -> Self {
        Self::Frame(value.into())
    }
}

impl From<FrameError> for CoreError {
    /// Converts [`FrameError`] into [`CoreError`].
    fn from(value: FrameError) -> Self {
        Self::Frame(value)
    }
}

impl From<MessageError> for CoreError {
    /// Converts [`MessageError`] into [`CoreError`].
    fn from(value: MessageError) -> Self {
        Self::Message(value)
    }
}
