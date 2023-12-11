//! # MAVLink frame

use crc_any::CRCu16;
use mavlib_spec::{MavLinkMessagePayload, MavLinkVersion};

use crate::consts::{MAVLINK_CHECKSUM_SIZE, MAVLINK_V2_SIGNATURE_LENGTH};
use crate::errors::{CoreError, FrameError, Result};
use crate::header::MavLinkHeader;
use crate::io::Read;
use crate::signature::MavLinkV2Signature;
use crate::types::{MAVLinkExtraCrc, MavLinkChecksum};

/// `MAVLink` frame.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MavLinkFrame {
    /// Generic `MAVLink` header.
    header: MavLinkHeader,
    /// Payload data.
    payload: MavLinkMessagePayload,
    /// `MAVLink` packet checksum.
    checksum: MavLinkChecksum,
    /// Signature.
    signature: Option<MavLinkV2Signature>,
}

/// `MAVLink` raw frame with everything but header encoded.
///
/// [`MavLinkRawFrame`] is a utility structure that holds links to [`MavLinkHeader`] and encoded
/// frame body. It is intended to be constructed and immediately converted to [`MavLinkFrame`] which
/// implements [`TryFrom<MavLinkRawFrame>`].
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MavLinkRawFrame<'a> {
    /// Generic `MAVLink` header.
    pub(crate) header: &'a MavLinkHeader,
    /// Frame body as bytes.
    pub(crate) body: &'a [u8],
}

impl<'a> MavLinkRawFrame<'a> {
    /// Default constructor.
    pub fn new(header: &'a MavLinkHeader, body: &'a [u8]) -> Self {
        Self { header, body }
    }
}

impl<'a> TryFrom<MavLinkRawFrame<'a>> for MavLinkFrame {
    type Error = CoreError;

    /// Converts [`MavLinkRawFrame`] into [`MavLinkFrame`].
    ///
    /// See [`MavLinkFrame::try_from_raw_frame`].
    fn try_from(value: MavLinkRawFrame) -> Result<Self> {
        Self::try_from_raw_frame(value)
    }
}

impl<'a> MavLinkRawFrame<'a> {
    /// Generic `MAVLink` header.
    pub fn header(&self) -> &'a MavLinkHeader {
        self.header
    }
    /// Frame body as bytes.
    pub fn body(&self) -> &'a [u8] {
        self.body
    }
}

