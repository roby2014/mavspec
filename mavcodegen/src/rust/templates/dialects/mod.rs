use crate::rust::RustGeneratorParams;
use mavspec::protocol::Dialect;
use serde::Serialize;

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

/// Input for [`DIALECT_MODULE`].
#[derive(Clone, Debug, Serialize)]
pub struct DialectModuleSpec<'a> {
    pub dialect: &'a Dialect,
    pub params: &'a RustGeneratorParams,
}

/// Dialect module root template.
///
/// Input: [`DialectModuleSpec`].
pub const DIALECT_MODULE: &str = r#"//! # MAVLink dialect `{{dialect.name}}`

use mavlib_core::errors::MavLinkMessageProcessingError;
use mavlib_core::{IntoMavLinkPayload, MavLinkMessagePayload, MavLinkVersion};

// MAVLink messages.
pub mod messages;

// MAVLink enums.
pub mod enums;

/// Enum containing all messages within `{{dialect.name}}` dialect.
#[derive(Clone, Debug)]
{{#if params.serde}}#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]{{/if}}
pub enum Message {
{{#each dialect.messages}}
    /// MAVLink message `{{name}}`.
    {{to-messages-enum-entry-name name}}(messages::{{to-message-struct-name name}}),
{{/each}}
}

impl TryFrom<&MavLinkMessagePayload> for Message {
    type Error = MavLinkMessageProcessingError;

    /// Decodes message from `MAVLink` payload.
    fn try_from(value: &MavLinkMessagePayload) -> Result<Self, Self::Error> {
        Self::decode(value)
    }
}

impl IntoMavLinkPayload for Message {
    /// Encodes message into `MAVLink` payload.
    fn encode(
        &self,
        version: MavLinkVersion,
    ) -> Result<MavLinkMessagePayload, MavLinkMessageProcessingError> {
        self.encode(version)
    }
}

impl Message {
    /// Decodes message from `MAVLink` payload.
    pub fn decode(
        payload: &MavLinkMessagePayload,
    ) -> Result<Self, MavLinkMessageProcessingError> {
        Ok(match payload.id() {
{{#each dialect.messages}}
            messages::{{to-message-mod-name name}}::MESSAGE_ID => Self::{{to-messages-enum-entry-name name}}(messages::{{to-message-struct-name name}}::try_from(payload)?),
{{/each}}
            id => return Err(MavLinkMessageProcessingError::UnsupportedMessageId(id)),
        })
    }

    /// Encodes message to `MAVLink` payload.
    pub fn encode(&self, version: MavLinkVersion) -> Result<MavLinkMessagePayload, MavLinkMessageProcessingError> {
        Ok(match self {
{{#each dialect.messages}}
            Self::{{to-messages-enum-entry-name name}}(message) => {message.encode(version)?}
{{/each}}
        })
    }
}
"#;
