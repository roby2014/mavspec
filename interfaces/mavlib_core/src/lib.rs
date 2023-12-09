//! # MAVLib Core.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

// Common constants
pub mod consts;

// Common types
pub mod types;

// MAVLink message trait
pub mod message;
pub use message::MavLinkMessage;

// MAVLink message payload
pub mod payload;
pub use payload::{IntoMavLinkPayload, MavLinkMessagePayload};

// MAVlink magic byte
pub mod stx;

// MAVLink version
mod version;
pub use version::MavLinkVersion;

// Errors
pub mod errors;

// Header
pub mod header;

// MAVLink frame
pub mod frame;
pub use frame::MavLinkFrame;
