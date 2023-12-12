//! # MAVLink header
//!
//! This module contains implementation for MAVLink packet header both for `MAVLink 1` and
//! `MAVLink 2`.

use mavlib_spec::MavLinkVersion;
use tbytes::{TBytesReader, TBytesReaderFor};

use crate::consts::{
    CHECKSUM_SIZE, HEADER_MAX_SIZE, HEADER_MIN_SIZE, HEADER_V1_SIZE, HEADER_V2_SIZE,
    MAVLINK_IFLAG_SIGNED, SIGNATURE_LENGTH, STX_V1, STX_V2,
};
use crate::errors::{CoreError, FrameError, Result};
use crate::io::Read;
use crate::stx::MavSTX;
use crate::types::{HeaderV1Bytes, HeaderV2Bytes, MessageId};

/// MAVLink frame header.
///
/// Header contains information relevant to for `MAVLink 1` and `MAVLink 2` packet formats.
///
/// See:
///  * [MAVLink 1 packet format](https://mavlink.io/en/guide/serialization.html#v1_packet_format).
///  * [MAVLink 2 packet format](https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format).
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    /// MAVLink protocol version.
    mavlink_version: MavLinkVersion,
    /// Payload length.
    payload_length: u8,
    /// Fields related to `MAVLink 2` headers.
    mavlink_v2_fields: Option<HeaderV2Fields>,
    /// Packet sequence number.
    sequence: u8,
    /// System `ID`.
    system_id: u8,
    /// Component `ID`.
    component_id: u8,
    /// Message `ID`.
    message_id: MessageId,
    /// Header as a sequence of bytes.
    bytes: [u8; HEADER_MAX_SIZE],
}

/// Fields related to `MAVLink 2` packet header.
///
/// See: [MAVLink 2 packet format](https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HeaderV2Fields {
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

impl TryFrom<HeaderV1Bytes> for Header {
    type Error = CoreError;

    /// Decodes [`Header`] from [`HeaderV1Bytes`] bytes.
    ///
    /// See [`Header::try_from_v1_bytes`].
    fn try_from(value: HeaderV1Bytes) -> Result<Self> {
        Self::try_from_v1_bytes(value)
    }
}

impl TryFrom<HeaderV2Bytes> for Header {
    type Error = CoreError;

    /// Decodes [`Header`] from [`HeaderV2Bytes`] bytes.
    ///
    /// See [`Header::try_from_v2_bytes`].
    fn try_from(value: HeaderV2Bytes) -> Result<Self> {
        Self::try_from_v2_bytes(value)
    }
}

impl TryFrom<&[u8]> for Header {
    type Error = CoreError;

    /// Decodes a slice of bytes into [`Header`].
    ///
    /// See [`Header::try_from_slice`].
    fn try_from(value: &[u8]) -> Result<Self> {
        Self::try_from_slice(value)
    }
}

impl Header {
    /// Initiates builder for [`Header`].
    ///
    /// Instead of constructor we use
    /// [builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
    /// pattern. An instance of [`HeaderBuilder`] returned by this function is initialized
    /// with default values. Once desired values are set, you can call [`HeaderBuilder::build`]
    /// to obtain [`Header`].
    pub fn builder() -> HeaderBuilder {
        HeaderBuilder::new()
    }

    /// MAVLink protocol version.
    ///
    /// MAVLink version defined by the magic byte (STX).
    ///
    /// See [`MavSTX`].
    pub fn mavlink_version(&self) -> MavLinkVersion {
        self.mavlink_version
    }

