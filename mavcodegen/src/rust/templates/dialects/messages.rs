use mavspec::protocol::{Message, MessageField};
use serde::Serialize;

use crate::rust::RustGeneratorParams;

/// Messages module root template.
///
/// Input: [`mavspec::protocol::Dialect::messages`].
pub const MESSAGES_MODULE_ROOT: &str = "\
//! MAVLink messages of `{{name}}` dialect.

{{#each messages}}
// MAVLink message `{{name}}`.
pub mod {{to-message-mod-name name}};
pub use {{to-message-mod-name name}}::{{to-message-struct-name name}};

{{/each}}\
";

/// Input for [`MESSAGE`] template.
///
/// Basically, this is a utility wrapper around `MAVSpec` [`Message`].
#[derive(Debug, Clone, Serialize)]
pub struct MessageSpec {
    /// Message ID.
    ///
    /// See [`Message::id`].
    pub id: u32,
    /// Message name.
    ///
    /// See [`Message::name`].
    pub name: String,
    /// Message fields in order of specification.
    ///
    /// See: [`Message::fields`].
    pub fields: Vec<MessageField>,
    /// Whether this message is compatible with `MAVLink 1` protocol version.
    ///
    /// See [`Message::is_v1_compatible`].
    pub is_v1_compatible: bool,
    /// Message fields applicable for `MAVLink 1` protocol version.
    ///
    /// See: [`Message::fields_v1`].
    pub fields_v1: Vec<MessageField>,
    /// Size of payload according to `MAVLink 1` protocol version.
    ///
    /// See: [`Message::size_v1`].
    pub payload_v1_size: usize,
    /// Message fields ordered according to `MAVLink 2` protocol specification.
    ///
    /// See: [Message::fields_v2].
    pub fields_v2: Vec<MessageField>,
    /// Size of payload according to `MAVLink 2` protocol version.
    ///
    /// See: [`Message::size_v2`].
    pub payload_v2_size: usize,
    /// Extension fields.
    ///
    /// See: [`Message::extension_fields`].
    pub extension_fields: Vec<MessageField>,
    /// Whether message has extension fields.
    ///
    /// See: [`Message::has_extension_fields`].
    pub has_extension_fields: bool,
    /// Path to the root module.
    pub module_path: String,
}

/// Message template.
///
/// Input: [`MessageSpec`].
pub const MESSAGE: &str = "\
//! MAVLink message `{{name}}` implementation.

use mavlib_core::errors::MavLinkMessageProcessingError;
use mavlib_core::{
    FromMavLinkPayload, IntoMavlinkPayload, MavLinkMessage, MavLinkMessagePayload, MavLinkVersion,
};

/// MAVLink message ID.
pub const MESSAGE_ID: u32 = {{id}};
/// Minimum supported MAVLink version.
pub const MIN_SUPPORTED_MAVLINK_VERSION: MavLinkVersion = MavLinkVersion::{{#if is_v1_compatible}}V1{{else}}V2{{/if}};

/// MAVLink message `{{name}}`.
///
/// Minimum supported MAVLink version is `MAVLink {{#if is_v1_compatible}}1{{else}}2{{/if}}`.
///
/// # Encoding/Decoding
/// 
/// Message encoding/decoding are provided by implementing [`FromMavLinkPayload`] and
/// [`IntoMavlinkPayload`] traits.
#[derive(Clone, Debug)]
pub struct {{to-message-struct-name name}} {
{{#each fields}}
    /// MAVLink field `{{name}}`.
    pub {{to-rust-var name}}: {{to-rust-type type}},
{{/each}}
}

impl MavLinkMessage for {{to-message-struct-name name}} {
    /// MAVLink message ID.
    ///
    /// See [`MavLinkMessage::id`].
    fn id(&self) -> u32 {
        MESSAGE_ID
    }

    /// Minimum supported MAVLink version.
    ///
    /// See [`MavLinkMessage::min_supported_mavlink_version`].
    fn min_supported_mavlink_version(&self) -> MavLinkVersion {
        MIN_SUPPORTED_MAVLINK_VERSION
    }
}

#[allow(clippy::derivable_impls)]
impl Default for {{to-message-struct-name name}} {
    /// Creates [`{{to-message-struct-name name}}`] initialized with default values.
    fn default() -> Self {
        Self {
{{#each fields}}
            {{to-rust-var name}}: {{to-rust-default-value type}},
{{/each}}
        }
    }
}

impl FromMavLinkPayload for {{to-message-struct-name name}} {
    /// Decodes [`MavLinkMessagePayload`] into [`{{to-message-struct-name name}}`] according to [`MavLinkVersion`].
    fn decode(payload: &MavLinkMessagePayload) -> Result<Self, MavLinkMessageProcessingError>
    where
        Self: MavLinkMessage + Sized,
    {
        match payload.version() {
            MavLinkVersion::V2 => v2::decode(payload.payload()),
{{#if is_v1_compatible}}
            MavLinkVersion::V1 => v1::decode(payload.payload()),
{{else}}
            version => {
                return Err(MavLinkMessageProcessingError::UnsupportedMavLinkVersion {
                    actual: version,
                    minimal: MIN_SUPPORTED_MAVLINK_VERSION,
                })
            }
{{/if}}
        }
    }
}

impl IntoMavlinkPayload for {{to-message-struct-name name}} {
    /// Encodes [`{{to-message-struct-name name}}`] into [`MavLinkMessagePayload`] according to [`MavLinkVersion`].
    fn encode(
        &self,
        version: MavLinkVersion,
    ) -> Result<MavLinkMessagePayload, MavLinkMessageProcessingError> {
        Ok(match version {
            MavLinkVersion::V2 => v2::encode(self)?,
{{#if is_v1_compatible}}
            MavLinkVersion::V1 => v1::encode(self)?,
{{else}}
            _ => {
                return Err(MavLinkMessageProcessingError::UnsupportedMavLinkVersion {
                    actual: version,
                    minimal: MIN_SUPPORTED_MAVLINK_VERSION,
                })
            }
{{/if}}
        })
    }
}

/// Encoding/decoding for [`{{to-message-struct-name name}}`] within `MAVLink 2` protocol.
///
/// See [MAVLink 2](https://mavlink.io/en/guide/mavlink_2.html).
pub mod v2 {
    use mavlib_core::errors::MavLinkMessageProcessingError;
    use mavlib_core::{MavLinkMessagePayload, MavLinkVersion};
    use tbytes::{TBytesWriterFor, TBytesReader, TBytesReaderFor, TBytesWriter};
    
    use super::{ {{to-message-struct-name name}}, MESSAGE_ID };

    /// Message [`{{to-message-struct-name name}}`] payload size (non-truncated) according to `MAVLink 2` protocol.
    pub const PAYLOAD_SIZE: usize = {{payload_v2_size}};

    /// Decodes into [`{{to-message-struct-name name}}`] message.
    ///
    /// If `payload` is less than expected, the remaining elements will be considered to be zeros.
    /// See [MAVLink 2 payload truncation](https://mavlink.io/en/guide/serialization.html#payload_truncation).
    ///
    /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///
    /// # Errors
    /// 
    /// Returns [`MavLinkMessageProcessingError::BufferError`] in case of malformed `payload`.
    pub fn decode(payload: &[u8]) -> Result<{{to-message-struct-name name}}, MavLinkMessageProcessingError> {
        let reader = TBytesReader::from(payload);

        Ok({{to-message-struct-name name}} {
            // Fields are reordered according to `MAVLink` specification
{{#each fields_v2}}
            {{to-rust-var name}}: reader.{{to-reader-fn type}}()?,
{{/each}}
        })
    }

    /// Encodes from [`{{to-message-struct-name name}}`] message.
    ///
    /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///
    /// Zero trailing bytes will be truncated.
    /// See [MAVLink 2 payload truncation](https://mavlink.io/en/guide/serialization.html#payload_truncation).
    ///
    /// # Errors
    /// 
    /// This function does not returns errors at the moment. The [`Result`] returning type is
    /// reserved for future implementations where such errors may happen.
    pub fn encode(
        message: &{{to-message-struct-name name}}
    ) -> Result<MavLinkMessagePayload, MavLinkMessageProcessingError> {
        let mut buf = [0u8; PAYLOAD_SIZE];
        let mut writer = TBytesWriter::from(buf.as_mut_slice());

        // Fields are reordered according to `MAVLink` specification
{{#each fields_v2}}
        writer.{{to-writer-fn type}}(message.{{to-rust-var name}})?;
{{/each}}

        let payload = MavLinkMessagePayload::new(MESSAGE_ID, buf.as_slice(), MavLinkVersion::V2);
        Ok(payload)
    }
}

{{#if is_v1_compatible}}
/// Encoding/decoding for [`{{to-message-struct-name name}}`] within `MAVLink 2` protocol.
///
/// See [MAVLink versions](https://mavlink.io/en/guide/mavlink_version.html).
pub mod v1 {
    use mavlib_core::errors::MavLinkMessageProcessingError;
    use mavlib_core::{MavLinkMessagePayload, MavLinkVersion};
    use tbytes::{TBytesWriterFor, TBytesReader, TBytesReaderFor, TBytesWriter};
    
    use super::{ {{to-message-struct-name name}}, MESSAGE_ID };

    /// Message [`{{to-message-struct-name name}}`] payload size according to `MAVLink 1` protocol.
    pub const PAYLOAD_SIZE: usize = {{payload_v1_size}};

    /// Decodes into [`{{to-message-struct-name name}}`] message.
    ///
    /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///
    /// # Errors
    /// 
    /// * Returns [`MavLinkMessageProcessingError::InvalidPayloadSize`] if `payload` has incorrect size.
    ///   Payload size is defined in [`PAYLOAD_SIZE`].
    /// * Returns [`MavLinkMessageProcessingError::BufferError`] in case of malformed `payload`.
    pub fn decode(payload: &[u8]) -> Result<{{to-message-struct-name name}}, MavLinkMessageProcessingError> {
        if payload.len() != PAYLOAD_SIZE {
            return Err(MavLinkMessageProcessingError::InvalidPayloadSize {
                actual: payload.len(),
                expected: PAYLOAD_SIZE,
            });
        }
        let reader = TBytesReader::from(payload);

        Ok({{to-message-struct-name name}} {
            // Fields are reordered according to `MAVLink` specification
{{#each fields_v1}}
            {{to-rust-var name}}: reader.{{to-reader-fn type}}()?,
{{/each}}
{{#if has_extension_fields}}
            // These fields are `MAVLink 2` extensions and will be populated with default values.
{{#each extension_fields}}
            {{to-rust-var name}}: {{to-rust-default-value type}},
{{/each}}
{{/if}}
        })
    }

    /// Encodes from [`{{to-message-struct-name name}}`] message.
    ///
    /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///
    /// # Errors
    /// 
    /// This function does not returns errors at the moment. The [`Result`] returning type is
    /// reserved for future implementations where such errors may happen.
    pub fn encode(
        message: &{{to-message-struct-name name}}
    ) -> Result<MavLinkMessagePayload, MavLinkMessageProcessingError> {
        let mut buf = [0u8; PAYLOAD_SIZE];
        let mut writer = TBytesWriter::from(buf.as_mut_slice());

        // Fields are reordered according to `MAVLink` specification
{{#each fields_v1}}
        writer.{{to-writer-fn type}}(message.{{to-rust-var name}})?;
{{/each}}
{{#if has_extension_fields}}
        // The following extension fields are ignored in `MAVLink 1`
{{#each extension_fields}}
        // message.{{to-rust-var name}}
{{/each}}
{{/if}}

        let payload = MavLinkMessagePayload::new(MESSAGE_ID, buf.as_slice(), MavLinkVersion::V1);
        Ok(payload)
    }
}
{{/if}}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_v2() {
        let payload = [1u8; v2::PAYLOAD_SIZE];

        let message = v2::decode(&payload).unwrap();
        assert!(matches!(message, {{to-message-struct-name name}} { .. }));

        let encoded_payload = v2::encode(&message).unwrap();
        assert_eq!(encoded_payload.payload(), payload.as_slice());

        let encoded_mavlink_payload = message.encode(MavLinkVersion::V2).unwrap();
        assert_eq!(encoded_mavlink_payload.payload(), payload);
        assert_eq!(encoded_mavlink_payload.id(), message.id());
        assert!(matches!(encoded_mavlink_payload.version(), MavLinkVersion::V2));
    }
{{#if is_v1_compatible}}

    #[test]
    fn basic_v1() {
        let payload = [1u8; v1::PAYLOAD_SIZE];

        let message = v1::decode(&payload).unwrap();
        assert!(matches!(message, {{to-message-struct-name name}} { .. }));

        let encoded_payload = v1::encode(&message).unwrap();
        assert_eq!(encoded_payload.payload(), payload.as_slice());

        let encoded_mavlink_payload = message.encode(MavLinkVersion::V1).unwrap();
        assert_eq!(encoded_mavlink_payload.payload(), payload);
        assert_eq!(encoded_mavlink_payload.id(), message.id());
        assert!(matches!(encoded_mavlink_payload.version(), MavLinkVersion::V1));
    }
{{/if}}
}
";

impl MessageSpec {
    /// Constructs from [`Message`] and [``].
    pub fn new(message: &Message, params: &RustGeneratorParams) -> Self {
        Self {
            id: message.id(),
            name: message.name().to_string(),
            fields: message.fields().to_vec(),
            // `MAVLink 1`
            is_v1_compatible: message.is_v1_compatible(),
            fields_v1: message.fields_v1(),
            payload_v1_size: message.size_v1(),
            // `MAVLink 2`
            fields_v2: message.fields_v2(),
            payload_v2_size: message.size_v2(),
            extension_fields: message.extension_fields(),
            has_extension_fields: message.has_extension_fields(),
            // Params
            module_path: params.module_path.clone(),
        }
    }
}

/// Input for [`INHERITED_MESSAGE`] template.
#[derive(Debug, Clone, Serialize)]
pub struct InheritedMessageSpec {
    /// MAVLink dialect name.
    pub dialect_name: String,
    /// MAVLink message name.
    pub message_name: String,
    /// Compatibility with `MAVLink 1` protocol.
    pub is_v1_compatible: bool,
}

impl InheritedMessageSpec {
    pub fn new(dialect_name: String, message: &Message) -> Self {
        Self {
            dialect_name,
            message_name: message.name().to_string(),
            is_v1_compatible: message.is_v1_compatible(),
        }
    }
}

/// Inherited message template.
///
/// Input: [`InheritedMessageSpec`].
pub const INHERITED_MESSAGE: &str = "\
//! MAVLink message `{{message_name}}` inherited from [`super::super::super::{{to-dialect-name dialect_name}}`] dialect.

use mavlib_core::MavLinkVersion;

use super::super::super::minimal as dialect;

/// MAVLink message ID originally defined in [`dialect::messages::{{to-message-mod-name message_name}}::MESSAGE_ID`].
pub const MESSAGE_ID: u32 = dialect::messages::{{to-message-mod-name message_name}}::MESSAGE_ID;
/// Minimum supported MAVLink version originally defined in [`dialect::messages::{{to-message-mod-name message_name}}::MIN_SUPPORTED_MAVLINK_VERSION`].
pub const MIN_SUPPORTED_MAVLINK_VERSION: MavLinkVersion =
    dialect::messages::{{to-message-mod-name message_name}}::MIN_SUPPORTED_MAVLINK_VERSION;

/// MAVLink message `{{message_name}}` originally defined in [`dialect::messages::{{to-message-struct-name message_name}}`].
pub type {{to-message-struct-name message_name}} = dialect::messages::{{to-message-struct-name message_name}};

/// Encoding/decoding for message [`{{to-message-struct-name message_name}}`] within `MAVLink 2` protocol originally defined in [`dialect::messages::{{to-message-mod-name message_name}}::v2`]..
pub use dialect::messages::{{to-message-mod-name message_name}}::v2;
{{#if is_v1_compatible}}

/// Encoding/decoding for message [`{{to-message-struct-name message_name}}`] within `MAVLink 1` protocol originally defined in [`dialect::messages::{{to-message-mod-name message_name}}::v1`]..
pub use dialect::messages::{{to-message-mod-name message_name}}::v1;
{{/if}}
";
