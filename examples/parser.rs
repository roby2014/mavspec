use mavspec::parser::XMLInspector;

fn main() {
    // Setup logger
    env_logger::builder()
        // Suppress everything below `info` for third-party modules.
        .filter_level(log::LevelFilter::Info)
        // Allow everything from current package
        .filter_module(env!("CARGO_PKG_NAME"), log::LevelFilter::Trace)
        .init();

    // Instantiate inspector and load list of XML definitions
    let inspector = XMLInspector::builder()
        .set_sources(vec![
            // Standard definitions from
            // https://github.com/mavlink/mavlink/tree/master/message_definitions/v1.0
            "./message_definitions/standard".to_string(),
            // Extra definitions which depend on standard dialects
            "./message_definitions/extra".to_string(),
        ])
        .set_include(vec![
            "common".to_string(),
            "crazyflight".to_string(),
            "ardupilotmega".to_string(),
            "matrixpilot".to_string(),
        ])
        .set_exclude(vec!["matrixpilot".to_string(), "paparazzi".to_string()])
        .build()
        .unwrap();

    // Parse all XML definitions
    let protocol = inspector.parse().unwrap();

    // Get `crazyflight` custom-defined dialect
    let crazyflight = protocol.dialects().get("crazyflight").unwrap();

    // Get `DUMMYFLIGHT_OUTCRY` message
    let outcry_message = crazyflight.messages().get(&54000u32).unwrap();
    assert_eq!(outcry_message.name(), "CRAZYFLIGHT_OUTCRY");
    log::warn!("\n`CRAZYFLIGHT_OUTCRY` message: {:#?}", outcry_message);

    // Get `HEARTBEAT` message which custom dialect inherits from `standard` dialect
    let heartbeat_message = crazyflight.messages().get(&0u32).unwrap();
    assert_eq!(heartbeat_message.name(), "HEARTBEAT");
    // Verify that `HEARTBEAT` message is defined in `minimal` dialect
    assert_eq!(
        heartbeat_message.defined_in().as_deref().unwrap(),
        "minimal"
    );
}
