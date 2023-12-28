use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};
use std::sync::Arc;

extern crate cargo_manifest;
use cargo_manifest::{Manifest, Value};
use mavinspect::protocol::Protocol;
use mavinspect::Inspector;

use crate::generator::{Generator, GeneratorParams};

/// Code builder for Rust generator.
#[derive(Clone, Debug, Default)]
pub struct BuildHelper {
    out_path: PathBuf,
    sources: Option<Vec<PathBuf>>,
    manifest_path: Option<PathBuf>,
    include_dialects: Option<HashSet<String>>,
    exclude_dialects: Option<HashSet<String>>,
    messages: Option<HashSet<String>>,
    protocol: Option<Arc<Protocol>>,
    all_enums: Option<bool>,
    serde: bool,
    generate_tests: Option<bool>,
}

/// Configuration builder for [`BuildHelper`].
#[derive(Clone, Debug, Default)]
pub struct BuildHelperConf(BuildHelper);

impl BuildHelper {
    /// Creates configuration builder for [`BuildHelper`].
    ///
    /// Takes `out_path` as argument. The output path for generated content.
    pub fn builder<T: Into<PathBuf>>(out_path: T) -> BuildHelperConf {
        BuildHelperConf(Self {
            out_path: out_path.into(),
            ..Default::default()
        })
    }

    /// Scans for dialects and generates MAVLink dialects.
    pub fn generate(&self) -> anyhow::Result<()> {
        if let Err(err) = remove_dir_all(&self.out_path) {
            log::debug!("Error while cleaning output directory: {err:?}");
        }

        let protocol = if let Some(protocol) = &self.protocol {
            protocol.clone()
        } else {
            let mut inspector_builder = Inspector::builder();

            let sources: Vec<&Path> = self
                .sources
                .as_ref()
                .unwrap()
                .iter()
                .map(|s| s.as_path())
                .collect();
            inspector_builder.set_sources(&sources);

            if let Some(include_dialects) = &self.include_dialects {
                inspector_builder
                    .set_include(&Vec::from_iter(include_dialects.iter().map(|d| d.as_str())));
            }
            if let Some(exclude_dialects) = &self.exclude_dialects {
                inspector_builder
                    .set_exclude(&Vec::from_iter(exclude_dialects.iter().map(|d| d.as_str())));
            }

            let mut protocol = inspector_builder.build()?.parse()?;
            if let Some(messages) = &self.messages {
                protocol.retain_messages(messages, !self.all_enums.unwrap_or(false));
            }

            Arc::new(protocol)
        };

        Generator::new(
            protocol,
            &self.out_path,
            GeneratorParams {
                serde: self.serde,
                messages: self.messages.clone(),
                all_enums: self.all_enums.unwrap_or(false),
                generate_tests: self.generate_tests.unwrap_or(false),
            },
        )
        .generate()?;

        Ok(())
    }

    /// Output path for autogenerated files.
    pub fn out_path(&self) -> &Path {
        self.out_path.as_path()
    }

    /// List of directories with MAVLink XML definitions.
    pub fn sources(&self) -> Option<Vec<&Path>> {
        self.sources
            .as_ref()
            .map(|sources| sources.iter().map(|src| src.as_path()).collect())
    }

    /// Path to `Cargo.toml` from which meta information will be extracted.
    ///
    /// You can specify configuration in your `Cargo.toml` file:
    ///
    /// ```toml
    /// [package.metadata.mavspec]
    /// messages = ["HEARTBEAT", "PROTOCOL_VERSION", "MAV_INSPECT_V1", "COMMAND_INT", "COMMAND_LONG"]
    /// all_enums = false
    /// generate_tests = false
    /// ```
    ///
    /// If [`Self::manifest_path`] is set, then the following parameters will be populated from keys in `Cargo.toml`:
    ///
    /// * [`Self::messages`] replaces `messages` key.
    /// * [`Self::all_enums`] replaces `all_enums` key.
    /// * [`Self::generate_tests`] replaces `generate_tests` key.
    ///
    /// Note that if set explicitly, these parameters has precedence over keys from manifest.
    pub fn manifest_path(&self) -> Option<&Path> {
        self.manifest_path.as_deref()
    }

    /// Included dialects.
    pub fn include_dialects(&self) -> Option<HashSet<&str>> {
        self.include_dialects.as_ref().map(|include_dialects| {
            include_dialects
                .iter()
                .map(|dialect| dialect.as_str())
                .collect()
        })
    }

