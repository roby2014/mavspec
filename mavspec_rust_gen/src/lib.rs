//! # Rust bindings generator for MAVSpec
//!
//! [`repository`](https://gitlab.com/mavka/libs/mavspec) |
//! [`crates.io`](https://crates.io/crates/mavspec) |
//! [`API docs`](https://docs.rs/mavspec/latest/mavspec/rust/gen) |
//! [`issues`](https://gitlab.com/mavka/libs/mavspec/-/issues)
//!
//! This module contains Rust bindings for [MAVSpec](https://gitlab.com/mavka/libs/mavinspect), a code generation
//! toolchain for [MAVLink](https://mavlink.io/en/) protocol.
//!
//! # Usage
//!
//! We provide a [`BuildHelper`] that can be integrated into your build pipeline.
//!
//! ```rust
//! # use std::fs::remove_dir_all;
//! use mavspec::rust::gen::BuildHelper;
//!
//! // Paths to XML definitions directories.
//! let sources = vec![
//!     "./message_definitions/standard",
//!     "./message_definitions/extra",
//! ];
//! // Output path
//! let destination = "../tmp/mavlink";
//! # let destination = "../tmp/mavlink/lib/default";
//!
//! // Generate rust bindings
//! BuildHelper::builder(destination)
//!     .set_sources(&sources)
//! #   .set_include_dialects(&["minimal"])
//!     .generate()
//!     .unwrap();
//! # remove_dir_all(destination).unwrap_or_default();
//! ```
//!
//! For better control over included dialects you may directly pass `Protocol` from
//! [MAVInspect](https://gitlab.com/mavka/libs/mavinspect)'s `Inspector`.
//!
//! ```rust
//! # use std::fs::remove_dir_all;
//! use std::path::Path;
//! use mavspec::rust::gen::BuildHelper;
//! use mavinspect::Inspector;
//!
//! // Paths to XML definitions directories.
//! let sources = vec![
//!     "./message_definitions/standard",
//!     "./message_definitions/extra",
//! ];
//!
//! // Parse XML definitions
//! let protocol = Inspector::builder()
//!     // Define paths to XML definitions directories
//!     .set_sources(&sources)
//! #   .set_include(&["minimal"])
//!     // Build configuration and parse dialects
//!     .build().unwrap()
//!     .parse().unwrap();
//!
//! // Output path
//! let destination = "../tmp/mavlink";
//! # let destination = "../tmp/mavlink/lib/from_protocol";
//!
//! // Generate rust bindings
//! BuildHelper::builder(destination)
//!     .set_protocol(protocol)
//!     .generate()
//!     .unwrap();
//! # remove_dir_all(destination).unwrap_or_default();
//! ```
//!
//! # Naming Conventions
//!
//! In `MAVSpec` we are trying to keep balance between names as they appear in MAVLink XML definitions and Rust naming
//! conventions. In most situation we favor the Rust way unless it introduces confusions. In case we failed, and you are
//! confused, all entities are supplemented with descriptions where canonical MAVlink names are mentioned. Here is the list
//! of the naming rules:
//!
//! * For **structs** and **enums** `MAVSpec` uses `UpperCamelCase`.
//! * For **message fields** we use `snake_case`.
//! * For **enum entries** (enum entries) we use `UpperCamelCase` with MAVLink enum name prefix stripped
//!   (whenever applicable). For example, if bitmask enum has name `IMPORTANCE_LEVEL` and flag name is
//!   `IMPORTANCE_LEVEL_THE_MATTER_OF_LIFE_AND_DEATH`, then flag name will be `TheMatterOfLifeAndDeath`.
//! * For **bitmask flags** (enum entries for enums which are bitmasks) we use `SCREAMING_SNAKE_CASE` with MAVLink enum name
//!   prefix stripped (whenever applicable). For example, if bitmask enum has name `VERY_IMPORTANT_FLAGS` and flag name is
//!   `VERY_IMPORTANT_FLAGS_THE_MATTER_OF_LIFE_AND_DEATH_FLAG`, then flag name will be `THE_MATTER_OF_LIFE_AND_DEATH_FLAG`.
//! * In the case of collision with rust keywords, we add underscore suffix. For example, `type` field of `HEARTBEAT`
//!   message will be encoded as `type_`.
//! * In the rare cases when symbolic name starts with numeric character, it will be prefixed with `_`.
//!
//! The last two cases of handling inconvenient names are not something of high aesthetic value but in our defence we
//! must say that all approaches we've considered looked equally ugly.
//!
//! # Fingerprints
//!
//! MAVInspect may skip code re-generation if dialects haven't changed. It uses 64-bit CRC fingerprint to monitor
//! changes. Set `fingerprints` feature flag to enable this behavior.
//!
//! This feature is useful for reducing build time during development and CI runs. Make sure that your releases are
//! clean and do not depend on fingerprints.
//!
//! # Unstable Features
//!
//! Unstable features are enabled by `unstable` feature flag. Such features are experimental and can be changed or
//! excluded in future releases.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(
    html_logo_url = "https://gitlab.com/mavka/libs/mavspec/-/raw/main/avatar.png?ref_type=heads",
    html_favicon_url = "https://gitlab.com/mavka/libs/mavspec/-/raw/main/avatar.png?ref_type=heads"
)]

mod build_helper;
pub use build_helper::{BuildHelper, BuildHelperBuilder};

pub mod utils;

pub(crate) mod conventions;
pub(crate) mod generator;
pub(crate) mod specs;
pub(crate) mod templates;
