//! # MAVLib Core.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

// MAVLink message trait
pub mod message;
pub use message::MavLinkMessage;

// MAVLink message payload
pub mod payload;
pub use payload::{FromMavLinkPayload, IntoMavlinkPayload, MavLinkMessagePayload};

// MAVLink 1 frame
pub mod mav_frame_v1;
pub use mav_frame_v1::MavLinkFrameV1;

// MAVLink 2 frame
pub mod mav_frame_v2;
pub use mav_frame_v2::MavLinkFrameV2;

// MAVlink magic byte
pub mod stx;
pub use stx::MavSTX;

// MAVLink version
mod version;
pub use version::MavLinkVersion;

// Errors
pub mod errors;
