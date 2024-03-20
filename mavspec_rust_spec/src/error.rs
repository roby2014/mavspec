//! # MAVLib errors

use crate::types::{MavLinkVersion, MessageId};

/// Errors related to MAVLink message specification and encoding/decoding.
///
/// All errors except [`SpecError::InvalidV1PayloadSize`] are related to wrong user input like
/// incorrectly chosen MAVLink dialect or protocol version.
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
    /// Invalid size of the `MAVLink 1` payload.
    InvalidV1PayloadSize {
        /// Actual payload size in bytes.
        actual: usize,
        /// Expected payload size.
        expected: usize,
    },
}
