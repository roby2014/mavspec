use bitflags::bitflags;
use mavspec::rust::spec::IntoPayload;

use mavspec::rust::derive::{Enum, Message};

const ONE: u8 = 1;
const FIVE: usize = 5;

bitflags! {
    #[derive(Clone, Copy, Debug, Default)]
    struct MyFlags: u8 {
        const FLAG_8 = 1;
        const FLAG_7 = 1 << 1;
        const FLAG_6 = 1 << 2;
        const FLAG_5 = 1 << 3;
        const FLAG_4 = 1 << 4;
        const FLAG_3 = 1 << 5;
        const FLAG_2 = 1 << 6;
        const FLAG_1 = 1 << 7;
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, Enum)]
enum MyEnum {
    #[default]
    OptionA = 0,
    OptionB = ONE,
    OptionC = 2,
    OptionD = 3,
    OptionE = 4,
}

#[derive(Clone, Debug, Message)]
#[message_id(40)]
#[crc_extra(32)]
struct MyMessage {
    scalar_i16: i16,
    array_u16_5: [u16; FIVE],
    array_u8_40: [u16; 40],

    #[base_type(u8)]
    enum_scalar_u8: MyEnum,

    #[base_type(u8)]
    enum_array_u8_4: [MyEnum; 4],
    #[base_type(u8)]
    enum_array_u8_40: [MyEnum; 40],

    #[base_type(u16)]
    #[repr_type(u8)]
    enum_large_scalar_u16: MyEnum,

    #[base_type(u16)]
    #[repr_type(u8)]
    enum_large_array_u16_4: [MyEnum; 4],

    #[base_type(f32)]
    #[repr_type(u8)]
    enum_f32: MyEnum,

    #[base_type(f32)]
    #[repr_type(u8)]
    enum_large_array_f64_5: [MyEnum; FIVE],

    #[bitmask]
    #[base_type(u8)]
    bitmask_scalar_u8: MyFlags,

    #[bitmask]
    #[base_type(u16)]
    #[repr_type(u8)]
    bitmask_large_scalar_u16: MyFlags,

    #[bitmask]
    #[base_type(u16)]
    #[repr_type(u8)]
    bitmask_large_array_u16_5: [MyFlags; FIVE],

    #[extension]
    ext_array_u32_4: [u32; 4],
    #[extension]
    ext_scalar_f64: f64,
}

fn make_message() -> MyMessage {
    MyMessage {
        enum_f32: MyEnum::OptionB,
        enum_array_u8_4: [
            MyEnum::OptionA,
            MyEnum::OptionB,
            MyEnum::OptionC,
            MyEnum::OptionD,
        ],
        enum_large_array_u16_4: [
            MyEnum::OptionA,
            MyEnum::OptionB,
            MyEnum::OptionC,
            MyEnum::OptionD,
        ],
        enum_large_array_f64_5: [
            MyEnum::OptionA,
            MyEnum::OptionB,
            MyEnum::OptionC,
            MyEnum::OptionD,
            MyEnum::OptionE,
        ],
        enum_large_scalar_u16: MyEnum::OptionC,
        enum_scalar_u8: MyEnum::OptionD,
        ..Default::default()
    }
}

fn encode_decode() {
    let message = make_message();
    log::info!("Message: {message:#?}");

    let payload = message
        .encode(mavspec::rust::spec::MavLinkVersion::V2)
        .unwrap();

    let decoded_message = MyMessage::try_from(&payload).unwrap();
    log::info!("Decoded message: {decoded_message:#?}");
}

pub fn main() {
    // Setup logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Info) // Suppress everything below `info` for third-party modules.
        .filter_module(env!("CARGO_PKG_NAME"), log::LevelFilter::Trace) // Allow everything from current package
        .init();

    // Encode/decode custom message
    encode_decode();
}

#[test]
fn test_example() {
    encode_decode();
}
