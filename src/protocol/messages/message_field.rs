#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::protocol::{MavType, Units, Value};
use crate::utils::{Buildable, Builder};

use super::message_field_invalid_value::MessageFieldInvalidValue;

/// MAVLink message field.
///
/// Represents field in MAVLink [`Message`](crate::protocol::Message).
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MessageField {
    name: String,
    description: String,
    r#type: MavType,
    r#enum: Option<String>,
    units: Option<Units>,
    bitmask: bool,
    print_format: Option<String>,
    default: Option<Value>,
    invalid: Option<MessageFieldInvalidValue>,
    instance: bool,
    extension: bool,
}

impl Buildable for MessageField {
    type Builder = MessageFieldBuilder;

    /// Creates [`MessageFieldBuilder`] initialised with current message field.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mavspec::protocol::{MessageField, MessageFieldBuilder};
    /// use mavspec::utils::{Buildable, Builder};
    ///
    /// let original = MessageField::builder()
    ///     .set_name("original".to_string())
    ///     .set_description("original".to_string())
    ///     .build();
    ///
    /// let updated = original.to_builder()
    ///     .set_description("updated".to_string())
    ///     .build();
    ///
    /// assert_eq!(updated.name(), "original");
    /// assert_eq!(updated.description(), "updated");
    /// ```
    fn to_builder(&self) -> MessageFieldBuilder {
        MessageFieldBuilder {
            field: self.clone(),
        }
    }
}

impl MessageField {
    /// Initiates builder.
    ///
    /// Instead of constructor we use
    /// [builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
    /// pattern. An instance of [`MessageFieldBuilder`] returned by this function is initialized
    /// with default values. Once desired values are set, you can call [`MessageFieldBuilder::build`] to
    /// obtain [`MessageField`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mavspec::protocol::MessageField;
    /// use mavspec::utils::Builder;
    ///
    /// let field = MessageField::builder()
    ///     .set_name("name".to_string())
    ///     .set_description("description".to_string())
    ///     .build();
    ///
    /// assert!(matches!(field, MessageField { .. }));
    /// assert_eq!(field.name(), "name");
    /// assert_eq!(field.description(), "description");
    /// ```
    pub fn builder() -> MessageFieldBuilder {
        MessageFieldBuilder::new()
    }

    /// Message field name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Message field description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Message field type.
    pub fn r#type(&self) -> &MavType {
        &self.r#type
    }

    /// Enum name which defines message values.
    pub fn r#enum(&self) -> Option<&str> {
        match &self.r#enum {
            None => None,
            Some(enm) => Some(enm.as_str()),
        }
    }

    /// Message field units.
    pub fn units(&self) -> Option<&Units> {
        match &self.units {
            None => None,
            Some(units) => Some(units),
        }
    }

    /// Set to `true` for bitmask fields, default `false`.
    pub fn bitmask(&self) -> bool {
        self.bitmask
    }

    /// Print format.
    ///
    /// C `printf`-like format (i.e. `0x%04x`).
    pub fn print_format(&self) -> Option<&str> {
        self.print_format.as_deref()
    }

    /// Specification for default value.
    pub fn default(&self) -> Option<&Value> {
        self.default.as_ref()
    }

    /// Defines how invalid values should be specified.
    ///
    /// Specifies a value that can be set on a field to indicate that the data is invalid: the
    /// recipient should ignore the field if it has this value. For example,
    /// `BATTERY_STATUS.current_battery` specifies `invalid="-1"`, so a battery that does not
    /// measure supplied current should set `BATTERY_STATUS.current_battery` to `-1`.
    pub fn invalid(&self) -> Option<&MessageFieldInvalidValue> {
        self.invalid.as_ref()
    }

    /// Instance flag.
    ///
    /// If `true`, this indicates that the message contains the information for a particular sensor
    /// or battery (e.g. Battery 1, Battery 2, etc.) and that this field indicates which sensor.
    /// Default is `false`.
    pub fn instance(&self) -> bool {
        self.instance
    }

    /// Whether this message field is extension or not.
    pub fn extension(&self) -> bool {
        self.extension
    }
}

/// Builder for [`MessageField`].
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MessageFieldBuilder {
    field: MessageField,
}

