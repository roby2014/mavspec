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
    fn generate_rust() {
        use mavinspect::Inspector;
        use mavspec_rust_gen::BuildHelper;

        let out_path = out_path();

        let protocol = Inspector::builder()
            .set_sources(&xml_definition_paths())
            .build()
            .unwrap()
            .parse()
            .unwrap();

        BuildHelper::builder(&out_path)
            .set_protocol(protocol)
            .set_serde(true)
            .set_generate_tests(true)
            .generate()
            .unwrap();

        remove_dir_all(&out_path).unwrap();
    }
}
