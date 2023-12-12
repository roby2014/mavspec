//! # MAVLib Core.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(feature = "std"), no_std)]

// MAVLink message trait
pub mod message;
pub use message::{MavLinkMessageInfo, MavLinkMessageSpec};

// MAVLink message payload
pub mod payload;
pub use payload::{IntoMavLinkPayload, MavLinkMessagePayload};

// MAVLink version
mod version;
pub use version::MavLinkVersion;

// Errors
mod errors;
pub use errors::MessageError;

mod dialect;
pub use dialect::MavLinkDialectSpec;
