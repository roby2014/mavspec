//! # Tiny types and type aliases
//!
//! Type aliases and tiny types (that require a few lines of code) used across the `mavlib_core` library.

use crate::consts::{
    HEADER_V1_SIZE, HEADER_V2_SIZE, SIGNATURE_LENGTH, SIGNATURE_TIMESTAMP_LENGTH,
    SIGNATURE_VALUE_LENGTH,
};

/// MAVLink message ID regardless of protocol.
pub type MessageId = u32;

/// MAVLink packet checksum.
///
/// MAVLink checksum is encoded with little endian (low byte, high byte).
///
/// See:
///  * [`MavLinkFrame::checksum`](crate::frame::Frame::checksum).
///  * [`MavLinkFrame::calculate_crc`](crate::frame::Frame::calculate_crc).
pub type Checksum = u16;

/// MAVLink extra CRC.
///
/// See:
///  * [`MavLinkFrame::checksum`](crate::frame::Frame::checksum).
///  * [`MavLinkFrame::calculate_crc`](crate::frame::Frame::calculate_crc).
///  * [CRC_EXTRA calculation](https://mavlink.io/en/guide/serialization.html#crc_extra) in MAVLink docs.
pub type ExtraCrc = u8;

/// `MAVLink 1` header as array of bytes.
pub type HeaderV1Bytes = [u8; HEADER_V1_SIZE];
/// `MAVLink 2` header as array of bytes.
pub type HeaderV2Bytes = [u8; HEADER_V2_SIZE];

/// `MAVLink 2` signature as array of bytes.
///
/// See:
///  * [`MavLinkFrameV2Signature`](crate::signature::Signature).
///  * [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html).
pub type SignatureBytes = [u8; SIGNATURE_LENGTH];
/// `MAVLink 2` signature link ID.
///
/// See:
///  * [`MavLinkFrameV2Signature`](crate::signature::Signature).
///  * `link id` field in [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html).
pub type SignatureLinkId = u8;
/// `MAVLink 2` signature timestamp.
///
/// See:
///  * [`MavLinkFrameV2Signature`](crate::signature::Signature).
///  * `tm.timestamp` field in [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html).
pub type SignatureTimestampBytes = [u8; SIGNATURE_TIMESTAMP_LENGTH];
/// `MAVLink 2` signature value.
///
/// See:
///  * [`MavLinkFrameV2Signature`](crate::signature::Signature).
///  * `signature` field in [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html).
pub type SignatureValueBytes = [u8; SIGNATURE_VALUE_LENGTH];
