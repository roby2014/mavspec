//! # MAVLink bindings generator for Rust
//!
//! # Usage
//!
//! The easiest way to generate code is to use [`BuildHelper`].
//!
//! ```rust
//! # use std::fs::remove_dir_all;
//! use mavcodegen::rust::BuildHelper;
//!
//! // Paths to XML definitions directories.
//! let sources = vec![
//!     "../message_definitions/standard",
//!     "../message_definitions/extra",
//! ];
//! // Output path
//! let destination = "../tmp/mavlink";
//!
//! BuildHelper::builder(&sources, destination)
//! #   .set_dialects(&["minimal"])
//!     .generate()
//!     .unwrap();
//! # remove_dir_all("../tmp/mavlink").unwrap();
//! ```
//!
//! For better control you may use [`Generator`] that consumes parsed MAVLink XML definitions provided by
//! [MAVSpec](https://gitlab.com/mavka/libs/mavspec)'s `XMLInspector`.
//!
//! ```rust
//! # use std::fs::remove_dir_all;
//! use std::path::Path;
//! use mavcodegen::rust::{Generator, GeneratorParams};
//! use mavspec::parser::XMLInspector;
//!
//! // Paths to XML definitions directories.
//! let sources = vec![
//!     "../message_definitions/standard",
//!     "../message_definitions/extra",
//! ];
//!
//! // Parse XML definitions
//! let protocol = XMLInspector::builder()
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
//! // Generate MAVLink dialects
//! let generator = Generator::new(
//!     protocol,
//!     &destination,
//!     GeneratorParams {
//!         serde: true,
//!         ..Default::default()
//!     },
//! );
//! generator.generate().unwrap();
//! # remove_dir_all(&destination).unwrap();
//! ```

mod conventions;

mod generator;
pub use generator::{Generator, GeneratorParams};

#[cfg(feature = "build_utils")]
mod build_helper;
#[cfg(feature = "build_utils")]
pub use build_helper::{BuildHelper, BuildHelperConf};

mod helpers;

mod templates;

pub mod utils;
