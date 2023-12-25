use mavinspect::protocol::{Enum, EnumEntry, MavType};
use serde::Serialize;

use crate::conventions::split_description;
use crate::generator::GeneratorParams;

/// Enums module root template.
///
/// Input: [`mavinspect::protocol::Dialect`].
pub const ENUMS_MODULE_ROOT: &str = "\
//! MAVLink enums of `{{name}}` dialect.

{{#each enums}}
pub mod {{to-enum-mod-name name}};
pub use {{to-enum-mod-name name}}::{{to-enum-rust-name name}};
{{/each}}

/// MAVLink enums of `{{name}}` dialect. 
pub enum Enums {
{{#each enums}}
    /// Mavlink enum `{{name}}`.
    {{to-enum-rust-name name}}({{to-enum-rust-name name}}),
{{/each}}
}
";

/// Input for [`ENUM`] template.
///
/// Basically, this is a utility wrapper around `MAVInspect` [`Enum`].
#[derive(Debug, Clone, Serialize)]
pub struct EnumSpec {
    name: String,
    description: Vec<String>,
    inferred_type: MavType,
    entries: Vec<EnumEntrySpec>,
    is_bitmask: bool,
    params: GeneratorParams,
}

/// Enum entry representation for template.
///
/// Basically, this is a utility wrapper around `MAVInspect` [`EnumEntry`].
#[derive(Debug, Clone, Serialize)]
pub struct EnumEntrySpec {
    value: u32,
    name: String,
    name_stripped: String,
    description: Vec<String>,
}

impl EnumEntrySpec {
    pub fn from_enum_entry(entry: &EnumEntry) -> Self {
        Self {
            value: entry.value(),
            name: entry.name().to_string(),
            name_stripped: entry.name_stripped().to_string(),
            description: split_description(entry.description()),
        }
    }
}

impl EnumSpec {
    pub fn new(mav_enum: &Enum, params: &GeneratorParams) -> EnumSpec {
        let mut entries: Vec<EnumEntrySpec> = mav_enum
            .entries()
            .values()
            .map(EnumEntrySpec::from_enum_entry)
            .collect();
        entries.sort_by_key(|entry| entry.value);

        EnumSpec {
            name: mav_enum.name().into(),
            description: split_description(mav_enum.description()),
            inferred_type: mav_enum.inferred_type(),
            entries,
            is_bitmask: mav_enum.bitmask(),
            params: params.clone(),
        }
    }
}

/// Enums template.
///
/// Input: [`EnumSpec`].
pub const ENUM: &str = r#"//! MAVLink `{{name}}` enum implementation.

{{#if is_bitmask}}
use mavspec::rust::spec::bitflags::bitflags;

bitflags! {
    #[allow(rustdoc::bare_urls)]
    #[allow(rustdoc::broken_intra_doc_links)]
    /// MAVLink bitmask `{{name}}`.
    ///
    {{#each description}}
    /// {{this}}
    {{/each}}
    #[derive(Copy, Clone, Debug, Default)]
{{#if params.serde}}
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
{{/if}}
    pub struct {{to-enum-rust-name name}}: {{to-rust-type inferred_type}} {
{{#each entries}}
        /// `{{name}}` flag.
        ///
{{#each description}}
        /// {{this}}
{{/each}}
        const {{to-enum-bitmap-entry-name name_stripped}} = {{value}};
{{/each}}
    }
}
{{else}}
use mavspec::rust::spec::MessageError;

#[cfg(not(doctest))]
#[allow(rustdoc::bare_urls)]
#[allow(rustdoc::broken_intra_doc_links)]
/// MAVLink enum `{{name}}`.
///
{{#each description}}
/// {{this}}
{{/each}}
#[derive(Copy, Clone, Debug, Default)]
#[repr({{to-rust-type inferred_type}})]
{{#if params.serde}}#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]{{/if}}
pub enum {{to-enum-rust-name name}} {
    #[default]
{{#each entries}}
    /// MAVLink enum entry `{{name}}`.
    ///
{{#each description}}
    /// {{this}}
{{/each}}
    {{to-enum-entry-name name_stripped}} = {{value}},
{{/each}}
}

impl TryFrom<{{to-rust-type inferred_type}}> for {{to-enum-rust-name name}} {
    type Error = MessageError;

    fn try_from(value: {{to-rust-type inferred_type}}) -> Result<Self, MessageError> {
        Self::try_from_discriminant(value)
    }
}

impl {{to-enum-rust-name name}} {
    /// Attempts to create [`{{to-enum-rust-name name}}`] variant from discriminant (raw value).
    ///
    /// # Errors
    ///
    /// * Returns [`MessageError::InvalidEnumValue`] if there is no enum variant corresponding to discriminant `value`.
    pub fn try_from_discriminant(value: {{to-rust-type inferred_type}}) -> Result<Self, MessageError> {
        Ok(match value {
{{#each entries}}
            {{value}} => Self::{{to-enum-entry-name name_stripped}},
{{/each}}
            _ => {
                return Err(MessageError::InvalidEnumValue {
                    enum_name: "{{name}}",
                    value: value.into(),
                })
            }
        })
    }
}
{{/if}}
"#;
