//! # MAVLib errors

use tbytes::errors::TBytesError;

use crate::types::{MavLinkVersion, MessageId};

/// Errors related to MAVLink message specification and encoding/decoding.
///
/// All errors except [`SpecError::Payload`] are related to wrong user input like incorrectly
/// chosen MAVLink dialect or protocol version.
///
/// [`SpecError::Payload`] is a wrapper around [`PayloadError`] which indicate that error is related
/// to message encoding/decoding from or into [`Payload`](crate::Payload). These errors indicate
/// that client or downstream libraries incorrectly constructed MAVLink payload or attempted to
/// decode `MAVLink 1` messages from `MAVLink 2` payload (due to zero trailing bytes truncation or
/// field extensions which are not supported for `MAVLink 1`).
#[derive(Clone, Debug)]
pub enum SpecError {
    /// MAVLink version is not supported.
    UnsupportedMavLinkVersion {
        /// Actual requested version.
        actual: MavLinkVersion,
        /// Minimum supported MAVLink version,
        minimal: MavLinkVersion,
    },
    /// MAVLink message with specified ID is not in dialect.
    NotInDialect(MessageId),
    /// Error during conversion to MAVLink enum.
    InvalidEnumValue {
        /// Enum name.
        enum_name: &'static str,
    },
    /// Errors related to message encoding/decoding from and into MAVLink
    /// [`Payload`](crate::Payload).
    Payload(PayloadError),
}

/// Errors related to invalid MAVLink [`Payload`](crate::Payload), always wrapped by
/// [`SpecError::Payload`].
///
/// These errors indicate that client or downstream libraries incorrectly constructed MAVLink payload
/// or attempted to decode `MAVLink 1` messages from `MAVLink 2` payload.
///
/// This error is never returned directly but only as wrapped by [`SpecError::Payload`].
#[derive(Clone, Debug)]
pub enum PayloadError {
    /// Invalid size of the provided payload.
    InvalidV1PayloadSize {
        /// Actual payload size in bytes.
        actual: usize,
        /// Expected payload size.
        expected: usize,
    },
    /// Error during encoding/decoding payload buffer.
    ///
    /// This error is internal to MAVSpec and potentially indicates a bug in implementation. You
    /// should not rely on this error since later versions may fix the implementation in way that
    /// such error may not occur at all.
    #[deprecated]
    Buffer(TBytesError),
}

impl From<PayloadError> for SpecError {
    fn from(value: PayloadError) -> Self {
        Self::Payload(value)
    }
}

impl From<TBytesError> for SpecError {
    /// Wrap array buffer errors as message processing errors.
    fn from(value: TBytesError) -> Self {
        #[allow(deprecated)]
        Self::Payload(PayloadError::Buffer(value))
    }
}
