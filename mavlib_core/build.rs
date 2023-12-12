use std::env::var;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

use mavcodegen::rust::{RustGenerator, RustGeneratorParams};
use mavspec::parser::XMLInspector;

fn main() {
    // Serde feature
    let has_serde_feature = var("CARGO_FEATURE_SERDE").is_ok();

    // MAVLink dialect features
    let has_minimal_feature = var("CARGO_FEATURE_MINIMAL").is_ok();
    let has_common_feature = var("CARGO_FEATURE_COMMON").is_ok();
    let has_ardupilotmega_feature = var("CARGO_FEATURE_ARDUPILOTMEGA").is_ok();
    let has_all_feature = var("CARGO_FEATURE_ALL").is_ok();

    // Define included dialects
    let included_dialects = {
        let mut included_dialects: Vec<String> = Default::default();

        if has_minimal_feature {
            included_dialects.push("minimal".to_string())
        }
        if has_common_feature {
            included_dialects.push("common".to_string())
        }
        if has_ardupilotmega_feature {
            included_dialects.push("ardupilotmega".to_string())
        }
        if has_all_feature {
            included_dialects.push("all".to_string())
        }

        included_dialects
    };

    // Define destination
    // let destination = PathBuf::from(var("OUT_DIR").unwrap()).join("mavlink");
    let destination = PathBuf::from("src").join("mavlink");

    // Clear all files within destination
    if let Err(e) = remove_dir_all(&destination) {
        eprint!("Error deleting directory: {:?}", e);
    }

    // Create stub if code-generation is no dialects included
    if included_dialects.is_empty() {
        // Ensure that root directory exists
        create_dir_all(&destination).unwrap();

        if !destination.join("mod.rs").exists() {
            let mut output = File::create(destination.join("mod.rs")).unwrap();
            write!(
                output,
                "// # MAVLink definitions (empty)\n/// Dialects\npub mod dialects {{ }}\n"
            )
            .unwrap();
        }

        return;
    }

    // Load dialects
    let sources = vec!["../message_definitions/standard".to_string()];
    let protocol = XMLInspector::builder()
        .set_sources(sources)
        .set_include(included_dialects)
        .build()
        .unwrap()
        .parse()
        .unwrap();

    // Generate source code
    RustGenerator::new(
        protocol,
        destination,
        RustGeneratorParams {
            module_path: "mavlib_core::mavlink".to_string(),
            serde: has_serde_feature,
        },
    )
    .generate()
    .unwrap();
}
