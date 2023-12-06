pub mod deprecated {
    use crate::parser::errors::XmlParseError;
    use crate::protocol::{Deprecated, DeprecatedSince};

    /// Low-level representation of MAVLink enum deprecation.
    ///
    /// Can be converted to `crate::protocol::DeprecatedEnum`.
    #[derive(Debug, Clone)]
    pub struct XmlDeprecated {
        pub since: String,
        pub replaced_by: String,
    }

    impl XmlDeprecated {
        /// Parses MAVLink enum deprecation XML definition to `crate::protocol::DeprecatedEnum`
        pub fn to_deprecated(&self) -> Result<Deprecated, XmlParseError> {
            let mut since_parts = self.since.split('-');
            let year: i32 = since_parts
                .next()
                .ok_or(XmlParseError::DeprecatedSinceDateIncorrectFormat(
                    self.since.clone(),
                ))?
                .parse()
                .map_err(XmlParseError::DeprecatedSinceIncorrectYear)?;
            let month: u8 = since_parts
                .next()
                .ok_or(XmlParseError::DeprecatedSinceDateIncorrectFormat(
                    self.since.clone(),
                ))?
                .parse()
                .map_err(XmlParseError::DeprecatedSinceIncorrectMonth)?;

            Ok(Deprecated::new(
                DeprecatedSince::new(year, month),
                self.replaced_by.to_string(),
            ))
        }
    }

    #[test]
    fn correct_deprecated_since_date_parsing() {
        let deprecated = XmlDeprecated {
            since: "2023-10".to_string(),
            replaced_by: "something".to_string(),
        }
        .to_deprecated()
        .unwrap();

        assert_eq!(deprecated.since().year(), 2023);
        assert_eq!(deprecated.since().month(), 10);
    }
}

/// Contains low-level representations of MAVLink XML definitions for enums.
///
/// Each representation has methods to parse to `crate::protocol` high level representations.
pub mod enums {
    use std::collections::HashMap;

    use crate::parser::errors::XmlParseError;
    use crate::protocol::traits::Builder;
    use crate::protocol::{
        Enum, EnumEntry, EnumEntryMavCmdFlags, EnumEntryMavCmdParam, MavType, Units, Value,
    };

    use super::deprecated::XmlDeprecated;

    /// Low-level representation of MAVLink enum entry.
    ///
    /// Can be converted to `crate::protocol::EnumEntry`.
    #[derive(Debug, Clone)]
    pub struct XmlEnumEntry {
        pub value: String,
        pub name: String,
        pub description: String,
        pub cmd_flags: XmlEnumEntryMavCmdFlags,
        pub params: Vec<XmlEnumEntryMavCmdParam>,
        pub deprecated: Option<XmlDeprecated>,
        pub wip: bool,
        pub defined_in: Option<String>,
    }

    impl XmlEnumEntry {
        /// Parses MAVLink enum entry XML definition to `crate::protocol::EnumEntry`
        pub fn to_enum_entry(&self) -> Result<EnumEntry, XmlParseError> {
            let mut params = Vec::new();

            for (i, param) in self.params.iter().enumerate() {
                match i {
                    0 => params.push(param.to_enum_entry_mav_cmd_param(&MavType::Float)?),
                    1 => params.push(param.to_enum_entry_mav_cmd_param(&MavType::Float)?),
                    2 => params.push(param.to_enum_entry_mav_cmd_param(&MavType::Float)?),
                    3 => params.push(param.to_enum_entry_mav_cmd_param(&MavType::Float)?),
                    4 => params.push(param.to_enum_entry_mav_cmd_param(&MavType::Int32)?),
                    5 => params.push(param.to_enum_entry_mav_cmd_param(&MavType::Int32)?),
                    6 => params.push(param.to_enum_entry_mav_cmd_param(&MavType::Float)?),
                    _ => {}
                }
            }

            let value = self
                .value
                .parse::<u32>()
                .map_err(XmlParseError::EnumEntryValueParseFailed)?;
            let flags = self.cmd_flags.to_enum_entry_mav_cmd_flags()?;
            let deprecated = match &self.deprecated {
                Some(deprecated) => Some(deprecated.to_deprecated()?),
                None => None,
            };

            Ok(EnumEntry::builder()
                .set_value(value)
                .set_name(self.name.clone())
                .set_description(self.description.clone())
                .set_cmd_flags(flags)
                .set_params(params)
                .set_wip(self.wip)
                .set_deprecated(deprecated)
                .set_defined_in(self.defined_in.clone())
                .build())
        }
    }

