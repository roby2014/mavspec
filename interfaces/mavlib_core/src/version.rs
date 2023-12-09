use crate::consts::MAVLINK_V1_HEADER_SIZE;
use crate::errors::MavLinkFrameDecodingError;
use crate::stx::MavSTX;

/// MAVLink version.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MavLinkVersion {
    /// MAVLink 1.
    #[default]
    V1,
    /// MAVLink 2.
    V2,
}

impl TryFrom<MavSTX> for MavLinkVersion {
    type Error = MavLinkFrameDecodingError;

    /// Tries to convert [`MavSTX`] into [`MavLinkVersion`].
    ///
    /// # Errors
    ///
    /// Returns [`MavLinkFrameDecodingError::InvalidMavLinkVersion`] if [`MavSTX::Unknown`] provided.
    fn try_from(value: MavSTX) -> Result<Self, Self::Error> {
        Ok(match value {
            MavSTX::MavLink1 => Self::V1,
            MavSTX::MavLink2 => Self::V2,
            MavSTX::Unknown(_) => return Err(MavLinkFrameDecodingError::InvalidMavLinkVersion),
        })
    }
}

impl MavLinkVersion {
    /// `MAVLink` packet header size for this protocol version.
    ///
    /// See [`MavLinkHeader`](crate::header::MavLinkHeader).
    pub fn header_size(&self) -> usize {
        match self {
            MavLinkVersion::V1 => MAVLINK_V1_HEADER_SIZE,
            MavLinkVersion::V2 => MAVLINK_V1_HEADER_SIZE,
        }
    }
}
