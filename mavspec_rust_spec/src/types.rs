//! # Tiny types and type aliases
//!
//! Type aliases and tiny types (that require a few lines of code) used across the `mavspec_rust_spec` library.

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
pub type CrcExtra = u8;

/// Type used to contain `dialect` identifier specified in MAVLink [XML definitions](https://mavlink.io/en/guide/xml_schema.html).
pub type DialectId = u32;

/// Type used to contain minor dialect `version` specified in MAVLink [XML definitions](https://mavlink.io/en/guide/xml_schema.html).
///
/// Dialect version appears in some messages like [HEARTBEAT](https://mavlink.io/en/messages/common.html#HEARTBEAT). In
/// such cases it is usually not directly set by user.
pub type DialectVersion = u8;

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

/// MAVLink protocol version trait.
pub trait MavLinkProtocolVersion {
    /// Version of MAVLink protocol.
    fn version(&self) -> MavLinkVersion;
}

impl MavLinkProtocolVersion for MavLinkVersion {
    #[inline]
    fn version(&self) -> MavLinkVersion {
        *self
    }
}

/// `MAVLink 1` protocol version marker.
#[derive(Clone, Copy, Debug, Default)]
pub struct MavLinkV1();

/// `MAVLink 2` protocol version marker.
#[derive(Clone, Copy, Debug, Default)]
pub struct MavLinkV2();

impl MavLinkProtocolVersion for MavLinkV1 {
    fn version(&self) -> MavLinkVersion {
        MavLinkVersion::V1
    }
}

impl MavLinkProtocolVersion for MavLinkV2 {
    fn version(&self) -> MavLinkVersion {
        MavLinkVersion::V2
    }
}
