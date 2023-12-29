pub mod enums;
pub mod messages;

/// Dialects module root template.
///
/// Input: [`mavinspect::protocol::Protocol`].
pub const DIALECTS_ROOT_MODULE: &str = r#"//! # Autogenerated MAVLink dialects
//!
//! > *Generated by [`MAVSpec`](https://gitlab.com/mavka/libs/mavspec)*
//!
//! Each dialect is packaged into a module with corresponding (`snake_cased`) name.
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

{{#each dialects}}
/// `{{name}}` dialect.
#[cfg(not(doctest))]
pub mod {{to-dialect-name name}};

{{/each}}
"#;

/// Dialect module root template.
///
/// Input: [`crate::generator::DialectSpec`].
pub const DIALECT_MODULE: &str = r#"//! # MAVLink dialect `{{name}}`

use mavspec::rust::spec::{
    IntoPayload, DialectSpec, Payload, MessageSpec,
    MavLinkVersion, MessageError,
};
use mavspec::rust::spec::types::{MessageId, DialectId, DialectVersion};

// MAVLink messages.
pub mod messages;
// MAVLink enums.
pub mod enums;

/// Dialect name as it appears in XML definition.
/// 
/// See [`DialectSpec::name`].
const NAME: &str = "{{name}}";
/// Dialect id as it appears in XML definition.
/// 
/// See [`DialectSpec::dialect`].
const ID: Option<DialectId> = {{to-dialect-id dialect_id}};
/// Dialect version as it appears in XML definition.
/// 
/// See [`DialectSpec::dialect`].
const VERSION: Option<DialectVersion> = {{to-dialect-ver version}};
/// [`Dialect`] specification.
///
/// See: [`DialectSpec`].
const SPEC: Dialect = Dialect {};

/// Dialect specification.
///
/// This struct can'be instantiated directly. The constant (and the only) instance is accessible
/// through [`spec`] function.  
#[derive(core::clone::Clone, core::fmt::Debug, Default)]
struct Dialect;

impl Dialect {
    /// Dialect name as it appears in XML definition.
    /// 
    /// See [`DialectSpec::name`].
    #[inline]
    pub fn name() -> &'static str {
        NAME
    }
    
    /// Returns `dialect` identifier as specified in MAVLink [XML definitions](https://mavlink.io/en/guide/xml_schema.html).
    //
    // See [`DialectSpec::dialect`].
    #[inline]
    fn dialect() -> Option<DialectId> {
        ID
    }
    
    /// Minor dialect `version` as specified in MAVLink [XML definitions](https://mavlink.io/en/guide/xml_schema.html).
    ///
    /// See [`DialectSpec::version`].
    #[inline]
    fn version() -> Option<DialectVersion> {
        VERSION
    }

    /// Message specification by `id`.
    /// 
    /// See [`DialectSpec::message_info`].
    #[inline]
    pub fn message_info(id: MessageId) -> Result<&'static dyn MessageSpec, MessageError> {
        message_info(id)
    }
}

impl DialectSpec for Dialect {
    /// Message specification by `id`.
    ///
    /// See [`DialectSpec::name`].
    #[inline]
    fn name(&self) -> &str {
        Self::name()
    }
    
    /// Returns `dialect` identifier as specified in MAVLink [XML definitions](https://mavlink.io/en/guide/xml_schema.html).
    //
    // See [`DialectSpec::dialect`].
    #[inline]
    fn dialect(&self) -> Option<DialectId> {
        Self::dialect()
    }
    
    /// Minor dialect `version` as specified in MAVLink [XML definitions](https://mavlink.io/en/guide/xml_schema.html).
    ///
    /// See [`DialectSpec::version`].
    #[inline]
    fn version(&self) -> Option<DialectVersion> {
        Self::version()
    }

    /// Message specification by `id`.
    /// 
    /// See [`DialectSpec::message_info`].
    #[inline]
    fn message_info(&self, id: MessageId) -> Result<&dyn MessageSpec, MessageError> {
        Self::message_info(id)
    }
}

/// Enum containing all messages within `{{name}}` dialect.
#[derive(core::clone::Clone, core::fmt::Debug)]
// {{#if params.serde}}#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]{{/if}}
#[allow(clippy::large_enum_variant)]
pub enum Message {
{{#each messages}}
    /// MAVLink message `{{name}}`.
    {{to-messages-enum-entry-name name}}(messages::{{to-message-struct-name name}}),
{{/each}}
}

/// Enum containing all raw messages within `{{name}}` dialect.
#[derive(core::clone::Clone, core::fmt::Debug)]
// {{#if params.serde}}#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]{{/if}}
#[allow(clippy::large_enum_variant)]
pub enum MessageRaw {
{{#each messages}}
    /// Raw MAVLink message `{{name}}`.
    {{to-messages-enum-entry-name name}}(messages::{{to-message-mod-name name}}::{{to-message-raw-struct-name name}}),
{{/each}}
}

impl core::convert::TryFrom<&Payload> for Message {
    type Error = MessageError;