    /// Fields related to `MAVLink 2` headers.
    ///
    /// See:
    ///  * [`HeaderV2Fields`].
    ///  * [MAVLink 2 packet format](https://mavlink.io/en/guide/serialization.html#mavlink2_packet_format).
    pub fn mavlink_v2_fields(&self) -> Option<&HeaderV2Fields> {
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
    pub fn message_id(&self) -> MessageId {
        self.message_id
    }

    /// Size of the header in bytes.
    ///
    /// Depends on the MAVLink protocol version.
    pub fn size(&self) -> usize {
        match self.mavlink_version {
            MavLinkVersion::V1 => HEADER_V1_SIZE,
            MavLinkVersion::V2 => HEADER_V2_SIZE,
        }
    }

    /// Returns `true` if `MAVLink 2` frame body should contain signature.
    ///
    /// See [Frame::signature](crate::frame::Frame::signature).
    pub fn is_signature_required(&self) -> Result<bool> {
        Ok(match self.mavlink_version {
            MavLinkVersion::V1 => false,
            MavLinkVersion::V2 => match self.mavlink_v2_fields {
                Some(HeaderV2Fields { incompat_flags, .. }) => {
                    incompat_flags & MAVLINK_IFLAG_SIGNED == MAVLINK_IFLAG_SIGNED
                }
                None => return Err(FrameError::InconsistentV2Header.into()),
            },
        })
    }

    /// Expected MAVLink frame body length.
    ///
    /// # Errors
    ///
    /// Returns [`FrameError::InconsistentV2Header`] if header does not have
    /// `MAVLink 2` specific fields.
    pub fn expected_body_length(&self) -> Result<usize> {
        Ok(match self.mavlink_version {
            MavLinkVersion::V1 => self.payload_length as usize + CHECKSUM_SIZE,
            MavLinkVersion::V2 => {
                if self.is_signature_required()? {
                    self.payload_length as usize + CHECKSUM_SIZE + SIGNATURE_LENGTH
                } else {
                    self.payload_length as usize + CHECKSUM_SIZE
                }
            }
        })
    }

    /// MAVLink packet header size for this protocol version.
    ///
    /// See [`MavLinkVersion`].
    pub fn header_size(version: MavLinkVersion) -> usize {
        match version {
            MavLinkVersion::V1 => HEADER_V1_SIZE,
            MavLinkVersion::V2 => HEADER_V1_SIZE,
        }
    }

    /// Read and decode [`Header`] from the instance of [`Read`].
    pub fn recv<R: Read>(reader: &mut R) -> Result<Self> {
        loop {
            // Read minimum amount of bytes required for a valid MAVLink header
            let mut buffer = [0u8; HEADER_MIN_SIZE];
            reader.read_exact(&mut buffer)?;

            // Look for the `magic` byte
            let mut mavlink_version: Option<MavLinkVersion> = None;
            let mut header_start_idx = buffer.len();
            for (i, &byte) in buffer.iter().enumerate() {
                if MavSTX::is_magic_byte(byte) {
                    header_start_idx = i;
                    mavlink_version = MavLinkVersion::try_from(MavSTX::from(byte)).ok();
                }
            }

            match mavlink_version {
                // If `magic` byte wasn't found, continue to the next attempt
                None => continue,
                // If `magic` byte is found, read the remaining bytes for the corresponding header
                // size and decode header.
                Some(version) => {
                    // Number of header bytes already available in buffer
                    let num_read_bytes = buffer.len() - header_start_idx;
                    // Slice of the header bytes already available in buffer
                    let header_start_bytes = &buffer[header_start_idx..buffer.len()];

                    // Copy all bytes starting from `header_start_idx` byte position from
                    // `buffer` to `header_bytes`, then read remaining bytes. Once all bytes
                    // required for the header of specific MAVLink protocol version, construct and
                    // return header.
                    return match version {
                        MavLinkVersion::V1 => {
                            let mut header_bytes = [0u8; HEADER_V1_SIZE];
                            header_bytes[0..num_read_bytes].copy_from_slice(header_start_bytes);
                            if num_read_bytes < HEADER_V1_SIZE {
                                reader.read_exact(
                                    &mut header_bytes[num_read_bytes..HEADER_V1_SIZE],
                                )?;
                            }
                            Self::try_from_v1_bytes(header_bytes)
                        }
                        MavLinkVersion::V2 => {
                            let mut header_bytes = [0u8; HEADER_V2_SIZE];
                            header_bytes[0..num_read_bytes].copy_from_slice(header_start_bytes);
                            if num_read_bytes < HEADER_V2_SIZE {
                                reader.read_exact(
                                    &mut header_bytes[num_read_bytes..HEADER_V2_SIZE],
                                )?;
                            }
                            Self::try_from_v2_bytes(header_bytes)
                        }
                    };
                }
            }
        }
    }

    /// Decodes [`Header`] from [`HeaderV1Bytes`] bytes.
    ///
    /// Used in [`TryFrom<HeaderV1Bytes>`] trait implementation for [`Header`].
    ///
    /// # Errors
    ///
    /// Returns [`FrameError::InvalidMavLinkVersion`] if `magic` byte is not equal to
    /// [`STX_V1`].
    pub fn try_from_v1_bytes(bytes: HeaderV1Bytes) -> Result<Self> {
        let magic = bytes[0];
        if magic != STX_V1 {
            return Err(FrameError::InvalidMavLinkVersion.into());
        }
        // Decode
        Self::try_from_slice(bytes.as_slice())
    }

    /// Decodes [`Header`] from [`HeaderV2Bytes`] bytes.
    ///
    /// Used in [`TryFrom<HeaderV2Bytes>`] trait implementation for [`Header`].
    ///
    /// # Errors
    ///
    /// Returns [`FrameError::InvalidMavLinkVersion`] if `magic` byte is not equal to
    /// [`STX_V2`].
    pub fn try_from_v2_bytes(bytes: HeaderV2Bytes) -> Result<Self> {
        let magic = bytes[0];
        if magic != STX_V2 {
            return Err(FrameError::InvalidMavLinkVersion.into());
        }
        // Decode
        Self::try_from_slice(bytes.as_slice())
    }

    /// Decodes a slice of bytes into [`Header`].
    ///
    /// Used in [`TryFrom<&[u8]>`](TryFrom) trait implementation for [`Header`].
    pub fn try_from_slice(bytes: &[u8]) -> Result<Self> {
        // Validate header
        Header::validate_slice(bytes)?;

        let reader = TBytesReader::from(bytes);

        let magic: u8 = reader.read()?;
        let mavlink_version: MavLinkVersion = MavLinkVersion::try_from(MavSTX::from(magic))?;
        let payload_length: u8 = reader.read()?;
        let mavlink_v2_fields = if let MavLinkVersion::V2 = mavlink_version {
            Some(HeaderV2Fields {
                incompat_flags: reader.read()?,
                compat_flags: reader.read()?,
            })
        } else {
            None
        };

        let sequence: u8 = reader.read()?;
        let system_id: u8 = reader.read()?;
        let component_id: u8 = reader.read()?;
        let message_id: MessageId = match mavlink_version {
            MavLinkVersion::V1 => {
                let version: u8 = reader.read()?;
                version as MessageId
            }
            MavLinkVersion::V2 => {
                let version_byte: [u8; 4] = [reader.read()?, reader.read()?, reader.read()?, 0];
                MessageId::from_le_bytes(version_byte)
            }
        };

        let mut header_bytes = [0u8; HEADER_MAX_SIZE];
        header_bytes[0..bytes.len()].copy_from_slice(bytes);

        Ok(Self {
            mavlink_version,
            payload_length,
            mavlink_v2_fields,
            sequence,
            system_id,
            component_id,
            message_id,
            bytes: header_bytes,
        })
    }

    /// Validates that provided header can be converted to [`Header`].
    pub fn validate_slice(value: &[u8]) -> Result<()> {
        // Validate that header has minimum required size
        if value.len() < HEADER_MIN_SIZE {
            return Err(FrameError::HeaderIsTooSmall.into());
        }

        // Retrieve and validate magic byte
        let magic = value[0];
        if !MavSTX::is_magic_byte(magic) {
            return Err(FrameError::InvalidMavLinkVersion.into());
        }

        // Validate that header contains enough data according to specific MAVLink protocol
        match magic {
            _ if magic == STX_V1 => {
                if value.len() < HEADER_V1_SIZE {
                    return Err(FrameError::V1HeaderIsTooSmall.into());
                }
            }
            _ if magic == STX_V2 => {
                if value.len() < HEADER_V2_SIZE {
                    return Err(FrameError::V2HeaderIsTooSmall.into());
                }
            }
            _ => return Err(FrameError::InvalidMavLinkVersion.into()),
        }

        Ok(())
    }

    /// [`Header`] CRC data.
    ///
    /// Returns all header data excluding `magic` byte.
    ///
    /// See:
    ///  * [`MavLinkFrame::calculate_crc`](crate::frame::Frame::calculate_crc).
    ///  * [MAVLink checksum](https://mavlink.io/en/guide/serialization.html#checksum) in MAVLink
    ///    protocol documentation.
    pub fn crc_data(&self) -> &[u8] {
        &self.bytes[1..self.size()]
    }
}

/// Builder for [`Header`].
///
/// Implements [builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
/// pattern for [`Header`].
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HeaderBuilder {
    header: Header,
}

impl HeaderBuilder {
    /// Default constructor.
    pub fn new() -> HeaderBuilder {
        Self::default()
    }

