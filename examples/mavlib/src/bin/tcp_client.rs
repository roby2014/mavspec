use mavlib_core::frame::MavLinkRawFrame;
use mavlib_core::header::MavLinkHeader;
use mavlib_core::stx::MavSTX;
use mavlib_core::{MavLinkFrame, MavLinkVersion};
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

/// Consumes stream up to the magic byte and returns corresponding `MAVLink` protocol version
fn seek_until_frame_start<T: Read>(
    reader: &mut BufReader<&mut T>,
) -> std::io::Result<MavLinkVersion> {
    Ok(loop {
        let received = reader.fill_buf()?;
        let mut skipped_bytes = received.len();
        let mut mavlink_version: Option<MavLinkVersion> = None;

        for (i, &byte) in received.iter().enumerate() {
            if MavSTX::is_magic_byte(byte) {
                skipped_bytes = i;
                mavlink_version = MavLinkVersion::try_from(MavSTX::from(byte)).ok();
                break;
            }
        }
        // Skip all bytes before the magic byte or the entire buffer if no magic byte found
        reader.consume(skipped_bytes);

        // Break and return MAVLink protocol version if valid MAVLink frame is identified
        if let Some(version) = mavlink_version {
            break version;
        }
    })
}

fn decode_header<T: Read>(
    reader: &mut BufReader<&mut T>,
    mavlink_version: MavLinkVersion,
) -> std::io::Result<MavLinkHeader> {
    // Wait until the header is received
    Ok(loop {
        let received = reader.fill_buf()?;
        let header_size = mavlink_version.header_size();

        // Create and return header if enough bytes received
        if received.len() >= header_size {
            let header = MavLinkHeader::try_from(received).unwrap();
            reader.consume(header.size());
            break header;
        }
    })
}

fn decode_frame<T: Read>(
    reader: &mut BufReader<&mut T>,
    header: &MavLinkHeader,
) -> std::io::Result<MavLinkFrame> {
    let body_length = header.expected_body_length().unwrap();

    // Wait until the entire frame received
    Ok(loop {
        let received = reader.fill_buf()?;

        if received.len() >= body_length {
            let body = &received[0..body_length];
            let raw_frame = MavLinkRawFrame::new(&header, &body);
            let frame = MavLinkFrame::try_from(raw_frame).unwrap();

            reader.consume(body_length);
            break frame;
        }
    })
}

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("0.0.0.0:5600")?;
    let mut reader = BufReader::new(&mut stream);

    loop {
        // Seek up to the magic byte
        let mavlink_version = seek_until_frame_start(&mut reader)?;

        // Decode header
        let header = decode_header(&mut reader, mavlink_version)?;

        // Decode the entire frame
        let frame = decode_frame(&mut reader, &header)?;

        // Create a message
        let msg = mavlib::mavlink::dialects::minimal::Message::decode(frame.payload());

        println!("HEADER #{}: {header:#?}", header.sequence());
        println!("FRAME #{}: {frame:#?}", header.sequence());
        println!("MESSAGE #{}: {msg:#?}", header.sequence());
    }
}
