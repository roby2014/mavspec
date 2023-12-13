//! # Constants

/// Maximum size of a payload. Payloads of greater size in most cases will be truncated or cause
/// errors.
pub const PAYLOAD_MAX_SIZE: usize = 255;

/// Maximum size of [MessageId](crate::types::MessageId) for `MAVLink 1` protocol.
pub const MESSAGE_ID_V1_MAX: u32 = u8::MAX as u32;

/// Maximum size of [MessageId](crate::types::MessageId) for `MAVLink 2` protocol.
pub const MESSAGE_ID_V2_MAX: u32 = 2u32.pow(24);