    /// Decodes message from MAVLink payload.
    fn try_from(value: &Payload) -> Result<Self, Self::Error> {
        Self::decode(value)
    }
}

impl core::convert::TryFrom<&Payload> for MessageRaw {
    type Error = MessageError;

    /// Decodes message from MAVLink payload.
    fn try_from(value: &Payload) -> Result<Self, Self::Error> {
        Self::decode(value)
    }
}

impl IntoPayload for Message {
    /// Encodes message into MAVLink payload.
    fn encode(
        &self,
        version: MavLinkVersion,
    ) -> Result<Payload, MessageError> {
        self.encode(version)
    }
}

impl IntoPayload for MessageRaw {
    /// Encodes raw message into MAVLink payload.
    fn encode(
        &self,
        version: MavLinkVersion,
    ) -> Result<Payload, MessageError> {
        self.encode(version)
    }
}

impl Message {
    /// Decodes message from MAVLink payload.
    pub fn decode(
        payload: &Payload,
    ) -> Result<Self, MessageError> {
        decode(payload)
    }

    /// Encodes message to MAVLink payload.
    pub fn encode(&self, version: MavLinkVersion) -> Result<Payload, MessageError> {
        encode(self, version)
    }
}

impl MessageRaw {
    /// Decodes raw message from MAVLink payload.
    pub fn decode(
        payload: &Payload,
    ) -> Result<Self, MessageError> {
        decode_raw(payload)
    }

    /// Encodes raw message to MAVLink payload.
    pub fn encode(&self, version: MavLinkVersion) -> Result<Payload, MessageError> {
        encode_raw(self, version)
    }
}

/// Dialect specification.
///
/// Returns the current dialect specification as [`DialectSpec`] trait object.
#[inline]
pub const fn spec() -> &'static dyn DialectSpec {
    &SPEC
}

/// Retrieve message specification by its `id`.
/// 
/// See [`DialectSpec::message_info`].
pub fn message_info(id: MessageId) -> Result<&'static dyn MessageSpec, MessageError> {
    Ok(match id {
{{#each messages}}
        {{id}} => &messages::{{to-message-mod-name name}}::MESSAGE_INFO,
{{/each}}
        _ => return Err(MessageError::UnsupportedMessageId(id)),
    })
}

/// Decodes [`Message`] from [`Payload`].
pub fn decode(payload: &Payload) -> Result<Message, MessageError> {
    Ok(match payload.id() {
{{#each messages}}
            messages::{{to-message-mod-name name}}::MESSAGE_ID => Message::{{to-messages-enum-entry-name name}}(messages::{{to-message-struct-name name}}::try_from(payload)?),
{{/each}}
        id => return Err(MessageError::UnsupportedMessageId(id)),
    })
}

/// Decodes [`MessageRaw`] from [`Payload`].
pub fn decode_raw(payload: &Payload) -> Result<MessageRaw, MessageError> {
    Ok(match payload.id() {
{{#each messages}}
            messages::{{to-message-mod-name name}}::MESSAGE_ID => MessageRaw::{{to-messages-enum-entry-name name}}(messages::{{to-message-mod-name name}}::{{to-message-raw-struct-name name}}::try_from(payload)?),
{{/each}}
        id => return Err(MessageError::UnsupportedMessageId(id)),
    })
}

/// Encodes [`Message`] into [`Payload`].
pub fn encode(msg: &Message, version: MavLinkVersion) -> Result<Payload, MessageError> {
    Ok(match msg {
{{#each messages}}
        Message::{{to-messages-enum-entry-name name}}(message) => {message.encode(version)?}
{{/each}}
    })
}

/// Encodes [`MessageRaw`] into [`Payload`].
pub fn encode_raw(msg: &MessageRaw, version: MavLinkVersion) -> Result<Payload, MessageError> {
    Ok(match msg {
{{#each messages}}
        MessageRaw::{{to-messages-enum-entry-name name}}(raw_message) => {raw_message.encode(version)?}
{{/each}}
    })
}

{{#if params.generate_tests}}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieve_message_info() {
        for id in [
{{#each messages}}
            {{id}},
{{/each}}
        ] {
            let msg_info = spec().message_info(id);
            assert!(msg_info.is_ok());
            assert_eq!(msg_info.unwrap().id(), id);
        }
    }
}
{{/if}}
"#;