    /// Low-level representation of MAVLink enum.
    ///
    /// Can be converted to `crate::protocol::Enum`.
    #[derive(Debug, Clone)]
    pub struct XmlEnum {
        pub name: String,
        pub bitmask: bool,
        pub description: String,
        pub entries: HashMap<String, XmlEnumEntry>,
        pub deprecated: Option<XmlDeprecated>,
        pub defined_in: Vec<String>,
    }

    impl XmlEnum {
        /// Parses MAVLink enum XML definition to `crate::protocol::Entry`
        pub fn to_enum(&self) -> Result<Enum, XmlParseError> {
            let mut entries = HashMap::new();
            for (name, entry) in &self.entries {
                entries.insert(name.clone(), entry.to_enum_entry()?);
            }

            let deprecated = match &self.deprecated {
                None => None,
                Some(deprecated) => Some(deprecated.to_deprecated()?),
            };

            Ok(Enum::builder()
                .set_name(self.name.clone())
                .set_description(self.description.clone())
                .set_entries(entries)
                .set_bitmask(self.bitmask)
                .set_deprecated(deprecated)
                .set_defined_in(self.defined_in.clone())
                .build())
        }
    }

    /// Low-level representation of MAVLink enum entry `MAV_CMD` flags.
    ///
    /// Can be converted to `crate::protocol::EnumEntryMavCmdFlags`.
    #[derive(Debug, Clone)]
    pub struct XmlEnumEntryMavCmdFlags {
        pub has_location: Option<bool>,
        pub is_destination: Option<bool>,
        pub mission_only: Option<bool>,
    }

    impl XmlEnumEntryMavCmdFlags {
        pub fn to_enum_entry_mav_cmd_flags(
            &self,
        ) -> Result<Option<EnumEntryMavCmdFlags>, XmlParseError> {
            let has_location = match &self.has_location {
                Some(true) => Some(true),
                _ => None,
            };
            let is_destination = match &self.is_destination {
                Some(true) => Some(true),
                _ => None,
            };
            let mission_only = match &self.mission_only {
                Some(true) => Some(true),
                _ => None,
            };

            Ok(match (has_location, is_destination, mission_only) {
                (None, None, None) => None,
                _ => Some(
                    EnumEntryMavCmdFlags::builder()
                        .set_has_location(has_location)
                        .set_is_destination(is_destination)
                        .set_mission_only(mission_only)
                        .build(),
                ),
            })
        }
    }

    /// Low-level representation of MAVLink enum entry `MAV_CMD` parameter.
    ///
    /// Can be converted to `crate::protocol::EnumEntryMavCmdParam`.
    #[derive(Debug, Clone)]
    pub struct XmlEnumEntryMavCmdParam {
        pub index: String,
        pub description: String,
        pub label: Option<String>,
        pub units: Option<String>,
        pub r#enum: Option<String>,
        pub decimal_places: Option<String>,
        pub increment: Option<String>,
        pub min_value: Option<String>,
        pub max_value: Option<String>,
        pub reserved: Option<String>,
        pub default: Option<String>,
    }

