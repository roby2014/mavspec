//! # MAVSpec
//!
//! Code-generation for [MAVLink](https://mavlink.io/en/) protocol based on
//! [MAVInspect](https://gitlab.com/mavka/libs/mavinspect).
//!
//! # Rust
//!
//! Check [`rust`] module for details.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(
    html_logo_url = "https://gitlab.com/mavka/libs/mavspec/-/raw/main/avatar.png?ref_type=heads",
    html_favicon_url = "https://gitlab.com/mavka/libs/mavspec/-/raw/main/avatar.png?ref_type=heads"
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "rust")]
pub mod rust {
    //! MAVSpec for Rust.

    pub use mavspec_rust_spec as spec;

    #[cfg(feature = "rust_gen")]
    pub use mavspec_rust_gen as gen;
}
