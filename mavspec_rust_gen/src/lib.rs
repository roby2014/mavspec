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
//!
//! // Generate rust bindings
//! BuildHelper::builder(destination)
//!     .set_sources(&sources)
//! #   .set_include_dialects(&["minimal"])
//!     .generate()
//!     .unwrap();
//! # remove_dir_all("../tmp/mavlink").unwrap();
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
//!
//! // Generate rust bindings
//! BuildHelper::builder(destination)
//!     .set_protocol(protocol)
//!     .generate()
//!     .unwrap();
//! # remove_dir_all("../tmp/mavlink").unwrap();
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
//! * In the case of collision with rust keywords, we use raw strings. For example, `type` field of `HEARTBEAT` message will
//!   be encoded as `r#type`.
//! * In the rare cases when symbolic name starts with numeric character, it will be prefixed with `_`.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(
    html_logo_url = "https://gitlab.com/mavka/libs/mavspec/-/raw/main/avatar.png?ref_type=heads",
    html_favicon_url = "https://gitlab.com/mavka/libs/mavspec/-/raw/main/avatar.png?ref_type=heads"
)]

mod conventions;
mod generator;
mod helpers;
mod templates;
pub mod utils;

mod build_helper;
pub use build_helper::{BuildHelper, BuildHelperConf};
