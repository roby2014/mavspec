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
