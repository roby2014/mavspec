//! # MAVLink frame

use crc_any::CRCu16;
use mavlib_spec::{MavLinkDialectSpec, MavLinkMessagePayload, MavLinkVersion};

use crate::consts::{CHECKSUM_SIZE, SIGNATURE_LENGTH};
use crate::errors::{FrameError, Result};
use crate::header::Header;
use crate::io::Read;
use crate::signature::Signature;
use crate::types::{Checksum, ExtraCrc};

/// MAVLink frame.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Frame {
    /// Generic MAVLink header.
    header: Header,
    /// Payload data.
    payload: MavLinkMessagePayload,
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
    /// See [`MavLinkMessagePayload`].
    pub fn payload(&self) -> &MavLinkMessagePayload {
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

    /// Read and decode [`Frame`] frame from the instance of [`Read`].
    pub fn recv<R: Read>(reader: &mut R) -> Result<Self> {
        // Retrieve header
        let header = Header::recv(reader)?;

        let body_length = header.expected_body_length()?;

        // Prepare buffer that will contain the entire message body (with signature if expected)
        #[cfg(feature = "std")]
        let mut body_buf = vec![0u8; body_length];
        #[cfg(not(feature = "std"))]
        let mut body_buf = {
            use mavlib_spec::payload::MAX_PAYLOAD_SIZE;
            [0u8; MAX_PAYLOAD_SIZE + SIGNATURE_LENGTH]
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
    /// * [`MavLinkDialectSpec`] for dialect specification.
    /// * [`Frame::calculate_crc`] for CRC implementation details.
    pub fn validate(&self, dialect_spec: &dyn MavLinkDialectSpec) -> Result<()> {
        let message_info = dialect_spec.message_info(self.header().message_id())?;
        self.validate_checksum(message_info.extra_crc())?;

        Ok(())
    }

    /// Converts slice of body bytes into [`Frame`].
    fn try_from_raw_body(header: &Header, body_bytes: &[u8]) -> Result<Self> {
        // Validate body size
        let body_length = header.expected_body_length()?;
        match header.mavlink_version() {
            MavLinkVersion::V1 => {
                if body_bytes.len() < body_length {
                    return Err(FrameError::V1PacketBodyIsTooSmall.into());
                }
            }
            MavLinkVersion::V2 => {
                if body_bytes.len() < body_length {
                    return Err(FrameError::V2PacketBodyIsTooSmall.into());
                }
            }
        }

        let payload_bytes = &body_bytes[0..header.payload_length() as usize];
        let payload = MavLinkMessagePayload::new(
            header.message_id(),
            payload_bytes,
            header.mavlink_version(),
        );

        // Decode checksum
        let checksum_start = header.payload_length() as usize;
        let checksum_bytes = [body_bytes[checksum_start], body_bytes[checksum_start + 1]];
        let checksum: Checksum = Checksum::from_le_bytes(checksum_bytes);

        let signature: Option<Signature> = if header.is_signature_required()? {
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
