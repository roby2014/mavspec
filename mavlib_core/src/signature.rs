//! # `MAVLink 2` packet signature
//!
//! Implements [`MavLinkV2Signature`].
//!
//! See [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html).

use crate::consts::{
    MAVLINK_V2_SIGNATURE_LENGTH, MAVLINK_V2_SIGNATURE_LINK_ID_LENGTH,
    MAVLINK_V2_SIGNATURE_TIMESTAMP_LENGTH, MAVLINK_V2_SIGNATURE_VALUE_LENGTH,
};
use crate::errors::FrameError;
use crate::types::{
    MavLinkV2SignatureBytes, MavLinkV2SignatureLinkId, MavLinkV2SignatureTimestamp,
    MavLinkV2SignatureValue,
};

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

impl TryFrom<&[u8]> for MavLinkV2Signature {
    type Error = FrameError;

    /// Decodes slice of bytes into [`MavLinkV2Signature`].
    ///
    /// See [`MavLinkV2Signature::try_from_slice`].
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_slice(value)
    }
}

impl From<MavLinkV2Signature> for MavLinkV2SignatureBytes {
    /// Encodes [`MavLinkV2Signature`] into [`MavLinkV2SignatureBytes`] byte array.
    ///
    /// See [`MavLinkV2Signature::to_byte_array`].
    fn from(value: MavLinkV2Signature) -> Self {
        value.to_byte_array()
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

    /// Decodes slice of bytes into [`MavLinkV2Signature`].
    ///
    /// Used in [`TryFrom<&[u8]>`](TryFrom) trait implementation for [`MavLinkV2Signature`].
    ///
    /// # Errors
    ///
    /// Returns [`FrameError::V2SignatureIsTooSmall`] if slice is too small to
    /// contain a valid signature.
    pub fn try_from_slice(bytes: &[u8]) -> Result<Self, FrameError> {
        if bytes.len() < MAVLINK_V2_SIGNATURE_LENGTH {
            return Err(FrameError::V2SignatureIsTooSmall);
        }

        let link_id = bytes[0];
        let mut timestamp: MavLinkV2SignatureTimestamp = Default::default();
        let mut signature: MavLinkV2SignatureValue = Default::default();

        let timestamp_start = MAVLINK_V2_SIGNATURE_LINK_ID_LENGTH;
        let timestamp_end = timestamp_start + MAVLINK_V2_SIGNATURE_TIMESTAMP_LENGTH;
        timestamp.copy_from_slice(&bytes[timestamp_start..timestamp_end]);

        let value_start = timestamp_end;
        let value_end = value_start + MAVLINK_V2_SIGNATURE_VALUE_LENGTH;
        signature.copy_from_slice(&bytes[timestamp_start..value_end]);

        Ok(Self {
            link_id,
            timestamp,
            signature,
        })
    }

    /// Encodes [`MavLinkV2Signature`] into [`MavLinkV2SignatureBytes`] byte array.
    ///
    /// Used in [`From<MavLinkV2Signature>`](From) trait implementation for [`MavLinkV2SignatureBytes`].
    pub fn to_byte_array(&self) -> MavLinkV2SignatureBytes {
        let mut bytes: MavLinkV2SignatureBytes = Default::default();

        bytes[0] = self.link_id;
        bytes[1..MAVLINK_V2_SIGNATURE_TIMESTAMP_LENGTH].copy_from_slice(&self.timestamp);
        bytes[1 + MAVLINK_V2_SIGNATURE_TIMESTAMP_LENGTH..MAVLINK_V2_SIGNATURE_LENGTH]
            .copy_from_slice(&self.signature);

        bytes
    }
}
