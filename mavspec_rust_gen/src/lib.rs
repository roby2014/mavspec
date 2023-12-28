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
