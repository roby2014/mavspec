use mavinspect::protocol::{Enum, EnumEntry, MavType};
use serde::Serialize;
use std::collections::HashMap;

use crate::conventions::split_description;
use crate::generator::GeneratorParams;
use crate::specs::Spec;

/// Input for enums root module template.
pub(crate) struct EnumsRootModuleSpec<'a> {
    dialect_name: &'a str,
    enums: &'a HashMap<String, Enum>,
    params: &'a GeneratorParams,
}

impl<'a> Spec for EnumsRootModuleSpec<'a> {
    fn params(&self) -> &GeneratorParams {
        self.params
    }
}

impl<'a> EnumsRootModuleSpec<'a> {
    pub(crate) fn new(
        dialect_name: &'a str,
        enums: &'a HashMap<String, Enum>,
        params: &'a GeneratorParams,
    ) -> Self {
        Self {
            dialect_name,
            enums,
            params,
        }
    }

    pub(crate) fn dialect_name(&self) -> &str {
        self.dialect_name
    }

    pub(crate) fn enums(&self) -> &HashMap<String, Enum> {
        self.enums
    }
}

/// Input for MAVLink enum implementation module template.
///
/// Basically, this is a utility wrapper around `MAVInspect` [`Enum`].
#[derive(Debug, Clone, Serialize)]
pub(crate) struct EnumImplModuleSpec<'a> {
    name: &'a str,
    description: Vec<String>,
    inferred_type: MavType,
    entries: Vec<EnumEntrySpec<'a>>,
    is_bitmask: bool,
    params: &'a GeneratorParams,
}

impl<'a> Spec for EnumImplModuleSpec<'a> {
    fn params(&self) -> &GeneratorParams {
        self.params
    }
}

impl<'a> EnumImplModuleSpec<'a> {
    pub(crate) fn new(mav_enum: &'a Enum, params: &'a GeneratorParams) -> Self {
        let mut entries: Vec<EnumEntrySpec> = mav_enum
            .entries()
            .values()
            .map(EnumEntrySpec::from_enum_entry)
            .collect();
        entries.sort_by_key(|entry| entry.value);

        EnumImplModuleSpec {
            name: mav_enum.name(),
            description: split_description(mav_enum.description()),
            inferred_type: mav_enum.inferred_type(),
            entries,
            is_bitmask: mav_enum.bitmask(),
            params,
        }
    }

    pub(crate) fn name(&self) -> &str {
        self.name
    }

    pub(crate) fn is_bitmask(&self) -> bool {
        self.is_bitmask
    }

    pub(crate) fn description(&self) -> &[String] {
        self.description.as_slice()
    }

    pub(crate) fn entries(&self) -> &[EnumEntrySpec] {
        self.entries.as_slice()
    }

    pub(crate) fn inferred_type(&self) -> &MavType {
        &self.inferred_type
    }
}

/// Enum entry representation for template.
///
/// Basically, this is a utility wrapper around `MAVInspect` [`EnumEntry`].
#[derive(Debug, Clone, Serialize)]
pub(crate) struct EnumEntrySpec<'a> {
    value: u32,
    name: &'a str,
    name_stripped: String,
    description: Vec<String>,
}

impl<'a> EnumEntrySpec<'a> {
    pub(crate) fn from_enum_entry(entry: &'a EnumEntry) -> Self {
        Self {
            value: entry.value(),
            name: entry.name(),
            name_stripped: entry.name_stripped().to_string(),
            description: split_description(entry.description()),
        }
    }

    pub(crate) fn value(&self) -> u32 {
        self.value
    }

    pub(crate) fn value_expr(&self) -> syn::Expr {
        syn::parse_str(format!("{}", self.value()).as_str()).unwrap()
    }

    pub(crate) fn name(&self) -> &str {
        self.name
    }

    pub(crate) fn name_stripped(&self) -> &str {
        self.name_stripped.as_str()
    }

    pub(crate) fn description(&self) -> &[String] {
        self.description.as_slice()
    }
}
