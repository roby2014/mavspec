//! # MAVCodeGen
//!
//! Code-generation for [MAVLink](https://mavlink.io/en/) protocol based on
//! [MAVSpec](https://gitlab.com/mavka/libs/mavspec).
//!
//! # Rust
//!
//! Check [`rust`] module for details.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(
    html_logo_url = "https://gitlab.com/mavka/libs/mavcodegen/-/raw/main/avatar.png?ref_type=heads",
    html_favicon_url = "https://gitlab.com/mavka/libs/mavcodegen/-/raw/main/avatar.png?ref_type=heads"
)]

#[cfg(feature = "rust")]
pub mod rust;
