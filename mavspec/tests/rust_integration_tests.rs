#[cfg(feature = "rust")]
mod tests {
    use std::fs::remove_dir_all;
    use std::path::PathBuf;

    fn xml_definition_paths() -> Vec<&'static str> {
        vec![
            "../message_definitions/standard",
            "../message_definitions/extra",
        ]
    }

    fn out_path() -> PathBuf {
        PathBuf::from("../tmp/mavlink")
    }

    #[test]
    fn generate() {
        use mavinspect::parser::XMLInspector;
        use mavspec::rust::{Generator, GeneratorParams};

        let out_path = out_path();

        let protocol = XMLInspector::builder()
            .set_sources(&xml_definition_paths())
            .build()
            .unwrap()
            .parse()
            .unwrap();

        let generator = Generator::new(
            protocol,
            &out_path,
            GeneratorParams {
                serde: true,
                ..Default::default()
            },
        );
        generator.generate().unwrap();

        // Clean up
        remove_dir_all(&out_path).unwrap();
    }
}
