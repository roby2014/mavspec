//! # MAVLink header
//!
//! This module contains implementation for MAVLink packet header both for `MAVLink 1` and
//! `MAVLink 2`.

use mavlib_spec::consts::{MESSAGE_ID_V1_MAX, MESSAGE_ID_V2_MAX};
use tbytes::{TBytesReader, TBytesReaderFor};

use crate::consts::{
    CHECKSUM_SIZE, HEADER_MAX_SIZE, HEADER_MIN_SIZE, HEADER_V1_SIZE, HEADER_V2_SIZE,
    MAVLINK_IFLAG_SIGNED, SIGNATURE_LENGTH, STX_V1, STX_V2,
};
use crate::errors::{CoreError, FrameError, Result};
use crate::io::Read;
use crate::protocol::{CompatFlags, IncompatFlags, MavSTX};
use crate::protocol::{HeaderV1Bytes, HeaderV2Bytes, MavLinkVersion, MessageId};

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
    v2_fields: Option<HeaderV2Fields>,
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
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HeaderV2Fields {
    /// Incompatibility Flags.
    ///
    /// Flags that must be understood for MAVLink compatibility (implementation discards packet if
    /// it does not understand flag).
    ///
    /// See: [MAVLink 2 incompatibility flags](https://mavlink.io/en/guide/serialization.html#incompat_flags).
    pub incompat_flags: IncompatFlags,
    /// Compatibility Flags.
    ///
    /// Flags that can be ignored if not understood (implementation can still handle packet even if
    /// it does not understand flag).
    ///
    /// See: [MAVLink 2 compatibility flags](https://mavlink.io/en/guide/serialization.html#compat_flags).
    pub compat_flags: CompatFlags,
}

