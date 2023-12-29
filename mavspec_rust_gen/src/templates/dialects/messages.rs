use std::collections::HashMap;

use mavinspect::protocol::{MavType, Message, MessageField, MessageId};
use serde::Serialize;

use crate::conventions::split_description;
use crate::generator::DialectSpec;
use crate::generator::GeneratorParams;

/// Input for [`MESSAGES_MODULE_ROOT`] template.
#[derive(Clone, Debug, Serialize)]
pub struct MessagesSpec<'a> {
    dialect_name: String,
    messages: &'a HashMap<MessageId, Message>,
}

impl<'a> MessagesSpec<'a> {
    pub fn new(dialect_spec: &'a DialectSpec) -> Self {
        Self {
            dialect_name: dialect_spec.name().to_string(),
            messages: dialect_spec.messages(),
        }
    }
}

/// Messages module root template.
///
/// Input: [`DialectSpec`].
pub const MESSAGES_MODULE_ROOT: &str = "\
//! MAVLink messages of `{{dialect_name}}` dialect.

{{#each messages}}
// MAVLink message `{{name}}`.
pub mod {{to-message-mod-name name}};
pub use {{to-message-mod-name name}}::{{to-message-struct-name name}};

{{/each}}\
";

/// Input for [`MESSAGE`] template.
///
/// Basically, this is a utility wrapper around `MAVInspect` [`Message`].
#[derive(Clone, Debug, Serialize)]
pub struct MessageSpec<'a> {
    id: u32,
    name: String,
    description: Vec<String>,
    fields: Vec<FieldSpec>,
    is_v1_compatible: bool,
    fields_v1: Vec<FieldSpec>,
    payload_v1_size: usize,
    fields_v2: Vec<FieldSpec>,
    payload_v2_size: usize,
    extension_fields: Vec<FieldSpec>,
    has_extension_fields: bool,
    crc_extra: u8,
    params: &'a GeneratorParams,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct FieldSpec {
    name: String,
    description: Vec<String>,
    r#type: MavType,
    is_enum: bool,
    is_bitmask: bool,
    enum_name: String,
    enum_type: MavType,
    is_array: bool,
    array_length: usize,
    cast_enum: bool,
    serde_arrays: bool,
}

impl FieldSpec {
    fn from_mavinspect_field(value: &MessageField, dialect_spec: &DialectSpec) -> FieldSpec {
        let mut spec = FieldSpec {
            name: value.name().into(),
            description: split_description(value.description()),
            r#type: value.r#type().clone(),
            is_array: value.r#type().is_array(),
            ..Default::default()
        };

        if let MavType::Array(_, len) = value.r#type() {
            spec.array_length = *len;

            if *len > 32 && dialect_spec.params().serde {
                spec.serde_arrays = true;
            }
        }

        if let Some(enum_name) = value.r#enum() {
            if let Some(field_enum) = dialect_spec.enums().get(enum_name) {
                spec.is_enum = true;
                spec.enum_name = field_enum.name().into();
                spec.enum_type = field_enum.inferred_type();
                spec.is_bitmask = field_enum.bitmask();
                spec.cast_enum = field_enum.inferred_type() < *value.r#type().base_type()
                    || field_enum.inferred_type() != *value.r#type().base_type();
            }
        }

        spec
    }

    fn from_mavinspect_fields(
        fields: &[MessageField],
        dialect_spec: &DialectSpec,
    ) -> Vec<FieldSpec> {
        fields
            .iter()
            .map(|fld| FieldSpec::from_mavinspect_field(fld, dialect_spec))
            .collect()
    }
}

impl<'a> MessageSpec<'a> {
    /// Constructs from [`Message`] and [`Dialect`].
    pub fn new(message: &Message, dialect_spec: &'a DialectSpec) -> Self {
        Self {
            id: message.id(),
            name: message.name().to_string(),
            description: split_description(message.description()),
            fields: FieldSpec::from_mavinspect_fields(message.fields(), dialect_spec),
            // `MAVLink 1`
            is_v1_compatible: message.is_v1_compatible(),
            fields_v1: FieldSpec::from_mavinspect_fields(
                message.fields_v1().as_slice(),
                dialect_spec,
            ),
            payload_v1_size: message.size_v1(),
            // `MAVLink 2`
            fields_v2: FieldSpec::from_mavinspect_fields(
                message.fields_v2().as_slice(),
                dialect_spec,
            ),
            payload_v2_size: message.size_v2(),
            extension_fields: FieldSpec::from_mavinspect_fields(
                message.extension_fields().as_slice(),
                dialect_spec,
            ),
            has_extension_fields: message.has_extension_fields(),
            // CRC
            crc_extra: message.crc_extra(),
            // Generator params
            params: dialect_spec.params(),
        }
    }
}

