use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::protocol::{Enum, Message, MessageId};

/// MAVLink dialect specification.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Dialect {
    name: String,
    version: Option<u8>,
    dialect: Option<u8>,
    messages: HashMap<MessageId, Message>,
    enums: HashMap<String, Enum>,
}

impl Dialect {
    /// Default constructor
    ///
    /// # Arguments
    ///
    /// * `name` dialect name.
    /// * `version` dialect version.
    /// * `version` dialect version (if provided).
    /// * `messages` map of messages.
    /// * `enums` map of enums.
    pub fn new(
        name: String,
        version: Option<u8>,
        dialect: Option<u8>,
        messages: HashMap<MessageId, Message>,
        enums: HashMap<String, Enum>,
    ) -> Self {
        Self {
            name,
            version,
            dialect,
            messages,
            enums,
        }
    }

    /// Dialect name.
    ///
    /// As a dialect we use a file base name of its XML definition (without extension). However,
    /// upon XML parsing loading, we convert this name to a canonical form
    /// ([`crate::parser::XmlDialectDefinition::canonize_name`]). This may help avoiding naming
    /// collisions when someone tries to generate source code based on the dialect name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Dialect version.
    pub fn version(&self) -> Option<u8> {
        self.version
    }

    /// Dialect version.
    pub fn dialect(&self) -> Option<u8> {
        self.dialect
    }

    /// Collection of dialect messages.
    pub fn messages(&self) -> &HashMap<MessageId, Message> {
        &self.messages
    }

    /// Collection of dialect enums.
    pub fn enums(&self) -> &HashMap<String, Enum> {
        &self.enums
    }
}
