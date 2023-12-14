//! # MAVLink frame

use crc_any::CRCu16;

use crate::consts::{CHECKSUM_SIZE, SIGNATURE_LENGTH};
use crate::errors::{FrameError, Result};
use crate::io::{Read, Write};
use crate::protocol::header::{Header, HeaderConf, HeaderV2Fields};
use crate::protocol::signature::Signature;
use crate::protocol::{
    Checksum, CompatFlags, CrcExtra, DialectSpec, IncompatFlags, MavLinkVersion, MessageId, Payload,
};
use crate::protocol::{MessageImpl, SignatureBytes};

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

/// Configuration builder for [`Frame`].
///
/// Implements [builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
/// pattern for [`Frame`].
#[derive(Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FrameConf {
    header_conf: HeaderConf,
    payload: Option<Payload>,
    crc_extra: Option<CrcExtra>,
}

impl Frame {
    /// Initiates builder for [`Frame`].
    ///
    /// Instead of constructor we use
    /// [builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
    /// pattern. An instance of [`FrameConf`] returned by this function is initialized
    /// with default values. Once desired values are set, you can call [`FrameConf::build`]
    /// to obtain [`Frame`].
    pub fn conf() -> FrameConf {
        FrameConf::new()
    }

    /// Generic MAVLink header.
    ///
    /// # Links
    ///
    /// * Header implementation: [`Header`].
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Payload data.
    ///
    /// Message data. Content depends on message type (i.e. `message_id`).
    ///
    /// # Links
    ///
    /// * Payload implementation: [`Payload`].
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
    /// # Links
    ///
    /// * [`Frame::calculate_crc`] for implementation.
    /// * [MAVLink checksum definition](https://mavlink.io/en/guide/serialization.html#checksum).
    /// * [CRC-16/MCRF4XX](https://ww1.microchip.com/downloads/en/AppNotes/00752a.pdf) (PDF).
    pub fn checksum(&self) -> Checksum {
        self.checksum
    }

    /// Signature.
    ///
    /// Signature to ensure the link is tamper-proof. Applicable only for `MAVLink 2` frames.
    ///
    /// # Links
    ///
    /// * [`Frame::is_signature_required`].
    /// * [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html).
    pub fn signature(&self) -> Option<&Signature> {
        self.signature.as_ref()
    }

    /// MAVLink protocol version defined by [`Header`].
    ///
    /// # Links
    ///
    /// * [MavLinkVersion]
    /// * [Header::mavlink_version]
    pub fn mavlink_version(&self) -> MavLinkVersion {
        self.header.mavlink_version()
    }

    /// Fields related to `MAVLink 2` headers.
    ///
    /// # Links
    ///
    /// * [Header::v2_fields] and [`HeaderV2Fields`].
    /// * [MAVLink 2 packet format](https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format).
    pub fn mavlink_v2_fields(&self) -> Option<&HeaderV2Fields> {
        self.header.v2_fields()
    }

    /// Payload length.
    ///
    /// Indicates length of the following `payload` section. This may be affected by payload truncation.
    ///
    /// # Links
    ///
    /// * [Header::payload_length].
    pub fn payload_length(&self) -> u8 {
        self.header.payload_length()
    }

    /// Packet sequence number.
    ///
    /// Used to detect packet loss. Components increment value for each message sent.
    ///
    /// # Links
    ///
    /// * [Header::sequence].
    pub fn sequence(&self) -> u8 {
        self.header.sequence()
    }

    /// System `ID`.
    ///
    /// `ID` of system (vehicle) sending the message. Used to differentiate systems on network.
    ///
    /// > Note that the broadcast address 0 may not be used in this field as it is an invalid source
    /// > address.
    ///
    /// # Links
    ///
    /// * [Header::system_id].
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
    ///
    /// # Links
    ///
    /// * [Header::component_id].
    pub fn component_id(&self) -> u8 {
        self.header.component_id()
    }

    /// Message `ID`.
    ///
    /// `ID` of MAVLink message. Defines how payload will be encoded and decoded.
    ///
    /// # Links
    ///
    /// * [Header::message_id].
    pub fn message_id(&self) -> MessageId {
        self.header.message_id()
    }

    /// Whether a frame body should contain signature.
    ///
    /// # Errors
    ///
    /// * Returns [FrameError::InconsistentV2Header] if [`Header::v2_fields`] are missing.
    ///
    /// # Links
    ///
    /// * [`Frame::signature`].
    #[inline]
    pub fn is_signature_required(&self) -> bool {
        self.header.is_signed()
    }

    /// Body length.
    ///
    /// # Errors
    ///
    /// Returns [`FrameError::InconsistentV2Header`] if frame is `MAVLink 2` but doesn't have `MAVLink 2` specific
    /// fields.
    pub fn body_length(&self) -> usize {
        self.header().expected_body_length()
    }

