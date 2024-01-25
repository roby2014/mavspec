//! # Rust core interfaces for MAVSpec
//!
//! <span style="font-size:24px">[🇺🇦](https://mavka.gitlab.io/home/a_note_on_the_war_in_ukraine/)</span>
//! [![`repository`](https://img.shields.io/gitlab/pipeline-status/mavka/libs/mavspec.svg?branch=main&label=repository)](https://gitlab.com/mavka/libs/mavspec)
//! [![`crates.io`](https://img.shields.io/crates/v/mavspec.svg)](https://crates.io/crates/mavspec)
//! [![`docs.rs`](https://img.shields.io/docsrs/mavspec.svg?label=docs.rs)](https://docs.rs/mavinspect/latest/mavspec/)
//! [![`issues`](https://img.shields.io/gitlab/issues/open/mavka/libs/mavspec.svg)](https://gitlab.com/mavka/libs/mavspec/-/issues/)
//!
//! This crate provides interfaces for [MAVLink](https://mavlink.io/en/) dialect definitions generated by
//! [MAVSpec](https://gitlab.com/mavka/libs/mavspec).
//!
//! Entities generated by MAVSpec-Rust depend on these interfaces. This crate also re-exports several 3rd party
//! libraries to simplify dependency management.
//!
//! # Payload
//!
//! [`Payload`] encapsulates MAVLink message payload and additional meta information required for encoding and decoding.
//! This struct depends on `alloc` conditional compilation feature.
//!
//! [`IntoPayload`] trait is implemented by objects which are capable to transform themselves into MAVLink payload.
//!
//! # Message
//!
//! [`MessageSpec`] trait should be implemented by objects which carry information about a message.
//!
//! [`MessageImpl`] trait corresponds to a concrete message implementation which both are [`MessageSpec`] and
//! [`IntoPayload`].
//!
//! # Dialect
//!
//! [`DialectSpec`] trait is implemented by dialect specifications. It contains metadata like dialect name, dialect ID,
//! dialect capabilities, or minor dialect version. It also exposes [`DialectSpec::message_info`] method which provides
//! message specifications for dialect messages.  
//!
//! # Types & Conventions
//!
//! Modules [`consts`] and [`types`] provide constants, type aliases, enums and wrapper types. The entities are intended
//! to grasp MAVLink protocol conventions and provide semantic meaning for otherwise unclear types and magic numbers.  
//!
//! # Errors
//!
//! All fallible functions and methods return [`MessageError`] on failure.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(
    html_logo_url = "https://gitlab.com/mavka/libs/mavspec/-/raw/main/avatar.png?ref_type=heads",
    html_favicon_url = "https://gitlab.com/mavka/libs/mavspec/-/raw/main/avatar.png?ref_type=heads"
)]
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
