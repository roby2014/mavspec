// #[cfg(test)]
// #[test]
// fn run_as_integration_test() {
//     main().unwrap();
// }

#[cfg(feature = "std")]
fn main() -> mavlib_core::errors::Result<()> {
    use mavlib_core::MavLinkFrame;
    use std::net::TcpStream;

    let mut stream = TcpStream::connect("0.0.0.0:5600")?;

    loop {
        // Decode the entire frame
        let frame = MavLinkFrame::recv(&mut stream)?;

        println!("FRAME #{}: {frame:#?}", frame.header().sequence());
        // println!("MESSAGE #{}: {msg:#?}", frame.header().sequence());
    }
}

#[cfg(not(feature = "std"))]
fn main() -> mavlib_core::errors::Result<()> {}
