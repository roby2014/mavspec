//! `MAVLink 1` message frame.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use super::{MavLinkMessagePayload, MavLinkVersion, MavSTX};

/// Minimum size of a `MAVLink 1` packet which makes sense.
pub const MIN_PACKET_SIZE: usize = 8;

/// Errors related to [`MavLinkFrameV1`] processing and validation.
pub enum MavLinkFrameV1ValidationError {
    /// Invalid `magic` byte [`MavSTX`] value.
    InvalidSTX(MavSTX),
    /// MavLink version of a `payload` does not matches `magic` byte.
    InvalidPayloadVersion(MavLinkVersion),
}

/// Errors which may happen during frame decoding.
pub enum MavLinkFrameV1DecodeError {
    /// Packet is too small to decode. See [`MIN_PACKET_SIZE`].
    PacketIsTooSmall(usize),
    /// Packet is too small to contain `payload`.
    IncompletePacket {
        /// Length of the `payload` in bytes.
        payload_length: usize,
        /// Length of the packet in bytes.
        packet_length: usize,
    },
}

/// `MAVLink 1` message frame.
///
/// All fields are intentionally made accessible for the client. Use [`MavLinkFrameV1::validate`] to
/// validate consistency of a frame.
///
/// See [MAVLink 1 packet format](https://mavlink.io/en/guide/serialization.html#v1_packet_format).
#[derive(Clone, Debug)]
pub struct MavLinkFrameV1 {
    /// Packet start marker (magic byte).
    ///
    /// Protocol-specific start-of-text (STX) marker used to indicate the beginning of a new packet.
    ///
    /// See [`MavSTX`].
    pub magic: MavSTX,
    /// Payload length.
    ///
    /// Indicates length of the following `payload` section (fixed for a particular message).
    pub payload_length: u8,
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
    pub message_id: u8,
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
}

impl Default for MavLinkFrameV1 {
    /// Instantiates [`MavLinkFrameV1`] with default values.
    ///
    /// STX is set to match `MAVLink 1` protocol version.
    fn default() -> Self {
        Self {
            magic: MavSTX::MavLink1,
            payload_length: 0,
            sequence: 0,
            system_id: 0,
            component_id: 0,
            message_id: 0,
            payload: Default::default(),
            checksum: 0,
        }
    }
}

impl MavLinkFrameV1 {
    /// Decodes [`MavLinkFrameV1`] from slice of bytes.
    ///
    /// # Errors
    ///
    /// * [`MavLinkFrameV1DecodeError::PacketIsTooSmall`] if packet is unreasonably small to decode.
    /// * [`MavLinkFrameV1DecodeError::IncompletePacket`] if packet is too small to contain `payload`.
    pub fn from_slice(packet: &[u8]) -> Result<Self, MavLinkFrameV1DecodeError> {
        let packet_length = packet.len();
        if packet_length < MIN_PACKET_SIZE {
            return Err(MavLinkFrameV1DecodeError::PacketIsTooSmall(packet_length));
        }

        let magic: MavSTX = packet[0].into();
        let payload_length: u8 = packet[1];
        let sequence: u8 = packet[2];
        let system_id: u8 = packet[3];
        let component_id: u8 = packet[4];
        let message_id: u8 = packet[5];
        let payload_bytes: &[u8] = &packet[6..packet_length - 2];
        let checksum: u16 =
            u16::from_le_bytes([packet[packet_length - 2], packet[packet_length - 1]]);

        // Return error if packet is too small to contain `payload`
        if payload_length as usize != payload_bytes.len() {
            return Err(MavLinkFrameV1DecodeError::IncompletePacket {
                payload_length: payload_length as usize,
                packet_length,
            });
        }

        let payload =
            MavLinkMessagePayload::new(message_id as u32, payload_bytes, MavLinkVersion::V1);

        Ok(Self {
            magic,
            payload_length,
            sequence,
            system_id,
            component_id,
            message_id,
            payload,
            checksum,
        })
    }

    /// Validates consistency of [`MavLinkFrameV1`].
    pub fn validate(&self) -> Result<(), MavLinkFrameV1ValidationError> {
        // The `magic` byte should be set to `MAVLink 1`
        match self.magic {
            MavSTX::MavLink1 => {}
            m => return Err(MavLinkFrameV1ValidationError::InvalidSTX(m)),
        }

        // Payload version should be `MAVLink 1`
        match self.payload.version() {
            MavLinkVersion::V1 => {}
            v => return Err(MavLinkFrameV1ValidationError::InvalidPayloadVersion(v)),
        }

        // Validate checksum
        self.validate_checksum()?;

        Ok(())
    }

    /// Validates `checksum`.
    pub fn validate_checksum(&self) -> Result<(), MavLinkFrameV1ValidationError> {
        // TODO: implement checksum validation
        Ok(())
    }
}
