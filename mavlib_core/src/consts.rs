//! # Constants

/// `MAVLink 1` packet start marker value.
///
/// # Links
///
/// * [`MavSTX::V1`](crate::stx::MavSTX::V1).
pub const STX_V1: u8 = 0xFE;
/// `MAVLink 2` packet start marker value.
///
/// # Links
///
/// * [`MavSTX::V2`](crate::stx::MavSTX::V2).
pub const STX_V2: u8 = 0xFD;

/// Minimum size of a MAVLink header (regardless of protocol).
pub const HEADER_MIN_SIZE: usize = HEADER_V1_SIZE;
/// Maximum size of a MAVLink header (regardless of protocol).
pub const HEADER_MAX_SIZE: usize = HEADER_V2_SIZE;
/// Size of the `MAVLink 1` header in bytes.
///
/// `MAVLink 1` header have the following format:
///
/// | Field            | Size in bytes |
/// |------------------|---------------|
/// | `magic` byte     | 1             |
/// | `payload_length` | 1             |
/// | `sequence`       | 1             |
/// | `system_id`      | 1             |
/// | `component_id`   | 1             |
/// | `message_id`     | 1             |
///
/// # Links
///
/// * [MAVLink 1 packet format](https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format).
pub const HEADER_V1_SIZE: usize = 6;
/// Size of the `MAVLink 2` header in bytes.
///
/// `MAVLink 2` header have the following format:
///
/// | Field            | Size in bytes |
/// |------------------|---------------|
/// | `magic` byte     | 1             |
/// | `incompat_flags` | 1             |
/// | `compat_flags`   | 1             |
/// | `payload_length` | 1             |
/// | `sequence`       | 1             |
/// | `system_id`      | 1             |
/// | `component_id`   | 1             |
/// | `message_id`     | 3             |
///
/// # Links
///
/// * [MAVLink 2 packet format](https://mavlink.io/en/guide/serialization.html#v1_packet_format).
pub const HEADER_V2_SIZE: usize = 10;

/// Size of MAVLink checksum in bytes.
pub const CHECKSUM_SIZE: usize = 2;

/// `MAVLink 2` "message is signed" incompatibility flag.
///
/// # Links
///
/// * `MAVLINK_IFLAG_SIGNED` field in [MAVLink 2 incompatibility flags](https://mavlink.io/en/guide/serialization.html#incompat_flags)
pub const MAVLINK_IFLAG_SIGNED: u8 = 0x01;

/// `MAVLink 2` signature link ID length in bytes.
///
/// # Links
///
/// * [`Signature`](crate::signature::Signature)
/// * `link id` field in [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html)
pub const SIGNATURE_LINK_ID_LENGTH: usize = 1;
/// `MAVLink 2` signature timestamp length in bytes.
///
/// # Links
///
///  * [`Signature`](crate::signature::Signature)
///  * `tm.timestamp` field in [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html)
pub const SIGNATURE_TIMESTAMP_LENGTH: usize = 6;
/// `MAVLink 2` signature value length in bytes.
///
/// # Links
///
///  * [`Signature`](crate::signature::Signature)
///  * `signature` field in [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html)
pub const SIGNATURE_VALUE_LENGTH: usize = 6;

/// `MAVLink 2` signature length in bytes.
///
/// # Links
///
///  * [`Signature`](crate::signature::Signature)
///  * [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html)
pub const SIGNATURE_LENGTH: usize =
    SIGNATURE_LINK_ID_LENGTH + SIGNATURE_TIMESTAMP_LENGTH + SIGNATURE_VALUE_LENGTH;
