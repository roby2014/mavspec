//! # Common constants

/// `MAVLink 1` packet start marker value.
///
/// See [`MavSTX::MavLink1`](crate::stx::MavSTX::MavLink1).
pub const STX_MAVLINK_1: u8 = 0xFE;
/// `MAVLink 2` packet start marker value.
///
/// See [`MavSTX::MavLink2`](crate::stx::MavSTX::MavLink2).
pub const STX_MAVLINK_2: u8 = 0xFD;

/// Minimum size of `MAVLink` header (regardless of protocol).
pub const MAVLINK_MIN_HEADER_SIZE: usize = MAVLINK_V1_HEADER_SIZE;
/// Size of the `MAVLink 1` header in bytes.
pub const MAVLINK_V1_HEADER_SIZE: usize = 8;
/// Size of the `MAVLink 2` header in bytes.
pub const MAVLINK_V2_HEADER_SIZE: usize = 10;

/// Size of `MAVLink` checksum in bytes.
pub const MAVLINK_CHECKSUM_SIZE: usize = 2;

/// `MAVLink 2` "message is signed" incompatibility flag.
///
/// See `MAVLINK_IFLAG_SIGNED` field in [MAVLink 2 incompatibility flags](https://mavlink.io/en/guide/serialization.html#incompat_flags)
pub const MAVLINK_V2_IFLAG_SIGNED: u8 = 0x01;

/// `MAVLink 2` signature link ID length in bytes.
///
/// See:
///  * [`MavLinkFrameV2Signature`](crate::frame::MavLinkV2Signature)
///  * `link id` field in [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html)
pub const MAVLINK_V2_SIGNATURE_LINK_ID_LENGTH: usize = 1;
/// `MAVLink 2` signature timestamp length in bytes.
///
/// See:
///  * [`MavLinkFrameV2Signature`](crate::frame::MavLinkV2Signature)
///  * `tm.timestamp` field in [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html)
pub const MAVLINK_V2_SIGNATURE_TIMESTAMP_LENGTH: usize = 6;
/// `MAVLink 2` signature value length in bytes.
///
/// See:
///  * [`MavLinkFrameV2Signature`](crate::frame::MavLinkV2Signature)
///  * `signature` field in [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html)
pub const MAVLINK_V2_SIGNATURE_VALUE_LENGTH: usize = 6;

/// `MAVLink 2` signature length in bytes.
///
/// See:
///  * [`MavLinkFrameV2Signature`](crate::frame::MavLinkV2Signature)
///  * [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html)
pub const MAVLINK_V2_SIGNATURE_LENGTH: usize = MAVLINK_V2_SIGNATURE_LINK_ID_LENGTH
    + MAVLINK_V2_SIGNATURE_TIMESTAMP_LENGTH
    + MAVLINK_V2_SIGNATURE_VALUE_LENGTH;
