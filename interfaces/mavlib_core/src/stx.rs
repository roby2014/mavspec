//! # MAVLink packet start marker
//!
//! [`MavSTX`] represents a protocol-specific start-of-text (STX) marker used to indicate the
//! beginning of a new packet.
//!
//! Any system that does not understand protocol version will skip the packet.
//!
//! See:
//! * [MAVLink 1 Packet Format](https://mavlink.io/en/guide/serialization.html#v1_packet_format).
//! * [MAVLink 2 Packet Format](https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format).

/// `MAVLink 1` packet start marker value.
///
/// See [`MavSTX::MavLink1`].
pub const STX_MAVLINK_1: u8 = 0xFE;
/// `MAVLink 2` packet start marker value.
///
/// See [`MavSTX::MavLink2`].
pub const STX_MAVLINK_2: u8 = 0xFD;

/// Packet start marker.
///
/// Protocol-specific start-of-text (STX) marker used to indicate the beginning of a new packet.
///
/// Any system that does not understand protocol version will skip the packet.
///
/// See:
/// * [MAVLink 1 Packet Format](https://mavlink.io/en/guide/serialization.html#v1_packet_format).
/// * [MAVLink 2 Packet Format](https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format).
#[derive(Clone, Copy, Debug)]
pub enum MavSTX {
    /// Designates `MAVLink 1` protocol, equals to [`STX_MAVLINK_1`].
    MavLink1,
    /// Designates `MAVLink 2` protocol, equals to [`STX_MAVLINK_2`].
    MavLink2,
    /// Unknown protocol.
    Unknown(u8),
}

impl Default for MavSTX {
    /// Creates [`MavSTX`] with default value.
    ///
    /// We assume unknown protocol with zero marker.
    fn default() -> Self {
        Self::Unknown(0)
    }
}

impl From<MavSTX> for u8 {
    /// Converts from `u8` into [`MavSTX`].
    fn from(value: MavSTX) -> Self {
        match value {
            MavSTX::MavLink1 => STX_MAVLINK_1,
            MavSTX::MavLink2 => STX_MAVLINK_2,
            MavSTX::Unknown(unknown) => unknown,
        }
    }
}

impl From<u8> for MavSTX {
    /// Converts from `u8` into [`MavSTX`].
    fn from(value: u8) -> Self {
        match value {
            STX_MAVLINK_1 => MavSTX::MavLink1,
            STX_MAVLINK_2 => MavSTX::MavLink2,
            unknown => MavSTX::Unknown(unknown),
        }
    }
}