    /// Builds [`Header`].
    pub fn build(&self) -> Result<Header> {
        self.validate()?;
        Ok(self.header)
    }

    /// Validates header for consistency.
    pub fn validate(&self) -> Result<()> {
        match self.header.mavlink_version {
            // `MAVLink 1` header should not have `MAVLink 2` fields
            MavLinkVersion::V1 => {
                if self.header.mavlink_v2_fields.is_some() {
                    return Err(FrameError::InconsistentV1Header.into());
                }
            }
            // `MAVLink 2` header should have `MAVLink 2` fields
            MavLinkVersion::V2 => {
                if self.header.mavlink_v2_fields.is_none() {
                    return Err(FrameError::InconsistentV2Header.into());
                }
            }
        }
        Ok(())
    }

    /// Sets MAVLink protocol version.
    ///
    /// See: [`Header::mavlink_version`].
    pub fn set_mavlink_version(&mut self, mavlink_version: MavLinkVersion) -> &mut Self {
        self.header.mavlink_version = mavlink_version;
        self
    }

    /// Sets fields related to `MAVLink 2` headers.
    ///
    /// See: [`Header::mavlink_v2_fields`].
    pub fn set_mavlink_v2_fields(
        &mut self,
        mavlink_v2_fields: Option<HeaderV2Fields>,
    ) -> &mut Self {
        self.header.mavlink_v2_fields = mavlink_v2_fields;
        self
    }