    /// Read and decode [`Frame`] frame from the instance of [`Read`].
    pub(crate) fn recv<R: Read>(reader: &mut R) -> Result<Self> {
        // Retrieve header
        let header = Header::recv(reader)?;

        let body_length = header.expected_body_length();

        // Prepare buffer that will contain the entire message body (with signature if expected)
        #[cfg(feature = "std")]
        let mut body_buf = vec![0u8; body_length];
        #[cfg(not(feature = "std"))]
        let mut body_buf = [0u8; PAYLOAD_MAX_SIZE + SIGNATURE_LENGTH];
        let body_bytes = &mut body_buf[0..body_length];

        // Read and decode
        reader.read_exact(body_bytes)?;
        let frame = Self::try_from_raw_body(&header, body_bytes)?;

        Ok(frame)
    }

    /// Encodes and sends [`Frame`] into an instance of [`Write`].
    pub(crate) fn send<W: Write>(&self, writer: &mut W) -> Result<usize> {
        // Validate payload length consistency
        if self.payload_length() != self.payload.length() {
            return Err(FrameError::InconsistentPayloadSize.into());
        }
        let payload_length = self.payload_length() as usize;

        // Send header
        let header_bytes_sent = self.header.send(writer)?;

        // Prepare a buffer
        #[cfg(not(feature = "alloc"))]
        let mut buf = [0u8; PAYLOAD_MAX_SIZE + SIGNATURE_LENGTH];
        #[cfg(feature = "alloc")]
        let mut buf = vec![0u8; self.body_length()];

        // Put payload into buffer
        buf[0..payload_length].copy_from_slice(self.payload.payload());

        // Put checksum into buffer
        let checksum_bytes: [u8; 2] = self.checksum.to_le_bytes();
        buf[payload_length..payload_length + 2].copy_from_slice(&checksum_bytes);

        // Put signature if required
        if let Some(signature) = self.signature {
            let signature_bytes: SignatureBytes = signature.to_byte_array();
            let sig_start_idx = payload_length + 2;
            buf[sig_start_idx..self.body_length()].copy_from_slice(&signature_bytes);
        }

        writer.write_all(buf.as_slice())?;

        Ok(header_bytes_sent + self.body_length())
    }

    /// Calculates CRC for [`Frame`] within `crc_extra`.
    ///
    /// Provided `crc_extra` depends on a dialect and contains a digest of message XML definition.
    ///
    /// # Links
    ///
    /// * [`Frame::checksum`].
    /// * [MAVLink checksum definition](https://mavlink.io/en/guide/serialization.html#checksum).
    /// * [CRC-16/MCRF4XX](https://ww1.microchip.com/downloads/en/AppNotes/00752a.pdf) (PDF).
    pub fn calculate_crc(&self, crc_extra: CrcExtra) -> Checksum {
        let mut crc_calculator = CRCu16::crc16mcrf4cc();

        crc_calculator.digest(self.header.crc_data());
        crc_calculator.digest(self.payload.payload());

        if let Some(signature) = self.signature {
            crc_calculator.digest(&signature.to_byte_array());
        }

        crc_calculator.digest(&[crc_extra]);

        crc_calculator.get_crc()
    }

