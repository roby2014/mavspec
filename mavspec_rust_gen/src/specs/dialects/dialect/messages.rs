use std::collections::HashMap;

use mavinspect::protocol::{MavType, Message, MessageField, MessageId};
use quote::format_ident;
use serde::Serialize;

use crate::conventions::{message_struct_name, split_description};
use crate::generator::GeneratorParams;
use crate::specs::dialects::dialect::DialectModuleSpec;
use crate::specs::Spec;

/// Input for `messages` root module template.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct MessagesRootModuleSpec<'a> {
    dialect_name: &'a str,
    messages: &'a HashMap<MessageId, Message>,
    params: &'a GeneratorParams,
}

impl<'a> Spec for MessagesRootModuleSpec<'a> {
    fn params(&self) -> &GeneratorParams {
        self.params
    }
}

impl<'a> MessagesRootModuleSpec<'a> {
    pub(crate) fn new(dialect_spec: &'a DialectModuleSpec, params: &'a GeneratorParams) -> Self {
        Self {
            dialect_name: dialect_spec.name(),
            messages: dialect_spec.messages(),
            params,
        }
    }

    pub(crate) fn dialect_name(&self) -> &str {
        self.dialect_name
    }

    pub(crate) fn messages(&self) -> &HashMap<MessageId, Message> {
        self.messages
    }
}

/// Input for MAVLink message module.
///
/// Basically, this is a utility wrapper around `MAVInspect` [`Message`].
#[derive(Clone, Debug, Serialize)]
pub struct MessageImplModuleSpec<'a> {
    id: u32,
    name: &'a str,
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

impl<'a> Spec for MessageImplModuleSpec<'a> {
    fn params(&self) -> &GeneratorParams {
        self.params
    }
}

impl<'a> MessageImplModuleSpec<'a> {
    /// Constructs from [`Message`] and [`DialectModuleSpec`].
    pub fn new(message: &'a Message, dialect_spec: &'a DialectModuleSpec) -> Self {
        Self {
            id: message.id(),
            name: message.name(),
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

    pub(crate) fn id(&self) -> u32 {
        self.id
    }

    pub(crate) fn name(&self) -> &str {
        self.name
    }

    pub(crate) fn ident(&self) -> syn::Ident {
        format_ident!("{}", message_struct_name(self.name))
    }

    pub(crate) fn description(&self) -> &[String] {
        self.description.as_slice()
    }

    pub(crate) fn fields(&self) -> &[FieldSpec] {
        self.fields.as_slice()
    }

    pub(crate) fn is_v1_compatible(&self) -> bool {
        self.is_v1_compatible
    }

    pub(crate) fn fields_v1(&self) -> &[FieldSpec] {
        self.fields_v1.as_slice()
    }

    pub(crate) fn payload_v1_size(&self) -> usize {
        self.payload_v2_size
    }

    pub(crate) fn fields_v2(&self) -> &[FieldSpec] {
        self.fields_v2.as_slice()
    }

    pub(crate) fn payload_v2_size(&self) -> usize {
        self.payload_v2_size
    }

    pub(crate) fn has_extension_fields(&self) -> bool {
        self.has_extension_fields
    }

    pub(crate) fn crc_extra(&self) -> u8 {
        self.crc_extra
    }
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
    requires_serde_arrays: bool,
}

impl FieldSpec {
    fn from_mavinspect_field(value: &MessageField, dialect_spec: &DialectModuleSpec) -> FieldSpec {
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
                spec.requires_serde_arrays = true;
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
        dialect_spec: &DialectModuleSpec,
    ) -> Vec<FieldSpec> {
        fields
            .iter()
            .map(|fld| FieldSpec::from_mavinspect_field(fld, dialect_spec))
            .collect()
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(crate) fn description(&self) -> &[String] {
        self.description.as_slice()
    }

    pub(crate) fn r#type(&self) -> &MavType {
        &self.r#type
    }

    pub(crate) fn enum_type(&self) -> &MavType {
        &self.enum_type
    }

    pub(crate) fn is_enum(&self) -> bool {
        self.is_enum
    }

    pub(crate) fn is_bitmask(&self) -> bool {
        self.is_bitmask
    }

    pub(crate) fn enum_name(&self) -> &str {
        self.enum_name.as_str()
    }

    pub(crate) fn is_array(&self) -> bool {
        self.is_array
    }

    pub(crate) fn array_length(&self) -> usize {
        self.array_length
    }

    pub(crate) fn cast_enum(&self) -> bool {
        self.cast_enum
    }

    pub(crate) fn requires_serde_arrays(&self) -> bool {
        self.requires_serde_arrays
    }
}

/// Input for modules containing references to already implemented messages.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct MessageInheritedModuleSpec<'a> {
    dialect_name: &'a str,
    message_name: &'a str,
    is_v1_compatible: bool,
    params: &'a GeneratorParams,
}

impl<'a> Spec for MessageInheritedModuleSpec<'a> {
    fn params(&self) -> &GeneratorParams {
        self.params
    }
}

impl<'a> MessageInheritedModuleSpec<'a> {
    pub fn new(dialect_name: &'a str, message: &'a Message, params: &'a GeneratorParams) -> Self {
        Self {
            dialect_name,
            message_name: message.name(),
            is_v1_compatible: message.is_v1_compatible(),
            params,
        }
    }

    pub(crate) fn dialect_name(&self) -> &str {
        self.dialect_name
    }

    pub(crate) fn message_name(&self) -> &str {
        self.message_name
    }

    pub(crate) fn is_v1_compatible(&self) -> bool {
        self.is_v1_compatible
    }
}