    /// Excluded dialects.
    pub fn exclude_dialects(&self) -> Option<HashSet<&str>> {
        self.exclude_dialects.as_ref().map(|include_dialects| {
            include_dialects
                .iter()
                .map(|dialect| dialect.as_str())
                .collect()
        })
    }

    /// If set, then defines the list of MAVLink messages to generate.
    pub fn messages(&self) -> Option<HashSet<&str>> {
        self.messages.as_ref().map(|include_dialects| {
            include_dialects
                .iter()
                .map(|dialect| dialect.as_str())
                .collect()
        })
    }

    /// MAVLink protocol containing parsed dialects.
    pub fn protocol(&self) -> Option<&Protocol> {
        match &self.protocol {
            None => None,
            Some(protocol) => Some(protocol.as_ref()),
        }
    }

    /// Whether only enums required for specified messages will be generated.
    pub fn all_enums(&self) -> bool {
        self.all_enums.unwrap_or(false)
    }

    /// [Serde](https://serde.rs/) support flag for generated entities.
    pub fn serde(&self) -> bool {
        self.serde
    }

    /// Tests generation flag.
    ///
    /// If set to `true`, then tests will be generated.
    pub fn generate_tests(&self) -> bool {
        self.generate_tests.unwrap_or(false)
    }

