use dialect::Message;
use mavlib_core::dialects::minimal as dialect;
use mavlib_core::io::{Read, Write};
use mavlib_core::{Frame, Receiver, Sender};

use mavlib_spec::MavLinkVersion;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

/// Listens to incoming frames and decodes `HEARTBEAT` message.
fn listen<R: Read>(reader: R) -> mavlib_core::errors::Result<()> {
    let mut receiver = Receiver::new(reader);

    loop {
        // Decode the entire frame
        let frame = receiver.recv()?;
        println!(
            "FRAME #{}: system_id={}, component_id={}",
            frame.sequence(),
            frame.system_id(),
            frame.component_id(),
        );

        // Validate frame in the context of dialect specification (including checksum and signature)
        frame.validate(dialect::spec())?;

        // Decode message
        if let Ok(msg) = dialect::decode(frame.payload()) {
            println!("MESSAGE #{}: {msg:#?}", frame.sequence());

            // If heartbeat is sent, print fields
            if let Message::Heartbeat(msg) = msg {
                println!("HEARTBEAT #{}: mavlink_version={msg:#?}", frame.sequence(),);
            }
        }
    }
}

/// Sends several `HEARTBEAT` and then stops
fn send_heartbeats<W: Write>(writer: W) -> mavlib_core::errors::Result<()> {
    // Create an instance of sender
    let mut sender = Sender::new(writer);

    // MAVLink node settings
    let mavlink_version = MavLinkVersion::V2;
    let system_id = 15;
    let component_id = 42;

    // Send several heartbeats
    for sequence in 0..10 {
        let message = dialect::messages::MsgHeartbeat {
            r#type: 6,
            autopilot: 0,
            base_mode: 0,
            custom_mode: 0,
            system_status: 4,
            mavlink_version: 3,
        };

        let frame = Frame::conf()
            .with_message(&message, mavlink_version)?
            .set_sequence(sequence)
            .set_system_id(system_id)
            .set_component_id(component_id)
            .build()?;

        sender.send(&frame)?;

        thread::sleep(Duration::from_millis(1000));
    }

    Ok(())
}

fn main() -> mavlib_core::errors::Result<()> {
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
