//! # MAVLib Core.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

// Common constants
pub mod consts;

// Common types
pub mod types;

// MAVlink magic byte
pub mod stx;

// Errors
pub mod errors;

// Header
pub mod header;

// MAVLink frame
pub mod frame;
pub use frame::MavLinkFrame;

// MAVLink I/O interface
pub mod io;

// MAVLink 2 signature
pub mod signature;

// Utils
pub mod utils;