/// Builder for [`Header`].
///
/// Implements [builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
/// pattern for [`Header`].
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HeaderBuilder {
    /// MAVLink protocol version.
    mavlink_version: Option<MavLinkVersion>,
    /// Payload length.
    payload_length: Option<u8>,
    /// Packet sequence number.
    sequence: Option<u8>,
    /// System `ID`.
    system_id: Option<u8>,
    /// Component `ID`.
    component_id: Option<u8>,
    /// Message `ID`.
    message_id: Option<MessageId>,
    /// Incompatibility Flags.
    pub incompat_flags: Option<u8>,
    /// Compatibility Flags.
    pub compat_flags: Option<u8>,
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

impl HeaderV2Fields {
    /// Returns `true` if `MAVLink 2` frame body should contain signature.
    ///
    /// Checks that [`MAVLINK_IFLAG_SIGNED`] flag is set for `incompat_flags`.
    ///
    /// #Links
    ///
    /// * [Frame::signature](crate::protocol::Frame::signature).
    pub fn is_signed(&self) -> bool {
        self.incompat_flags & MAVLINK_IFLAG_SIGNED == MAVLINK_IFLAG_SIGNED
    }

    /// Sets whether `MAVLink 2` frame body should contain signature.
    ///
    /// Sets [`MAVLINK_IFLAG_SIGNED`] flag for `incompat_flags`.
    ///
    /// #Links
    ///
    /// * [Frame::signature](crate::protocol::Frame::signature).
    pub fn set_is_signed(&mut self, flag: bool) {
        self.incompat_flags =
            self.incompat_flags & !MAVLINK_IFLAG_SIGNED | (MAVLINK_IFLAG_SIGNED & flag as u8);
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
    pub fn v2_fields(&self) -> Option<&HeaderV2Fields> {
        self.v2_fields.as_ref()
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
    /// #Links
    ///
    /// * [Frame::signature](crate::protocol::Frame::signature).
    pub fn is_signed(&self) -> Result<bool> {
        Ok(match self.mavlink_version {
            MavLinkVersion::V1 => false,
            MavLinkVersion::V2 => match self.v2_fields {
                Some(v2_fields) => v2_fields.is_signed(),
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
                if self.is_signed()? {
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
    pub(crate) fn recv<R: Read>(reader: &mut R) -> Result<Self> {
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
            v2_fields: mavlink_v2_fields,
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
                    return Err(FrameError::HeaderV1IsTooSmall.into());
                }
            }
            _ if magic == STX_V2 => {
                if value.len() < HEADER_V2_SIZE {
                    return Err(FrameError::HeaderV2IsTooSmall.into());
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
    ///  * [`MavLinkFrame::calculate_crc`](crate::protocol::Frame::calculate_crc).
    ///  * [MAVLink checksum](https://mavlink.io/en/guide/serialization.html#checksum) in MAVLink
    ///    protocol documentation.
    pub fn crc_data(&self) -> &[u8] {
        &self.bytes[1..self.size()]
    }
}

impl HeaderBuilder {
    /// Default constructor.
    pub fn new() -> HeaderBuilder {
        Self::default()
    }

    /// Builds [`Header`].
    pub fn build(&self) -> Result<Header> {
        self.validate()?;

        let mut header = Header::default();

        // Set required fields
        macro_rules! set_required_field {
            ($field: ident) => {
                match self.$field {
                    Some($field) => header.$field = $field,
                    None => {
                        return Err(FrameError::HeaderFieldIsNone(stringify!($field).into()).into())
                    }
                }
            };
        }
        set_required_field!(mavlink_version);
        set_required_field!(payload_length);
        set_required_field!(sequence);
        set_required_field!(system_id);
        set_required_field!(component_id);
        set_required_field!(message_id);

        // `MAVLink 2` fields
        if let Some(MavLinkVersion::V2) = self.mavlink_version {
            let mut v2_fields = HeaderV2Fields::default();

            if let Some(incompat_flags) = self.incompat_flags {
                v2_fields.incompat_flags = incompat_flags;
            }
            if let Some(compat_flags) = self.compat_flags {
                v2_fields.compat_flags = compat_flags;
            }

            header.v2_fields = Some(v2_fields);
        }

        Ok(header)
    }

    /// Validates header builder data for consistency.
    ///
    /// # Errors
    ///
    /// * Returns [`FrameError::InconsistentV1Header`] if MAVLink version is set to [`MavLinkVersion::V1`] but either
    /// [Self::set_incompat_flags] or [`Self::set_compat_flags`] were set. These fields are not allowed for `MAVLink 1`
    /// headers.
    /// * Returns [`FrameError::HeaderFieldIsNone`] if required fields are missing.
    pub fn validate(&self) -> Result<()> {
        // Validate required fields
        macro_rules! required_field {
            ($field: ident) => {
                if self.$field.is_none() {
                    return Err(FrameError::HeaderFieldIsNone(stringify!($field).into()).into());
                }
            };
        }
        required_field!(mavlink_version);
        required_field!(payload_length);
        required_field!(sequence);
        required_field!(system_id);
        required_field!(component_id);
        required_field!(message_id);

        // Validate `MAVLink 1` configuration
        if let Some(MavLinkVersion::V1) = self.mavlink_version {
            // Incompatibility and compatibility flags should not be set
            if self.incompat_flags.is_some() || self.compat_flags.is_some() {
                return Err(FrameError::InconsistentV1Header.into());
            }
            // Message ID should by a 8-bit value
            match self.message_id {
                Some(message_id) if message_id > MESSAGE_ID_V1_MAX => {
                    return Err(FrameError::InvalidMavLinkVersion.into())
                }
                _ => {}
            }
        }

        // Validate `MAVLink 2` configuration
        if let Some(MavLinkVersion::V2) = self.mavlink_version {
            // Message ID should by a 24-bit value
            match self.message_id {
                Some(message_id) if message_id > MESSAGE_ID_V2_MAX => {
                    return Err(FrameError::InvalidMavLinkVersion.into())
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Sets MAVLink protocol version.
    ///
    /// See: [`Header::mavlink_version`].
    pub fn set_mavlink_version(&mut self, mavlink_version: MavLinkVersion) -> &mut Self {
        self.mavlink_version = Some(mavlink_version);
        self
    }

    /// Sets incompatibility flags for `MAVLink 2` header.
    ///
    /// # Errors
    ///
    /// Does not returns error directly but if both MAVLink version is set to [`MavLinkVersion::V1`] and incompatibility
    /// flags are present, then [`FrameError::InconsistentV1Header`] error will be returned by [`Self::build`].
    ///
    /// #Links
    ///
    /// * [`HeaderV2Fields`].
    pub fn set_incompat_flags(&mut self, incompat_flags: IncompatFlags) -> &mut Self {
        self.incompat_flags = Some(incompat_flags);
        self
    }

    /// Sets compatibility flags for `MAVLink 2` header.
    ///
    /// # Errors
    ///
    /// Does not returns error directly but if both MAVLink version is set to [`MavLinkVersion::V1`] and compatibility
    /// flags are present, then [`FrameError::InconsistentV1Header`] error will be returned by [`Self::build`].
    ///
    /// #Links
    ///
    /// * [`HeaderV2Fields`].
    pub fn set_compat_flags(&mut self, compat_flags: CompatFlags) -> &mut Self {
        self.compat_flags = Some(compat_flags);
        self
    }

    /// Sets payload length.
    ///
    /// See: [`Header::payload_length`].
    pub fn set_payload_length(&mut self, payload_length: u8) -> &mut Self {
        self.payload_length = Some(payload_length);
        self
    }

    /// Sets packet sequence number.
    ///
    /// See: [`Header::sequence`].
    pub fn set_sequence(&mut self, sequence: u8) -> &mut Self {
        self.sequence = Some(sequence);
        self
    }

    /// Sets system `ID`.
    ///
    /// See: [`Header::system_id`].
    pub fn set_system_id(&mut self, system_id: u8) -> &mut Self {
        self.system_id = Some(system_id);
        self
    }

    /// Sets component `ID`.
    ///
    /// See: [`Header::component_id`].
    pub fn set_component_id(&mut self, component_id: u8) -> &mut Self {
        self.component_id = Some(component_id);
        self
    }

    /// Sets message `ID`.
    ///
    /// See: [`Header::message_id`].
    pub fn set_message_id(&mut self, message_id: u32) -> &mut Self {
        self.message_id = Some(message_id);
        self
    }

    /// Sets whether `MAVLink 2` frame body should contain signature.
    ///
    /// Sets [`MAVLINK_IFLAG_SIGNED`] flag for `incompat_flags`.
    ///
    /// #Links
    ///
    /// * [Frame::signature](crate::protocol::Frame::signature).
    pub fn set_is_signed(&mut self, flag: bool) -> &mut Self {
        match self.incompat_flags {
            // Add incompatibility flags if absent
            None => {
                self.incompat_flags = Some(MAVLINK_IFLAG_SIGNED & flag as u8);
            }
            // Set `MAVLINK_IFLAG_SIGNED` for existing flags
            Some(incompat_flags) => {
                self.incompat_flags = Some(
                    incompat_flags & !MAVLINK_IFLAG_SIGNED | (MAVLINK_IFLAG_SIGNED & flag as u8),
                );
            }
        }
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
    fn set_get_is_signature_required() {
        let mut fields = HeaderV2Fields::default();
        assert!(!fields.is_signed());

        fields.set_is_signed(true);
        assert!(fields.is_signed());

        fields.incompat_flags = 0b01010100;
        assert!(!fields.is_signed());

        fields.set_is_signed(true);
        assert!(fields.is_signed());
        assert_eq!(fields.incompat_flags, 0b01010101);

        fields.set_is_signed(false);
        assert_eq!(fields.incompat_flags, 0b01010100);
    }

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

        assert!(matches!(header.mavlink_version(), MavLinkVersion::V1));
        assert_eq!(header.payload_length(), 8u8);
        assert_eq!(header.sequence(), 1u8);
        assert_eq!(header.system_id(), 10u8);
        assert_eq!(header.component_id(), 255u8);
        assert_eq!(header.message_id(), 0u32);
        assert!(header.v2_fields().is_none());
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

        assert!(matches!(header.mavlink_version(), MavLinkVersion::V2));
        assert_eq!(header.payload_length(), 8u8);
        assert_eq!(header.v2_fields().unwrap().incompat_flags, 1u8);
        assert_eq!(header.v2_fields().unwrap().compat_flags, 0u8);
        assert_eq!(header.sequence(), 1u8);
        assert_eq!(header.system_id(), 10u8);
        assert_eq!(header.component_id(), 255u8);
        assert_eq!(header.message_id(), 0u32);
    }

    #[test]
    fn build_v1_header() {
        let header = Header::builder()
            .set_mavlink_version(MavLinkVersion::V1)
            .set_payload_length(10)
            .set_sequence(5)
            .set_system_id(10)
            .set_component_id(240)
            .set_message_id(42)
            .build();

        assert!(header.is_ok());
        let header = header.unwrap();

        assert!(matches!(header.mavlink_version(), MavLinkVersion::V1));
        assert!(header.v2_fields().is_none());
        assert_eq!(header.payload_length(), 10);
        assert_eq!(header.sequence(), 5);
        assert_eq!(header.system_id(), 10);
        assert_eq!(header.component_id(), 240);
        assert_eq!(header.message_id(), 42);
    }

    #[test]
    fn build_v2_header() {
        let header = Header::builder()
            .set_mavlink_version(MavLinkVersion::V2)
            .set_incompat_flags(1)
            .set_compat_flags(8)
            .set_payload_length(10)
            .set_sequence(5)
            .set_system_id(10)
            .set_component_id(240)
            .set_message_id(42)
            .set_is_signed(true)
            .build();

        assert!(header.is_ok());
        let header = header.unwrap();

        assert!(matches!(header.mavlink_version(), MavLinkVersion::V2));
        assert!(header.v2_fields().is_some());
        assert_eq!(header.v2_fields().unwrap().incompat_flags, 1);
        assert_eq!(header.v2_fields().unwrap().compat_flags, 8);
        assert_eq!(header.payload_length(), 10);
        assert_eq!(header.sequence(), 5);
        assert_eq!(header.system_id(), 10);
        assert_eq!(header.component_id(), 240);
        assert_eq!(header.message_id(), 42);
        assert_eq!(
            header.v2_fields.unwrap().incompat_flags,
            MAVLINK_IFLAG_SIGNED
        );
    }

    #[test]
    fn builder_validates_required_fields() {
        let header = Header::builder()
            .set_mavlink_version(MavLinkVersion::V2)
            .build();

        assert!(header.is_err());
        assert!(matches!(
            header,
            Err(CoreError::Frame(FrameError::HeaderFieldIsNone(_)))
        ));
    }
}
