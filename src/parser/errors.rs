use std::io::Error;
use std::num::ParseIntError;
use std::str::ParseBoolError;

use thiserror::Error;

use crate::protocol::errors::TypeParseError;
use crate::protocol::errors::{InvalidValueParseError, ParseError, ValueParseError};

/// Errors which may happen during XML definitions discovery
#[derive(Debug, Error)]
pub enum XmlInspectionError {
    /// Two dialect XML definitions with the same canonical names.
    ///
    /// See [`crate::parser::XmlDialectDefinition::canonize_name`].
    #[error("dialects `{first}`, `{second}` have the same canonical name: `{canonical}`")]
    NamingCollision {
        /// First dialect name.
        first: String,
        /// Second dialect name.
        second: String,
        /// Colliding canonical name.
        canonical: String,
    },
    /// IO error.
    #[error("IO error: {0}")]
    IoError(std::io::Error),
}

impl From<std::io::Error> for XmlInspectionError {
    /// Convert [`std::io::Error`] into [`XmlInspectionError::IoError`].
    fn from(value: Error) -> Self {
        Self::IoError(value)
    }
}

/// Errors which may happen during XML parsing
#[derive(Debug, Clone, Error)]
pub enum XmlParseError {
    /// XML tags appears in invalid order.
    #[error("incorrect tags order: tag `{inner}` appears inside tag {outer:?}")]
    TagsNotInOrder {
        /// Outer tag.
        outer: Option<String>,
        /// Inner tag.
        inner: String,
    },
    /// Trying to close tag which was not immediately opened.
    #[error("incorrect closing tag: opened with `{opened_with:?}` but closed with {closed_with}")]
    UnexpectedClosingTag {
        /// Opening tag.
        opened_with: Option<String>,
        /// Closing tag.
        closed_with: String,
    },

    /// Impossible to parse dialect version.
    #[error("impossible to parse dialect version: {0:?}")]
    VersionParseFailed(ParseIntError),
    /// Impossible to parse dialect ID.
    #[error("impossible to parse dialect ID: {0:?}")]
    DialectIdParseFailed(ParseIntError),

    /// Found enum entry `<entry>` outside of enum definition `<enum>`.
    #[error("found `<entry>` tag outside of `<enum>` tag")]
    EnumEntryWithoutEnum,
    /// Impossible to parse enum entry value.
    #[error("impossible to parse enum entry value: {0:?}")]
    EnumEntryValueParseFailed(ParseIntError),

    /// Found `<param>` tag outside of `<enum>` tag.
    #[error("found `<param>` tag outside of `<enum>` tag")]
    MavCmdParamWithoutEnumEntry,
    /// Invalid `hasLocation` field value in enum entry.
    #[error("invalid `hasLocation` field value: {0:?}")]
    EnumEntryMavCmdFlagsInvalidHasLocation(ParseBoolError),
    /// Invalid `isDestination` field value in enum entry.
    #[error("invalid `isDestination` field value in enum entry: {0:?}")]
    EnumEntryMavCmdFlagsInvalidIsDestination(ParseBoolError),
    /// Invalid `missionOnly` field value in enum entry.
    #[error("invalid `missionOnly` field value in enum entry: {0:?}")]
    EnumEntryMavCmdFlagsInvalidMissionOnly(ParseBoolError),
    /// Invalid `index` field value in enum entry param.
    #[error("invalid `index` field value in enum entry param: {0:?}")]
    EnumEntryMavCmdParamInvalidIndex(ParseIntError),
    /// Invalid `decimalPlaces` field value in enum entry param.
    #[error("invalid `decimalPlaces` field value in enum entry param: {0:?}")]
    EnumEntryMavCmdParamInvalidDecimalPlaces(ParseIntError),
    /// Invalid `increment` field value in enum entry param.
    #[error("invalid `increment` field value in enum entry param: {0:?}")]
    EnumEntryMavCmdParamInvalidIncrement(ValueParseError),
    /// Invalid `minValue` field value in enum entry param.
    #[error("invalid `minValue` field value in enum entry param: {0:?}")]
    EnumEntryMavCmdParamInvalidMinValue(ValueParseError),
    /// Invalid `maxValue` field value in enum entry param.
    #[error("invalid `maxValue` field value in enum entry param: {0:?}")]
    EnumEntryMavCmdParamInvalidMaxValue(ValueParseError),
    /// Invalid `reserved` field value in enum entry param.
    #[error("invalid `reserved` field value in enum entry param: {0:?}")]
    EnumEntryMavCmdParamInvalidReserved(ParseBoolError),
    /// Invalid `default` field value in enum entry param.
    #[error("invalid `default` field value in enum entry param: {0:?}")]
    EnumEntryMavCmdParamInvalidDefaultValue(ValueParseError),

    /// Invalid message ID.
    #[error("invalid message ID: {0:?}")]
    IncorrectMessageId(ParseIntError),

    /// Impossible to parse message field type
    #[error("invalid message field type: {0:?}")]
    IncorrectMessageFieldType(TypeParseError),
    /// Invalid message field `instance` field.
    #[error("invalid message field `instance` field: {0:?}")]
    IncorrectMessageFieldInstance(ParseBoolError),
    /// Invalid message field `default` field.
    #[error("invalid message field `default` field: {0:?}")]
    IncorrectMessageFieldDefaultValue(ValueParseError),

    /// Invalid deprecated since date format.
    #[error("invalid deprecated since date format: {0}")]
    DeprecatedSinceDateIncorrectFormat(String),
    /// Invalid deprecated since year or month.
    #[error("invalid deprecated since year: {0}")]
    DeprecatedSinceIncorrectYear(ParseIntError),
    /// Invalid deprecated since year or month.
    #[error("invalid deprecated since month: {0}")]
    DeprecatedSinceIncorrectMonth(ParseIntError),

    /// Invalid value
    #[error("invalid value")]
    IncorrectInvalidValue(InvalidValueParseError),

    /// Invalid units
    #[error("invalid units: {0:?}")]
    IncorrectUnits(ParseError),

    /// Found message field `<field>` tag outside of message definition `<message>`.
    #[error("found `<field>` tag outside of `<message>` tag")]
    MessageFieldWithoutMessage,
    /// Found `<extensions>` tag outside of message definition `<message>`.
    #[error("found `<extensions>` tag outside of `<message>` tag")]
    MessageExtensionsWithoutMessage,

    /// Found `<description>` tag outside of any appropriate tag.
    #[error("found `<description>` tag outside of any appropriate tag")]
    DescriptionWithoutSubject,
    /// Found `<deprecated>` tag outside of any appropriate tag.
    #[error("found `<deprecated>` tag outside of any appropriate tag")]
    DeprecationWithoutSubject,
    /// Found `<wip>` tag outside of any appropriate tag.
    #[error("found `<wip>` tag outside of any appropriate tag")]
    WipWithoutSubject,
}
