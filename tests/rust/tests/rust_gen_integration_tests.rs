mod tests {
    use mavspec::rust::spec::{MavLinkVersion, MessageSpec};

    #[test]
    fn dialect_is_present() {
        use dialect::enums::{SmallBitmask, SmallEnum};
        use dialect::messages::MavInspectV1;
        use mavspec_tests_rust::dialects::mav_inspect_test as dialect;

        let message = MavInspectV1 {
            plain_uint8: 10,
            plain_int16: -1000,
            small_array: [1, 2, 3, 4],
            large_array: [500; 40],
            small_enum_native: SmallEnum::First,
            small_bitmask_native: SmallBitmask::FIRST | SmallBitmask::SECOND,
            // And so forth (lots of test cases)
            ..Default::default()
        };

        let payload = message.encode(MavLinkVersion::V2).unwrap();

        assert!(matches!(payload.version(), MavLinkVersion::V2));
        assert_eq!(payload.id(), dialect::messages::mav_inspect_v1::spec().id());
    }

    #[test]
    fn inconvenient_names_are_correctly_represented() {
        use dialect::enums::{SmallBitmask, SmallEnum, _1stClassCitizen, _2ndChanceFlags};
        use mavspec_tests_rust::dialects::mav_inspect_test as dialect;

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

        let payload = message.encode(MavLinkVersion::V2).unwrap();

        assert!(matches!(payload.version(), MavLinkVersion::V2));
        assert_eq!(payload.id(), dialect::messages::mav_inspect_v1::spec().id());
    }

    #[test]
    fn derivable_traits() {
        use dialect::messages::MavInspectV1;
        use mavspec_tests_rust::dialects::mav_inspect_test as dialect;

        #[derive(Clone, Debug, Default)]
        struct MessageWrapper(MavInspectV1);

        let msg = MavInspectV1::default();
        let msg_wrapped = MessageWrapper::default();

        assert_eq!(msg.id(), msg_wrapped.0.id());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serde_support() {
        use serde::{Deserialize, Serialize};

        use dialect::messages::MavInspectV1;
        use mavspec_tests_rust::dialects::mav_inspect_test as dialect;

        #[derive(Clone, Debug, Default, Serialize, Deserialize)]
        struct MessageWrapper(MavInspectV1);

        let msg = MavInspectV1::default();
        let msg_wrapped = MessageWrapper::default();

        assert_eq!(msg.id(), msg_wrapped.0.id());
    }
}
