//! # MAVLib errors

use tbytes::errors::TBytesError;

use crate::types::MavLinkVersion;

/// Errors related to MAVLink message encoding/decoding.
#[derive(Debug, Clone, Copy)]
pub enum MessageError {
    /// MAVLink version is not supported.
    UnsupportedMavLinkVersion {
        /// Actual requested version.
        actual: MavLinkVersion,
        /// Minimum supported MAVLink version,
        minimal: MavLinkVersion,
    },
    /// MAVLink message with specified ID is not supported.
    UnsupportedMessageId(u32),
    /// Invalid size of the provided payload.
    InvalidPayloadSize {
        /// Actual payload size in bytes.
        actual: usize,
        /// Expected payload size.
        expected: usize,
    },
    /// Error during decoding payload buffer.
    BufferError(TBytesError),
}

impl From<TBytesError> for MessageError {
    /// Wrap array buffer errors as message processing errors.
    fn from(value: TBytesError) -> Self {
        Self::BufferError(value)
    }
}
