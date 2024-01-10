use std::collections::HashMap;
use std::ffi::OsStr;
#[cfg(feature = "fingerprints")]
use std::fs::read_to_string;
use std::fs::{create_dir_all, File};
#[cfg(feature = "fingerprints")]
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "fingerprints")]
use base64::{engine::general_purpose, Engine as _};
use handlebars::Handlebars;
use serde::Serialize;

use crate::templates::dialects::messages::MessagesSpec;
use mavinspect::protocol::{
    Dialect, DialectId, DialectVersion, Enum, Message, MessageId, Protocol,
};

use super::conventions;
use super::helpers::register_helpers;
use super::templates;

/// [`Generator`] parameters.
#[derive(Clone, Debug, Default, Serialize)]
pub struct GeneratorParams {
    pub serde: bool,
    pub generate_tests: bool,
}

/// Specification for dialect-level templates.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct DialectSpec<'a> {
    name: String,
    version: Option<DialectVersion>,
    dialect_id: Option<DialectId>,
    messages: HashMap<MessageId, Message>,
    enums: HashMap<String, Enum>,
    params: &'a GeneratorParams,
}

impl<'a> DialectSpec<'a> {
    pub(crate) fn new(dialect: &'a Dialect, params: &'a GeneratorParams) -> Self {
        Self {
            name: dialect.name().to_string(),
            version: dialect.version(),
            dialect_id: dialect.dialect(),
            messages: dialect.messages().clone(),
            enums: dialect.enums().clone(),
            params,
        }
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(crate) fn messages(&self) -> &HashMap<MessageId, Message> {
        &self.messages
    }

    pub(crate) fn enums(&self) -> &HashMap<String, Enum> {
        &self.enums
    }

    pub(crate) fn params(&self) -> &GeneratorParams {
        self.params
    }
}

/// Rust code generator.
pub struct Generator<'a> {
    protocol: Arc<Protocol>,
    path: PathBuf,
    params: GeneratorParams,
    handlebars: Handlebars<'a>,
}

impl<'a> Generator<'a> {
    /// Default constructor.
    pub fn new<T: ?Sized + AsRef<OsStr>>(
        protocol: Arc<Protocol>,
        path: &T,
        params: GeneratorParams,
    ) -> Self {
        let handlebars = Self::build_handlebars().unwrap();

        Self {
            protocol,
            path: PathBuf::from(path),
            params,
            handlebars,
        }
    }

    /// Generate Rust bindings.
    pub fn generate(&self) -> anyhow::Result<()> {
        log::info!("Generating Rust code from MAVLink protocol.");

        #[cfg(feature = "fingerprints")]
        if !self.fingerprint_has_updated()? {
            log::info!("Fingerprint hasn't changed. Skipping.");
            return Ok(());
        }

        self.generate_root_module()?;
        self.generate_dialects()?;

        #[cfg(feature = "fingerprints")]
        self.generate_fingerprint()?;

        log::info!(
            "Generation results: {}",
            self.path.canonicalize().unwrap().to_str().unwrap()
        );

        Ok(())
    }

    #[cfg(feature = "fingerprints")]
    fn fingerprint(&self) -> String {
        general_purpose::STANDARD_NO_PAD.encode(self.protocol.fingerprint().to_le_bytes())
    }

    #[cfg(feature = "fingerprints")]
    fn fingerprint_has_updated(&self) -> anyhow::Result<bool> {
        if self.fingerprint_path().exists() {
            let existing_fingerprint = read_to_string(self.fingerprint_path())?;
            return Ok(existing_fingerprint != self.fingerprint());
        }

        Ok(true)
    }

    #[cfg(feature = "fingerprints")]
    fn generate_fingerprint(&self) -> anyhow::Result<()> {
        let mut file = File::create(self.fingerprint_path())?;
        file.write_all(self.fingerprint().as_bytes())?;
        Ok(())
    }

    fn generate_root_module(&self) -> anyhow::Result<()> {
        // Ensure that root directory exists
        create_dir_all(self.path.as_path())?;

        let file = File::create(self.root_module_file_path("mod.rs"))?;
        self.handlebars
            .render_to_write("mod.rs", self.protocol.as_ref(), file)?;
        log::debug!("Generated: root module.");

        Ok(())
    }

    fn generate_dialects(&self) -> anyhow::Result<()> {
        // Ensure that dialects directory exists
        create_dir_all(self.dialects_dir())?;

        // Generate root module for all dialects
        let file = File::create(self.dialects_mod_rs())?;
        self.handlebars
            .render_to_write("dialects/mod.rs", self.protocol.as_ref(), file)?;
        log::debug!("Generated: 'dialects' root module.");

        // Generate individual dialects
        for dialect in self.protocol.dialects().values() {
            let dialect_spec = DialectSpec::new(dialect, &self.params);
            self.generate_dialect(&dialect_spec)?;
        }

        Ok(())
    }

    fn generate_dialect(&self, dialect_spec: &DialectSpec) -> anyhow::Result<()> {
        create_dir_all(self.dialect_dir(dialect_spec.name()))?;

        let file = File::create(self.dialect_mod_rs(dialect_spec.name()))?;
        self.handlebars
            .render_to_write("dialects/{dialect}/mod.rs", &dialect_spec, file)?;
        log::debug!(
            "Generated: 'dialects::{}' root module.",
            dialect_spec.name()
        );

        self.generate_enums(dialect_spec)?;
        self.generate_messages(dialect_spec)?;

        Ok(())
    }