    impl XmlEnumEntryMavCmdParam {
        pub fn to_enum_entry_mav_cmd_param(
            &self,
            r#type: &MavType,
        ) -> Result<EnumEntryMavCmdParam, XmlParseError> {
            let index = self
                .index
                .parse::<u8>()
                .map_err(XmlParseError::EnumEntryMavCmdParamInvalidIndex)?;
            let units = match &self.units {
                None => None,
                Some(units) => Some(
                    units
                        .parse::<Units>()
                        .map_err(XmlParseError::IncorrectUnits)?,
                ),
            };
            let decimal_places = match &self.decimal_places {
                None => None,
                Some(dec) => Some(
                    dec.parse::<u8>()
                        .map_err(XmlParseError::EnumEntryMavCmdParamInvalidDecimalPlaces)?,
                ),
            };
            let increment = match &self.increment {
                None => None,
                Some(inc) => Some(
                    Value::parse(inc, r#type)
                        .map_err(XmlParseError::EnumEntryMavCmdParamInvalidIncrement)?,
                ),
            };
            let min_value = match &self.min_value {
                None => None,
                Some(min) => Some(
                    Value::parse(min, r#type)
                        .map_err(XmlParseError::EnumEntryMavCmdParamInvalidMinValue)?,
                ),
            };
            let max_value = match &self.max_value {
                None => None,
                Some(max) => Some(
                    Value::parse(max, r#type)
                        .map_err(XmlParseError::EnumEntryMavCmdParamInvalidMaxValue)?,
                ),
            };
            let reserved = match &self.reserved {
                None => false,
                Some(reserved) => reserved
                    .parse::<bool>()
                    .map_err(XmlParseError::EnumEntryMavCmdParamInvalidReserved)?,
            };
            let default = match &self.default {
                None => None,
                Some(def) => Some(
                    Value::parse(def, r#type)
                        .map_err(XmlParseError::EnumEntryMavCmdParamInvalidDefaultValue)?,
                ),
            };

            Ok(EnumEntryMavCmdParam::builder()
                .set_index(index)
                .set_description(self.description.clone())
                .set_label(self.label.clone())
                .set_units(units)
                .set_enum(self.r#enum.clone())
                .set_decimal_places(decimal_places)
                .set_increment(increment)
                .set_min_value(min_value)
                .set_max_value(max_value)
                .set_reserved(reserved)
                .set_default(default)
                .build())
        }
    }
}

pub mod messages {
    use super::deprecated::XmlDeprecated;
    use crate::parser::errors::XmlParseError;
    use crate::protocol::traits::Builder;
    use crate::protocol::{
        MavType, Message, MessageField, MessageFieldInvalidValue, MessageId, Units, Value,
    };

    #[derive(Debug, Clone)]
    pub struct XmlMessageField {
        pub field_type: String,
        pub name: String,
        pub description: String,
        pub r#enum: Option<String>,
        pub units: Option<String>,
        pub bitmask: bool,
        pub print_format: Option<String>,
        pub default: Option<String>,
        pub invalid: Option<String>,
        pub instance: bool,
        pub extension: bool,
    }

    impl XmlMessageField {
        pub fn to_message_field(&self) -> Result<MessageField, XmlParseError> {
            let r#type = self
                .field_type
                .parse::<MavType>()
                .map_err(XmlParseError::IncorrectMessageFieldType)?;
            let units = match &self.units {
                None => None,
                Some(units) => Some(
                    units
                        .parse::<Units>()
                        .map_err(XmlParseError::IncorrectUnits)?,
                ),
            };
            let default = match &self.default {
                None => None,
                Some(default) => Some(
                    Value::parse(default, &r#type)
                        .map_err(XmlParseError::IncorrectMessageFieldDefaultValue)?,
                ),
            };
            let invalid = match self.invalid.clone() {
                Some(invalid) => Some(
                    MessageFieldInvalidValue::parse(invalid.as_str(), &r#type)
                        .map_err(XmlParseError::IncorrectInvalidValue)?,
                ),
                None => None,
            };

            Ok(MessageField::builder()
                .set_name(self.name.clone())
                .set_description(self.description.clone())
                .set_type(r#type)
                .set_enum(self.r#enum.clone())
                .set_units(units)
                .set_bitmask(self.bitmask)
                .set_print_format(self.print_format.clone())
                .set_default(default)
                .set_invalid(invalid)
                .set_instance(self.instance)
                .set_extension(self.extension)
                .build())
        }
    }

    #[derive(Debug, Clone)]
    pub struct XmlMessage {
        pub id: String,
        pub name: String,
        pub description: String,
        pub fields: Vec<XmlMessageField>,
        pub wip: bool,
        pub deprecated: Option<XmlDeprecated>,
        pub defined_in: Option<String>,
    }

    impl XmlMessage {
        pub fn to_message(&self) -> Result<Message, XmlParseError> {
            let message_id = self
                .id
                .parse::<MessageId>()
                .map_err(XmlParseError::IncorrectMessageId)?;

            let mut fields: Vec<MessageField> = Vec::new();

            for xml_fld in &self.fields {
                fields.push(xml_fld.to_message_field()?);
            }

            let deprecated = match &self.deprecated {
                Some(deprecated) => Some(deprecated.to_deprecated()?),
                None => None,
            };

            Ok(Message::builder()
                .set_id(message_id)
                .set_name(self.name.clone())
                .set_description(self.description.clone())
                .set_fields(fields)
                .set_wip(self.wip)
                .set_deprecated(deprecated)
                .set_defined_in(self.defined_in.clone())
                .build())
        }
    }
}
