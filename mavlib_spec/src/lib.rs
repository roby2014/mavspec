//! # MAVLib Core.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

// MAVLink message trait
pub mod message;
pub use message::MavLinkMessage;

// MAVLink message payload
pub mod payload;
pub use payload::{IntoMavLinkPayload, MavLinkMessagePayload};

// MAVLink version
mod version;
pub use version::MavLinkVersion;

// Errors
pub mod errors;
