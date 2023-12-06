#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::protocol::traits::{Buildable, Builder};
use crate::protocol::{Deprecated, MessageField};

/// Type of MAVLink message ID
pub type MessageId = u32;

/// MAVLink message
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Message {
    /// Unique message ID.
    id: MessageId,
    /// Message name (expected to be unique).
    name: String,
    /// Message description.
    description: String,
    /// List of message fields.
    fields: Vec<MessageField>,
    /// Whether this message definition is in work in progress status.
    wip: bool,
    /// Deprecation status.
    deprecated: Option<Deprecated>,
    /// Dialect name where this message was defined.
    defined_in: Option<String>,
}

impl Buildable for Message {
    type Builder = MessageBuilder;

    /// Creates [`MessageBuilder`] initialised with current message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mavspec::protocol::{Message, MessageBuilder};
    /// use mavspec::protocol::traits::{Buildable, Builder};
    ///
    /// let original = Message::builder()
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
    fn to_builder(&self) -> MessageBuilder {
        MessageBuilder {
            message: self.clone(),
        }
    }
}

impl Message {
    /// Initiates builder.
    ///
    /// Instead of constructor we use
    /// [builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
    /// pattern. An instance of [`MessageBuilder`] returned by this function is initialized
    /// with default values. Once desired values are set, you can call [`MessageBuilder::build`] to
    /// obtain [`Message`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mavspec::protocol::Message;
    /// use mavspec::protocol::traits::Builder;
    ///
    /// let message = Message::builder()
    ///     .set_name("name".to_string())
    ///     .set_description("description".to_string())
    ///     .build();
    ///
    /// assert!(matches!(message, Message { .. }));
    /// assert_eq!(message.name(), "name");
    /// assert_eq!(message.description(), "description");
    /// ```
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }

    /// Unique message ID within dialect.
    pub fn id(&self) -> MessageId {
        self.id
    }

    /// Message name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Message description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// List of message fields.
    pub fn fields(&self) -> &[MessageField] {
        &self.fields
    }

    /// Work in progress status.
    pub fn wip(&self) -> bool {
        self.wip
    }

    /// Deprecation status.
    pub fn deprecated(&self) -> Option<&Deprecated> {
        self.deprecated.as_ref()
    }

    /// Dialect name where this message was defined.
    ///
    /// We track dialect in which message was defined to help to optimise code generation.
    pub fn defined_in(&self) -> Option<&String> {
        self.defined_in.as_ref()
    }

    /// Size of the fields payload according to
    /// [MAVLink 2](https://mavlink.io/en/guide/mavlink_2.html) protocol version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mavspec::protocol::{MavType, Message, MessageField};
    /// use mavspec::protocol::traits::Builder;
    ///
    /// let msg = Message::builder()
    ///     .set_fields(vec![
    ///         MessageField::builder()
    ///             .set_type(MavType::Array(Box::new(MavType::Float), 3))  // Must be 12
    ///             .build(),
    ///         MessageField::builder()
    ///             .set_type(MavType::Array(Box::new(MavType::UInt8), 5))  // Must be 5
    ///             .build(),
    ///         MessageField::builder()
    ///             .set_type(MavType::Float)                               // Must be 4
    ///             .build(),
    ///     ])
    ///     .build();
    ///
    /// assert_eq!(msg.size_v2(), 21);
    /// ```
    pub fn size_v2(&self) -> usize {
        self.fields_v2()
            .iter()
            .fold(0, |acc, fld| acc + fld.r#type().size())
    }

    /// Size of the fields payload according to
    /// [MAVLink 2](https://mavlink.io/en/guide/mavlink_2.html) protocol version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mavspec::protocol::{MavType, Message, MessageField};
    /// use mavspec::protocol::traits::Builder;
    ///
    /// let msg = Message::builder()
    ///     .set_fields(vec![
    ///         MessageField::builder()
    ///             .set_type(MavType::Array(Box::new(MavType::Float), 3))  // Must be 12
    ///             .build(),
    ///         MessageField::builder()
    ///             .set_type(MavType::Array(Box::new(MavType::UInt8), 5))  // Must be 5
    ///             .build(),
    ///         MessageField::builder()
    ///             .set_type(MavType::Float)                               // Ignored
    ///             .set_extension(true)
    ///             .build(),
    ///     ])
    ///     .build();
    ///
    /// assert_eq!(msg.size_v1(), 17);
    /// ```
    pub fn size_v1(&self) -> usize {
        self.fields_v1()
            .iter()
            .fold(0, |acc, fld| acc + fld.r#type().size())
    }

    /// Returns index of the first extension field if any.
    ///
    /// See:
    ///  * [MAVLink fields reordering](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///  * MAVLink [extension fields](https://mavlink.io/en/guide/define_xml_element.html#message_extensions).
    ///  * [`MessageField::extension`].
    pub fn extension_fields_idx(&self) -> Option<usize> {
        for (i, field) in self.fields.iter().enumerate() {
            if field.extension() {
                return Some(i);
            }
        }
        None
    }

    /// Returns whether this message has extension fields.
    ///
    /// See:
    ///  * [MAVLink fields reordering](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///  * MAVLink [extension fields](https://mavlink.io/en/guide/define_xml_element.html#message_extensions).
    ///  * [`MessageField::extension`].
    pub fn has_extension_fields(&self) -> bool {
        self.extension_fields_idx().is_some()
    }

    /// Returns fields applicable to [MavLink 1](https://mavlink.io/en/guide/mavlink_version.html)
    /// protocol version.
    ///
    /// Basically, as required by specification, all extension fields are excluded.
    ///
    /// See: [MAVLink message extensions](https://mavlink.io/en/guide/define_xml_element.html#message_extensions).
    pub fn fields_v1(&self) -> Vec<MessageField> {
        self.base_fields_reordered()
    }

    /// Returns fields applicable to [MAVLink 2](https://mavlink.io/en/guide/mavlink_2.html)
    /// protocol version.
    ///
    /// All [extension fields](https://mavlink.io/en/guide/define_xml_element.html#message_extensions)
    /// will be included.
    ///
    /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering)
    pub fn fields_v2(&self) -> Vec<MessageField> {
        self.reordered_fields()
    }

    /// Returns reordered fields according to
    /// [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
    pub fn reordered_fields(&self) -> Vec<MessageField> {
        /// Custom comparison function.
        fn compare(left: &MessageField, right: &MessageField) -> Ordering {
            let left_type_size = left.r#type().base_type().size();
            let right_type_size = right.r#type().base_type().size();
            left_type_size.cmp(&right_type_size).reverse()
        }

        // Fields must be rearranged only before the first extension field.
        let arrange_before_idx = self.extension_fields_idx();

        match arrange_before_idx {
            // Arrange everything
            None => {
                let mut rearranged = self.fields.clone();
                rearranged.sort_by(compare);
                rearranged
            }
            // Arrange only non-extension fields.
            Some(idx) => {
                let mut rearrangable = self.fields[0..idx].to_vec();
                rearrangable.sort_by(compare);

                let mut fields = self.fields.clone();
                for (i, field) in rearrangable.iter().enumerate() {
                    fields[i] = field.clone()
                }

                fields
            }
        }
    }

    /// Returns base fields.
    ///
    /// Base fields are the fields which are not marked as extensions.
    ///
    /// These fields are not reordered. To get reordered base fields use [`Message::fields_v1`].
    ///
    /// See: [MAVLink message extensions](https://mavlink.io/en/guide/define_xml_element.html#message_extensions).
    pub fn base_fields(&self) -> Vec<MessageField> {
        self.fields()
            .iter()
            .cloned()
            .filter(|field| !field.extension())
            .collect()
    }

    /// Returns base fields reordered according to `MAVLink` specification.
    ///
    /// Base fields are the fields which are not marked as extensions.
    ///
    /// See:
    ///  * [MAVLink fields reordering](https://mavlink.io/en/guide/serialization.html#field_reordering).
    ///  * [MAVLink message extensions](https://mavlink.io/en/guide/define_xml_element.html#message_extensions).
    pub fn base_fields_reordered(&self) -> Vec<MessageField> {
        self.reordered_fields()
            .iter()
            .cloned()
            .filter(|field| !field.extension())
            .collect()
    }

    /// Returns extension fields.
    ///
    /// See: [MAVLink message extensions](https://mavlink.io/en/guide/define_xml_element.html#message_extensions).
    pub fn extension_fields(&self) -> Vec<MessageField> {
        self.fields()
            .iter()
            .cloned()
            .filter(|field| field.extension())
            .collect()
    }

    /// Whether this message is compatible with MAVLink 1.
    ///
    /// See: [MAVLink versions](https://mavlink.io/en/guide/mavlink_version.html).
    pub fn is_v1_compatible(&self) -> bool {
        self.id <= 255
    }
}

