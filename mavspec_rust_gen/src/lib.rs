//! # Rust bindings generator for [MAVSpec](https://gitlab.com/mavka/libs/mavspec).
//!
//! Generates [MAVLink](https://mavlink.io/en/) bindings for Rust.
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
//!     "../message_definitions/standard",
//!     "../message_definitions/extra",
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
//! For better control you may directly pass `Protocol` from [MAVInspect](https://gitlab.com/mavka/libs/mavinspect)'s
//! `Inspector`.
//!
//! ```rust
//! # use std::fs::remove_dir_all;
//! use std::path::Path;
//! use mavspec::rust::gen::BuildHelper;
//! use mavinspect::Inspector;
//!
//! // Paths to XML definitions directories.
//! let sources = vec![
//!     "../message_definitions/standard",
//!     "../message_definitions/extra",
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
//! #   .set_include_dialects(&["minimal"])
//!     .generate()
//!     .unwrap();
//! # remove_dir_all(&destination).unwrap();
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
