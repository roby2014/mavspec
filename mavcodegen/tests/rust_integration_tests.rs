#[cfg(feature = "rust")]
mod tests {
    use std::fs::remove_dir_all;
    use std::path::{Path, PathBuf};

    const MODULE_PATH: &str = "mavlib::mavlink";

    fn xml_definition_paths() -> Vec<String> {
        vec![
            "../message_definitions/standard".to_string(),
            "../message_definitions/extra".to_string(),
        ]
    }

    fn out_path() -> PathBuf {
        Path::new("../tmp/mavlink").to_path_buf()
    }

    #[test]
    fn generate() {
        use mavcodegen::rust::{RustGenerator, RustGeneratorParams};
        use mavspec::parser::XMLInspector;

        let protocol = XMLInspector::builder()
            .set_sources(xml_definition_paths())
            .build()
            .unwrap()
            .parse()
            .unwrap();

        let generator = RustGenerator::new(
            protocol,
            out_path(),
            RustGeneratorParams {
                module_path: MODULE_PATH.to_string(),
                serde: true,
            },
        );
        generator.generate().unwrap();

        // Clean up
        remove_dir_all(out_path()).unwrap();
    }
}
