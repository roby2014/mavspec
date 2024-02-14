//! # MAVLink payload

#[cfg(feature = "alloc")]
extern crate alloc;

use core::cmp::min;

#[cfg(feature = "std")]
use std::fmt::{Debug, Formatter};

use crate::consts::PAYLOAD_MAX_SIZE;
use crate::errors::SpecError;
use crate::types::{MavLinkVersion, MessageId};

#[cfg(feature = "alloc")]
type PayloadContainer = alloc::vec::Vec<u8>;
#[cfg(not(feature = "alloc"))]
use no_alloc_payload_container::PayloadContainer;

/// MAVlink message payload.
///
/// Encapsulates MAVLink payload.
/// In `no_std` non-allocating targets it uses fixed-sized
/// arrays of bytes. Otherwise, payload is stored on heap as a dynamically sized sequence.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(not(feature = "std"), derive(Debug))]
pub struct Payload {
    /// MAVLink message ID.
    id: MessageId,
    /// Message payload as a sequence of bytes.
    payload: PayloadContainer,
    /// Payload length.
    length: usize,
    /// MAVLink protocol version.
    version: MavLinkVersion,
}

#[allow(clippy::derivable_impls)]
impl Default for Payload {
    /// Creates [`Payload`] populated with default values.
    fn default() -> Self {
        Self {
            id: MessageId::default(),
            payload: Payload::container_default(),
            version: MavLinkVersion::default(),
            length: PAYLOAD_MAX_SIZE,
        }
    }
}

#[cfg(feature = "std")]
impl Debug for Payload {
    /// Formats [`Payload`] with `payload` truncated up to `max_size`.
    ///
    /// This is important for `no_std` implementations where `payload` has fixed size of
    /// [`PAYLOAD_MAX_SIZE`] bytes.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let payload = match self.version {
            MavLinkVersion::V1 => &self.payload[0..min(self.payload.len(), self.length)],
            MavLinkVersion::V2 => {
                &self.payload[0..min(self.payload.len(), Self::truncated_length(&self.payload))]
            }
        };

        f.debug_struct("payload")
            .field("id", &self.id)
            .field("payload", &payload)
            .field("length", &self.length)
            .field("version", &self.version)
            .finish()
    }
}

impl Payload {
    /// Default constructor.
    ///
    /// Upon creation, the length of the provided payload will define
    /// [`Payload::length`] the maximum length of the
    /// [`Payload::bytes`].
    ///
    /// If `payload` is longer than [`PAYLOAD_MAX_SIZE`], all trailing elements will be ignored.
    pub fn new(id: MessageId, payload: &[u8], version: MavLinkVersion) -> Self {
        let max_size = min(PAYLOAD_MAX_SIZE, payload.len());

        // Define length based on MAVLink protocol version since `MAVLink 2` requires payload truncation.
        let length = match version {
            MavLinkVersion::V1 => max_size,
            MavLinkVersion::V2 => Self::truncated_length(&payload[0..max_size]),
        };

        let payload = Self::container_from_slice(payload, length);

        Self {
            id,
            payload,
            length,
            version,
        }
    }

    /// MAVLink message ID.
    pub fn id(&self) -> MessageId {
        self.id
    }

    /// Message payload as bytes.
    ///
    /// For `MAVLink 2` zero trailing bytes will be truncated.
    /// See [MAVLink 2 payload truncation](https://mavlink.io/en/guide/serialization.html#payload_truncation).
    pub fn bytes(&self) -> &[u8] {
        &self.payload[0..self.length]
    }

    /// MAVLink protocol version.
    ///
    /// See [`MavLinkVersion`].
    pub fn version(&self) -> MavLinkVersion {
        self.version
    }

    /// Payload size in bytes.
    ///
    /// Note that for `MAVLink 2` payloads trailing zero bytes are truncated.  
    ///
    /// See [`Payload::bytes`].
    pub fn length(&self) -> u8 {
        self.length as u8
    }

    /// Upgrade payload to `MAVLink 2` protocol version in-place.
    ///
    /// The reverse procedure is not possible since `MAVLink 2` payload may contain extra fields
    /// and its trailing zero bytes are truncated.
    ///
    /// To replace an existing payload by value, use [`Payload::upgraded`].
    pub fn upgrade(&mut self) {
        self.version = MavLinkVersion::V2;
        self.length = Self::truncated_length(self.bytes());
    }

