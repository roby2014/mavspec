use mavinspect::protocol::Protocol;

use crate::conventions::dialect_mod_name;
use crate::generator::GeneratorParams;
use crate::specs::Spec;

pub(crate) struct DialectsRootModuleSpec<'a> {
    params: &'a GeneratorParams,
    module_names: Vec<String>,
}

impl<'a> Spec for DialectsRootModuleSpec<'a> {
    fn params(&self) -> &GeneratorParams {
        self.params
    }
}

impl<'a> DialectsRootModuleSpec<'a> {
    pub(crate) fn new(protocol: &Protocol, params: &'a GeneratorParams) -> Self {
        Self {
            module_names: protocol
                .dialects()
                .map(|dialect| dialect_mod_name(dialect.name().into()))
                .collect(),
            params,
        }
    }

    pub(crate) fn module_names(&self) -> &[String] {
        self.module_names.as_slice()
    }
}