    /// Sets payload length.
    ///
    /// See: [`Header::payload_length`].
    pub fn set_payload_length(&mut self, payload_length: u8) -> &mut Self {
        self.header.payload_length = payload_length;
        self
    }

    /// Sets packet sequence number.
    ///
    /// See: [`Header::sequence`].
    pub fn set_sequence(&mut self, sequence: u8) -> &mut Self {
        self.header.sequence = sequence;
        self
    }

    /// Sets system `ID`.
    ///
    /// See: [`Header::system_id`].
    pub fn system_id(&mut self, system_id: u8) -> &mut Self {
        self.header.system_id = system_id;
        self
    }

    /// Sets component `ID`.
    ///
    /// See: [`Header::component_id`].
    pub fn set_component_id(&mut self, component_id: u8) -> &mut Self {
        self.header.component_id = component_id;
        self
    }

    /// Sets message `ID`.
    ///
    /// See: [`Header::message_id`].
    pub fn set_message_id(&mut self, message_id: u32) -> &mut Self {
        self.header.message_id = message_id;
        self
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;
    use crate::consts::STX_V1;
    use std::io::Cursor;

    #[test]
    fn read_v1_header() {
        let mut buffer = Cursor::new(vec![
            12,     // \
            24,     //  | Junk bytes
            240,    // /
            STX_V1, // magic byte
            8,      // payload_length
            1,      // sequence
            10,     // system ID
            255,    // component ID
            0,      // message ID
        ]);

        let header = Header::recv(&mut buffer).unwrap();

        assert!(matches!(header.mavlink_version, MavLinkVersion::V1));
        assert_eq!(header.payload_length, 8u8);
        assert_eq!(header.sequence, 1u8);
        assert_eq!(header.system_id, 10u8);
        assert_eq!(header.component_id, 255u8);
        assert_eq!(header.message_id, 0u32);
        assert!(header.mavlink_v2_fields.is_none());
    }

    #[test]
    fn read_v2_header() {
        let mut reader = Cursor::new(vec![
            12,     // \
            24,     //  |Junk bytes
            240,    // /
            STX_V2, // magic byte
            8,      // payload_length
            1,      // incompatibility flags
            0,      // compatibility flags
            1,      // sequence
            10,     // system ID
            255,    // component ID
            0,      // \
            0,      //  | message ID
            0,      // /
        ]);

        let header = Header::recv(&mut reader).unwrap();

        assert!(matches!(header.mavlink_version, MavLinkVersion::V2));
        assert_eq!(header.payload_length, 8u8);
        assert_eq!(header.mavlink_v2_fields.unwrap().incompat_flags, 1u8);
        assert_eq!(header.mavlink_v2_fields.unwrap().compat_flags, 0u8);
        assert_eq!(header.sequence, 1u8);
        assert_eq!(header.system_id, 10u8);
        assert_eq!(header.component_id, 255u8);
        assert_eq!(header.message_id, 0u32);
    }
}
