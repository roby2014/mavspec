use std::net::TcpStream;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dialect::enums::{MavAutopilot, MavModeFlag, MavState, MavType};
use dialect::Message;
use mavlib_core::dialects::minimal as dialect;
use mavlib_core::io::{Read, Write};

use mavlib_core::consts::{SIGNATURE_SECRET_KEY_LENGTH, SIGNATURE_TIMESTAMP_OFFSET};
use mavlib_core::protocol::MavLinkVersion;
use mavlib_core::protocol::{MavTimestamp, SecretKey, SignatureConf};
use mavlib_core::utils::MavSha256;
use mavlib_core::{Frame, Receiver, Sender};

/// Listen to incoming frames and decode `HEARTBEAT` message.
fn listen<R: Read>(reader: R) -> mavlib_core::errors::Result<()> {
    let mut receiver = Receiver::new(reader);
    let secret_key = secret_key();

    loop {
        // Decode the entire frame
        let frame = receiver.recv()?;

        // Validate frame in the context of dialect specification (including checksum)
        if frame.validate_checksum(dialect::spec()).is_err() {
            log::warn!("INVALID FRAME #{}: {frame:#?}", frame.sequence());
            // Proceed to the next frame
            continue;
        }

        log::info!(
            "FRAME #{}: system_id={}, component_id={}",
            frame.sequence(),
            frame.system_id(),
            frame.component_id(),
        );

        // Calculate and print signature if frame is signed
        if frame.is_signed() {
            let signature = frame.calculate_signature(&mut MavSha256::default(), &secret_key);
            log::info!("{signature:#?}");
        }

        // Decode message
        let decoded = dialect::decode(frame.payload());
        if let Ok(msg) = decoded {
            log::info!("MESSAGE #{}: {msg:?}", frame.sequence());

            // If heartbeat is sent, print fields
            if let Message::Heartbeat(msg) = msg {
                log::info!(
                    "HEARTBEAT #{}: mavlink_version={:#?}",
                    frame.sequence(),
                    msg.mavlink_version
                );
            }
        } else {
            log::warn!("DECODE ERROR #{}: {decoded:?}", frame.sequence());
        }
    }
}

/// Send several `HEARTBEAT` and then stops.
fn send_heartbeats<W: Write>(writer: W) -> mavlib_core::errors::Result<()> {
    // Create an instance of sender
    let mut sender = Sender::new(writer);
    let secret_key = secret_key();

    // MAVLink node settings
    let mavlink_version = MavLinkVersion::V2;
    let system_id = 15;
    let component_id = 42;

    // Send several heartbeats
    for sequence in 0..10 {
        let message = dialect::messages::Heartbeat {
            r#type: MavType::MavTypeFixedWing,
            autopilot: MavAutopilot::MavAutopilotGeneric,
            base_mode: MavModeFlag::MAV_MODE_FLAG_TEST_ENABLED
                & MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
            custom_mode: 0,
            system_status: MavState::MavStateActive,
            mavlink_version: 3,
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let mav_timestamp = timestamp / 10 + SIGNATURE_TIMESTAMP_OFFSET * 10u64.pow(5);

        let mut frame = Frame::builder()
            .set_sequence(sequence)
            .set_system_id(system_id)
            .set_component_id(component_id)
            .build_for(&message, mavlink_version)?;

        frame
            .add_signature(
                &mut MavSha256::default(),
                SignatureConf {
                    link_id: 0,
                    timestamp: MavTimestamp::from_raw_u64(mav_timestamp),
                    secret: secret_key,
                },
            )?
            .remove_signature();

        sender.send(&frame)?;

        thread::sleep(Duration::from_millis(1000));
    }

    Ok(())
}

fn secret_key() -> SecretKey {
    let mut secret_key = [0u8; SIGNATURE_SECRET_KEY_LENGTH];
    secret_key[0..6].copy_from_slice(b"abcdef");
    secret_key
}

fn main() -> mavlib_core::errors::Result<()> {
    // Setup logger
    env_logger::builder()
        // Suppress everything below `info` for third-party modules.
        .filter_level(log::LevelFilter::Info)
        // Allow everything from current package
        .filter_module(env!("CARGO_PKG_NAME"), log::LevelFilter::Trace)
        .init();

    let client = TcpStream::connect("0.0.0.0:5600")?;
    let reader = client.try_clone()?;

    // Spawn a thread which will listen to incoming messages
    thread::spawn(move || -> mavlib_core::errors::Result<()> { listen(reader) });

    send_heartbeats(client)
}

#[cfg(not(feature = "std"))]
fn main() -> mavlib_core::errors::Result<()> {
    Ok(())
}
