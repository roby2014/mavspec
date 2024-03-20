use mavspec::rust::derive::{Dialect, Enum, Message};
use mavspec::rust::spec::{IntoPayload, MavLinkVersion};

#[repr(u8)]
#[derive(Copy, Clone, Debug, Default, Enum)]
enum Mood {
    #[default]
    Serious = 0,
    Grumpy = 1,
    Delighted = 2,
    Confused = 3,
}

#[derive(Clone, Debug, Message)]
#[message_id(42)]
struct Howdy {
    #[base_type(u8)]
    mood: Mood,
}

#[derive(Clone, Debug, Message)]
#[message_id(43)]
struct Good {
    #[base_type(u8)]
    mood: Mood,
}

#[derive(Clone, Debug, Message)]
#[message_id(44)]
struct AndYou {
    #[base_type(u8)]
    mood: Mood,
}

mod extra {
    use super::*;

    #[derive(Clone, Debug, Message)]
    #[message_id(1001)]
    pub struct FeelingBad {
        #[base_type(u8)]
        mood: Mood,
    }

    #[derive(Clone, Debug, Message)]
    #[message_id(1002)]
    pub struct HelpPackage {
        #[base_type(u8)]
        mood: Mood,
    }

    #[derive(Clone, Debug, Message)]
    #[message_id(1003)]
    pub struct Thanks {
        #[base_type(u8)]
        mood: Mood,
    }
}

#[derive(Dialect)]
#[dialect(1099)]
#[version(99)]
enum SmallTalk {
    Howdy(Howdy),
    Good(Good),
    AndYou(AndYou),
    FeelingBad(extra::FeelingBad),
    HelpPackage(extra::HelpPackage),
    Thanks(extra::Thanks),
}

fn encode_decode() {
    use mavspec::rust::spec::Dialect;

    let message = SmallTalk::Howdy(Howdy {
        mood: Mood::Delighted,
    });

    let payload = message.encode(MavLinkVersion::V2).unwrap();

    let decoded_message = SmallTalk::decode(&payload).unwrap();

    match decoded_message {
        SmallTalk::Howdy(_) => {}
        _ => panic!("invalid message!"),
    }
}

pub fn main() {
    // Setup logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Info) // Suppress everything below `info` for third-party modules.
        .filter_module(env!("CARGO_PKG_NAME"), log::LevelFilter::Trace) // Allow everything from current package
        .init();

    // Encode/decode custom dialect
    encode_decode();
}

#[test]
fn test_example() {
    encode_decode();
}
