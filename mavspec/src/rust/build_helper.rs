use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};

extern crate cargo_manifest;
use cargo_manifest::{Manifest, Value};
use mavinspect::parser::XMLInspector;

use crate::rust::{Generator, GeneratorParams};

/// Code builder for Rust generator.
#[derive(Clone, Debug, Default)]
pub struct BuildHelper {
    sources: Vec<PathBuf>,
    out_path: PathBuf,
    manifest_path: Option<PathBuf>,
    dialects: Option<HashSet<String>>,
    messages: Option<HashSet<String>>,
    all_enums: bool,
    serde: bool,
}

/// Configuration builder for [`BuildHelper`].
#[derive(Clone, Debug, Default)]
pub struct BuildHelperConf(BuildHelper);

impl BuildHelper {
    /// Creates configuration builder for [`BuildHelper`].
    ///
    /// # Arguments
    ///
    /// * `sources` - paths to XML definitions directories.
    /// * `out_path` - output path where sources will be generated.
    pub fn builder<S: ?Sized + AsRef<OsStr>, O: ?Sized + AsRef<OsStr>>(
        sources: &[&S],
        out_path: &O,
    ) -> BuildHelperConf {
        BuildHelperConf(Self {
            sources: sources.iter().map(|&src| PathBuf::from(src)).collect(),
            out_path: PathBuf::from(out_path),
            ..Default::default()
        })
    }

    /// Scans for dialects and generates MAVLink dialects.
    pub fn generate(&self) -> anyhow::Result<()> {
        if let Err(err) = remove_dir_all(&self.out_path) {
            log::debug!("Error while cleaning output directory: {err:?}");
        }

        let mut inspector_builder = XMLInspector::builder();

        let sources: Vec<&Path> = self.sources.iter().map(|s| s.as_path()).collect();
        inspector_builder.set_sources(&sources);

        if let Some(dialects) = &self.dialects {
            inspector_builder.set_include(&Vec::from_iter(dialects.iter().map(|d| d.as_str())));
        }

        let protocol = inspector_builder.build()?.parse()?;

        Generator::new(
            protocol,
            &self.out_path,
            GeneratorParams {
                serde: self.serde,
                messages: self.messages.clone(),
                all_enums: self.all_enums,
            },
        )
        .generate()?;

        Ok(())
    }
}

impl BuildHelperConf {
    /// Default constructor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds [`BuildHelper`] from configuration.
    pub fn build(&self) -> BuildHelper {
        self.0.clone()
    }

    /// Builds [`BuildHelper`] and use it to generates dialects according to configuration.
    pub fn generate(&self) -> anyhow::Result<()> {
        self.0.generate()
    }

    /// Set path to `Cargo.toml` manifest.
    ///
    /// You can control which messages to include by specifying `messages` key in your `Cargo.toml`:
    ///
    /// ```toml
    /// [package.metadata.mavspec]
    /// messages = ["HEARTBEAT", "PROTOCOL_VERSION", "MAV_INSPECT_V1", "COMMAND_INT", "COMMAND_LONG"]
    /// all_enums = false
    /// ```
    ///
    /// The `all_enums` key defines whether only enums required for specified messages will be generated.
    ///
    /// Overrides configuration set by [`Self::set_messages`] and [`Self::set_all_enums`].
    pub fn set_manifest_path<T: ?Sized + AsRef<OsStr>>(&mut self, manifest_path: &T) -> &mut Self {
        self.0.manifest_path = Some(PathBuf::from(manifest_path));
        self.apply_manifest_config();
        self
    }

    /// Set dialects list. Only dialects from this list will be generated.
    ///
    /// This does not apply to included dialects. If specified dialect has `<include>` tag, all these dialects will be
    /// generated as well.
    pub fn set_dialects<T: ToString>(&mut self, dialects: &[T]) -> &mut Self {
        self.0.dialects = Some(HashSet::from_iter(dialects.iter().map(|s| s.to_string())));
        self
    }

    /// Defines which messages will be generated.
    ///
    /// Overrides configuration defined by [`Self::set_manifest_path`].
    pub fn set_messages<T: ToString>(&mut self, messages: &[T]) -> &mut Self {
        self.0.messages = Some(HashSet::from_iter(messages.iter().map(|s| s.to_string())));
        self
    }

    /// Defines whether only enums required for specified messages will be generated.
    ///
    /// Overrides `all_enums` flag set by [`Self::set_manifest_path`].
    ///
    /// Does not have effect if message filtering is not enabled.
    pub fn set_all_enums(&mut self, all_enums: bool) -> &mut Self {
        self.0.all_enums = all_enums;
        self
    }

    /// Enables/disables [Serde](https://serde.rs/) support for generated entities.
    pub fn set_serde(&mut self, serde: bool) -> &mut Self {
        self.0.serde = serde;
        self
    }

    fn apply_manifest_config(&mut self) {
        if let Some(manifest_path) = &self.0.manifest_path {
            let manifest = Manifest::from_path(manifest_path).unwrap();
            if let Some(package) = manifest.package {
                if let Some(metadata) = package.metadata {
                    if let Some(spec) = metadata.get("mavspec") {
                        if let Some(Value::Array(msgs)) = spec.get("messages") {
                            self.0.messages = Some(HashSet::from_iter(
                                msgs.iter().map(|v| v.to_string().replace('"', "")),
                            ));
                        }
                        if let Some(Value::Boolean(all)) = spec.get("all_enums") {
                            self.0.all_enums = *all;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_dir_all;
    use std::path::Path;

    #[test]
    fn test_build_helper_conf_paths() {
        // `&[&str]` sources, `Path` destination
        BuildHelper::builder(
            &[
                "../message_definitions/standard",
                "../message_definitions/extra",
            ],
            &Path::new("../tmp/mavlink"),
        )
        .set_dialects(&["minimal"])
        .generate()
        .unwrap();

        // `Vec<Path>` sources, `&str` destination
        BuildHelper::builder(
            &vec![
                Path::new("../message_definitions/standard"),
                Path::new("../message_definitions/extra"),
            ],
            "../tmp/mavlink",
        )
        .set_dialects(&["minimal"])
        .generate()
        .unwrap();

        remove_dir_all("../tmp/mavlink").unwrap();
    }
}
