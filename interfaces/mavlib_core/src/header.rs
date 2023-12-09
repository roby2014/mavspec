//! # `MAVLink` header
//!
//! This module contains implementation for `MAVLink` packet header both for `MAVLink 1` and
//! `MAVLink 2`.

use crate::consts::{
    MAVLINK_CHECKSUM_SIZE, MAVLINK_MIN_HEADER_SIZE, MAVLINK_V1_HEADER_SIZE, MAVLINK_V2_HEADER_SIZE,
    MAVLINK_V2_IFLAG_SIGNED, MAVLINK_V2_SIGNATURE_LENGTH, STX_MAVLINK_1, STX_MAVLINK_2,
};
use crate::errors::MavLinkFrameDecodingError;
use crate::stx::MavSTX;
use crate::types::{MavLinkMessageId, MavLinkV1Header, MavLinkV2Header};
use crate::MavLinkVersion;
use tbytes::{TBytesReader, TBytesReaderFor};

/// MAVLink frame header.
///
/// Header contains information relevant to for `MAVLink 1` and `MAVLink 2` packet formats.
///
/// See:
///  * [MAVLink 1 packet format](https://mavlink.io/en/guide/serialization.html#v1_packet_format).
///  * [MAVLink 2 packet format](https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format).
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MavLinkHeader {
    /// `MAVLink` protocol version.
    pub(crate) mavlink_version: MavLinkVersion,
    /// Payload length.
    pub(crate) payload_length: u8,
    /// Fields related to `MAVLink 2` headers.
    pub(crate) mavlink_v2_fields: Option<MavLinkV2HeaderFields>,
    /// Packet sequence number.
    pub(crate) sequence: u8,
    /// System `ID`.
    pub(crate) system_id: u8,
    /// Component `ID`.
    pub(crate) component_id: u8,
    /// Message `ID`.
    pub(crate) message_id: MavLinkMessageId,
}

/// Fields related to `MAVLink 2` packet header.
///
/// See: [MAVLink 2 packet format](https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MavLinkV2HeaderFields {
    /// Incompatibility Flags.
    ///
    /// Flags that must be understood for MAVLink compatibility (implementation discards packet if
    /// it does not understand flag).
    ///
    /// See: [MAVLink 2 incompatibility flags](https://mavlink.io/en/guide/serialization.html#incompat_flags).
    pub incompat_flags: u8,
    /// Compatibility Flags.
    ///
    /// Flags that can be ignored if not understood (implementation can still handle packet even if
    /// it does not understand flag).
    ///
    /// See: [MAVLink 2 compatibility flags](https://mavlink.io/en/guide/serialization.html#compat_flags).
    pub compat_flags: u8,
}

impl TryFrom<MavLinkV1Header> for MavLinkHeader {
    type Error = MavLinkFrameDecodingError;