impl Builder for MessageFieldBuilder {
    type Buildable = MessageField;

    /// Creates [`MessageField`] from builder.
    fn build(&self) -> MessageField {
        // We need this to get error when `MessageField` changes
        #[allow(clippy::match_single_binding)]
        match self.field.clone() {
            MessageField {
                name,
                description,
                r#type,
                r#enum,
                units,
                bitmask,
                print_format,
                default,
                invalid,
                instance,
                extension,
            } => MessageField {
                name,
                description,
                r#type,
                r#enum,
                units,
                bitmask,
                print_format,
                default,
                invalid,
                instance,
                extension,
            },
        }
    }
}

impl MessageFieldBuilder {
    /// Creates builder instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets message field name.
    ///
    /// See: [`MessageField::name`].
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.field.name = name;
        self
    }

    /// Sets message field description.
    ///
    /// See: [`MessageField::description`].
    pub fn set_description(&mut self, description: String) -> &mut Self {
        self.field.description = description;
        self
    }

    /// Sets message field type.
    ///
    /// See: [`MessageField::type`].
    pub fn set_type(&mut self, r#type: MavType) -> &mut Self {
        self.field.r#type = r#type;
        self
    }

    /// Sets enum name which defines message values.
    ///
    /// See: [`MessageField::enum`].
    pub fn set_enum(&mut self, r#enum: Option<String>) -> &mut Self {
        self.field.r#enum = r#enum;
        self
    }

    /// Sets message field units.
    ///
    /// See: [`MessageField::units`].
    pub fn set_units(&mut self, units: Option<Units>) -> &mut Self {
        self.field.units = units;
        self
    }

    /// Sets message field units.
    ///
    /// See: [`MessageField::bitmask`].
    pub fn set_bitmask(&mut self, bitmask: bool) -> &mut Self {
        self.field.bitmask = bitmask;
        self
    }

    /// Sets print format.
    ///
    /// See [`MessageField::print_format`].
    pub fn set_print_format(&mut self, print_format: Option<String>) -> &mut Self {
        self.field.print_format = print_format;
        self
    }

    /// Specification for default value.
    ///
    /// See: [`MessageField::default`].
    pub fn set_default(&mut self, default: Option<Value>) -> &mut Self {
        self.field.default = default;
        self
    }

    /// Sets specification for invalid field value (if applicable).
    ///
    /// See: [`MessageField::invalid`].
    pub fn set_invalid(&mut self, invalid: Option<MessageFieldInvalidValue>) -> &mut Self {
        self.field.invalid = invalid;
        self
    }

    /// Sets instance flag.
    ///
    /// See: [`MessageField::instance`].
    pub fn set_instance(&mut self, instance: bool) -> &mut Self {
        self.field.instance = instance;
        self
    }

    /// Sets whether this message field is extension or not
    pub fn set_extension(&mut self, extension: bool) -> &mut Self {
        self.field.extension = extension;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_field_builder() {
        let field = MessageFieldBuilder::new()
            .set_name("name".to_string())
            .set_description("description".to_string())
            .set_type(MavType::Float)
            .set_enum(Some("MAV_CMD".to_string()))
            .set_units(Some(Units::Ampere))
            .set_bitmask(true)
            .set_print_format(Some("format".to_string()))
            .set_default(Some(Value::UInt16(u16::MAX)))
            .set_invalid(Some(MessageFieldInvalidValue::Value(Value::Max(
                MavType::Float,
            ))))
            .set_instance(true)
            .set_extension(true)
            .build();

        assert!(matches!(field, MessageField { .. }));
        assert_eq!(field.name(), "name");
        assert_eq!(field.description(), "description");
        assert!(matches!(field.r#type(), MavType::Float));
        assert_eq!(field.r#enum.clone().unwrap(), "MAV_CMD");
        assert!(matches!(field.units(), Some(Units::Ampere)));
        assert!(field.bitmask());
        assert_eq!(field.print_format.clone().unwrap(), "format");
        assert!(matches!(field.default, Some(Value::UInt16(u16::MAX))));
        assert!(matches!(
            field.invalid(),
            Some(MessageFieldInvalidValue::Value(Value::Max(MavType::Float,)))
        ));
        assert!(field.instance());
        assert!(field.extension());
    }
}
