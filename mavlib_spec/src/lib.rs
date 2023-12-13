//! # MAVLib Core.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(feature = "std"), no_std)]

// MAVLink message trait
pub mod message;
#[doc(inline)]
pub use message::{MessageImpl, MessageInfo, MessageSpec};

// MAVLink message payload
pub mod payload;
#[doc(inline)]
pub use payload::{IntoPayload, Payload};

// Errors
mod errors;
#[doc(inline)]
pub use errors::MessageError;

// Types
pub mod types;
#[doc(inline)]
pub use types::MavLinkVersion;

// Consts
pub mod consts;

// Dialect
mod dialect;
#[doc(inline)]
pub use dialect::DialectSpec;
