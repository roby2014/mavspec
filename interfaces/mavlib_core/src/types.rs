//! # Common types

use crate::consts::{
    MAVLINK_V1_HEADER_SIZE, MAVLINK_V2_HEADER_SIZE, MAVLINK_V2_SIGNATURE_TIMESTAMP_LENGTH,
    MAVLINK_V2_SIGNATURE_VALUE_LENGTH,
};

/// `MAVLink` message ID regardless of protocol.
pub type MavLinkMessageId = u32;

/// `MAVLink` packet checksum.
///
/// `MAVLink` checksum is encoded with little endian (low byte, high byte).
///
/// See [MavLinkFrame::checksum](crate::frame::MavLinkFrame::checksum).
pub type MavLinkChecksum = u16;

/// `MAVLink 1` header as array of bytes.
pub type MavLinkV1Header = [u8; MAVLINK_V1_HEADER_SIZE];
/// `MAVLink 2` header as array of bytes.
pub type MavLinkV2Header = [u8; MAVLINK_V2_HEADER_SIZE];

/// `MAVLink 2` signature link ID.
///
/// See:
///  * [`MavLinkFrameV2Signature`](crate::frame::MavLinkV2Signature)
///  * `link id` field in [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html)
pub type MavLinkV2SignatureLinkId = u8;
/// `MAVLink 2` signature timestamp.
///
/// See:
///  * [`MavLinkFrameV2Signature`](crate::frame::MavLinkV2Signature)
///  * `tm.timestamp` field in [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html)
pub type MavLinkV2SignatureTimestamp = [u8; MAVLINK_V2_SIGNATURE_TIMESTAMP_LENGTH];
/// `MAVLink 2` signature value.
///
/// See:
///  * [`MavLinkFrameV2Signature`](crate::frame::MavLinkV2Signature)
///  * `signature` field in [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html)
pub type MavLinkV2SignatureValue = [u8; MAVLINK_V2_SIGNATURE_VALUE_LENGTH];
