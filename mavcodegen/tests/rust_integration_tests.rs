#[cfg(feature = "rust")]
mod tests {
    use std::path::{Path, PathBuf};

    const MODULE_PATH: &str = "mavlib::mavlink";

    fn default_xml_definition_paths() -> Vec<String> {
        vec![
            "../message_definitions/standard".to_string(),
            "../message_definitions/extra".to_string(),
        ]
    }

    fn default_out_path() -> PathBuf {
        Path::new("../tmp/mavlib/src/mavlink").to_path_buf()
    }

    #[test]
    fn generate() {
        use mavcodegen::rust::{RustGenerator, RustGeneratorParams};
        use mavspec::parser::XMLInspector;

        let protocol = XMLInspector::new(default_xml_definition_paths())
            .unwrap()
            .parse()
            .unwrap();

        let generator = RustGenerator::new(
            protocol,
            default_out_path(),
            RustGeneratorParams {
                module_path: MODULE_PATH.to_string(),
                serde: true,
            },
        );
        generator.generate().unwrap();
    }
}
