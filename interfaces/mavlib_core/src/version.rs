

/// MAVLink version.
#[derive(Clone, Copy, Debug, Default)]
pub enum MavLinkVersion {
    /// MAVLink 1.
    #[default]
    V1,
    /// MAVLink 2.
    V2,
}
