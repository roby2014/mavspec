mod tests {
    use std::collections::HashSet;
    use std::fs::remove_dir_all;
    use std::path::PathBuf;

    use mavinspect::protocol::Microservices;
    use mavinspect::Inspector;

    use mavspec_rust_gen::BuildHelper;

    const CARGO_MANIFEST_PATH_TESTS: &str = "../tests/rust/Cargo.toml";
    const CARGO_MANIFEST_PATH_EXAMPLES: &str = "../examples/rust/Cargo.toml";

    fn xml_definition_paths() -> Vec<&'static str> {
        vec![
            "./message_definitions/standard",
            "./message_definitions/extra",
        ]
    }

    fn out_path() -> PathBuf {
        PathBuf::from("../tmp/mavlink")
    }

    #[test]
    fn generate_rust_from_protocol() {
        let out_path = out_path().join("from_protocol");

        if let Err(err) = remove_dir_all(&out_path) {
            log::debug!("Can't delete temporary directory '{out_path:?}': {err:?}. Proceed.");
        }

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

        remove_dir_all(out_path).unwrap();
    }

    #[test]
    fn generate_rust_from_manifest() {
        let out_path = out_path().join("from_manifest");

        if let Err(err) = remove_dir_all(&out_path) {
            log::debug!("Can't delete temporary directory '{out_path:?}': {err:?}. Proceed.");
        }

        let helper = BuildHelper::builder(&out_path)
            .set_sources(&xml_definition_paths())
            .set_manifest_path(CARGO_MANIFEST_PATH_TESTS)
            .build()
            .unwrap();

        assert!(helper.generate_tests());
        assert_eq!(
            HashSet::from_iter(helper.messages().unwrap().iter().copied()),
            HashSet::from([
                "DEFAULT",
                "1ST_CLASS_MESSAGE",
                "PROTOCOL_VERSION",
                "TRY_FROM",
                "DEBUG",
                "MAV_INSPECT_V1",
                "INTO",
                "CLONE",
                "FROM",
                "TRY_INTO",
                "COPY",
            ])
        );
        assert!(helper
            .microservices()
            .unwrap()
            .contains(Microservices::HEARTBEAT));
        assert!(helper
            .microservices()
            .unwrap()
            .contains(Microservices::COMMAND));

        helper.generate().unwrap();
        remove_dir_all(out_path).unwrap();
    }

    #[test]
    fn test_examples_rust_generation() {
        let out_path = out_path().join("examples_rust_generation");
        let included_dialects = vec!["MAVInspect_test".to_string()];
        let sources = xml_definition_paths();
        let serde_feature_enabled = true;

        BuildHelper::builder(&out_path)
            .set_sources(&sources)
            .set_manifest_path(CARGO_MANIFEST_PATH_EXAMPLES)
            .set_include_dialects(&included_dialects)
            .set_serde(serde_feature_enabled)
            .generate()
            .unwrap();

        remove_dir_all(out_path).unwrap();
    }
}