    fn apply_manifest_config(&mut self) -> anyhow::Result<()> {
        if let Some(manifest_path) = &self.manifest_path {
            let manifest = Manifest::from_path(manifest_path)?;
            if let Some(package) = manifest.package {
                if let Some(metadata) = package.metadata {
                    if let Some(spec) = metadata.get("mavspec") {
                        if let Some(Value::Array(msgs)) = spec.get("messages") {
                            if self.messages.is_none() {
                                self.messages = Some(HashSet::from_iter(
                                    msgs.iter().map(|v| v.to_string().replace('"', "")),
                                ));
                            }
                        }
                        if let Some(Value::Boolean(all)) = spec.get("all_enums") {
                            self.all_enums = Some(*all);
                        }
                        if let Some(Value::Boolean(generate_tests)) = spec.get("generate_tests") {
                            self.generate_tests = Some(*generate_tests);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl BuildHelperConf {
    /// Default constructor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds [`BuildHelper`] from configuration.
    pub fn build(&self) -> anyhow::Result<BuildHelper> {
        let mut helper = self.0.clone();
        if helper.manifest_path.is_some() {
            helper.apply_manifest_config()?;
        }

        Ok(helper)
    }

    /// Builds [`BuildHelper`] and use it to generates dialects according to configuration.
    pub fn generate(&self) -> anyhow::Result<()> {
        self.build()?.generate()
    }

    /// Set paths to MAVLink XML definitions directories. Discards [`Self::set_protocol`].
    ///
    /// If sources are set then [`Self::set_sources`] wil be discarded and MAVLink message definitions will be read from
    /// these specified `sources`. This enables parameters related to XML definitions parsing and filtering.
    ///
    /// The following parameters will take effect:
    ///
    /// * [`Self::set_include_dialects`],
    /// * [`Self::set_exclude_dialects`],
    /// * [`Self::set_messages`],
    /// * [`Self::set_all_enums`],
    /// * [`Self::set_manifest_path`].
    pub fn set_sources<T>(&mut self, sources: &[T]) -> &mut Self
    where
        T: ?Sized + Into<PathBuf> + Clone,
    {
        self.0.sources = Some(sources.iter().cloned().map(|src| src.into()).collect());
        self.0.manifest_path = None;
        self
    }

    /// Set path to `Cargo.toml` manifest.
    ///
    /// You can control which messages to include by specifying `messages` key in your `Cargo.toml`:
    ///
    /// ```toml
    /// [package.metadata.mavspec]
    /// messages = ["HEARTBEAT", "PROTOCOL_VERSION", "MAV_INSPECT_V1", "COMMAND_INT", "COMMAND_LONG"]
    /// all_enums = false
    /// generate_tests = false
    /// ```
    ///
    /// The `all_enums` key defines whether only enums required for specified messages will be generated.
    ///
    /// The following parameters have precedence over configuration defined in Cargo manifest:
    ///
    /// * [`Self::set_messages`] replaces `messages` key.
    /// * [`Self::set_all_enums`] replaces `all_enums` key.
    /// * [`Self::set_generate_tests`] replaces `generate_tests` key.
    pub fn set_manifest_path<T: ?Sized + AsRef<OsStr>>(&mut self, manifest_path: &T) -> &mut Self {
        self.0.manifest_path = Some(PathBuf::from(manifest_path));
        self
    }

    /// Set dialects list. Only dialects from this list will be generated.
    ///
    /// Does not includes dialects ignored by [`Self::set_exclude_dialects`].
    ///
    /// This does not apply to dialect dependencies. If specified dialect has `<include>` tag, all these dialects will
    /// be generated as well.
    pub fn set_include_dialects<T: ToString>(&mut self, include_dialects: &[T]) -> &mut Self {
        self.0.include_dialects = Some(HashSet::from_iter(
            include_dialects.iter().map(|s| s.to_string()),
        ));
        self
    }

    /// Set dialects exclusion list. Dialects from this list will not be generated.
    ///
    /// Has precedence over [`Self::set_include_dialects`].
    ///
    /// This does not apply to dialect dependencies. If a dialect has `<include>` tag, all these dialects will
    /// be generated as well.
    pub fn set_exclude_dialects<T: ToString>(&mut self, include_dialects: &[T]) -> &mut Self {
        self.0.include_dialects = Some(HashSet::from_iter(
            include_dialects.iter().map(|s| s.to_string()),
        ));
        self
    }

    /// Defines which messages will be generated.
    ///
    /// Overrides `messages` configuration key defined by [`Self::set_manifest_path`].
    pub fn set_messages<T: ToString>(&mut self, messages: &[T]) -> &mut Self {
        self.0.messages = Some(HashSet::from_iter(messages.iter().map(|s| s.to_string())));
        self
    }

    /// Defines whether only enums required for specified messages will be generated.
    ///
    /// Overrides `all_enums` configuration flag set by [`Self::set_manifest_path`].
    ///
    /// Does not have effect if message filtering is not enabled.
    pub fn set_all_enums(&mut self, all_enums: bool) -> &mut Self {
        self.0.all_enums = Some(all_enums);
        self
    }

    /// Enables/disables [Serde](https://serde.rs/) support for generated entities.
    pub fn set_serde(&mut self, serde: bool) -> &mut Self {
        self.0.serde = serde;
        self
    }

    /// Manually define MAVInspect [`Protocol`].
    ///
    /// If set, then [`Self::set_sources`] will be discarded and all parameters controlling MAVLink XML definitions will
    /// be ignored.
    ///
    /// These parameters will be ignored upon protocol setting:
    ///
    /// * [`Self::set_include_dialects`],
    /// * [`Self::set_exclude_dialects`],
    /// * [`Self::set_messages`],
    /// * [`Self::set_all_enums`],
    /// * [`Self::set_manifest_path`].
    pub fn set_protocol(&mut self, protocol: Protocol) -> &mut Self {
        self.0.protocol = Some(Arc::new(protocol));
        self.0.sources = None;
        self
    }

    /// Enables/disables tests generation.
    ///
    /// Set to `true` if you want include autogenerated tests.
    ///
    /// Overrides `generate_tests` configuration flag set by [`Self::set_manifest_path`].
    pub fn set_generate_tests(&mut self, generate_tests: bool) -> &mut Self {
        self.0.generate_tests = Some(generate_tests);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_dir_all;
    use std::path::Path;

    #[test]
    fn build_helper_basic() {
        BuildHelper::builder(&Path::new("../tmp/mavlink"))
            .set_sources(&[
                "../message_definitions/standard",
                "../message_definitions/extra",
            ])
            .set_include_dialects(&["minimal"])
            .generate()
            .unwrap();

        remove_dir_all("../tmp/mavlink").unwrap();
    }

    #[test]
    fn build_helper_new_generic() {
        // Accepts `&str`.
        BuildHelper::builder("../tmp/mavlink");

        // Accepts `String`.
        BuildHelper::builder("../tmp/mavlink".to_string());

        // Accepts `&Path`.
        BuildHelper::builder(Path::new("../tmp/mavlink"));

        // Accepts `PathBuf`
        BuildHelper::builder(Path::new("../tmp").join("mavlink"));
    }

    #[test]
    fn build_helper_set_sources_generic() {
        // Accepts `&str`.
        BuildHelper::builder("../tmp/mavlink").set_sources(&["../message_definitions/standard"]);

        // Accepts `String`.
        BuildHelper::builder("../tmp/mavlink")
            .set_sources(&["../message_definitions/standard".to_string()]);

        // Accepts `&Path`.
        BuildHelper::builder("../tmp/mavlink")
            .set_sources(&[Path::new("../message_definitions/standard")]);

        // Accepts `PathBuf`
        BuildHelper::builder("../tmp/mavlink")
            .set_sources(&[Path::new("../message_definitions").join("extra")]);
    }
}
