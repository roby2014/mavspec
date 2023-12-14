//! # Errors
//!
//! This errors used in `mavlib_core`.
//!
//! The top-level error is [`CoreError`]. Library API returns versions of this error possibly wrapping other types of
//! errors.
//!
//! We also re-export errors from [`mavlib_spec`] crate to provide a full specification of MAVLink-related errors.

use tbytes::errors::TBytesError;

#[cfg(feature = "std")]
use thiserror::Error;

// Re-export `mavlib_spec` errors.
#[doc(no_inline)]
pub use mavlib_spec::MessageError;

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
#[derive(Clone, Debug)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum FrameError {
    /// MAVLink header is too small.
    #[cfg_attr(feature = "std", error("header is too small"))]
    HeaderIsTooSmall,
    /// `MAVLink 1` header is too small.
    #[cfg_attr(feature = "std", error("MAVLink 1 header is too small"))]
    HeaderV1IsTooSmall,
    /// `MAVLink 2` header is too small.
    #[cfg_attr(feature = "std", error("MAVLink 2 header is too small"))]
    HeaderV2IsTooSmall,
    /// Incorrect MAVLink version.
    #[cfg_attr(feature = "std", error("invalid MAVLink version"))]
    InvalidMavLinkVersion,
    /// `MAVLink 1` version is out of bounds.
    #[cfg_attr(feature = "std", error("`MAVLink 1` version is out of bounds"))]
    MavLinkVersionV1OutOfBounds,
    /// `MAVLink 2` version is out of bounds.
    #[cfg_attr(feature = "std", error("`MAVLink 2` version is out of bounds"))]
    MavLinkVersionV2OutOfBounds,
    /// Inconsistent `MAVLink 1` header: `MAVLink 2` fields are defined.
    #[cfg_attr(feature = "std", error("inconsistent MAVLink 1 header"))]
    InconsistentV1Header,
    /// Inconsistent `MAVLink 2` header: `MAVLink 2` fields are not defined.
    #[cfg_attr(feature = "std", error("inconsistent MAVLink 2 header"))]
    InconsistentV2Header,
    /// `MAVLink 1` packet body is too small.
    #[cfg_attr(feature = "std", error("MAVLink 1 packet body is too small"))]
    PacketV1BodyIsTooSmall,
    /// `MAVLink 2` packet body is too small.
    #[cfg_attr(feature = "std", error("MAVLink 2 packet body is too small"))]
    PacketV2BodyIsTooSmall,
    /// `MAVLink 2` signature is too small.
    #[cfg_attr(feature = "std", error("MAVLink 2 signature is too small"))]
    SignatureIsTooSmall,
    /// `MAVLink 2` signature is missing but [`MAVLINK_IFLAG_SIGNED`](crate::consts::MAVLINK_IFLAG_SIGNED) is set..
    #[cfg_attr(
        feature = "std",
        error("MAVLink 2 signature is missing but `MAVLINK_IFLAG_SIGNED` is set")
    )]
    SignatureIsMissing,
    /// Buffer error.
    #[cfg_attr(feature = "std", error("MAVLink 2 signature is too small"))]
    Buffer(TBytesError),
    /// Upon calculation CRC does not match received [MavLinkFrame::checksum](crate::Frame::checksum).
    #[cfg_attr(feature = "std", error("checksum validation failed"))]
    InvalidChecksum,

    /// Missing [`HeaderBuilder`](crate::protocol::header::HeaderConf) field when building a
    /// [`Header`](crate::protocol::header::Header).
    #[cfg_attr(
        feature = "std",
        error("can't build header since field `{0}` is missing")
    )]
    MissingHeaderField(String),
    /// Missing [`FrameBuilder`](crate::protocol::frame::FrameConf) field when building a
    /// [`Frame`](crate::protocol::frame::Frame).
    #[cfg_attr(
    feature = "std",
    error("can't build frame since field `{0}` is missing")
    )]
    MissingFrameField(String),
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
