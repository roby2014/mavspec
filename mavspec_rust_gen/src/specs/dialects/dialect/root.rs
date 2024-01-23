use std::collections::HashMap;

use mavinspect::protocol::{Dialect, DialectId, DialectVersion, Enum, Message, MessageId};
use serde::Serialize;

use crate::generator::GeneratorParams;
use crate::specs::Spec;

/// Specification for dialect module template.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct DialectModuleSpec<'a> {
    name: &'a str,
    version: Option<DialectVersion>,
    dialect_id: Option<DialectId>,
    messages: &'a HashMap<MessageId, Message>,
    enums: &'a HashMap<String, Enum>,
    params: &'a GeneratorParams,
}

impl<'a> Spec for DialectModuleSpec<'a> {
    fn params(&self) -> &GeneratorParams {
        self.params
    }
}

impl<'a> DialectModuleSpec<'a> {
    pub(crate) fn new(dialect: &'a Dialect, params: &'a GeneratorParams) -> Self {
        Self {
            name: dialect.name(),
            version: dialect.version(),
            dialect_id: dialect.dialect(),
            messages: dialect.messages(),
            enums: dialect.enums(),
            params,
        }
    }

    pub(crate) fn name(&self) -> &str {
        self.name
    }

    pub(crate) fn messages(&self) -> &HashMap<MessageId, Message> {
        self.messages
    }

    pub(crate) fn enums(&self) -> &HashMap<String, Enum> {
        self.enums
    }

    pub(crate) fn dialect_id(&self) -> Option<DialectId> {
        self.dialect_id
    }

    pub(crate) fn version(&self) -> Option<DialectVersion> {
        self.version
    }
}
