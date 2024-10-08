use std::env::var;
use std::path::Path;
use std::process::Command;

use mavspec::rust::gen::BuildHelper;

/// Updates git submodules.
///
/// It step is specific to MAVSpec repository structure.
fn update_git_submodules() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    if let Err(error) = Command::new("git")
        .arg("submodule")
        .arg("update")
        .arg("--init")
        .current_dir(src_dir)
        .status()
    {
        eprintln!("Unable to update Git submodules: {error}");
    }
}

fn main() {
    update_git_submodules();

    // Use Cargo feature flags to identify which dialects to include
    let included_dialects = {
        let mut included_dialects: Vec<String> = Default::default();

        // List of dialect names (case-sensitive)
        let dialects = vec!["common", "MAVInspect_test", "minimal", "standard"];
        for dialect in dialects {
            // We need to convert dialect name to canonical form before comparing with Cargo feature
            let feature_name =
                mavspec::rust::gen::utils::dialect_module_name(dialect).to_ascii_uppercase();
            // Include dialect if corresponding Cargo feature is enabled
            if var(format!("CARGO_FEATURE_{}", feature_name)).is_ok() {
                included_dialects.push(dialect.to_string())
            }
        }

        included_dialects
    };

    // Path to Cargo manifest directory (package root)
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    // Output path for autogenerated code
    let destination = Path::new(&var("OUT_DIR").unwrap()).join("mavlink");
    // List of directories with MAVLink message definitions.
    //
    // Note that `../../message_definitions` was symlinked into package directory. It is necessary to make
    // `cargo publish` work properly. Although we don't publish example packages, we want to keep them as close as
    // possible to what people might use in production.
    let sources = [
        manifest_dir.join("message_definitions").join("standard"),
        manifest_dir.join("message_definitions").join("extra"),
    ];
    // Path to Cargo.toml
    let manifest_path = manifest_dir.join("Cargo.toml");
    // Serde support feature flag
    let serde_feature_enabled = var("CARGO_FEATURE_SERDE").is_ok();

    // Generate Rust source code for MAVLink dialects
    BuildHelper::builder(destination)
        .set_sources(&sources)
        .set_manifest_path(&manifest_path)
        .set_serde(serde_feature_enabled)
        .set_include_dialects(&included_dialects)
        .generate()
        .unwrap();
}
