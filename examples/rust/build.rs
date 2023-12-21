use std::env::var;
use std::path::Path;

use mavcodegen::rust::BuildHelper;

fn main() {
    let included_dialects = {
        let mut included_dialects: Vec<String> = Default::default();

        let dialects = vec!["common", "MAVSpec_test", "minimal", "standard"];
        for dialect in dialects {
            let feature_name =
                mavcodegen::rust::utils::dialect_module_name(dialect).to_ascii_uppercase();
            if var(format!("CARGO_FEATURE_{}", feature_name)).is_ok() {
                included_dialects.push(dialect.to_string())
            }
        }

        included_dialects
    };

    let sources = vec![
        Path::new("../../message_definitions/standard"),
        Path::new("../../message_definitions/extra"),
    ];
    let destination = Path::new(&var("OUT_DIR").unwrap()).join("mavlink");
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let serde_feature_enabled = var("CARGO_FEATURE_SERDE").is_ok();

    BuildHelper::builder(&sources, &destination)
        .set_manifest_path(&manifest_path)
        .set_serde(serde_feature_enabled)
        .set_dialects(&included_dialects)
        .generate()
        .unwrap();
}