impl MavLinkFrame {
    /// Generic `MAVLink` header.
    ///
    /// See [`MavLinkHeader`].
    pub fn header(&self) -> &MavLinkHeader {
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

    /// `MAVLink` packet checksum.
    ///
    /// `CRC-16/MCRF4XX` [checksum](https://mavlink.io/en/guide/serialization.html#checksum) for
    /// message (excluding magic byte).
    ///
    /// Includes [CRC_EXTRA](https://mavlink.io/en/guide/serialization.html#crc_extra) byte.
    ///
    /// Checksum is encoded with little endian (low byte, high byte).
    ///
    /// See:
    ///  * [`MavLinkFrame::calculate_crc`] for implementation.
    ///  * [MAVLink checksum definition](https://mavlink.io/en/guide/serialization.html#checksum).
    ///  * [CRC-16/MCRF4XX](https://ww1.microchip.com/downloads/en/AppNotes/00752a.pdf) (PDF).
    pub fn checksum(&self) -> MavLinkChecksum {
        self.checksum
    }

    /// Signature.
    ///
    /// Signature to ensure the link is tamper-proof.
    pub fn signature(&self) -> Option<&MavLinkV2Signature> {
        self.signature.as_ref()
    }

    /// `MAVLink` protocol version defined by [`MavLinkHeader`].
    ///
    /// See:
    ///  * [MavLinkVersion]
    ///  * [MavLinkHeader::mavlink_version]
    pub fn mavlink_version(&self) -> MavLinkVersion {
        self.header.mavlink_version()
    }

    /// Read and decode [`MavLinkFrame`] frame from the instance of [`Read`].
    pub fn recv<R: Read>(reader: &mut R) -> Result<Self> {
        // Retrieve header
        let header = MavLinkHeader::recv(reader)?;

        let body_length = header.expected_body_length()?;

        // Prepare buffer that will contain the entire message body (with signature if expected)
        #[cfg(feature = "std")]
        let mut body_buf = vec![0u8; body_length];
        #[cfg(not(feature = "std"))]
        let mut body_buf = {
            use mavlib_spec::payload::MAX_PAYLOAD_SIZE;
            [0u8; MAX_PAYLOAD_SIZE + MAVLINK_V2_SIGNATURE_LENGTH]
        };
        let body_bytes = &mut body_buf[0..body_length];

        reader.read_exact(body_bytes)?;

        let raw_frame = MavLinkRawFrame::new(&header, body_bytes);
        let frame = MavLinkFrame::try_from_raw_frame(raw_frame)?;

        Ok(frame)
    }

    /// Calculates CRC for [`MavLinkFrame`] within `extra_crc`.
    ///
    /// Provided `extra_crc` depends on a dialect and contains a digest of message XML definition.
    ///
    /// See:
    ///  * [`MavLinkFrame::checksum`].
    ///  * [MAVLink checksum definition](https://mavlink.io/en/guide/serialization.html#checksum).
    ///  * [CRC-16/MCRF4XX](https://ww1.microchip.com/downloads/en/AppNotes/00752a.pdf) (PDF).
    pub fn calculate_crc(&self, extra_crc: MAVLinkExtraCrc) -> MavLinkChecksum {
        let mut crc_calculator = CRCu16::crc16mcrf4cc();

        crc_calculator.digest(self.header.crc_data());
        crc_calculator.digest(self.payload.payload());

        if let Some(signature) = self.signature {
            crc_calculator.digest(&signature.to_byte_array());
        }

        crc_calculator.digest(&[extra_crc]);

        crc_calculator.get_crc()
    }

    /// Validates [`MavLinkFrame::checksum`] using provided `extra_crc`.
    ///
    /// See: [`MavLinkFrame::calculate_crc`] for CRC implementation details.
    pub fn validate_crc(&self, extra_crc: MAVLinkExtraCrc) -> Result<()> {
        if self.calculate_crc(extra_crc) != self.checksum {
            return Err(FrameError::InvalidChecksum.into());
        }

        Ok(())
    }

    /// Converts [`MavLinkRawFrame`] into [`MavLinkFrame`].
    pub fn try_from_raw_frame(raw_frame: MavLinkRawFrame) -> Result<Self> {
        let body = raw_frame.body;
        let header = raw_frame.header;

        // Validate body size
        let body_length = header.expected_body_length()?;
        match header.mavlink_version() {
            MavLinkVersion::V1 => {
                if body.len() < body_length {
                    return Err(FrameError::V1PacketBodyIsTooSmall.into());
                }
            }
            MavLinkVersion::V2 => {
                if body.len() < body_length {
                    return Err(FrameError::V2PacketBodyIsTooSmall.into());
                }
            }
        }

        let payload_bytes = &body[0..header.payload_length() as usize];
        let payload = MavLinkMessagePayload::new(
            header.message_id(),
            payload_bytes,
            header.mavlink_version(),
        );

        // Decode checksum
        let checksum_start = header.payload_length() as usize;
        let checksum_bytes = [body[checksum_start], body[checksum_start + 1]];
        let checksum: MavLinkChecksum = MavLinkChecksum::from_le_bytes(checksum_bytes);

        let signature: Option<MavLinkV2Signature> = if header.is_signature_required()? {
            let signature_start = checksum_start + MAVLINK_CHECKSUM_SIZE;
            let signature_bytes =
                &body[signature_start..signature_start + MAVLINK_V2_SIGNATURE_LENGTH];
            Some(MavLinkV2Signature::try_from(signature_bytes)?)
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
