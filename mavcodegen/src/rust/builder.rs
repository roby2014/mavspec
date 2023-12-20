use std::collections::HashSet;
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};

extern crate cargo_manifest;
use cargo_manifest::{Manifest, Value};
use mavspec::parser::XMLInspector;

use crate::rust::{Generator, GeneratorParams};

/// Code builder for Rust generator.
pub struct Builder {
    manifest_path: PathBuf,
    sources: Vec<PathBuf>,
    out_path: PathBuf,
    dialects: Option<HashSet<String>>,
    messages: Option<HashSet<String>>,
    all_enums: bool,
    serde: bool,
}

impl Builder {
    /// Creates an instance of builder.
    ///
    /// # Arguments
    ///
    /// * `manifest_path` - path to `Cargo.toml` manifest.
    /// * `sources` - paths to XML definitions directories.
    /// * `out_path` - output path where sources will be generated.
    /// * `dialects` - dialects to generate.
    /// * `serde` - `serde` support flag .
    pub fn new(
        manifest_path: &Path,
        sources: &[&Path],
        out_path: &Path,
        dialects: Option<&Vec<String>>,
        serde: bool,
    ) -> Self {
        let mut builder = Self {
            manifest_path: PathBuf::from(manifest_path),
            sources: sources.iter().map(|&src| PathBuf::from(src)).collect(),
            out_path: PathBuf::from(out_path),
            messages: None,
            all_enums: false,
            serde,
            dialects: dialects.map(|dialects| HashSet::from_iter(dialects.iter().cloned())),
        };

        builder.apply_manifest_config();

        builder
    }

    /// Scans for dialects and builds Rust sources.
    pub fn build(&self) -> anyhow::Result<()> {
        if let Err(err) = remove_dir_all(&self.out_path) {
            log::debug!("Error while cleaning output directory: {err:?}");
        }

        let mut inspector_builder = XMLInspector::builder();
        inspector_builder.set_sources(
            self.sources
                .iter()
                .map(|s| s.to_str().unwrap().to_string())
                .collect(),
        );

        if let Some(dialects) = &self.dialects {
            inspector_builder.set_include(Vec::from_iter(dialects.iter().cloned()));
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

    fn apply_manifest_config(&mut self) {
        let manifest = Manifest::from_path(&self.manifest_path).unwrap();
        if let Some(package) = manifest.package {
            if let Some(metadata) = package.metadata {
                if let Some(spec) = metadata.get("mavcodegen") {
                    if let Some(Value::Array(msgs)) = spec.get("messages") {
                        self.messages = Some(HashSet::from_iter(
                            msgs.iter().map(|v| v.to_string().replace('"', "")),
                        ));
                    }
                    if let Some(Value::Boolean(all)) = spec.get("all_enums") {
                        self.all_enums = *all;
                    }
                }
            }
        }
    }
}
