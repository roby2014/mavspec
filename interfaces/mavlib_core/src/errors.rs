//! # MAVLib errors

use tbytes::errors::TBytesError;

use super::version::MavLinkVersion;

// Reexport `MAVLink 1` frame errors
pub use super::mav_frame_v1::{MavLinkFrameV1DecodeError, MavLinkFrameV1ValidationError};
// Reexport `MAVLink 2` frame errors
pub use super::mav_frame_v2::{MavLinkFrameV2DecodeError, MavLinkFrameV2ValidationError};

/// Errors related to MAVLink message encoding/decoding.
#[derive(Debug, Clone, Copy)]
pub enum MavLinkMessageProcessingError {
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

impl From<TBytesError> for MavLinkMessageProcessingError {
    /// Wrap array buffer errors as message processing errors.
    fn from(value: TBytesError) -> Self {
        Self::BufferError(value)
    }
}