/// [`Builder`] for [`Message`].
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MessageBuilder {
    message: Message,
}

impl Builder for MessageBuilder {
    type Buildable = Message;

    fn build(&self) -> Message {
        // We need this to get error when `Message` changes
        #[allow(clippy::match_single_binding)]
        match self.message.clone() {
            Message {
                id,
                name,
                description,
                fields,
                wip,
                deprecated,
                defined_in,
            } => Message {
                id,
                name,
                description,
                fields,
                wip,
                deprecated,
                defined_in,
            },
        }
    }
}

impl MessageBuilder {
    /// Creates builder instance.
    ///
    /// Instantiates builder with default values for [`Message`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets unique message ID within dialect.
    ///
    /// See: [`Message::id`].
    pub fn set_id(&mut self, id: MessageId) -> &mut Self {
        self.message.id = id;
        self
    }

    /// Sets message name.
    ///
    /// See: [`Message::name`].
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.message.name = name;
        self
    }

    /// Sets message description.
    ///
    /// See: [`Message::description`].
    pub fn set_description(&mut self, description: String) -> &mut Self {
        self.message.description = description;
        self
    }

    /// Sets list of message fields.
    ///
    /// See: [`Message::fields`].
    pub fn set_fields(&mut self, fields: Vec<MessageField>) -> &mut Self {
        self.message.fields = fields;
        self
    }

    /// Sets work in progress status.
    ///
    /// See: [`Message::wip`].
    pub fn set_wip(&mut self, wip: bool) -> &mut Self {
        self.message.wip = wip;
        self
    }

    /// Sets deprecation status.
    ///
    /// See: [`Message::deprecated`].
    pub fn set_deprecated(&mut self, deprecated: Option<Deprecated>) -> &mut Self {
        self.message.deprecated = deprecated;
        self
    }

    /// Sets dialect name where this message was defined.
    ///
    /// See: [`Message::defined_in`].
    pub fn set_defined_in(&mut self, defined_in: Option<String>) -> &mut Self {
        self.message.defined_in = defined_in;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{DeprecatedSince, MavType, MessageFieldBuilder};

    fn make_fields(fields: &[MavType]) -> Vec<MessageField> {
        fields
            .iter()
            .enumerate()
            .map(|(i, t)| {
                MessageField::builder()
                    .set_type(t.clone())
                    .set_name(i.to_string())
                    .build()
            })
            .collect()
    }

    #[test]
    fn message_builder() {
        let message = MessageBuilder::new()
            .set_name("name".to_string())
            .set_description("description".to_string())
            .set_fields(make_fields(&[
                MavType::Int16,
                MavType::UInt16,
                MavType::Float,
            ]))
            .set_wip(true)
            .set_deprecated(Some(Deprecated::new(
                DeprecatedSince::new(2023, 10),
                "new".to_string(),
            )))
            .set_defined_in(Some("dialect".to_string()))
            .build();

        assert!(matches!(message, Message { .. }));
        assert_eq!(message.name(), "name");
        assert_eq!(message.description(), "description");
        assert_eq!(message.fields().len(), 3);
        assert!(matches!(message.fields()[0].r#type(), MavType::Int16));
        assert!(message.wip());
        assert!(matches!(message.deprecated(), Some(Deprecated { .. })));
    }

    #[test]
    fn fields_v1_v2() {
        let message = MessageBuilder::new()
            .set_fields(vec![
                MessageFieldBuilder::new()
                    .set_name("first".to_string())
                    .build(),
                MessageFieldBuilder::new()
                    .set_name("second".to_string())
                    .build(),
                MessageFieldBuilder::new()
                    .set_name("third".to_string())
                    .build(),
                MessageFieldBuilder::new()
                    .set_name("fourth (extension)".to_string())
                    .set_extension(true)
                    .build(),
                MessageFieldBuilder::new()
                    .set_name("fifth (extension)".to_string())
                    .set_extension(true)
                    .build(),
            ])
            .build();

        assert_eq!(message.fields().len(), 5);
        assert_eq!(message.fields_v2().len(), 5);
        assert_eq!(message.fields_v1().len(), 3);
        assert_eq!(message.extension_fields().len(), 2);
    }

    #[test]
    fn basic_fields_reordering() {
        let message = Message::builder()
            .set_fields(make_fields(&[
                MavType::Int16,
                MavType::UInt16,
                MavType::UInt32,
                MavType::UInt8,
                MavType::Float,
            ]))
            .build();

        let reordered = message.reordered_fields();

        assert_eq!(reordered.get(0).unwrap().name(), "2");
        assert_eq!(reordered.get(1).unwrap().name(), "4");
        assert_eq!(reordered.get(2).unwrap().name(), "0");
        assert_eq!(reordered.get(3).unwrap().name(), "1");
        assert_eq!(reordered.get(4).unwrap().name(), "3");
    }

    #[test]
    fn extensions_fields_are_not_reordered() {
        let mut fields = make_fields(&[
            MavType::Int16,
            MavType::UInt16,
            MavType::UInt32,
            MavType::UInt8,
            MavType::Float,
        ]);
        fields[3] = fields[3].to_builder().set_extension(true).build();
        fields[4] = fields[4].to_builder().set_extension(true).build();

        let message = Message::builder().set_fields(fields).build();

        let reordered = message.reordered_fields();

        assert_eq!(reordered.get(0).unwrap().name(), "2");
        assert_eq!(reordered.get(1).unwrap().name(), "0");
        assert_eq!(reordered.get(2).unwrap().name(), "1");
        assert_eq!(reordered.get(3).unwrap().name(), "3");
        assert_eq!(reordered.get(4).unwrap().name(), "4");
    }
}
