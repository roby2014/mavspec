//! # MAVLink frame

use crc_any::CRCu16;

use crate::consts::{CHECKSUM_SIZE, SIGNATURE_LENGTH};
use crate::errors::{FrameError, Result};
use crate::io::Read;
use crate::protocol::header::{Header, HeaderV2Fields};
use crate::protocol::signature::Signature;
use crate::protocol::{Checksum, DialectSpec, ExtraCrc, MavLinkVersion, MessageId, Payload};

/// MAVLink frame.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Frame {
    /// Generic MAVLink header.
    header: Header,
    /// Payload data.
    payload: Payload,
    /// MAVLink packet checksum.
    checksum: Checksum,
    /// Signature.
    signature: Option<Signature>,
}

impl Frame {
    /// Generic MAVLink header.
    ///
    /// See [`Header`].
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Payload data.
    ///
    /// Message data. Content depends on message type (i.e. `message_id`).
    ///
    /// See [`Payload`].
    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    /// MAVLink packet checksum.
    ///
    /// `CRC-16/MCRF4XX` [checksum](https://mavlink.io/en/guide/serialization.html#checksum) for
    /// message (excluding magic byte).
    ///
    /// Includes [CRC_EXTRA](https://mavlink.io/en/guide/serialization.html#crc_extra) byte.
    ///
    /// Checksum is encoded with little endian (low byte, high byte).
    ///
    /// See:
    ///  * [`Frame::calculate_crc`] for implementation.
    ///  * [MAVLink checksum definition](https://mavlink.io/en/guide/serialization.html#checksum).
    ///  * [CRC-16/MCRF4XX](https://ww1.microchip.com/downloads/en/AppNotes/00752a.pdf) (PDF).
    pub fn checksum(&self) -> Checksum {
        self.checksum
    }

    /// Signature.
    ///
    /// Signature to ensure the link is tamper-proof.
    pub fn signature(&self) -> Option<&Signature> {
        self.signature.as_ref()
    }

    /// MAVLink protocol version defined by [`Header`].
    ///
    /// See:
    ///  * [MavLinkVersion]
    ///  * [Header::mavlink_version]
    pub fn mavlink_version(&self) -> MavLinkVersion {
        self.header.mavlink_version()
    }

    /// Fields related to `MAVLink 2` headers.
    ///
    /// See:
    ///  * [`HeaderV2Fields`].
    ///  * [MAVLink 2 packet format](https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format).
    pub fn mavlink_v2_fields(&self) -> Option<&HeaderV2Fields> {
        self.header.v2_fields()
    }

    /// Payload length.
    ///
    /// Indicates length of the following `payload` section. This may be affected by payload truncation.
    pub fn payload_length(&self) -> u8 {
        self.header.payload_length()
    }

    /// Packet sequence number.
    ///
    /// Used to detect packet loss. Components increment value for each message sent.
    pub fn sequence(&self) -> u8 {
        self.header.sequence()
    }

    /// System `ID`.
    ///
    /// `ID` of system (vehicle) sending the message. Used to differentiate systems on network.
    ///
    /// > Note that the broadcast address 0 may not be used in this field as it is an invalid source
    /// > address.
    pub fn system_id(&self) -> u8 {
        self.header.system_id()
    }

    /// Component `ID`.
    ///
    /// `ID` of component sending the message. Used to differentiate components in a system (e.g.
    /// autopilot and a camera). Use appropriate values in
    /// [MAV_COMPONENT](https://mavlink.io/en/messages/common.html#MAV_COMPONENT).
    ///
    /// > Note that the broadcast address `MAV_COMP_ID_ALL` may not be used in this field as it is
    /// > an invalid source address.
    pub fn component_id(&self) -> u8 {
        self.header.component_id()
    }

    /// Message `ID`.
    ///
    /// `ID` of message type in payload.
    ///
    /// Used to decode data back into message object.
    pub fn message_id(&self) -> MessageId {
        self.header.message_id()
    }

    /// Read and decode [`Frame`] frame from the instance of [`Read`].
    pub(crate) fn recv<R: Read>(reader: &mut R) -> Result<Self> {
        // Retrieve header
        let header = Header::recv(reader)?;

        let body_length = header.expected_body_length()?;

        // Prepare buffer that will contain the entire message body (with signature if expected)
        #[cfg(feature = "std")]
        let mut body_buf = vec![0u8; body_length];
        #[cfg(not(feature = "std"))]
        let mut body_buf = {
            use crate::consts::PAYLOAD_MAX_SIZE;
            [0u8; PAYLOAD_MAX_SIZE + SIGNATURE_LENGTH]
        };
        let body_bytes = &mut body_buf[0..body_length];

        // Read and decode
        reader.read_exact(body_bytes)?;
        let frame = Self::try_from_raw_body(&header, body_bytes)?;

        Ok(frame)
    }

