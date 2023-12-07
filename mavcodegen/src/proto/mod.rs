//! # Protobuf bindings generator

use std::path::PathBuf;

use mavspec::protocol::Protocol;

/// # Protobuf code generator
pub struct ProtobufGenerator {
    protocol: Protocol,
    path: PathBuf,
}

impl ProtobufGenerator {
    /// Default constructor.
    pub fn new(protocol: Protocol, path: PathBuf) -> Self {
        Self { protocol, path }
    }

    /// Generate Protobuf definitions.
    pub fn generate(&self) -> anyhow::Result<()> {
        println!(
            "Dialects: {:?}. Path: {:?}",
            self.protocol.dialects().keys(),
            self.path
        );
        Ok(())
    }
}
