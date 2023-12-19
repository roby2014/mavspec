use std::fs::{create_dir_all, File};
use std::path::PathBuf;

use handlebars::Handlebars;
use serde::Serialize;

use crate::rust::templates::dialects::DialectModuleSpec;
use mavspec::protocol::{Dialect, Protocol};

use super::conventions;
use super::helpers::register_helpers;
use super::templates;

/// # [`RustGenerator`] parameters
#[derive(Debug, Clone, Serialize)]
pub struct RustGeneratorParams {
    /// Path to the autogenerated module within target crate.
    ///
    /// For example: `my_crate::mavlink`.
    pub module_path: String,
    /// Add `serde` support for all generated entities
    pub serde: bool,
}

/// # Rust code generator
pub struct RustGenerator<'a> {
    protocol: Protocol,
    path: PathBuf,
    params: RustGeneratorParams,
    handlebars: Handlebars<'a>,
}

impl<'a> RustGenerator<'a> {
    /// Default constructor.
    pub fn new(protocol: Protocol, path: PathBuf, params: RustGeneratorParams) -> Self {
        let handlebars = Self::build_handlebars().unwrap();

        Self {
            protocol,
            path,
            params,
            handlebars,
        }
    }

    /// Generate Rust bindings.
    pub fn generate(&self) -> anyhow::Result<()> {
        log::info!("Generating Rust code from MAVLink protocol.");

        self.generate_root_module()?;
        self.generate_dialects()?;

        log::info!(
            "Generation results: {}",
            self.path.canonicalize().unwrap().to_str().unwrap()
        );

        Ok(())
    }

    fn generate_root_module(&self) -> anyhow::Result<()> {
        // Ensure that root directory exists
        create_dir_all(&self.path)?;

        let file = File::create(self.root_module_file_path("mod.rs"))?;
        self.handlebars
            .render_to_write("mod.rs", &self.protocol, file)?;
        log::debug!("Generated: root module.");

        Ok(())
    }

    fn generate_dialects(&self) -> anyhow::Result<()> {
        // Ensure that dialects directory exists
        create_dir_all(self.dialects_dir())?;

        // Generate root module for all dialects
        let file = File::create(self.dialects_mod_rs())?;
        self.handlebars
            .render_to_write("dialects/mod.rs", &self.protocol, file)?;
        log::debug!("Generated: 'dialects' root module.");

        // Generate individual dialects
        for dialect in self.protocol.dialects().values() {
            self.generate_dialect(dialect)?;
        }

        Ok(())
    }

    fn generate_dialect(&self, dialect: &Dialect) -> anyhow::Result<()> {
        create_dir_all(self.dialect_dir(dialect.name()))?;

        let file = File::create(self.dialect_mod_rs(dialect.name()))?;
        self.handlebars.render_to_write(
            "dialects/{dialect}/mod.rs",
            &DialectModuleSpec {
                dialect,
                params: &self.params,
            },
            file,
        )?;
        log::debug!("Generated: 'dialects::{}' root module.", dialect.name());

        self.generate_enums(dialect)?;
        self.generate_messages(dialect)?;

        Ok(())
    }

    fn generate_enums(&self, dialect: &Dialect) -> anyhow::Result<()> {
        create_dir_all(self.enums_dir(dialect.name()))?;

        let file = File::create(self.enums_mod_rs(dialect.name()))?;
        self.handlebars
            .render_to_write("dialects/{dialect}/enums/mod.rs", dialect, file)?;
        log::debug!(
            "Generated: 'dialects::{}::enums' root module.",
            dialect.name()
        );

        for mav_enum in dialect.enums().values() {
            let file = File::create(self.enum_file(dialect.name(), mav_enum.name()))?;

            self.handlebars.render_to_write(
                "dialects/{dialect}/enums/{enum}.rs",
                &templates::dialects::enums::EnumSpec::new(mav_enum, &self.params),
                file,
            )?;

            log::trace!(
                "Generated: message '{}' for dialect '{}'.",
                mav_enum.name(),
                dialect.name(),
            );
        }

        Ok(())
    }

    fn generate_messages(&self, dialect: &Dialect) -> anyhow::Result<()> {
        create_dir_all(self.messages_dir(dialect.name()))?;

        let file = File::create(self.messages_mod_rs(dialect.name()))?;
        self.handlebars
            .render_to_write("dialects/{dialect}/messages/mod.rs", dialect, file)?;
        log::debug!(
            "Generated: 'dialects::{}::messages' root module.",
            dialect.name()
        );

        for message in dialect.messages().values() {
            let file = File::create(self.message_file(dialect.name(), message.name()))?;

            match message.defined_in() {
                Some(dialect_name) if dialect_name != dialect.name() => {
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
                        dialect.name(),
                        dialect_name
                    );
                }
                _ => {
                    self.handlebars.render_to_write(
                        "dialects/{dialect}/messages/{message}.rs",
                        &templates::dialects::messages::MessageSpec::new(
                            message,
                            dialect,
                            &self.params,
                        ),
                        file,
                    )?;
                    log::trace!(
                        "Generated: message '{}' for dialect '{}'.",
                        message.name(),
                        dialect.name(),
                    );
                }
            }
        }
        log::debug!("Generated: all '{}' dialect messages.", dialect.name());

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
