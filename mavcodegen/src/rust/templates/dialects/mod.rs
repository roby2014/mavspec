pub mod enums;
pub mod messages;

/// Dialects module root template.
///
/// Input: [`mavspec::protocol::Protocol`].
pub const DIALECTS_ROOT_MODULE: &str = "\
//! MAVLink dialects.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

{{#each dialects}}
/// `{{name}}` dialect.
pub mod {{to-dialect-name name}};

{{/each}}
";

/// Dialect module root template.
///
/// Input: [`mavspec::protocol::Dialect`].
pub const DIALECT_MODULE: &str = "\
//! MAVLink dialect `{{name}}`.

use mavlib_core::errors::MavLinkMessageProcessingError;
use mavlib_core::{FromMavLinkPayload, IntoMavlinkPayload, MavLinkMessagePayload, MavLinkVersion};

// MAVLink messages.
pub mod messages;

// MAVLink enums.
pub mod enums;

/// Enum containing all messages within `{{name}}` dialect.
pub enum Message {
{{#each messages}}
    /// MAVLink message `{{name}}`.
    {{to-messages-enum-entry-name name}}(messages::{{to-message-struct-name name}}),
{{/each}}
}

impl Message {
    /// Decodes message from MAVLink payload.
    pub fn decode(
        payload: &MavLinkMessagePayload,
    ) -> Result<Self, MavLinkMessageProcessingError> {
        Ok(match payload.id() {
{{#each messages}}
            messages::{{to-message-mod-name name}}::MESSAGE_ID => Self::{{to-messages-enum-entry-name name}}(messages::{{to-message-struct-name name}}::decode(payload)?),
{{/each}}
            id => return Err(MavLinkMessageProcessingError::UnsupportedMessageId(id)),
        })
    }

    /// Encodes message to MAVLink payload.
    pub fn encode(&self, version: MavLinkVersion) -> Result<MavLinkMessagePayload, MavLinkMessageProcessingError> {
        Ok(match self {
{{#each messages}}
            Self::{{to-messages-enum-entry-name name}}(message) => {message.encode(version)?}
{{/each}}
        })
    }
}
";
