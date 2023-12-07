//! MAVLink patload.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::cmp::min;

use super::{MavLinkMessage, MavLinkVersion};
use crate::errors::MavLinkMessageProcessingError;

/// Maximum size of a payload. Payloads of greater size in most cases will be truncated or cause
/// errors.
pub const MAX_PAYLOAD_SIZE: usize = 255;

#[cfg(not(feature = "alloc"))]
type PayloadContainer = [u8; MAX_PAYLOAD_SIZE];
#[cfg(feature = "alloc")]
type PayloadContainer = alloc::vec::Vec<u8>;

/// MAVlink message payload.
///
/// Encapsulates `MAVLink` payload. In `no_std` non-allocating targets it uses fixed-sized
/// arrays of bytes. Otherwise payload is stored as a [`Vec`].
#[derive(Clone, Debug)]
pub struct MavLinkMessagePayload {
    /// MAVLink message ID.
    id: u32,
    /// Message payload as a sequence of bytes.
    payload: PayloadContainer,
    /// Payload size.
    max_size: usize,
    /// MAVLink protocol version.
    version: MavLinkVersion,
}

#[allow(clippy::derivable_impls)]
impl Default for MavLinkMessagePayload {
    /// Creates [`MavLinkMessagePayload`] populated with default values.
    fn default() -> Self {
        Self {
            id: u32::default(),
            payload: MavLinkMessagePayload::container_default(),
            version: MavLinkVersion::default(),
            max_size: MAX_PAYLOAD_SIZE,
        }
    }
}

impl MavLinkMessagePayload {
    /// Default constructor.
    ///
    /// Upon creation, the length of the provided payload will define
    /// [`MavLinkMessagePayload::max_size`] the maximum length of the
    /// [`MavLinkMessagePayload::payload`].
    ///
    /// If `payload` is longer than [`MAX_PAYLOAD_SIZE`], all trailing elements will be ignored.
    pub fn new(id: u32, payload: &[u8], version: MavLinkVersion) -> Self {
        let max_size = min(MAX_PAYLOAD_SIZE, payload.len());
        let payload = Self::container_from_slice(payload, max_size);
        Self {
            id,
            payload,
            max_size,
            version,
        }
    }

    /// Constructs [`MavLinkMessagePayload`] with specified payload size.
    ///
    /// If `max_size` is greater than [`MAX_PAYLOAD_SIZE`], then it will be ignored and
    /// [`MAX_PAYLOAD_SIZE`] will be used instead.
    pub fn new_sized(id: u32, payload: &[u8], version: MavLinkVersion, max_size: usize) -> Self {
        let max_size = min(MAX_PAYLOAD_SIZE, max_size);
        let payload = Self::container_from_slice(payload, max_size);
        Self {
            id,
            payload,
            max_size,
            version,
        }
    }

    /// `MAVLink` message ID.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Message payload.
    ///
    /// For `MAVLink 2` zero trailing bytes will be truncated.
    /// See [MAVLink 2 payload truncation](https://mavlink.io/en/guide/serialization.html#payload_truncation).
    pub fn payload(&self) -> &[u8] {
        if let MavLinkVersion::V2 = self.version {
            self.truncated()
        } else if self.payload.len() < self.max_size {
            &self.payload
        } else {
            &self.payload[0..self.max_size]
        }
    }

    /// MAVLink protocol version.
    ///
    /// See [`MavLinkVersion`].
    pub fn version(&self) -> MavLinkVersion {
        self.version
    }

    /// Maximum length in bytes of the available payload.
    ///
    /// See [`MavLinkMessagePayload::payload`].
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Payload with trailing zeros truncated.
    pub fn truncated(&self) -> &[u8] {
        let n: usize = self.max_size;
        // Assume that all elements are zeros
        let mut num_non_zero = 0usize;
        // Seek from the end to start
        for i in 1..=n {
            // Stop when non-zero element is found
            if self.payload[n - i] != 0u8 {
                num_non_zero = n - i + 1;
                break;
            }
        }

        &self.payload[0..num_non_zero]
    }

