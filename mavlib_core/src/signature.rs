//! # `MAVLink 2` packet signature
//!
//! Implements [`Signature`].
//!
//! See [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html).

use crate::consts::{
    SIGNATURE_LENGTH, SIGNATURE_LINK_ID_LENGTH, SIGNATURE_TIMESTAMP_LENGTH, SIGNATURE_VALUE_LENGTH,
};
use crate::errors::Result;
use crate::errors::{CoreError, FrameError};
use crate::types::{SignatureBytes, SignatureLinkId, SignatureTimestampBytes, SignatureValueBytes};

/// `MAVLink 2` packet signature.
///
/// See [MAVLink 2 message signing](https://mavlink.io/en/guide/message_signing.html).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Signature {
    /// Signature link ID.
    ///
    /// See [link Ids](https://mavlink.io/en/guide/message_signing.html#link_ids) in MAVLink docs.
    pub(crate) link_id: SignatureLinkId,
    /// Signature timestamp.
    ///
    /// See [timestamp handling](https://mavlink.io/en/guide/message_signing.html#timestamp) in MAVLink docs.
    pub(crate) timestamp: SignatureTimestampBytes,
    /// Signature value.
    ///
    /// See [signature specification](https://mavlink.io/en/guide/message_signing.html#signature) in MAVLink docs.
    pub(crate) signature: SignatureValueBytes,
}

impl TryFrom<&[u8]> for Signature {
    type Error = CoreError;

    /// Decodes slice of bytes into [`Signature`].
    ///
    /// See [`Signature::try_from_slice`].
    fn try_from(value: &[u8]) -> Result<Self> {
        Self::try_from_slice(value)
    }
}

impl From<Signature> for SignatureBytes {
    /// Encodes [`Signature`] into [`SignatureBytes`] byte array.
    ///
    /// See [`Signature::to_byte_array`].
    fn from(value: Signature) -> Self {
        value.to_byte_array()
    }
}

impl Signature {
    /// Signature link ID.
    pub fn link_id(&self) -> SignatureLinkId {
        self.link_id
    }

    /// Signature timestamp.
    pub fn timestamp(&self) -> SignatureTimestampBytes {
        self.timestamp
    }

    /// Signature value.
    pub fn signature(&self) -> SignatureValueBytes {
        self.signature
    }

    /// Decodes slice of bytes into [`Signature`].
    ///
    /// Used in [`TryFrom<&[u8]>`](TryFrom) trait implementation for [`Signature`].
    ///
    /// # Errors
    ///
    /// Returns [`FrameError::V2SignatureIsTooSmall`] if slice is too small to
    /// contain a valid signature.
    pub fn try_from_slice(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < SIGNATURE_LENGTH {
            return Err(FrameError::V2SignatureIsTooSmall.into());
        }

        let link_id = bytes[0];
        let mut timestamp: SignatureTimestampBytes = Default::default();
        let mut signature: SignatureValueBytes = Default::default();

        let timestamp_start = SIGNATURE_LINK_ID_LENGTH;
        let timestamp_end = timestamp_start + SIGNATURE_TIMESTAMP_LENGTH;
        timestamp.copy_from_slice(&bytes[timestamp_start..timestamp_end]);

        let value_start = timestamp_end;
        let value_end = value_start + SIGNATURE_VALUE_LENGTH;
        signature.copy_from_slice(&bytes[timestamp_start..value_end]);

        Ok(Self {
            link_id,
            timestamp,
            signature,
        })
    }

    /// Encodes [`Signature`] into [`SignatureBytes`] byte array.
    ///
    /// Used in [`From<MavLinkV2Signature>`](From) trait implementation for [`SignatureBytes`].
    pub fn to_byte_array(&self) -> SignatureBytes {
        let mut bytes: SignatureBytes = Default::default();

        bytes[0] = self.link_id;
        bytes[1..SIGNATURE_TIMESTAMP_LENGTH].copy_from_slice(&self.timestamp);
        bytes[1 + SIGNATURE_TIMESTAMP_LENGTH..SIGNATURE_LENGTH].copy_from_slice(&self.signature);

        bytes
    }
}
