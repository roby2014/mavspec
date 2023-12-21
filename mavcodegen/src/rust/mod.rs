//! # Rust bindings generator

mod conventions;

mod generator;
pub use generator::{Generator, GeneratorParams};

#[cfg(feature = "build_utils")]
mod build_helper;
#[cfg(feature = "build_utils")]
pub use build_helper::BuildHelper;

mod helpers;

mod templates;

pub mod utils;
