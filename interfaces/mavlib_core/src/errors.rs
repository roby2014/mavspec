//! # MAVLib errors

use tbytes::errors::TBytesError;

use super::version::MavLinkVersion;

/// Errors related to `MAVLink` frame decoding.
#[derive(Debug, Clone, Copy)]
pub enum MavLinkFrameDecodingError {
    /// `MAVLink` header is too small.
    HeaderIsTooSmall,
    /// `MAVLink 1` header is too small.
    V1HeaderIsTooSmall,
    /// `MAVLink 2` header is too small.
    V2HeaderIsTooSmall,
    /// Incorrect `MAVLink` version.
    InvalidMavLinkVersion,
    /// Inconsistent `MAVLink 1` header: `MAVLink 2` fields are defined.
    InconsistentV1Header,
    /// Inconsistent `MAVLink 2` header: `MAVLink 2` fields are not defined.
    InconsistentV2Header,
    /// Error while reading from buffer.
    BufferDecodeError(TBytesError),
    /// `MAVLink 1` packet body is too small.
    V1PacketBodyIsTooSmall,
    /// `MAVLink 2` packet body is too small.
    V2PacketBodyIsTooSmall,
    /// `MAVLink 2` signature is too small.
    V2SignatureIsTooSmall,
}

impl From<TBytesError> for MavLinkFrameDecodingError {
    /// Converts [`TBytesError`] into [`MavLinkFrameDecodingError`].
    fn from(value: TBytesError) -> Self {
        MavLinkFrameDecodingError::BufferDecodeError(value)
    }
}

/// Errors related to `MAVLink` message encoding/decoding.
#[derive(Debug, Clone, Copy)]
pub enum MavLinkMessageProcessingError {
    /// `MAVLink` version is not supported.
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