    /// Creates [`PayloadContainer`] populated with values from slice.
    fn container_from_slice(value: &[u8], max_size: usize) -> PayloadContainer {
        // Truncate value up to maximum possible length
        let value: &[u8] = if value.len() > max_size {
            &value[0..max_size]
        } else {
            value
        };

        #[cfg(not(feature = "alloc"))]
        let payload: PayloadContainer = {
            let mut no_std_payload = [0u8; MAX_PAYLOAD_SIZE];
            let max_len = if MAX_PAYLOAD_SIZE < value.len() {
                MAX_PAYLOAD_SIZE
            } else {
                value.len()
            };
            no_std_payload[0..max_len].copy_from_slice(&value[0..max_len]);
            no_std_payload
        };
        #[cfg(feature = "alloc")]
        let payload: PayloadContainer = if value.len() < max_size {
            println!();
            let mut payload = vec![0u8; max_size];
            payload[0..value.len()].copy_from_slice(value);
            payload
        } else {
            PayloadContainer::from(value)
        };

        payload
    }

    /// Creates [`PayloadContainer`] populated with default values.
    fn container_default() -> PayloadContainer {
        #[cfg(not(feature = "alloc"))]
        let default = [0u8; MAX_PAYLOAD_SIZE];
        #[cfg(feature = "alloc")]
        let default = PayloadContainer::default();
        default
    }
}

/// MAVLink message decoder.
///
/// Decodes [`MavLinkMessagePayload`] into [`MavLinkMessage`].
pub trait FromMavLinkPayload {
    /// Creates [`MavLinkMessage`] from specified `payload`.
    fn decode(payload: &MavLinkMessagePayload) -> Result<Self, MavLinkMessageProcessingError>
    where
        Self: MavLinkMessage + Sized;
}

/// MAVLink message encoder.
///
/// Decodes [`MavLinkMessage`] into [`MavLinkMessagePayload`].
pub trait IntoMavlinkPayload: MavLinkMessage {
    /// Encodes message into MAVLink payload.
    ///
    /// # Errors
    ///
    /// * Returns [`MavLinkMessageProcessingError::UnsupportedMavLinkVersion`] if specified
    /// MAVLink `version` is not supported.
    fn encode(
        &self,
        version: MavLinkVersion,
    ) -> Result<MavLinkMessagePayload, MavLinkMessageProcessingError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        // Small initial payload
        let payload = MavLinkMessagePayload::new(0, &[1, 2, 3, 4, 5, 6u8], MavLinkVersion::V1);
        assert_eq!(payload.max_size(), 6);
        assert_eq!(payload.payload().len(), 6);
        assert_eq!(payload.payload(), &[1, 2, 3, 4, 5, 6u8]);

        // Payload with trailing zeros
        let payload = MavLinkMessagePayload::new(0, &[1, 2, 3, 4, 0, 0u8], MavLinkVersion::V1);
        assert_eq!(payload.max_size(), 6);
        assert_eq!(payload.payload().len(), 6);

        // Large initial payload
        let payload =
            MavLinkMessagePayload::new(0, &[1u8; MAX_PAYLOAD_SIZE * 2], MavLinkVersion::V1);
        assert_eq!(payload.max_size(), MAX_PAYLOAD_SIZE);
        assert_eq!(payload.payload().len(), MAX_PAYLOAD_SIZE);
    }

    #[test]
    fn new_sized() {
        // Small initial payload
        let payload =
            MavLinkMessagePayload::new_sized(0, &[1, 2, 3, 4, 5, 6u8], MavLinkVersion::V1, 4);
        assert_eq!(payload.max_size(), 4);
        assert_eq!(payload.payload().len(), 4);
        assert_eq!(payload.payload(), &[1, 2, 3, 4u8]);

        // Small initial payload, excess size
        let payload = MavLinkMessagePayload::new_sized(0, &[1, 2, 3, 4u8], MavLinkVersion::V1, 6);
        assert_eq!(payload.max_size(), 6);
        assert_eq!(payload.payload().len(), 6);
        assert_eq!(payload.payload(), &[1, 2, 3, 4, 0, 0u8]);

        // Large initial payload
        let payload = MavLinkMessagePayload::new_sized(
            0,
            &[1u8; MAX_PAYLOAD_SIZE * 2],
            MavLinkVersion::V1,
            MAX_PAYLOAD_SIZE * 2,
        );
        assert_eq!(payload.max_size(), MAX_PAYLOAD_SIZE);
        assert_eq!(payload.payload().len(), MAX_PAYLOAD_SIZE);
    }

    #[test]
    fn truncated() {
        let payload = MavLinkMessagePayload::new(0, &[1, 2, 3, 4, 0, 0u8], MavLinkVersion::V1);
        assert_eq!(payload.truncated(), [1, 2, 3, 4u8]);
    }
}
