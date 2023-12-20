use std::env::var;
use std::path::Path;

use mavcodegen::rust::Builder;

fn main() {
    let included_dialects = {
        let mut included_dialects: Vec<String> = Default::default();

        let dialects = vec![
            "all",
            "ardupilotmega",
            "asluav",
            "avssuas",
            "common",
            "csairlink",
            "cubepilot",
            "development",
            "icarous",
            "matrixpilot",
            "minimal",
            "paparazzi",
            "standard",
            "ualberta",
            "uavionix",
        ];
        for dialect in dialects {
            if var(format!("CARGO_FEATURE_{}", dialect.to_ascii_uppercase())).is_ok() {
                included_dialects.push(dialect.to_string())
            }
        }

        included_dialects
    };

    // let destination = PathBuf::from(var("OUT_DIR").unwrap()).join("mavlink");
    let destination = Path::new("src").join("mavlink");
    let sources = vec![
        Path::new("../message_definitions/standard"),
        Path::new("../message_definitions/extra"),
    ];

    let builder = Builder::new(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
        &sources,
        &destination,
        Some(&included_dialects),
        var("CARGO_FEATURE_SERDE").is_ok(),
    );
    builder.build().unwrap();
}
