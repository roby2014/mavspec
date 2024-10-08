//! # MAVSpec
//!
//! <span style="font-size:24px">[🇺🇦](https://mavka.gitlab.io/home/a_note_on_the_war_in_ukraine/)</span>
//! [![`repository`](https://img.shields.io/gitlab/pipeline-status/mavka/libs/mavspec.svg?branch=main&label=repository)](https://gitlab.com/mavka/libs/mavspec)
//! [![`crates.io`](https://img.shields.io/crates/v/mavspec.svg)](https://crates.io/crates/mavspec)
//! [![`docs.rs`](https://img.shields.io/docsrs/mavspec.svg?label=docs.rs)](https://docs.rs/mavinspect/latest/mavspec/)
//! [![`issues`](https://img.shields.io/gitlab/issues/open/mavka/libs/mavspec.svg)](https://gitlab.com/mavka/libs/mavspec/-/issues/)
//!
//! Code-generation for [MAVLink](https://mavlink.io/en/) protocol based on
//! [MAVInspect](https://gitlab.com/mavka/libs/mavinspect).
//!
//! # Rust
//!
//! Check [`rust`] module for details.
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
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "rust")]
pub mod rust {
    //! # MAVSpec's code generation toolchain for Rust
    #![cfg_attr(
        feature = "rust_gen",
        doc = "\n\nCheck [`gen`] module documentation to learn about code generation specifics."
    )]

    #[doc(inline)]
    pub use mavspec_rust_derive as derive;
    #[doc(inline)]
    pub use mavspec_rust_spec as spec;

    #[cfg(feature = "rust_gen")]
    #[doc(inline)]
    pub use mavspec_rust_gen as gen;
}