    /// Decodes [`MavLinkHeader`] from [`MavLinkV1Header`].
    ///
    /// # Errors
    ///
    /// Returns [`MavLinkFrameDecodingError::InvalidMavLinkVersion`] if `magic` byte is not equal to
    /// [`STX_MAVLINK_1`].
    fn try_from(value: MavLinkV1Header) -> Result<Self, Self::Error> {
        let magic = value[0];
        if magic != STX_MAVLINK_1 {
            return Err(MavLinkFrameDecodingError::InvalidMavLinkVersion);
        }
        // Decode
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<MavLinkV2Header> for MavLinkHeader {
    type Error = MavLinkFrameDecodingError;

    /// Decodes [`MavLinkHeader`] from [`MavLinkV2Header`].
    ///
    /// # Errors
    ///
    /// Returns [`MavLinkFrameDecodingError::InvalidMavLinkVersion`] if `magic` byte is not equal to
    /// [`STX_MAVLINK_2`].
    fn try_from(value: MavLinkV2Header) -> Result<Self, Self::Error> {
        let magic = value[0];
        if magic != STX_MAVLINK_2 {
            return Err(MavLinkFrameDecodingError::InvalidMavLinkVersion);
        }
        // Decode
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<&[u8]> for MavLinkHeader {
    type Error = MavLinkFrameDecodingError;

    /// Decodes a slice of bytes into [`MavLinkHeader`]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // Validate header
        MavLinkHeader::validate_slice(value)?;

        let reader = TBytesReader::from(value);

        let magic: u8 = reader.read()?;
        let mavlink_version: MavLinkVersion = MavLinkVersion::try_from(MavSTX::from(magic))?;
        let payload_length: u8 = reader.read()?;
        let mavlink_v2_fields = if let MavLinkVersion::V2 = mavlink_version {
            Some(MavLinkV2HeaderFields {
                incompat_flags: reader.read()?,
                compat_flags: reader.read()?,
            })
        } else {
            None
        };

        let sequence: u8 = reader.read()?;
        let system_id: u8 = reader.read()?;
        let component_id: u8 = reader.read()?;
        let message_id: MavLinkMessageId = match mavlink_version {
            MavLinkVersion::V1 => {
                let version: u8 = reader.read()?;
                version as MavLinkMessageId
            }
            MavLinkVersion::V2 => {
                let version_byte: [u8; 4] = [reader.read()?, reader.read()?, reader.read()?, 0];
                MavLinkMessageId::from_le_bytes(version_byte)
            }
        };

        Ok(Self {
            mavlink_version,
            payload_length,
            mavlink_v2_fields,
            sequence,
            system_id,
            component_id,
            message_id,
        })
    }
}

impl MavLinkHeader {
    /// Initiates builder for [`MavLinkHeader`].
    ///
    /// Instead of constructor we use
    /// [builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
    /// pattern. An instance of [`MavLinkHeaderBuilder`] returned by this function is initialized
    /// with default values. Once desired values are set, you can call [`MavLinkHeaderBuilder::build`]
    /// to obtain [`MavLinkHeader`].
    pub fn builder() -> MavLinkHeaderBuilder {
        MavLinkHeaderBuilder::new()
    }

    /// `MAVLink` protocol version.
    ///
    /// `MAVLink` version defined by the magic byte (STX).
    ///
    /// See [`MavSTX`].
    pub fn mavlink_version(&self) -> MavLinkVersion {
        self.mavlink_version
    }

    /// Fields related to `MAVLink 2` headers.
    ///
    /// See:
    ///  * [`MavLinkV2HeaderFields`].
    ///  * [MAVLink 2 packet format](https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format).
    pub fn mavlink_v2_fields(&self) -> Option<&MavLinkV2HeaderFields> {
        self.mavlink_v2_fields.as_ref()
    }

    /// Payload length.
    ///
    /// Indicates length of the following `payload` section. This may be affected by payload truncation.
    pub fn payload_length(&self) -> u8 {
        self.payload_length
    }

    /// Packet sequence number.
    ///
    /// Used to detect packet loss. Components increment value for each message sent.
    pub fn sequence(&self) -> u8 {
        self.sequence
    }

    /// System `ID`.
    ///
    /// `ID` of system (vehicle) sending the message. Used to differentiate systems on network.
    ///
    /// > Note that the broadcast address 0 may not be used in this field as it is an invalid source
    /// > address.
    pub fn system_id(&self) -> u8 {
        self.system_id
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
        self.component_id
    }

    /// Message `ID`.
    ///
    /// `ID` of message type in payload.
    ///
    /// Used to decode data back into message object.
    pub fn message_id(&self) -> MavLinkMessageId {
        self.message_id
    }

    /// Size of the header in bytes.
    ///
    /// Depends on the `MAVLink` protocol version.
    pub fn size(&self) -> usize {
        match self.mavlink_version {
            MavLinkVersion::V1 => MAVLINK_V1_HEADER_SIZE,
            MavLinkVersion::V2 => MAVLINK_V2_HEADER_SIZE,
        }
    }

    /// Returns `true` if `MAVLink 2` frame body should contain signature.
    ///
    /// See [MavLinkFrame::signature](crate::frame::MavLinkFrame::signature).
    pub fn is_signature_required(&self) -> Result<bool, MavLinkFrameDecodingError> {
        Ok(match self.mavlink_version {
            MavLinkVersion::V1 => false,
            MavLinkVersion::V2 => match self.mavlink_v2_fields {
                Some(MavLinkV2HeaderFields { incompat_flags, .. }) => {
                    incompat_flags & MAVLINK_V2_IFLAG_SIGNED == MAVLINK_V2_IFLAG_SIGNED
                }
                None => return Err(MavLinkFrameDecodingError::InconsistentV2Header),
            },
        })
    }

    /// Expected `MAVLink` frame body length.
    ///
    /// # Errors
    ///
    /// Returns [`MavLinkFrameDecodingError::InconsistentV2Header`] if header does not have
    /// `MAVLink 2` specific fields.
    pub fn expected_body_length(&self) -> Result<usize, MavLinkFrameDecodingError> {
        Ok(match self.mavlink_version {
            MavLinkVersion::V1 => self.payload_length as usize + MAVLINK_CHECKSUM_SIZE,
            MavLinkVersion::V2 => {
                if self.is_signature_required()? {
                    self.payload_length as usize
                        + MAVLINK_CHECKSUM_SIZE
                        + MAVLINK_V2_SIGNATURE_LENGTH
                } else {
                    self.payload_length as usize + MAVLINK_CHECKSUM_SIZE
                }
            }
        })
    }

    /// Validates that provided header can be converted to [`MavLinkHeader`].
    pub fn validate_slice(value: &[u8]) -> Result<(), MavLinkFrameDecodingError> {
        // Validate that header has minimum required size
        if value.len() < MAVLINK_MIN_HEADER_SIZE {
            return Err(MavLinkFrameDecodingError::HeaderIsTooSmall);
        }

        // Retrieve and validate magic byte
        let magic = value[0];
        if !MavSTX::is_magic_byte(magic) {
            return Err(MavLinkFrameDecodingError::InvalidMavLinkVersion);
        }

        // Validate that header contains enough data according to specific `MAVLink` protocol
        match magic {
            _ if magic == STX_MAVLINK_1 => {
                if value.len() < MAVLINK_V1_HEADER_SIZE {
                    return Err(MavLinkFrameDecodingError::V1HeaderIsTooSmall);
                }
            }
            _ if magic == STX_MAVLINK_2 => {
                if value.len() < MAVLINK_V2_HEADER_SIZE {
                    return Err(MavLinkFrameDecodingError::V2HeaderIsTooSmall);
                }
            }
            _ => return Err(MavLinkFrameDecodingError::InvalidMavLinkVersion),
        }

        Ok(())
    }
}

/// Builder for [`MavLinkHeader`].
///
/// Implements [builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
/// pattern for [`MavLinkHeader`].
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MavLinkHeaderBuilder {
    header: MavLinkHeader,
}

impl MavLinkHeaderBuilder {
    /// Default constructor.
    pub fn new() -> MavLinkHeaderBuilder {
        Self::default()
    }

    /// Builds [`MavLinkHeader`].
    pub fn build(&self) -> Result<MavLinkHeader, MavLinkFrameDecodingError> {
        self.validate()?;
        Ok(self.header)
    }

    /// Validates header for consistency.
    pub fn validate(&self) -> Result<(), MavLinkFrameDecodingError> {
        match self.header.mavlink_version {
            // `MAVLink 1` header should not have `MAVLink 2` fields
            MavLinkVersion::V1 => {
                if self.header.mavlink_v2_fields.is_some() {
                    return Err(MavLinkFrameDecodingError::InconsistentV1Header);
                }
            }
            // `MAVLink 2` header should have `MAVLink 2` fields
            MavLinkVersion::V2 => {
                if self.header.mavlink_v2_fields.is_none() {
                    return Err(MavLinkFrameDecodingError::InconsistentV2Header);
                }
            }
        }
        Ok(())
    }

    /// Sets `MAVLink` protocol version.
    ///
    /// See: [`MavLinkHeader::mavlink_version`].
    pub fn set_mavlink_version(&mut self, mavlink_version: MavLinkVersion) -> &mut Self {
        self.header.mavlink_version = mavlink_version;
        self
    }

    /// Sets fields related to `MAVLink 2` headers.
    ///
    /// See: [`MavLinkHeader::mavlink_v2_fields`].
    pub fn set_mavlink_v2_fields(
        &mut self,
        mavlink_v2_fields: Option<MavLinkV2HeaderFields>,
    ) -> &mut Self {
        self.header.mavlink_v2_fields = mavlink_v2_fields;
        self
    }

    /// Sets payload length.
    ///
    /// See: [`MavLinkHeader::payload_length`].
    pub fn set_payload_length(&mut self, payload_length: u8) -> &mut Self {
        self.header.payload_length = payload_length;
        self
    }

    /// Sets packet sequence number.
    ///
    /// See: [`MavLinkHeader::sequence`].
    pub fn set_sequence(&mut self, sequence: u8) -> &mut Self {
        self.header.sequence = sequence;
        self
    }

    /// Sets system `ID`.
    ///
    /// See: [`MavLinkHeader::system_id`].
    pub fn system_id(&mut self, system_id: u8) -> &mut Self {
        self.header.system_id = system_id;
        self
    }

    /// Sets component `ID`.
    ///
    /// See: [`MavLinkHeader::component_id`].
    pub fn set_component_id(&mut self, component_id: u8) -> &mut Self {
        self.header.component_id = component_id;
        self
    }

    /// Sets message `ID`.
    ///
    /// See: [`MavLinkHeader::message_id`].
    pub fn set_message_id(&mut self, message_id: u32) -> &mut Self {
        self.header.message_id = message_id;
        self
    }
}
