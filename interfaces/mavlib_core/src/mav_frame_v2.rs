//! `MAVLink 2` message packet.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use super::{MavLinkMessagePayload, MavLinkVersion, MavSTX};

/// Minimum size of a `MAVLink 2` packet which makes sense.
pub const MIN_PACKET_SIZE: usize = 12;
/// Index of a byte of a packet where message payload starts.
pub const PAYLOAD_START_BYTE_IDX: usize = 10;
/// MAVLink incompatibility flag: message is signed.
///
/// See: [MAVLink 2 incompatibility flags](https://mavlink.io/en/guide/serialization.html#incompat_flags).
pub const MAVLINK_IFLAG_SIGNED: u8 = 0x01;
/// Length of `MAVLink 2` message signature.
///
/// See: [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html).
pub const MAVLINK_SIGNATURE_LENGTH: usize = 13;
/// Maximum possible value for `MAVLink 2` message `ID`.
///
/// We store message `ID` (which is a 3-bytes unsigned integer) as [`u32`].
pub const MAX_MESSAGE_ID: u32 = 2u32.pow(24);

/// Errors related to [`MavLinkFrameV2`] processing and validation.
pub enum MavLinkFrameV2ValidationError {
    /// Invalid `magic` byte [`MavSTX`] value.
    InvalidSTX(MavSTX),
    /// MavLink version of a `payload` does not matches `magic` byte.
    InconsistentMavLinkVersion(MavSTX, MavLinkVersion),
    /// MAVlink message `ID` is out of bounds.
    ///
    /// See [`MAX_MESSAGE_ID`].
    MessageIdIsOutOfBounds(u32),
}

/// Type alias for `MAVLink 2` message signature.
///
/// See: [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html).
pub type MavLinkFrameV2Signature = [u8; MAVLINK_SIGNATURE_LENGTH];

/// Errors which may happen during frame decoding.
pub enum MavLinkFrameV2DecodeError {
    /// Packet is too small to decode. See [`MIN_PACKET_SIZE`].
    PacketIsTooSmall(usize),
    /// Packet is too small to contain `payload`.
    IncompletePacket {
        /// Length of the `payload` in bytes.
        payload_length: usize,
        /// Length of the packet in bytes.
        packet_length: usize,
    },
    /// Incompatibility flag is set for `signature` but signature is missing or has a wrong size.
    MalformedSignature {
        /// Byte position where `signature` starts.
        starts: usize,
        /// Length of the packet in bytes.
        packet_length: usize,
    },
}

/// `MAVLink 2` message frame.
///
/// All fields are intentionally made accessible for the client. Use [`MavLinkFrameV2::validate`] to
/// validate consistency of a frame.
///
/// See [MAVLink 2 packet format](https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format).
#[derive(Clone, Debug)]
pub struct MavLinkFrameV2 {
    /// Packet start marker (`magic` byte).
    ///
    /// Protocol-specific start-of-text (STX) marker used to indicate the beginning of a new packet.
    /// Any system that does not understand protocol version will skip the packet.
    ///
    /// See [`MavSTX`].
    pub magic: MavSTX,
    /// Payload length.
    ///
    /// Indicates length of the following `payload` section. This may be affected by payload truncation.
    pub payload_length: u8,
    /// Incompatibility Flags.
    ///
    /// Flags that must be understood for MAVLink compatibility (implementation discards packet if
    /// it does not understand flag).
    ///
    /// See: [MAVLink 2 incompatibility flags](https://mavlink.io/en/guide/serialization.html#incompat_flags).
    pub incompat_flags: u8,
    /// Compatibility Flags.
    ///
    /// Flags that can be ignored if not understood (implementation can still handle packet even if
    /// it does not understand flag).
    ///
    /// See: [MAVLink 2 compatibility flags](https://mavlink.io/en/guide/serialization.html#compat_flags).
    pub compat_flags: u8,
    /// Packet sequence number.
    ///
    /// Used to detect packet loss. Components increment value for each message sent.
    pub sequence: u8,
    /// System `ID`.
    ///
    /// `ID` of system (vehicle) sending the message. Used to differentiate systems on network.
    ///
    /// > Note that the broadcast address 0 may not be used in this field as it is an invalid source
    /// > address.
    pub system_id: u8,
    /// Component `ID`.
    ///
    /// `ID` of component sending the message. Used to differentiate components in a system (e.g.
    /// autopilot and a camera). Use appropriate values in
    /// [MAV_COMPONENT](https://mavlink.io/en/messages/common.html#MAV_COMPONENT).
    ///
    /// > Note that the broadcast address `MAV_COMP_ID_ALL` may not be used in this field as it is
    /// > an invalid source address.
    pub component_id: u8,
    /// Message `ID`.
    ///
    /// `ID` of message type in payload.
    ///
    /// Used to decode data back into message object.
    pub message_id: u32,
    /// Payload data.
    ///
    /// Message data. Content depends on message type (i.e. `message_id`).
    pub payload: MavLinkMessagePayload,
    /// Checksum (low byte, high byte).
    ///
    /// `CRC-16/MCRF4XX` [checksum](https://mavlink.io/en/guide/serialization.html#checksum) for
    /// message (excluding magic byte).
    ///
    /// Includes [CRC_EXTRA](https://mavlink.io/en/guide/serialization.html#crc_extra) byte.
    ///
    /// See: [CRC-16/MCRF4XX](https://ww1.microchip.com/downloads/en/AppNotes/00752a.pdf) (PDF).
    pub checksum: u16,
    /// Signature.
    ///
    /// Signature to ensure the link is tamper-proof.
    pub signature: Option<MavLinkFrameV2Signature>,
}

