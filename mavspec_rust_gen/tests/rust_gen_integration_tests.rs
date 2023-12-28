mod tests {
    use mavspec_rust_gen::BuildHelper;
    use std::collections::HashSet;
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
    fn generate_rust_from_protocol() {
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

    #[test]
    fn generate_rust_from_manifest() {
        let helper = BuildHelper::builder("../tmp/mavlink")
            .set_sources(&xml_definition_paths())
            .set_manifest_path("../tests/rust/Cargo.toml")
            .build()
            .unwrap();

        assert!(helper.generate_tests());
        assert!(!helper.all_enums());
        assert_eq!(
            helper.messages().unwrap(),
            HashSet::from([
                "HEARTBEAT",
                "PROTOCOL_VERSION",
                "MAV_INSPECT_V1",
                "COMMAND_INT",
                "COMMAND_LONG"
            ])
        );

        helper.generate().unwrap();
    }
}
