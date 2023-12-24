//! # MAVLib Core.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(feature = "std"), no_std)]

pub use bitflags;
pub use tbytes;

pub mod message;
#[doc(inline)]
pub use message::{MessageImpl, MessageInfo, MessageSpec};

pub mod payload;
#[doc(inline)]
pub use payload::{IntoPayload, Payload};

mod errors;
#[doc(inline)]
pub use errors::MessageError;

pub mod types;
#[doc(inline)]
pub use types::MavLinkVersion;

pub mod consts;

mod dialect;
#[doc(inline)]
pub use dialect::DialectSpec;
