use crate::rust::RustGeneratorParams;
use mavspec::protocol::{Enum, EnumEntry, MavType};
use serde::Serialize;

/// Enums module root template.
///
/// Input: [`mavspec::protocol::Dialect`].
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
/// Basically, this is a utility wrapper around `MAVSpec` [`Enum`].
#[derive(Debug, Clone, Serialize)]
pub struct EnumSpec {
    name: String,
    inferred_type: MavType,
    entries: Vec<EnumEntry>,
    is_bitmask: bool,
    params: RustGeneratorParams,
}

impl EnumSpec {
    pub fn new(mav_enum: &Enum, params: &RustGeneratorParams) -> EnumSpec {
        let mut entries: Vec<EnumEntry> = mav_enum.entries().values().cloned().collect();
        entries.sort_by_key(|entry| entry.value());

        EnumSpec {
            name: mav_enum.name().into(),
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
use mavlib_spec::bitflags::bitflags;

bitflags! {
    /// MAVLink bitmask `{{name}}`.
    #[derive(Copy, Clone, Debug, Default)]
{{#if params.serde}}#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]{{/if}}
    pub struct {{to-enum-rust-name name}}: {{to-rust-type inferred_type}} {
{{#each entries}}
        /// `{{name}}` flag.
        const {{name}} = {{value}};
{{/each}}
    }
}


{{else}}
use mavlib_spec::MessageError;

/// MAVLink enum `{{name}}`.
#[derive(Copy, Clone, Debug, Default)]
#[repr({{to-rust-type inferred_type}})]
{{#if params.serde}}#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]{{/if}}
pub enum {{to-enum-rust-name name}} {
    #[default]
{{#each entries}}
    /// MAVLink enum entry `{{name}}`.
    {{to-enum-entry-name name}} = {{value}},
{{/each}}
}

impl TryFrom<{{to-rust-type inferred_type}}> for {{to-enum-rust-name name}} {
    type Error = MessageError;

    fn try_from(value: {{to-rust-type inferred_type}}) -> Result<Self, Self::Error> {
        Self::try_from_discriminant(value)
    }
}

impl {{to-enum-rust-name name}} {
    /// Attempts to create [`{{to-enum-rust-name name}}`] variant from discriminant (raw value).
    ///
    /// # Errors
    ///
    /// * Returns [`MessageError::InvalidEnumValue`] if there is no enum varian corresponding to discriminant `value`.
    pub fn try_from_discriminant(value: {{to-rust-type inferred_type}}) -> Result<Self, MessageError> {
        Ok(match value {
{{#each entries}}
            {{value}} => Self::{{to-enum-entry-name name}},
{{/each}}
            _ => {
                return Err(MessageError::InvalidEnumValue {
                    enum_name: "{{name}}".into(),
                    value: value.into(),
                })
            }
        })
    }
}

{{/if}}
"#;
