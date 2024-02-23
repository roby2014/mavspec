use mavinspect::protocol::{Dialect, DialectId, DialectVersion, Enum, Message};
use serde::Serialize;

use crate::generator::GeneratorParams;
use crate::specs::Spec;

/// Specification for dialect module template.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct DialectModuleSpec<'a> {
    name: &'a str,
    version: Option<DialectVersion>,
    dialect_id: Option<DialectId>,
    messages: Vec<&'a Message>,
    enums: Vec<&'a Enum>,
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
            messages: Vec::from_iter(dialect.messages()),
            enums: Vec::from_iter(dialect.enums()),
            params,
        }
    }

    pub(crate) fn name(&self) -> &str {
        self.name
    }

    pub(crate) fn messages(&self) -> &[&Message] {
        self.messages.as_slice()
    }

    pub(crate) fn enums(&self) -> &[&Enum] {
        self.enums.as_slice()
    }

    pub(crate) fn dialect_id(&self) -> Option<DialectId> {
        self.dialect_id
    }

    pub(crate) fn version(&self) -> Option<DialectVersion> {
        self.version
    }

    pub(crate) fn get_enum_by_name(&self, name: &str) -> Option<&Enum> {
        self.enums
            .iter()
            .find(|&&mav_enum| mav_enum.name() == name)
            .copied()
    }
}