    /// Validates [`Frame::checksum`] using provided `crc_extra`.
    ///
    /// # Links
    ///
    /// * [`Frame::calculate_crc`] for CRC implementation details.
    pub fn validate_checksum(&self, crc_extra: CrcExtra) -> Result<()> {
        if self.calculate_crc(crc_extra) != self.checksum {
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
        self.validate_checksum(message_info.crc_extra())?;

        // Check that signature is present if `MAVLINK_IFLAG_SIGNED` flag is set
        if self.signature.is_none() && self.header.is_signed() {
            return Err(FrameError::SignatureIsMissing.into());
        }

        Ok(())
    }

    /// Converts slice of body bytes into [`Frame`].
    fn try_from_raw_body(header: &Header, body_bytes: &[u8]) -> Result<Self> {
        // Validate body size
        let body_length = header.expected_body_length();
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

        let signature: Option<Signature> = if header.is_signed() {
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

impl FrameConf {
    /// Default constructor
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds [`Frame`].
    ///
    /// Validates frame configuration and creates an instance of [`Frame`].
    ///
    /// # Errors
    ///
    /// * Returns various variants of [`FrameError`] (wrapped by [`CoreError`](crate::errors::CoreError)) if validation
    /// fails.
    pub fn build(&mut self) -> Result<Frame> {
        // Create payload
        let payload = match &self.payload {
            Some(payload) => payload.clone(),
            None => return Err(FrameError::MissingFrameField("payload".into()).into()),
        };

        // Build header
        let header = self
            .header_conf
            .set_payload_length(payload.length())
            .build()?;

        // Prepare frame
        let mut frame = Frame {
            header,
            payload,
            checksum: 0,
            signature: None,
        };

        // Calculate checksum
        match self.crc_extra {
            Some(crc_extra) => frame.checksum = frame.calculate_crc(crc_extra),
            None => return Err(FrameError::MissingFrameField("crc_extra".into()).into()),
        }

        Ok(frame)
    }

    /// Sets MAVLink protocol version.
    ///
    /// See: [`Header::mavlink_version`].
    pub fn set_mavlink_version(&mut self, mavlink_version: MavLinkVersion) -> &mut Self {
        self.header_conf.set_mavlink_version(mavlink_version);
        self
    }

    /// Sets incompatibility flags for `MAVLink 2` header.
    ///
    /// # Errors
    ///
    /// Does not returns error directly but if both MAVLink version is set to [`MavLinkVersion::V1`] and incompatibility
    /// flags are present, then [`FrameError::InconsistentV1Header`] error will be returned by [`Self::build`].
    ///
    /// # Links
    ///
    /// * [`HeaderV2Fields`].
    pub fn set_incompat_flags(&mut self, incompat_flags: IncompatFlags) -> &mut Self {
        self.header_conf.set_incompat_flags(incompat_flags);
        self
    }

    /// Sets compatibility flags for `MAVLink 2` header.
    ///
    /// # Errors
    ///
    /// Does not returns error directly but if both MAVLink version is set to [`MavLinkVersion::V1`] and compatibility
    /// flags are present, then [`FrameError::InconsistentV1Header`] error will be returned by [`Self::build`].
    ///
    /// # Links
    ///
    /// * [`HeaderV2Fields`].
    pub fn set_compat_flags(&mut self, compat_flags: CompatFlags) -> &mut Self {
        self.header_conf.set_compat_flags(compat_flags);
        self
    }

    /// Sets packet sequence number.
    ///
    /// # Links
    ///
    /// * [`Frame::sequence`].
    pub fn set_sequence(&mut self, sequence: u8) -> &mut Self {
        self.header_conf.set_sequence(sequence);
        self
    }

    /// Sets system `ID`.
    ///
    /// # Links
    ///
    /// * [`Frame::system_id`].
    pub fn set_system_id(&mut self, system_id: u8) -> &mut Self {
        self.header_conf.set_system_id(system_id);
        self
    }

    /// Sets component `ID`.
    ///
    /// # Links
    ///
    /// * [`Frame::component_id`].
    pub fn set_component_id(&mut self, component_id: u8) -> &mut Self {
        self.header_conf.set_component_id(component_id);
        self
    }

    /// Sets message `ID`.
    ///
    /// # Links
    ///
    /// * [`Frame::message_id`].
    pub fn set_message_id(&mut self, message_id: MessageId) -> &mut Self {
        self.header_conf.set_message_id(message_id);
        self
    }

    /// Sets `CRC_EXTRA`.
    ///
    /// # Links
    ///
    /// * [`Frame::checksum`] is calculated using [`CrcExtra`].
    pub fn set_crc_extra(&mut self, crc_extra: CrcExtra) -> &mut Self {
        self.crc_extra = Some(crc_extra);
        self
    }

    /// Sets payload data.
    ///
    /// # Links
    ///
    /// * [`Frame::payload`]
    pub fn set_payload(&mut self, payload: Payload) -> &mut Self {
        self.payload = Some(payload);
        self
    }

    /// Imports MAVLink message.
    ///
    /// Imports and encodes MAVLink message. Uses [`MessageImpl::crc_extra`] to create a checksum.
    ///
    /// Uses [`MessageImpl`] to define:
    ///
    /// * [`Frame::message_id`]
    /// * [`Frame::payload_length`]
    /// * [`Frame::payload`]
    /// * [`Frame::checksum`]
    pub fn with_message(
        &mut self,
        message: &dyn MessageImpl,
        mavlink_version: MavLinkVersion,
    ) -> Result<&mut Self> {
        let payload = message.encode(mavlink_version)?;

        self.set_message_id(message.id());
        self.set_mavlink_version(mavlink_version);
        self.set_payload(payload);
        self.set_crc_extra(message.crc_extra());

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    #[cfg(feature = "minimal")]
    fn test_builder() -> Result<()> {
        use crate::dialects::minimal::messages::MsgHeartbeat;

        let message = MsgHeartbeat::default();
        let frame = Frame::conf()
            .with_message(&message, MavLinkVersion::V2)?
            .set_sequence(17)
            .set_system_id(22)
            .set_component_id(17)
            .build();

        frame.unwrap();

        Ok(())
    }
}
