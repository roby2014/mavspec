use mavspec::parser::errors::XmlInspectionError;
use mavspec::parser::XMLInspector;

fn default_dialect_paths() -> Vec<String> {
    vec![
        "./message_definitions/standard".to_string(),
        "./message_definitions/extra".to_string(),
    ]
}

#[test]
fn naming_collisions_are_avoided() {
    let paths = vec!["./message_definitions/colliding".to_string()];
    let inspector = XMLInspector::new(paths);
    assert!(
        inspector.is_err(),
        "XMLInspector should recognize naming collisions"
    );
    assert!(matches!(
        inspector,
        Err(XmlInspectionError::NamingCollision { .. })
    ))
}

#[test]
fn wrong_paths_do_not_cause_panic() {
    let paths = vec![
        "./message_definitions/invalid".to_string(), // Non-existing
        "./message_definitions/standard".to_string(),
    ];
    let inspector = XMLInspector::new(paths);
    assert!(
        inspector.is_err(),
        "XMLInspector should return error for non-existing paths"
    );
    assert!(matches!(inspector, Err(XmlInspectionError::IoError(_))))
}

#[test]
fn empty_paths_do_not_cause_errors() {
    let paths = vec![
        "./tests".to_string(), // Clearly don't have any definitions
        "./message_definitions/standard".to_string(),
    ];
    let inspector = XMLInspector::new(paths);
    assert!(
        inspector.is_ok(),
        "XMLInspector should non return error for paths without definitions"
    );
}

#[test]
fn xml_definitions_are_loaded() {
    let inspector = XMLInspector::new(default_dialect_paths());
    assert!(inspector.is_ok(), "failed to instantiate XMLInspector");
    let inspector = inspector.unwrap();

    assert!(!inspector.definitions().is_empty(), "no definitions loaded");
}

#[test]
fn default_message_definitions_are_parsed() {
    let inspector = XMLInspector::new(default_dialect_paths()).unwrap();
    let protocol = inspector.parse();
    assert!(protocol.is_ok(), "failed to instantiate XMLInspector");
    let protocol = protocol.unwrap();

    // Check that some dialects are parsed
    assert!(!protocol.dialects().is_empty(), "no dialects parsed");
    // Check that minimal dialect is parsed
    assert!(
        protocol.dialects().get("minimal").is_some(),
        "`minimal` dialect should be parsed"
    );
}

#[test]
fn default_minimal_dialect_is_parsed_correctly() {
    let inspector = XMLInspector::new(default_dialect_paths()).unwrap();
    let protocol = inspector.parse().unwrap();

    let minimal = protocol.dialects().get("minimal").unwrap();

    assert_eq!(minimal.name(), "minimal", "incorrect dialect name");

    assert!(
        minimal.enums().get("MAV_AUTOPILOT").is_some(),
        "`minimal` dialect should contain `MAV_AUTOPILOT` enum"
    );

    assert!(
        minimal.messages().get(&0u32).is_some(),
        "`minimal` dialect should contain `HEARTBEAT` message"
    );
    assert_eq!(
        minimal.messages().get(&0u32).unwrap().name(),
        "HEARTBEAT",
        "`minimal` dialect should contain `HEARTBEAT` message"
    );

    assert!(
        minimal.messages().get(&300u32).is_some(),
        "`minimal` dialect should contain `PROTOCOL_VERSION` message"
    );
    assert_eq!(
        minimal.messages().get(&300u32).unwrap().name(),
        "PROTOCOL_VERSION",
        "`minimal` dialect should contain `PROTOCOL_VERSION` message"
    );
}
