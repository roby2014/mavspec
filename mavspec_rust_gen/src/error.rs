//! # MAVSpec errors related to generating Rust code

use std::sync::Arc;

use cargo_manifest::Error as ManifestError;
use mavinspect::errors::Error as InspectError;

/// Result returned by all user-facing functions in MAVSpec Rust generation module.
pub type RustGenResult<T> = Result<T, RustGenError>;

/// Error returned by all user-facing functions in MAVSpec Rust generation module.
#[derive(Clone, Debug, thiserror::Error)]
pub enum RustGenError {
    /// I/O error.
    #[error("I/O error: {0:?}")]
    Io(Arc<std::io::Error>),
    /// XML specification error.
    #[error("XML inspection error: {0:?}")]
    Inspect(Arc<InspectError>),
    /// Cargo manifest parsing error.
    #[error("Cargo manifest error: {0:?}")]
    Manifest(Arc<ManifestError>),
}

impl From<std::io::Error> for RustGenError {
    fn from(value: std::io::Error) -> Self {
        RustGenError::Io(Arc::new(value))
    }
}

impl From<InspectError> for RustGenError {
    fn from(value: InspectError) -> Self {
        RustGenError::Inspect(Arc::new(value))
    }
}

impl From<ManifestError> for RustGenError {
    fn from(value: ManifestError) -> Self {
        RustGenError::Manifest(Arc::new(value))
    }
}
