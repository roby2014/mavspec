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

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let included_dialects = {
        let mut included_dialects: Vec<String> = vec!["MAVInspect_test".to_string()];

        let dialects = vec!["common", "minimal", "standard"];
        for dialect in dialects {
            let feature_name =
                mavspec::rust::gen::utils::dialect_module_name(dialect).to_ascii_uppercase();
            if var(format!("CARGO_FEATURE_{}", feature_name)).is_ok() {
                included_dialects.push(dialect.to_string())
            }
        }

        included_dialects
    };

    let destination = Path::new(&var("OUT_DIR").unwrap()).join("mavlink");
    let sources = [
        manifest_dir.join("message_definitions").join("standard"),
        manifest_dir.join("message_definitions").join("extra"),
    ];
    let manifest_path = manifest_dir.join("Cargo.toml");
    let serde_feature_enabled = var("CARGO_FEATURE_SERDE").is_ok();

    BuildHelper::builder(destination)
        .set_sources(&sources)
        .set_manifest_path(&manifest_path)
        .set_include_dialects(&included_dialects)
        .set_serde(serde_feature_enabled)
        .generate()
        .unwrap();
}
