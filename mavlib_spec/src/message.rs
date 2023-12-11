//! # MAVLink message
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

use super::MavLinkVersion;

/// Generic MAVLink message.
pub trait MavLinkMessage {
    /// `MAVLink` message ID.
    ///
    /// In `MAVLink 2` message ID is a 24-bit unsigned integer stored as [`u32`].
    ///
    /// `MAVLink 1` supports only 8-bit message ID.
    ///
    /// # Links
    ///
    ///  - [MAVLink 2](https://mavlink.io/en/guide/mavlink_2.html)
    ///  - [MAVLink serialization](https://mavlink.io/en/guide/serialization.html)
    fn id(&self) -> u32;

    /// Minimum supported `MAVLink` protocol version.
    ///
    /// Messages supporting both `MAVLink 1` and `MAVLink 2` will return [`MavLinkVersion::V1`].
    ///
    /// Messages which make sense only in `MAVLink 2` will return [`MavLinkVersion::V2`],
    fn min_supported_mavlink_version(&self) -> MavLinkVersion;
}
