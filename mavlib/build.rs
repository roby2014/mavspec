use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

use mavcodegen::rust::{Generator, GeneratorParams};
use mavspec::parser::XMLInspector;

fn main() {
    let has_serde_feature = std::env::var("CARGO_FEATURE_SERDE").is_ok();

    // let destination = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("mavlink");
    let destination = PathBuf::from("src").join("mavlink");

    // Create stub if code-generation is turned off
    // TODO: remove before release
    if std::env::var("ALLOW_AUTOGEN").unwrap_or("false".to_string()) == "false" {
        // Ensure that root directory exists
        create_dir_all(&destination).unwrap();

        if !destination.join("mod.rs").exists() {
            let mut output = File::create(destination.join("mod.rs")).unwrap();
            write!(output, "// # MAVLink definitions stub").unwrap();
        }

        return;
    }

    let sources = vec!["../../message_definitions/extra".to_string()];

    let protocol = XMLInspector::builder()
        .set_sources(sources)
        .build()
        .unwrap()
        .parse()
        .unwrap();

    Generator::new(
        protocol,
        &destination,
        GeneratorParams {
            serde: has_serde_feature,
            ..Default::default()
        },
    )
    .generate()
    .unwrap();
}