    /// Calculates CRC for [`Frame`] within `extra_crc`.
    ///
    /// Provided `extra_crc` depends on a dialect and contains a digest of message XML definition.
    ///
    /// See:
    ///  * [`Frame::checksum`].
    ///  * [MAVLink checksum definition](https://mavlink.io/en/guide/serialization.html#checksum).
    ///  * [CRC-16/MCRF4XX](https://ww1.microchip.com/downloads/en/AppNotes/00752a.pdf) (PDF).
    pub fn calculate_crc(&self, extra_crc: ExtraCrc) -> Checksum {
        let mut crc_calculator = CRCu16::crc16mcrf4cc();

        crc_calculator.digest(self.header.crc_data());
        crc_calculator.digest(self.payload.payload());

        if let Some(signature) = self.signature {
            crc_calculator.digest(&signature.to_byte_array());
        }

        crc_calculator.digest(&[extra_crc]);

        crc_calculator.get_crc()
    }

    /// Validates [`Frame::checksum`] using provided `extra_crc`.
    ///
    /// See: [`Frame::calculate_crc`] for CRC implementation details.
    pub fn validate_checksum(&self, extra_crc: ExtraCrc) -> Result<()> {
        if self.calculate_crc(extra_crc) != self.checksum {
            return Err(FrameError::InvalidChecksum.into());
        }

        Ok(())
    }

    /// Validates frame in the context of specific dialect.
    ///
    /// Receives dialect specification in `dialect_spec`, ensures that message with such ID
    /// exists in this dialect, and compares checksums using `EXTRA_CRC`.
    ///
    /// # Errors
    ///
    /// * Returns [`CoreError::Message`](crate::errors::CoreError::Message) if message discovery failed.  
    /// * Returns [`FrameError::InvalidChecksum`] (wrapped by [`CoreError`](crate::errors::CoreError)) if checksum
    ///   validation failed.
    ///
    /// # Links
    ///
    /// * [`DialectSpec`] for dialect specification.
    /// * [`Frame::calculate_crc`] for CRC implementation details.
    pub fn validate(&self, dialect_spec: &dyn DialectSpec) -> Result<()> {
        let message_info = dialect_spec.message_info(self.header().message_id())?;
        self.validate_checksum(message_info.extra_crc())?;

        // Check that signature is present if `MAVLINK_IFLAG_SIGNED` flag is set
        if self.signature.is_none() && self.header.is_signed()? {
            return Err(FrameError::SignatureIsMissing.into());
        }

        Ok(())
    }

    /// Converts slice of body bytes into [`Frame`].
    fn try_from_raw_body(header: &Header, body_bytes: &[u8]) -> Result<Self> {
        // Validate body size
        let body_length = header.expected_body_length()?;
        match header.mavlink_version() {
            MavLinkVersion::V1 => {
                if body_bytes.len() < body_length {
                    return Err(FrameError::PacketV1BodyIsTooSmall.into());
                }
            }
            MavLinkVersion::V2 => {
                if body_bytes.len() < body_length {
                    return Err(FrameError::PacketV2BodyIsTooSmall.into());
                }
            }
        }

        let payload_bytes = &body_bytes[0..header.payload_length() as usize];
        let payload = Payload::new(header.message_id(), payload_bytes, header.mavlink_version());

        // Decode checksum
        let checksum_start = header.payload_length() as usize;
        let checksum_bytes = [body_bytes[checksum_start], body_bytes[checksum_start + 1]];
        let checksum: Checksum = Checksum::from_le_bytes(checksum_bytes);

        let signature: Option<Signature> = if header.is_signed()? {
            let signature_start = checksum_start + CHECKSUM_SIZE;
            let signature_bytes = &body_bytes[signature_start..signature_start + SIGNATURE_LENGTH];
            Some(Signature::try_from(signature_bytes)?)
        } else {
            None
        };

        Ok(Self {
            header: *header,
            payload,
            checksum,
            signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use crc_any::CRCu16;

    #[test]
    fn crc_calculation_algorithm_accepts_sequential_digests() {
        // We just want to test that CRC algorithm is invariant in respect to the way we feed it
        // data.

        let data = [124, 12, 22, 34, 2, 148, 82, 201, 72, 0, 18, 215, 37, 63u8];
        let split_at: usize = data.len() / 2;

        // Get all data as one slice
        let mut crc_calculator_bulk = CRCu16::crc16mcrf4cc();
        crc_calculator_bulk.digest(&data);

        // Get data as two chunks sequentially
        let mut crc_calculator_seq = CRCu16::crc16mcrf4cc();
        crc_calculator_seq.digest(&data[0..split_at]);
        crc_calculator_seq.digest(&data[split_at..data.len()]);

        assert_eq!(crc_calculator_bulk.get_crc(), crc_calculator_seq.get_crc());
    }
}
