//! # Tiny types and type aliases
//!
//! Type aliases and tiny types (that require a few lines of code) used across the `mavlib_spec` library.

/// MAVLink message ID regardless of protocol.
///
/// * For `MAVLink 1` message ID is a 8-bit unsigned integer.
/// * For `MAVLink 2` message ID is a 24-bit unsigned integer.
pub type MessageId = u32;

/// MAVLink extra CRC byte.
///
/// # Links
///
///  * [CRC_EXTRA calculation](https://mavlink.io/en/guide/serialization.html#crc_extra) in MAVLink docs.
pub type ExtraCrc = u8;

/// MAVLink protocol version.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MavLinkVersion {
    /// `MAVLink 1` protocol version.
    #[default]
    V1,
    /// `MAVLink 2` protocol version.
    V2,
}