    /// Upgrade protocol version to `MAVLink 2` replacing payload by value.
    ///
    /// The reverse procedure is not possible since `MAVLink 2` payload may contain extra fields
    /// and its trailing zero bytes are truncated.
    ///
    /// To upgrade payload in-place, use [`Payload::upgrade`].
    pub fn upgraded(self) -> Self {
        Self::new(self.id, self.bytes(), MavLinkVersion::V2)
    }

    fn truncated_length(slice: &[u8]) -> usize {
        let n: usize = slice.len();
        // Assume that all elements are zeros
        let mut num_non_zero = 0usize;
        // Seek from the end to start
        for i in 1..=n {
            // Stop when non-zero element is found
            if slice[n - i] != 0u8 {
                num_non_zero = n - i + 1;
                break;
            }
        }

        num_non_zero
    }

    fn container_from_slice(value: &[u8], max_size: usize) -> PayloadContainer {
        // Truncate value up to maximum possible length
        let value: &[u8] = if value.len() > max_size {
            &value[0..max_size]
        } else {
            value
        };

        #[cfg(not(feature = "alloc"))]
        let payload: PayloadContainer = PayloadContainer {
            content: {
                let mut no_std_payload = [0u8; PAYLOAD_MAX_SIZE];
                let max_len = if PAYLOAD_MAX_SIZE < value.len() {
                    PAYLOAD_MAX_SIZE
                } else {
                    value.len()
                };
                no_std_payload[0..max_len].copy_from_slice(&value[0..max_len]);
                no_std_payload
            },
        };
        #[cfg(feature = "alloc")]
        let payload: PayloadContainer = if value.len() < max_size {
            let mut payload = alloc::vec![0u8; max_size];
            payload[0..value.len()].copy_from_slice(value);
            payload
        } else {
            PayloadContainer::from(value)
        };

        payload
    }

    fn container_default() -> PayloadContainer {
        PayloadContainer::default()
    }
}

/// MAVLink message encoder.
///
/// Decodes MAVLink message into [`Payload`].
pub trait IntoPayload {
    /// Encodes message into MAVLink payload.
    ///
    /// # Errors
    ///
    /// * Returns [`SpecError::UnsupportedMavLinkVersion`] if specified
    /// MAVLink `version` is not supported.
    fn encode(&self, version: MavLinkVersion) -> Result<Payload, SpecError>;
}

#[cfg(not(feature = "alloc"))]
mod no_alloc_payload_container {
    use crate::payload::PAYLOAD_MAX_SIZE;

    #[derive(Clone, Debug)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct PayloadContainer {
        #[cfg_attr(feature = "serde", serde(with = "serde_arrays"))]
        pub(super) content: [u8; PAYLOAD_MAX_SIZE],
    }

    impl Default for PayloadContainer {
        fn default() -> Self {
            Self {
                content: [0u8; PAYLOAD_MAX_SIZE],
            }
        }
    }

    impl<Idx> core::ops::Index<Idx> for PayloadContainer
    where
        Idx: core::slice::SliceIndex<[u8]>,
    {
        type Output = Idx::Output;

        fn index(&self, index: Idx) -> &Self::Output {
            &self.content[index]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_payload() {
        // Small initial payload
        let payload = Payload::new(0, &[1, 2, 3, 4, 5, 6u8], MavLinkVersion::V1);
        assert_eq!(payload.length(), 6);
        assert_eq!(payload.bytes().len(), 6);
        assert_eq!(payload.bytes(), &[1, 2, 3, 4, 5, 6u8]);

        // Payload with trailing zeros V1
        let payload = Payload::new(0, &[1, 2, 3, 4, 0, 0u8], MavLinkVersion::V1);
        assert_eq!(payload.length(), 6);
        assert_eq!(payload.bytes().len(), 6);

        // Payload with trailing zeros V2
        let payload = Payload::new(0, &[1, 2, 3, 4, 0, 0u8], MavLinkVersion::V2);
        assert_eq!(payload.length(), 4);
        assert_eq!(payload.bytes().len(), 4);

        // Large initial payload
        let payload = Payload::new(0, &[1u8; PAYLOAD_MAX_SIZE * 2], MavLinkVersion::V1);
        assert_eq!(payload.length() as usize, PAYLOAD_MAX_SIZE);
        assert_eq!(payload.bytes().len(), PAYLOAD_MAX_SIZE);
    }

    #[test]
    fn truncated_length() {
        assert_eq!(Payload::truncated_length(&[1, 2, 3, 4, 5, 6u8]), 6);
        assert_eq!(Payload::truncated_length(&[1, 2, 3, 4, 0, 0u8]), 4);
    }
}
