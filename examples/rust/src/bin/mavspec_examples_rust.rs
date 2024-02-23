#[cfg(any(feature = "minimal", feature = "common", feature = "mav_inspect_test"))]
use mavspec::rust::spec::MavLinkVersion;

/// Constructs messages from various MAVLink dialects.
///
/// Dialects are managed by corresponding Cargo features:
///
/// * `minimal`
/// * `common`
/// * `mav_inspect_test`
pub fn play_with_dialects() {
    #[cfg(feature = "minimal")]
    {
        use dialect::enums::{MavAutopilot, MavModeFlag, MavState, MavType};
        use mavspec::rust::spec::{Dialect, IntoPayload};
        use mavspec_examples_rust::dialects::minimal as dialect;

        let message = dialect::messages::Heartbeat {
            type_: MavType::FixedWing,
            autopilot: MavAutopilot::Generic,
            base_mode: MavModeFlag::TEST_ENABLED | MavModeFlag::MANUAL_INPUT_ENABLED,
            custom_mode: 0,
            system_status: MavState::Active,
            mavlink_version: dialect::Minimal::version().unwrap(),
        };

        log::info!("{message:#?}");

        let payload = dialect::Minimal::encode(&message.into(), MavLinkVersion::V2).unwrap();
        log::info!("Payload for `Heartbeat` message: {payload:?}");

        let decoded_message = dialect::Minimal::decode(&payload).unwrap();
        if let dialect::Minimal::Heartbeat(message) = decoded_message {
            log::info!("`Heartbeat` message decoded from payload: {message:#?}");
        }
    }

    #[cfg(feature = "common")]
    {
        use dialect::enums::{MavCmd, MavFrame, SpeedType};
        use mavspec::rust::spec::{Dialect, IntoPayload};
        use mavspec_examples_rust::dialects::common as dialect;

        let message = dialect::messages::CommandInt {
            target_system: 10,
            target_component: 1,
            frame: MavFrame::Global,
            command: MavCmd::DoChangeSpeed,
            current: 0,
            autocontinue: 0,
            param1: (SpeedType::Airspeed as u8) as f32,
            param2: 40.0, // 40 m/s
            param3: 70.0, // 70%
            param4: 0.0,
            x: 0,
            y: 0,
            z: 0.0,
        };

        log::info!("{message:#?}");

        let payload = dialect::Common::encode(&message.into(), MavLinkVersion::V2).unwrap();
        log::info!("Payload for `CommandInt` message: {payload:?}");

        let decoded_message = dialect::Common::decode(&payload).unwrap();
        if let dialect::Common::CommandInt(message) = decoded_message {
            log::info!("`CommandInt` message decoded from payload: {message:#?}");
        }
    }

    #[cfg(feature = "mav_inspect_test")]
    {
        use dialect::enums::{SmallBitmask, SmallEnum, _1stClassCitizen, _2ndChanceFlags};
        use mavspec::rust::spec::{Dialect, IntoPayload};
        use mavspec_examples_rust::dialects::mav_inspect_test as dialect;

        let message = dialect::messages::MavInspectV1 {
            plain_uint8: 10,
            plain_int16: -1000,
            small_array: [1, 2, 3, 4],
            large_array: [500; 40],
            small_enum_native: SmallEnum::First,
            small_bitmask_native: SmallBitmask::FIRST | SmallBitmask::SECOND,

            // Inconvenient names
            //
            // Rust keyword is suffixed with underscore
            type_: 1,
            // Entities that starts with numeric characters are prefixed with underscore
            _1st_class_citizen: _1stClassCitizen::_1stOption,
            // Same for bitmap flags
            _2nd_chance_flags: _2ndChanceFlags::_1ST_FLAG,

            // And so forth (lots of test cases)
            ..Default::default()
        };

        log::info!("{message:#?}");

        let payload = dialect::MavInspectTest::encode(&message.into(), MavLinkVersion::V2).unwrap();
        log::info!("Payload for `MavInspectV1` message: {payload:?}");

        let decoded_message = dialect::MavInspectTest::decode(&payload).unwrap();
        if let dialect::MavInspectTest::MavInspectV1(message) = decoded_message {
            log::info!("`MavInspectV1` message decoded from payload: {message:#?}");
        }
    }
}

fn main() {
    // Setup logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Info) // Suppress everything below `info` for third-party modules.
        .filter_module(env!("CARGO_PKG_NAME"), log::LevelFilter::Trace) // Allow everything from current package
        .init();

    // Play with messages from various dialects
    play_with_dialects()
}

#[cfg(test)]
mod tests {
    #[test]
    fn rust_example_test() {
        super::play_with_dialects();
    }
}
