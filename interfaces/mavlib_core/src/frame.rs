//! # MAVLink frame

use crate::consts::{
    MAVLINK_CHECKSUM_SIZE, MAVLINK_V2_SIGNATURE_LENGTH, MAVLINK_V2_SIGNATURE_LINK_ID_LENGTH,
    MAVLINK_V2_SIGNATURE_TIMESTAMP_LENGTH, MAVLINK_V2_SIGNATURE_VALUE_LENGTH,
};
use crate::errors::MavLinkFrameDecodingError;
use crate::header::MavLinkHeader;
use crate::types::{
    MavLinkChecksum, MavLinkV2SignatureLinkId, MavLinkV2SignatureTimestamp, MavLinkV2SignatureValue,
};
use crate::{MavLinkMessagePayload, MavLinkVersion};

/// `MAVLink` frame.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MavLinkFrame {
    /// Generic `MAVLink` header.
    pub(crate) header: MavLinkHeader,
    /// Payload data.
    pub(crate) payload: MavLinkMessagePayload,
    /// `MAVLink` packet checksum.
    pub(crate) checksum: MavLinkChecksum,
    /// Signature.
    pub(crate) signature: Option<MavLinkV2Signature>,
}

/// `MAVLink 2` packet signature.
///
/// See [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MavLinkV2Signature {
    /// Signature link ID.
    ///
    /// See [link Ids](https://mavlink.io/en/guide/message_signing.html#link_ids) in `MAVLink` docs.
    pub(crate) link_id: MavLinkV2SignatureLinkId,
    /// Signature timestamp.
    ///
    /// See [timestamp handling](https://mavlink.io/en/guide/message_signing.html#timestamp) in `MAVLink` docs.
    pub(crate) timestamp: MavLinkV2SignatureTimestamp,
    /// Signature value.
    ///
    /// See [signature specification](https://mavlink.io/en/guide/message_signing.html#signature) in `MAVLink` docs.
    pub(crate) signature: MavLinkV2SignatureValue,
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

impl TryFrom<&[u8]> for MavLinkV2Signature {
    type Error = MavLinkFrameDecodingError;

    /// Reads array into [`MavLinkV2Signature`].
    ///
    /// # Errors
    ///
    /// Returns [`MavLinkFrameDecodingError::V2SignatureIsTooSmall`] if slice is too small to
    /// contain a valid signature.
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < MAVLINK_V2_SIGNATURE_LENGTH {
            return Err(MavLinkFrameDecodingError::V2SignatureIsTooSmall);
        }

        let link_id = value[0];
        let mut timestamp: MavLinkV2SignatureTimestamp = Default::default();
        let mut signature: MavLinkV2SignatureValue = Default::default();

        let timestamp_start = MAVLINK_V2_SIGNATURE_LINK_ID_LENGTH;
        let timestamp_end = timestamp_start + MAVLINK_V2_SIGNATURE_TIMESTAMP_LENGTH;
        timestamp.copy_from_slice(&value[timestamp_start..timestamp_end]);

        let value_start = timestamp_end;
        let value_end = value_start + MAVLINK_V2_SIGNATURE_VALUE_LENGTH;
        signature.copy_from_slice(&value[timestamp_start..value_end]);

        Ok(Self {
            link_id,
            timestamp,
            signature,
        })
    }
}

impl MavLinkV2Signature {
    /// Signature link ID.
    pub fn link_id(&self) -> MavLinkV2SignatureLinkId {
        self.link_id
    }

    /// Signature timestamp.
    pub fn timestamp(&self) -> MavLinkV2SignatureTimestamp {
        self.timestamp
    }

    /// Signature value.
    pub fn signature(&self) -> MavLinkV2SignatureValue {
        self.signature
    }
}

impl<'a> MavLinkRawFrame<'a> {
    /// Default constructor.
    pub fn new(header: &'a MavLinkHeader, body: &'a [u8]) -> Self {
        Self { header, body }
    }
}

impl<'a> TryFrom<MavLinkRawFrame<'a>> for MavLinkFrame {
    type Error = MavLinkFrameDecodingError;

    fn try_from(value: MavLinkRawFrame) -> Result<Self, Self::Error> {
        let body = value.body;
        let header = value.header;

        // Validate body size
        let body_length = header.expected_body_length()?;
        match header.mavlink_version {
            MavLinkVersion::V1 => {
                if body.len() < body_length {
                    return Err(MavLinkFrameDecodingError::V1PacketBodyIsTooSmall);
                }
            }
            MavLinkVersion::V2 => {
                if body.len() < body_length {
                    return Err(MavLinkFrameDecodingError::V2PacketBodyIsTooSmall);
                }
            }
        }

        let payload_bytes = &body[0..header.payload_length as usize];
        let payload =
            MavLinkMessagePayload::new(header.message_id, payload_bytes, header.mavlink_version);

        // Decode checksum
        let checksum_start = header.payload_length as usize;
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
    /// See [CRC-16/MCRF4XX](https://ww1.microchip.com/downloads/en/AppNotes/00752a.pdf) (PDF).
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
        self.header.mavlink_version
    }
}
