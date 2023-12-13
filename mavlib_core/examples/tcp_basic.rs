// #[cfg(test)]
// #[test]
// fn run_as_integration_test() {
//     main().unwrap();
// }

#[cfg(feature = "std")]
fn main() -> mavlib_core::errors::Result<()> {
    use dialect::Message;
    use mavlib_core::dialects::minimal as dialect;
    use mavlib_core::Frame;
    use std::net::TcpStream;

    let mut stream = TcpStream::connect("0.0.0.0:5600")?;

    loop {
        // Decode the entire frame
        let frame = Frame::recv(&mut stream)?;
        println!(
            "FRAME #{}: system_id={}, component_id={}",
            frame.sequence(),
            frame.system_id(),
            frame.component_id(),
        );

        // Validate frame in the context of dialect specification (including checksum)
        frame.validate(dialect::spec())?;

        // Decode message
        if let Ok(msg) = dialect::decode(frame.payload()) {
            println!("MESSAGE #{}: {msg:#?}", frame.sequence());

            // Access message fields
            if let Message::Heartbeat(msg) = msg {
                println!(
                    "HEARTBEAT #{}: mavlink_version={:#?}",
                    frame.sequence(),
                    msg.mavlink_version,
                );
            }
        }
    }
}

#[cfg(not(feature = "std"))]
fn main() -> mavlib_core::errors::Result<()> {
    Ok(())
}
