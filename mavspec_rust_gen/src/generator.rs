use std::ffi::OsStr;
#[cfg(feature = "fingerprints")]
use std::fs::read_to_string;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "fingerprints")]
use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;

use mavinspect::protocol::{Dialect, Enum, Protocol};

use crate::conventions;
use crate::specs::dialects::dialect::enums::{
    EnumImplModuleSpec, EnumInheritedModuleSpec, EnumsRootModuleSpec,
};
use crate::specs::dialects::dialect::messages::{
    MessageImplModuleSpec, MessageInheritedModuleSpec, MessagesRootModuleSpec,
};
use crate::specs::dialects::dialect::DialectModuleSpec;
use crate::specs::dialects::DialectsRootModuleSpec;
use crate::templates;

/// [`Generator`] parameters.
#[derive(Clone, Debug, Default, Serialize)]
pub struct GeneratorParams {
    pub serde: bool,
    pub generate_tests: bool,
}

/// Rust code generator.
pub struct Generator {
    protocol: Arc<Protocol>,
    path: PathBuf,
    params: GeneratorParams,
}

impl Generator {
    /// Default constructor.
    pub fn new<T: ?Sized + AsRef<OsStr>>(
        protocol: Arc<Protocol>,
        path: &T,
        params: GeneratorParams,
    ) -> Self {
        Self {
            protocol,
            path: PathBuf::from(path),
            params,
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
        create_dir_all(self.path.as_path())?;

        let mut file = File::create(self.root_module_file_path("mod.rs"))?;
        let content = prettyplease::unparse(&templates::root_module());

        file.write_all(content.as_bytes())?;
        log::debug!("Generated: root module.");

        Ok(())
    }

    fn generate_dialects(&self) -> anyhow::Result<()> {
        create_dir_all(self.dialects_dir())?;

        let mut file = File::create(self.dialects_mod_rs())?;
        let content = prettyplease::unparse(&templates::dialects::dialects_root_module(
            &DialectsRootModuleSpec::new(self.protocol.as_ref(), &self.params),
        ));

        file.write_all(content.as_bytes())?;
        log::debug!("Generated: 'dialects' root module.");

        for dialect in self.protocol.dialects() {
            let dialect_spec = DialectModuleSpec::new(dialect, &self.params);
            self.generate_dialect(&dialect_spec)?;
        }

        Ok(())
    }

    fn generate_dialect(&self, dialect_spec: &DialectModuleSpec) -> anyhow::Result<()> {
        create_dir_all(self.dialect_dir(dialect_spec.name()))?;

        let mut file = File::create(self.dialect_mod_rs(dialect_spec.name()))?;
        let content =
            prettyplease::unparse(&templates::dialects::dialect::dialect_module(dialect_spec));

        file.write_all(content.as_bytes())?;
        log::debug!(
            "Generated: 'dialects::{}' root module.",
            dialect_spec.name()
        );

        self.generate_enums(dialect_spec)?;
        self.generate_messages(dialect_spec)?;

        Ok(())
    }

    fn generate_enums(&self, dialect_spec: &DialectModuleSpec) -> anyhow::Result<()> {
        create_dir_all(self.enums_dir(dialect_spec.name()))?;

        let mut file = File::create(self.enums_mod_rs(dialect_spec.name()))?;
        let content =
            prettyplease::unparse(&templates::dialects::dialect::enums::enums_root_module(
                &EnumsRootModuleSpec::new(dialect_spec, &self.params),
            ));

        file.write_all(content.as_bytes())?;
        log::debug!(
            "Generated: 'dialects::{}::enums' root module.",
            dialect_spec.name()
        );

        for mav_enum in dialect_spec.enums() {
            let mut file = File::create(self.enum_file(dialect_spec.name(), mav_enum.name()))?;

            let content = if let Some(inherited_from_dialect) =
                self.enum_inherited_from(mav_enum, dialect_spec.name())
            {
                prettyplease::unparse(&templates::dialects::dialect::enums::enum_inherited_module(
                    &EnumInheritedModuleSpec::new(
                        mav_enum,
                        inherited_from_dialect.name(),
                        &self.params,
                    ),
                ))
            } else {
                prettyplease::unparse(&templates::dialects::dialect::enums::enum_module(
                    &EnumImplModuleSpec::new(mav_enum, &self.params),
                ))
            };

            file.write_all(content.as_bytes())?;
            log::trace!(
                "Generated: enum '{}' for dialect '{}'.",
                mav_enum.name(),
                dialect_spec.name(),
            );
        }

        Ok(())
    }

    fn generate_messages(&self, dialect_spec: &DialectModuleSpec) -> anyhow::Result<()> {
        create_dir_all(self.messages_dir(dialect_spec.name()))?;

        let mut file = File::create(self.messages_mod_rs(dialect_spec.name()))?;
        let content = prettyplease::unparse(
            &templates::dialects::dialect::messages::messages_root_module(
                &MessagesRootModuleSpec::new(dialect_spec, &self.params),
            ),
        );

        file.write_all(content.as_bytes())?;
        log::debug!(
            "Generated: 'dialects::{}::messages' root module.",
            dialect_spec.name()
        );

        for message in dialect_spec.messages() {
            let mut file = File::create(self.message_file(dialect_spec.name(), message.name()))?;

            match message.defined_in() {
                Some(dialect_name) if dialect_name != dialect_spec.name() => {
                    let content = prettyplease::unparse(
                        &templates::dialects::dialect::messages::inherited_message_module(
                            &MessageInheritedModuleSpec::new(
                                dialect_name.as_str(),
                                message,
                                &self.params,
                            ),
                        ),
                    );

                    file.write_all(content.as_bytes())?;
                    log::trace!(
                        "Message '{}' in dialect '{}' is inherited from dialect '{}'.",
                        message.name(),
                        dialect_spec.name(),
                        dialect_name
                    );
                }
                _ => {
                    let content = prettyplease::unparse(
                        &templates::dialects::dialect::messages::message_module(
                            &MessageImplModuleSpec::new(message, dialect_spec),
                        ),
                    );

                    file.write_all(content.as_bytes())?;
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
            .join(conventions::dialect_mod_name(dialect_name.to_string()))
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
            .join(conventions::enum_file_name(enum_name))
    }

    fn messages_dir(&self, dialect_name: &str) -> PathBuf {
        self.dialect_dir(dialect_name).join("messages")
    }

    fn messages_mod_rs(&self, dialect_name: &str) -> PathBuf {
        self.messages_dir(dialect_name).join("mod.rs")
    }

    fn message_file(&self, dialect_name: &str, message_name: &str) -> PathBuf {
        self.messages_dir(dialect_name)
            .join(conventions::message_file_name(message_name))
    }

    fn enum_inherited_from(&self, mav_enum: &Enum, dialect_name: &str) -> Option<&Dialect> {
        for defined_in_dialect_name in mav_enum.defined_in() {
            let defined_in_dialect = self
                .protocol
                .get_dialect_by_name(defined_in_dialect_name.as_str())
                .unwrap();
            if defined_in_dialect_name == dialect_name {
                continue;
            }

            let original_enum = defined_in_dialect.get_enum_by_name(mav_enum.name());
            let original_enum = if let Some(original_enum) = original_enum {
                original_enum
            } else {
                continue;
            };

            if original_enum.fingerprint() == mav_enum.fingerprint() {
                return Some(defined_in_dialect);
            }
        }
        None
    }
}
