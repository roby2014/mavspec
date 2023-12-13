use crate::{MessageError, MessageSpec};

/// Interface for autogenerated or custom MAVLink dialect.
pub trait DialectSpec {
    /// Dialect name as it appears in XML definition.
    fn name(&self) -> &str;

    /// Message specification by `id`.
    ///
    /// Clients may access this method to retrieve message specification prior to decoding it from
    /// payload.
    ///
    /// # Errors
    ///
    /// Returns [`MessageError::UnsupportedMessageId`] if message with specified ID is not supported.
    fn message_info(&self, id: u32) -> Result<&dyn MessageSpec, MessageError>;
}
