//! # Rust bindings generator

mod conventions;

mod generator;
pub use generator::{Generator, GeneratorParams};

#[cfg(feature = "build_utils")]
mod builder;
#[cfg(feature = "build_utils")]
pub use builder::Builder;

mod helpers;

mod templates;