impl Default for MavLinkFrameV2 {
    /// Instantiates [`MavLinkFrameV2`] with default values.
    ///
    /// STX is set to match `MAVLink 2` protocol version.  
    fn default() -> Self {
        Self {
            magic: MavSTX::MavLink2,
            payload_length: 0,
            incompat_flags: 0,
            compat_flags: 0,
            sequence: 0,
            system_id: 0,
            component_id: 0,
            message_id: 0,
            payload: Default::default(),
            checksum: 0,
            signature: None,
        }
    }
}

impl MavLinkFrameV2 {
    /// Decodes [`MavLinkFrameV2`] from slice of bytes.
    ///
    /// # Errors
    ///
    /// * [`MavLinkFrameV2DecodeError::PacketIsTooSmall`] if packet is unreasonably small to decode.
    /// * [`MavLinkFrameV2DecodeError::IncompletePacket`] if packet is too small to contain `payload`.
    /// * [`MavLinkFrameV2DecodeError::MalformedSignature`] if `signature` is incomplete or has a wrong size.
    pub fn from_slice(packet: &[u8]) -> Result<Self, MavLinkFrameV2DecodeError> {
        let packet_length = packet.len();

        // Return error if packet is unreasonably small
        if packet_length < MIN_PACKET_SIZE {
            return Err(MavLinkFrameV2DecodeError::PacketIsTooSmall(packet_length));
        }

        let magic: MavSTX = packet[0].into();
        let payload_length: u8 = packet[1];
        let incompat_flags: u8 = packet[2];
        let compat_flags: u8 = packet[3];
        let sequence: u8 = packet[4];
        let system_id: u8 = packet[5];
        let component_id: u8 = packet[6];
        let message_id: u32 = u32::from_le_bytes([packet[7], packet[8], packet[9], 0]);

        // Calculate where `payload` ends and `checksum` starts
        let checksum_start_idx = PAYLOAD_START_BYTE_IDX + payload_length as usize;
        // Return error if packet is too small to contain `payload`
        if packet_length < checksum_start_idx {
            return Err(MavLinkFrameV2DecodeError::IncompletePacket {
                payload_length: payload_length as usize,
                packet_length,
            });
        }

        // Read `payload` and `checksum`
        let payload_bytes: &[u8] = &packet[PAYLOAD_START_BYTE_IDX..checksum_start_idx];
        let checksum: u16 =
            u16::from_le_bytes([packet[checksum_start_idx], packet[checksum_start_idx + 1]]);

        // Read `signature`
        let signature: Option<MavLinkFrameV2Signature> =
            if incompat_flags & MAVLINK_IFLAG_SIGNED == MAVLINK_IFLAG_SIGNED {
                let signature_start_idx = checksum_start_idx + 2;

                // Return error if signature has invalid size
                if packet_length - signature_start_idx != MAVLINK_SIGNATURE_LENGTH {
                    return Err(MavLinkFrameV2DecodeError::MalformedSignature {
                        starts: signature_start_idx,
                        packet_length,
                    });
                }

                // Start from zero bytes
                let mut signature: MavLinkFrameV2Signature = [0u8; MAVLINK_SIGNATURE_LENGTH];
                // Copy the rest of the packet as `signature`
                signature.copy_from_slice(&packet[signature_start_idx..packet_length]);

                Some(signature)
            } else {
                None
            };

        // Construct payload
        let payload = MavLinkMessagePayload::new(message_id, payload_bytes, MavLinkVersion::V1);

        Ok(Self {
            magic,
            payload_length,
            incompat_flags,
            compat_flags,
            sequence,
            system_id,
            component_id,
            message_id,
            payload,
            checksum,
            signature,
        })
    }

    /// Validates consistency of a frame.
    pub fn validate(&self) -> Result<(), MavLinkFrameV2ValidationError> {
        // The `magic` byte should be set to `MAVLink 2`
        match self.magic {
            MavSTX::MavLink2 => {}
            _ => return Err(MavLinkFrameV2ValidationError::InvalidSTX(self.magic)),
        }

        // Payload version should be `MAVLink 2`
        match self.payload.version() {
            MavLinkVersion::V2 => {}
            _ => {
                return Err(MavLinkFrameV2ValidationError::InconsistentMavLinkVersion(
                    self.magic,
                    self.payload.version(),
                ))
            }
        }

        // Message ID should be 24-bit unsigned integer
        if self.message_id > MAX_MESSAGE_ID {
            return Err(MavLinkFrameV2ValidationError::MessageIdIsOutOfBounds(
                self.message_id,
            ));
        }

        // Validate checksum
        self.validate_checksum()?;

        Ok(())
    }

    /// Validates `checksum`.
    pub fn validate_checksum(&self) -> Result<(), MavLinkFrameV2ValidationError> {
        // TODO: implement checksum validation
        Ok(())
    }
}
