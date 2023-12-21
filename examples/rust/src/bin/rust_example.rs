/// Constructs messages from various MAVLink dialects.
///
/// Dialects are managed by corresponding Cargo features:
///
/// * `minimal`
/// * `common`
/// * `mav_spec_test`
pub fn main() {
    #[cfg(feature = "minimal")]
    {
        use dialect::enums::{MavAutopilot, MavModeFlag, MavState, MavType};
        use rust_example::dialects::minimal as dialect;

        let message = dialect::messages::Heartbeat {
            r#type: MavType::MavTypeFixedWing,
            autopilot: MavAutopilot::MavAutopilotGeneric,
            base_mode: MavModeFlag::MAV_MODE_FLAG_TEST_ENABLED
                | MavModeFlag::MAV_MODE_FLAG_MANUAL_INPUT_ENABLED,
            custom_mode: 0,
            system_status: MavState::MavStateActive,
            mavlink_version: dialect::spec().version().unwrap(),
        };

        println!("{message:#?}");
    }

    #[cfg(feature = "common")]
    {
        use dialect::enums::{MavCmd, MavFrame, SpeedType};
        use rust_example::dialects::common as dialect;

        let message = dialect::messages::CommandInt {
            target_system: 10,
            target_component: 1,
            frame: MavFrame::MavFrameGlobal,
            command: MavCmd::MavCmdDoChangeSpeed,
            current: 0,
            autocontinue: 0,
            param1: (SpeedType::SpeedTypeAirspeed as u8) as f32,
            param2: 40.0, // 40 m/s
            param3: 70.0, // 70%
            param4: 0.0,
            x: 0,
            y: 0,
            z: 0.0,
        };

        println!("{message:#?}");
    }

    #[cfg(feature = "mav_spec_test")]
    {
        use dialect::enums::{SmallBitmask, SmallEnum};
        use rust_example::dialects::mav_spec_test as dialect;

        let message = dialect::messages::MavSpecV1 {
            plain_uint8: 10,
            plain_int16: -1000,
            small_array: [1, 2, 3, 4],
            large_array: [500; 40],
            small_enum_native: SmallEnum::SmallEnumFirst,
            small_bitmask_native: SmallBitmask::SMALL_BITMASK_FIRST
                | SmallBitmask::SMALL_BITMASK_SECOND,
            // And so forth (lots of test cases)
            ..Default::default()
        };

        println!("{message:#?}");
    }
}