/// Message template.
///
/// Input: [`MessageSpec`].
pub const MESSAGE: &str = r#"//! # MAVLink `{{name}}` message implementation.

use mavspec::rust::spec::{
    IntoPayload, MavLinkVersion, MessageError, MessageImpl, MessageInfo, MessageSpec, Payload,
};
use mavspec::rust::spec::types::{MessageId, CrcExtra};

/// `{{name}}` message ID.
pub(crate) const MESSAGE_ID: MessageId = {{id}};
/// `{{name}}` `EXTRA_CRC` calculated from message XML definition.
pub(crate) const EXTRA_CRC: CrcExtra = {{crc_extra}};
/// `{{name}}` generic message info that contains all message metadata.
pub(crate) const MESSAGE_INFO: MessageInfo = MessageInfo::new(MESSAGE_ID, EXTRA_CRC);

/// MAVLink message `{{name}}` specification
#[inline]
pub const fn spec() -> &'static dyn MessageSpec {
    &MESSAGE_INFO
}

#[allow(rustdoc::bare_urls)]
#[allow(rustdoc::broken_intra_doc_links)]
#[allow(rustdoc::invalid_rust_codeblocks)]
/// MAVLink `{{name}}` message.
///
/// Minimum supported MAVLink version is `MAVLink {{#if is_v1_compatible}}1{{else}}2{{/if}}`.
///
/// # Description
///
{{#each description}}
/// {{this}}
{{/each}}
///
/// # Encoding/Decoding
/// 
/// Message encoding/decoding are provided by implementing [`core::convert::TryFrom<Payload>`] for
/// [`{{to-message-struct-name name}}`] (encoding) and [`IntoPayload`] (decoding) traits.
#[derive(core::clone::Clone, core::fmt::Debug)]
{{#if params.serde}}
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
{{/if}}
pub struct {{to-message-struct-name name}} {
{{#each fields}}
    /// MAVLink field `{{name}}`.
    ///
{{#each description}}
    /// {{this}}
{{/each}}
{{#if serde_arrays}}
    #[cfg_attr(feature = "serde", serde(with = "serde_arrays"))]
{{/if}}
{{#if is_enum}}
{{#if is_array}}
    pub {{to-rust-var name}}: [super::super::enums::{{to-enum-rust-name enum_name}}; {{array_length}}],
{{else}}
    pub {{to-rust-var name}}: super::super::enums::{{to-enum-rust-name enum_name}},
{{/if}}
{{else}}
    pub {{to-rust-var name}}: {{to-rust-type type}},
{{/if}}
{{/each}}
}

/// Raw representation of [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) MAVLink message.
///
/// Minimum supported MAVLink version is `MAVLink {{#if is_v1_compatible}}1{{else}}2{{/if}}`.
#[derive(core::clone::Clone, core::fmt::Debug)]
{{#if params.serde}}#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]{{/if}}
pub struct {{to-message-raw-struct-name name}} {
{{#each fields}}
    /// MAVLink field `{{name}}`.
{{#if serde_arrays}}
    #[cfg_attr(feature = "serde", serde(with = "serde_arrays"))]
{{/if}}
    pub {{to-rust-var name}}: {{to-rust-type type}},
{{/each}}
}

impl MessageSpec for {{to-message-struct-name name}} {
    /// MAVLink message ID.
    ///
    /// See [`MessageSpec::id`].
    #[inline]
    fn id(&self) -> MessageId {
        MESSAGE_ID
    }

    /// Minimum supported MAVLink version.
    ///
    /// See [`MessageSpec::min_supported_mavlink_version`].
    #[inline]
    fn min_supported_mavlink_version(&self) -> MavLinkVersion {
        MESSAGE_INFO.min_supported_mavlink_version()
    }
    
    /// Message `EXTRA_CRC` calculated from message XML definition.
    ///
    /// See: [`MessageSpec::crc_extra`].
    #[inline]
    fn crc_extra(&self) -> CrcExtra {
        EXTRA_CRC
    }
}

impl MessageSpec for {{to-message-raw-struct-name name}} {
    /// MAVLink message ID.
    ///
    /// See [`MessageSpec::id`].
    #[inline]
    fn id(&self) -> MessageId {
        MESSAGE_ID
    }

    /// Minimum supported MAVLink version.
    ///
    /// See [`MessageSpec::min_supported_mavlink_version`].
    #[inline]
    fn min_supported_mavlink_version(&self) -> MavLinkVersion {
        MESSAGE_INFO.min_supported_mavlink_version()
    }
    
    /// Message `EXTRA_CRC` calculated from message XML definition.
    ///
    /// See: [`MessageSpec::crc_extra`].
    #[inline]
    fn crc_extra(&self) -> CrcExtra {
        EXTRA_CRC
    }
}

// Implement `MessageImpl` that combines `MessageSpec` with `IntoPayload`
impl MessageImpl for {{to-message-struct-name name}} {}

// Implement `MessageImpl` that combines `MessageSpec` with `IntoPayload`
impl MessageImpl for {{to-message-raw-struct-name name}} {}

#[allow(clippy::derivable_impls)]
impl core::default::Default for {{to-message-struct-name name}} {
    /// Creates [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) initialized with default values.
    fn default() -> Self {
        Self {
{{#each fields}}
{{#if is_enum}}
{{#if is_array}}
            {{to-rust-var name}}: [super::super::enums::{{to-enum-rust-name enum_name}}::default(); {{array_length}}],
{{else}}
            {{to-rust-var name}}: super::super::enums::{{to-enum-rust-name enum_name}}::default(),
{{/if}}
{{else}}
            {{to-rust-var name}}: {{to-rust-default-value type}},
{{/if}}
{{/each}}
        }
    }
}

#[allow(clippy::derivable_impls)]
impl core::default::Default for {{to-message-raw-struct-name name}} {
    /// Creates [`{{to-message-raw-struct-name name}}`] initialized with default values.
    fn default() -> Self {
        {{to-message-struct-name name}}::default().into()
    }
}

impl core::convert::TryFrom<&Payload> for {{to-message-struct-name name}} {
    type Error = MessageError;

    /// Decodes [`Payload`] into [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) according to [`MavLinkVersion`].
    #[inline]
    fn try_from(value: &Payload) -> Result<Self, Self::Error> {
        Self::try_from_payload(value)
    }
}

impl core::convert::TryFrom<&Payload> for {{to-message-raw-struct-name name}} {
    type Error = MessageError;

    /// Decodes [`Payload`] into [`{{to-message-raw-struct-name name}}`] according to [`MavLinkVersion`].
    #[inline]
    fn try_from(value: &Payload) -> Result<Self, Self::Error> {
        Self::try_from_payload(value)
    }
}

impl IntoPayload for {{to-message-struct-name name}} {
    /// Encodes [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) into [`Payload`] according to [`MavLinkVersion`].
    #[inline]
    fn encode(
        &self,
        version: MavLinkVersion,
    ) -> Result<Payload, MessageError> {
        self.encode(version)
    }
}

impl IntoPayload for {{to-message-raw-struct-name name}} {
    /// Encodes [`{{to-message-raw-struct-name name}}`] into [`Payload`] according to [`MavLinkVersion`].
    #[inline]
    fn encode(
        &self,
        version: MavLinkVersion,
    ) -> Result<Payload, MessageError> {
        self.encode(version)
    }
}

impl {{to-message-struct-name name}} {
    /// Decodes [`Payload`] into [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) according to [`MavLinkVersion`].
    pub fn try_from_payload(value: &Payload) -> Result<Self, MessageError> {
        match value.version() {
            MavLinkVersion::V2 => v2::decode(value.bytes()),
{{#if is_v1_compatible}}
            MavLinkVersion::V1 => v1::decode(value.bytes()),
{{else}}
            version => {
                Err(MessageError::UnsupportedMavLinkVersion {
                    actual: version,
                    minimal: MESSAGE_INFO.min_supported_mavlink_version(),
                })
            }
{{/if}}
        }
    }

    /// Encodes [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) into [`Payload`] according to [`MavLinkVersion`].
    pub fn encode(
        &self,
        version: MavLinkVersion,
    ) -> Result<Payload, MessageError> {
        Ok(match version {
            MavLinkVersion::V2 => v2::encode(self)?,
{{#if is_v1_compatible}}
            MavLinkVersion::V1 => v1::encode(self)?,
{{else}}
            _ => {
                return Err(MessageError::UnsupportedMavLinkVersion {
                    actual: version,
                    minimal: MESSAGE_INFO.min_supported_mavlink_version(),
                })
            }
{{/if}}
        })
    }
    
    /// Converts into raw [`{{to-message-raw-struct-name name}}`].
    pub fn to_raw_message(&self) -> {{to-message-raw-struct-name name}} {
        {{to-message-raw-struct-name name}} {
{{#each fields}}
{{#if is_enum}}
    {{#if is_bitmask}}
        {{#if is_array}}
            {{to-rust-var name}}: self.{{to-rust-var name}}.map(|i| i.bits(){{#if cast_enum}} as {{to-rust-base-type type}}{{/if}}),
        {{else}}
            {{to-rust-var name}}: self.{{to-rust-var name}}.bits(){{#if cast_enum}} as {{to-rust-type type}}{{/if}},
        {{/if}}
    {{else}}
        {{#if is_array}}
            {{to-rust-var name}}: self.{{to-rust-var name}}.map(|i| i as {{to-rust-base-type type}}),
        {{else}}
            {{to-rust-var name}}: self.{{to-rust-var name}} as {{to-rust-type type}},
        {{/if}}
    {{/if}}
{{else}}
            {{to-rust-var name}}: self.{{to-rust-var name}},
{{/if}}
{{/each}}            
        }
    }
}

impl {{to-message-raw-struct-name name}} {
    /// Decodes [`Payload`] into [`{{to-message-raw-struct-name name}}`] according to [`MavLinkVersion`].
    pub fn try_from_payload(value: &Payload) -> Result<Self, MessageError> {
        match value.version() {
            MavLinkVersion::V2 => v2::decode_raw(value.bytes()),
{{#if is_v1_compatible}}
            MavLinkVersion::V1 => v1::decode_raw(value.bytes()),
{{else}}
            version => {
                Err(MessageError::UnsupportedMavLinkVersion {
                    actual: version,
                    minimal: MESSAGE_INFO.min_supported_mavlink_version(),
                })
            }
{{/if}}
        }
    }
    
    /// Encodes [`{{to-message-raw-struct-name name}}`] into [`Payload`] according to [`MavLinkVersion`].
    pub fn encode(
        &self,
        version: MavLinkVersion,
    ) -> Result<Payload, MessageError> {
        Ok(match version {
            MavLinkVersion::V2 => v2::encode_raw(self)?,
{{#if is_v1_compatible}}
            MavLinkVersion::V1 => v1::encode_raw(self)?,
{{else}}
            _ => {
                return Err(MessageError::UnsupportedMavLinkVersion {
                    actual: version,
                    minimal: MESSAGE_INFO.min_supported_mavlink_version(),
                })
            }
{{/if}}
        })
    }

    /// Attempts to convert [`{{to-message-raw-struct-name name}}`] into [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}).
    pub fn try_to_message(&self) -> Result<{{to-message-struct-name name}}, MessageError> {
        Ok({{to-message-struct-name name}} {
{{#each fields}}
{{#if is_enum}}
{{#if is_bitmask}}
    {{#if is_array}}
            {{to-rust-var name}}: {
                use super::super::enums::{{to-enum-rust-name enum_name}} as Enum;
                let mut values: [Enum; {{array_length}}] = [Enum::default(); {{array_length}}];
                #[allow(clippy::needless_range_loop)]
                for i in 0 .. {{array_length}} {
                    values[i] = Enum::from_bits_retain(
                        self.{{to-rust-var name}}[i]{{#if cast_enum}} as {{to-rust-type enum_type}}{{/if}},
                    )
                }
                values
            },
    {{else}}
            {{to-rust-var name}}: super::super::enums::{{to-enum-rust-name enum_name}}::from_bits_retain(
                self.{{to-rust-var name}}{{#if cast_enum}} as {{to-rust-type enum_type}}{{/if}}
            ),
    {{/if}}
{{else}}
    {{#if is_array}}
            {{to-rust-var name}}: {
                use super::super::enums::{{to-enum-rust-name enum_name}} as Enum;
                let mut values: [Enum; {{array_length}}] = [Enum::default(); {{array_length}}];
                #[allow(clippy::needless_range_loop)]
                for i in 0 .. {{array_length}} {
                    values[i] = Enum::try_from_discriminant(
                        self.{{to-rust-var name}}[i]{{#if cast_enum}} as {{to-rust-type enum_type}}{{/if}},
                    )?
                }
                values
            },
    {{else}}
            {{to-rust-var name}}: super::super::enums::{{to-enum-rust-name enum_name}}::try_from_discriminant(
                self.{{to-rust-var name}}{{#if cast_enum}} as {{to-rust-type enum_type}}{{/if}}
            )?,
    {{/if}}
{{/if}}
{{else}}
            {{to-rust-var name}}: self.{{to-rust-var name}},
{{/if}}
{{/each}}            
        })
    }
}

impl core::convert::TryFrom<{{to-message-raw-struct-name name}}> for {{to-message-struct-name name}} {
    type Error = MessageError;

    /// Converts raw [`{{to-message-raw-struct-name name}}`] into enriched [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}).
    ///
    /// # Errors
    ///
    /// * Returns [`MessageError::InvalidEnumValue`] if [`{{to-message-raw-struct-name name}}`] contains invalid values
    /// for MAVLink enums in [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}).
    fn try_from(value: {{to-message-raw-struct-name name}}) -> Result<Self, Self::Error> {
        value.try_to_message()
    }
}

impl core::convert::From<{{to-message-struct-name name}}> for {{to-message-raw-struct-name name}} {
    /// Converts enriched [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) into raw [`{{to-message-raw-struct-name name}}`].
    fn from(value: {{to-message-struct-name name}}) -> Self {
        value.to_raw_message()
    }
}

/// Encoding/decoding for [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) within `MAVLink 2` protocol.
///
/// See [MAVLink 2](https://mavlink.io/en/guide/mavlink_2.html).
pub mod v2 {
    use mavspec::rust::spec::{Payload, MavLinkVersion, MessageError};
    use mavspec::rust::spec::tbytes::{TBytesWriterFor, TBytesReader, TBytesReaderFor, TBytesWriter};
    
    use super::{ {{to-message-struct-name name}}, {{to-message-raw-struct-name name}}, MESSAGE_ID };

    /// Message [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) payload size (non-truncated) according to `MAVLink 2` protocol.
    pub const PAYLOAD_SIZE: usize = {{payload_v2_size}};
    
    /// Decodes into [`{{to-message-raw-struct-name name}}`] message.
    ///
    /// If `payload` is less than expected, the remaining elements will be considered to be zeros.
    /// See [MAVLink 2 payload truncation](https://mavlink.io/en/guide/serialization.html#payload_truncation).
    ///
    /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///
    /// # Errors
    /// 
    /// * Returns [`MessageError::BufferError`] in case of malformed `payload`.
    pub fn decode_raw(payload: &[u8]) -> Result<{{to-message-raw-struct-name name}}, MessageError> {
        let mut full_payload = [0u8; PAYLOAD_SIZE];
        full_payload[0..payload.len()].copy_from_slice(payload);

        let reader = TBytesReader::from(full_payload.as_slice());      

        Ok({{to-message-raw-struct-name name}} {
            // Fields are reordered according to MAVLink specification
{{#each fields_v2}}
            {{to-rust-var name}}: reader.{{to-reader-fn type}}()?,
{{/each}}
        })
    }

    /// Decodes into [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) message.
    ///
    /// If `payload` is less than expected, the remaining elements will be considered to be zeros.
    /// See [MAVLink 2 payload truncation](https://mavlink.io/en/guide/serialization.html#payload_truncation).
    ///
    /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///
    /// # Errors
    /// 
    /// * Returns [`MessageError::BufferError`] in case of malformed `payload`.
    /// * Returns [`MessageError::InvalidEnumValue`] if invalid value was provided for MAVLink enum.
    pub fn decode(payload: &[u8]) -> Result<{{to-message-struct-name name}}, MessageError> {
        decode_raw(payload)?.try_into()
    }
    
    /// Encodes [`{{to-message-raw-struct-name name}}`] message into MAVLink [`Payload`].
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
    pub fn encode_raw(
        message: &{{to-message-raw-struct-name name}}
    ) -> Result<Payload, MessageError> {
        let mut buf = [0u8; PAYLOAD_SIZE];
        let mut writer = TBytesWriter::from(buf.as_mut_slice());

        // Fields are reordered according to MAVLink specification
{{#each fields_v2}}
        writer.{{to-writer-fn type}}(message.{{to-rust-var name}})?;
{{/each}}

        let payload = Payload::new(MESSAGE_ID, buf.as_slice(), MavLinkVersion::V2);
        Ok(payload)
    }

    /// Encodes [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) message into MAVLink [`Payload`].
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
    ) -> Result<Payload, MessageError> {
        encode_raw(&message.to_raw_message())
    }
}

{{#if is_v1_compatible}}
/// Encoding/decoding for [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) within `MAVLink 1` protocol.
///
/// See [MAVLink versions](https://mavlink.io/en/guide/mavlink_version.html).
pub mod v1 {
    use mavspec::rust::spec::{Payload, MavLinkVersion, MessageError};
    use mavspec::rust::spec::tbytes::{TBytesWriterFor, TBytesReader, TBytesReaderFor, TBytesWriter};
    
    use super::{ {{to-message-struct-name name}}, {{to-message-raw-struct-name name}}, MESSAGE_ID };

    /// Message [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) payload size according to `MAVLink 1` protocol.
    pub const PAYLOAD_SIZE: usize = {{payload_v1_size}};

    /// Decodes [`Payload`] into [`{{to-message-raw-struct-name name}}`] message.
    ///
    /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///
    /// # Errors
    /// 
    /// * Returns [`MessageError::InvalidPayloadSize`] if `payload` has incorrect size.
    ///   Payload size is defined in [`PAYLOAD_SIZE`].
    /// * Returns [`MessageError::BufferError`] in case of malformed `payload`.
    pub fn decode_raw(payload: &[u8]) -> Result<{{to-message-raw-struct-name name}}, MessageError> {
        if payload.len() != PAYLOAD_SIZE {
            return Err(MessageError::InvalidPayloadSize {
                actual: payload.len(),
                expected: PAYLOAD_SIZE,
            });
        }
        let reader = TBytesReader::from(payload);

        Ok({{to-message-raw-struct-name name}} {
            // Fields are reordered according to MAVLink specification
{{#each fields_v1}}
            {{to-rust-var name}}: reader.{{to-reader-fn type}}()?,
{{/each}}
{{#if has_extension_fields}}
            // Set default values for `MAVLink 2` extensions field
            .. core::default::Default::default()
{{/if}}
        })
    }
    
    /// Decodes [`Payload`] into [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) message.
    ///
    /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///
    /// # Errors
    /// 
    /// * Returns [`MessageError::InvalidPayloadSize`] if `payload` has incorrect size.
    ///   Payload size is defined in [`PAYLOAD_SIZE`].
    /// * Returns [`MessageError::BufferError`] in case of malformed `payload`.
    /// * Returns [`MessageError::InvalidEnumValue`] if invalid value was provided for MAVLink enum.
    pub fn decode(payload: &[u8]) -> Result<{{to-message-struct-name name}}, MessageError> {
        decode_raw(payload)?.try_into()
    }

    /// Encodes [`{{to-message-raw-struct-name name}}`] message into MAVLink [`Payload`].
    ///
    /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///
    /// # Errors
    /// 
    /// This function does not returns errors at the moment. The [`Result`] returning type is
    /// reserved for future implementations where such errors may happen.
    pub fn encode_raw(
        message: &{{to-message-raw-struct-name name}}
    ) -> Result<Payload, MessageError> {
        let mut buf = [0u8; PAYLOAD_SIZE];
        let mut writer = TBytesWriter::from(buf.as_mut_slice());

        // Fields are reordered according to MAVLink specification
{{#each fields_v1}}
        writer.{{to-writer-fn type}}(message.{{to-rust-var name}})?;
{{/each}}
{{#if has_extension_fields}}
        // The following extension fields are ignored in `MAVLink 1`
{{#each extension_fields}}
        // message.{{to-rust-var name}}
{{/each}}
{{/if}}

        let payload = Payload::new(MESSAGE_ID, buf.as_slice(), MavLinkVersion::V1);
        Ok(payload)
    }
    
    /// Encodes [`{{to-message-struct-name name}}`](struct@self::{{to-message-struct-name name}}) message into MAVLink [`Payload`].
    ///
    /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///
    /// # Errors
    /// 
    /// This function does not returns errors at the moment. The [`Result`] returning type is
    /// reserved for future implementations where such errors may happen.
    pub fn encode(
        message: &{{to-message-struct-name name}}
    ) -> Result<Payload, MessageError> {
        encode_raw(&message.to_raw_message())
    }
}
{{/if}}

{{#if params.generate_tests}}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_v2() {
        let message = {{to-message-struct-name name}}::default();
        let encoded_payload = v2::encode(&message).unwrap();
        let decoded_message = v2::decode(encoded_payload.bytes()).unwrap();
        let encoded_versioned_payload = message.encode(MavLinkVersion::V2).unwrap();

        assert_eq!(decoded_message.id(), message.id());
        
        assert_eq!(encoded_payload.id(), message.id());
        assert!(matches!(encoded_payload.version(), MavLinkVersion::V2));
        
        assert_eq!(encoded_versioned_payload.id(), message.id());
        assert!(matches!(encoded_versioned_payload.version(), MavLinkVersion::V2));
    }
{{#if is_v1_compatible}}

    #[test]
    fn basic_v1() {
        let message = {{to-message-struct-name name}}::default();
        let encoded_payload = v1::encode(&message).unwrap();
        let decoded_message = v1::decode(encoded_payload.bytes()).unwrap();
        let encoded_versioned_payload = message.encode(MavLinkVersion::V1).unwrap();

        assert_eq!(decoded_message.id(), message.id());
        
        assert_eq!(encoded_payload.id(), message.id());
        assert!(matches!(encoded_payload.version(), MavLinkVersion::V1));
        
        assert_eq!(encoded_versioned_payload.id(), message.id());
        assert!(matches!(encoded_versioned_payload.version(), MavLinkVersion::V1));
    }
{{/if}}
}
{{/if}}
"#;

/// Input for [`INHERITED_MESSAGE`] template.
#[derive(Clone, Debug, Serialize)]
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

use mavspec::rust::spec::MessageInfo;

use super::super::super::{{to-dialect-name dialect_name}} as dialect;

pub(crate) const MESSAGE_ID: u32 = dialect::messages::{{to-message-mod-name message_name}}::MESSAGE_ID;
pub(crate) const MESSAGE_INFO: MessageInfo =
    dialect::messages::{{to-message-mod-name message_name}}::MESSAGE_INFO;

/// Originally defined in [`{{to-dialect-name dialect_name}}::messages::{{to-message-struct-name message_name}}`](dialect::messages::{{to-message-struct-name message_name}}).
pub type {{to-message-struct-name message_name}} = dialect::messages::{{to-message-struct-name message_name}};

/// Originally defined in [`{{to-dialect-name dialect_name}}::messages::{{to-message-mod-name message_name}}::{{to-message-raw-struct-name message_name}}`](dialect::messages::{{to-message-mod-name message_name}}::{{to-message-raw-struct-name message_name}}).
pub type {{to-message-raw-struct-name message_name}} = dialect::messages::{{to-message-mod-name message_name}}::{{to-message-raw-struct-name message_name}};

/// Re-exported from [`{{to-dialect-name dialect_name}}`](dialect) dialect.
pub use dialect::messages::{{to-message-mod-name message_name}}::spec;

/// Re-exported from [`{{to-dialect-name dialect_name}}`](dialect) dialect.
pub use dialect::messages::{{to-message-mod-name message_name}}::v2;
{{#if is_v1_compatible}}

/// Re-exported from [`{{to-dialect-name dialect_name}}`](dialect) dialect.
pub use dialect::messages::{{to-message-mod-name message_name}}::v1;
{{/if}}
";