    fn generate_enums(&self, dialect_spec: &DialectSpec) -> anyhow::Result<()> {
        create_dir_all(self.enums_dir(dialect_spec.name()))?;

        let file = File::create(self.enums_mod_rs(dialect_spec.name()))?;
        self.handlebars
            .render_to_write("dialects/{dialect}/enums/mod.rs", dialect_spec, file)?;
        log::debug!(
            "Generated: 'dialects::{}::enums' root module.",
            dialect_spec.name()
        );

        for mav_enum in dialect_spec.enums().values() {
            let file = File::create(self.enum_file(dialect_spec.name(), mav_enum.name()))?;

            self.handlebars.render_to_write(
                "dialects/{dialect}/enums/{enum}.rs",
                &templates::dialects::enums::EnumSpec::new(mav_enum, &self.params),
                file,
            )?;

            log::trace!(
                "Generated: enum '{}' for dialect '{}'.",
                mav_enum.name(),
                dialect_spec.name(),
            );
        }

        Ok(())
    }

    fn generate_messages(&self, dialect_spec: &DialectSpec) -> anyhow::Result<()> {
        create_dir_all(self.messages_dir(dialect_spec.name()))?;

        let file = File::create(self.messages_mod_rs(dialect_spec.name()))?;
        self.handlebars.render_to_write(
            "dialects/{dialect}/messages/mod.rs",
            &MessagesSpec::new(dialect_spec),
            file,
        )?;
        log::debug!(
            "Generated: 'dialects::{}::messages' root module.",
            dialect_spec.name()
        );

        for message in dialect_spec.messages().values() {
            let file = File::create(self.message_file(dialect_spec.name(), message.name()))?;

            match message.defined_in() {
                Some(dialect_name) if dialect_name != dialect_spec.name() => {
                    self.handlebars.render_to_write(
                        "dialects/{dialect}/messages/{message:inherited}.rs",
                        &templates::dialects::messages::InheritedMessageSpec::new(
                            dialect_name.clone(),
                            message,
                        ),
                        file,
                    )?;
                    log::trace!(
                        "Message '{}' in dialect '{}' is inherited from dialect '{}'.",
                        message.name(),
                        dialect_spec.name(),
                        dialect_name
                    );
                }
                _ => {
                    self.handlebars.render_to_write(
                        "dialects/{dialect}/messages/{message}.rs",
                        &templates::dialects::messages::MessageSpec::new(message, dialect_spec),
                        file,
                    )?;
                    log::trace!(
                        "Generated: message '{}' for dialect '{}'.",
                        message.name(),
                        dialect_spec.name(),
                    );
                }
            }
        }
        log::debug!("Generated: all '{}' dialect messages.", dialect_spec.name());

        Ok(())
    }

    fn build_handlebars<'h>() -> anyhow::Result<Handlebars<'h>> {
        let mut reg = Handlebars::new();

        register_helpers(&mut reg);

        reg.register_template_string("mod.rs", templates::ROOT_MODULE)?;

        reg.register_template_string("dialects/mod.rs", templates::dialects::DIALECTS_ROOT_MODULE)?;
        reg.register_template_string(
            "dialects/{dialect}/mod.rs",
            templates::dialects::DIALECT_MODULE,
        )?;
        reg.register_template_string(
            "dialects/{dialect}/messages/mod.rs",
            templates::dialects::messages::MESSAGES_MODULE_ROOT,
        )?;
        reg.register_template_string(
            "dialects/{dialect}/messages/{message}.rs",
            templates::dialects::messages::MESSAGE,
        )?;
        reg.register_template_string(
            "dialects/{dialect}/messages/{message:inherited}.rs",
            templates::dialects::messages::INHERITED_MESSAGE,
        )?;
        reg.register_template_string(
            "dialects/{dialect}/enums/mod.rs",
            templates::dialects::enums::ENUMS_MODULE_ROOT,
        )?;
        reg.register_template_string(
            "dialects/{dialect}/enums/{enum}.rs",
            templates::dialects::enums::ENUM,
        )?;

        Ok(reg)
    }

    #[cfg(feature = "fingerprints")]
    fn fingerprint_path(&self) -> PathBuf {
        self.path.join(".fingerprint")
    }

    fn root_module_file_path(&self, filename: &str) -> PathBuf {
        self.path.join(filename)
    }

    fn dialects_dir(&self) -> PathBuf {
        self.path.join("dialects")
    }

    fn dialects_mod_rs(&self) -> PathBuf {
        self.dialects_dir().join("mod.rs")
    }

    fn dialect_dir(&self, dialect_name: &str) -> PathBuf {
        self.dialects_dir()
            .join(conventions::dialect_name(dialect_name.to_string()))
    }

    fn dialect_mod_rs(&self, dialect_name: &str) -> PathBuf {
        self.dialect_dir(dialect_name).join("mod.rs")
    }

    fn enums_dir(&self, dialect_name: &str) -> PathBuf {
        self.dialect_dir(dialect_name).join("enums")
    }

    fn enums_mod_rs(&self, dialect_name: &str) -> PathBuf {
        self.enums_dir(dialect_name).join("mod.rs")
    }

    fn enum_file(&self, dialect_name: &str, enum_name: &str) -> PathBuf {
        self.enums_dir(dialect_name)
            .join(conventions::enum_file_name(enum_name.to_string()))
    }

    fn messages_dir(&self, dialect_name: &str) -> PathBuf {
        self.dialect_dir(dialect_name).join("messages")
    }

    fn messages_mod_rs(&self, dialect_name: &str) -> PathBuf {
        self.messages_dir(dialect_name).join("mod.rs")
    }

    fn message_file(&self, dialect_name: &str, message_name: &str) -> PathBuf {
        self.messages_dir(dialect_name)
            .join(conventions::message_file_name(message_name.to_string()))
    }
}
