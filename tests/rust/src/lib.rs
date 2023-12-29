//! Tests for MAVSpec Rust code generation

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod mavlink {
    include!(concat!(env!("OUT_DIR"), "/mavlink/mod.rs"));
}
pub use mavlink::dialects;
