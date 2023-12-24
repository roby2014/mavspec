//! # MAVLib
//!
//! This is toy implementation of a MAVLink-speaking library made to show capabilities of
//! `MAVSpec`.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;

// pub mod api;
pub mod errors;

mod mavlink {
    include!("mavlink/mod.rs");
    // include!(concat!(env!("OUT_DIR"), "/mavlink/mod.rs"));
}
pub use mavlink::dialects;
