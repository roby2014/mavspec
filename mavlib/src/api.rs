//! # MAVLib API

// General considerations
//
// `Endpoint` is a communication GateWay for MAVLink
// For example: EndpointTCPClient, EndpointTCPServer...
//
// `Node` is a collection of endpoints of various types
// Node receives and sends MAVLink frames from and to its endpoints.
// All endpoints within a Node considered to communicate within one MAVLink namespace.
// Node is responsible for message signing.
// Node has its own system_id/component_id.
//
// Client creates a node and subscribes to messages:
//   - via direct blocking `recv`
//   - via infinite iterator which spits messages
//   - via mpsc channels `events`

pub mod endpoint {
    //! MAVLink endpoints

    use mavlib_core::MavLinkFrame;
    use std::io::{BufReader, BufWriter};
    use std::net::{SocketAddr, TcpStream};

    /// MAVLink endpoint
    pub trait Endpoint: Iterator {
        fn recv(&self) -> MavLinkFrame;
        fn send(&self, frame: MavLinkFrame);
    }

    /// Builder for [`Endpoint`].
    pub trait EndpointBuilder {
        /// Endpoint which going to be built
        type Endpoint: Endpoint;

        /// Commit building process and create an endpoint.
        fn build(&self) -> Self::Endpoint;
    }

    #[derive(Debug)]
    pub struct EndpointTcpClient {
        reader: BufReader<TcpStream>,
        writer: BufWriter<TcpStream>,
    }

    #[derive(Clone, Copy, Debug, Default)]
    pub struct EndpointTcpClientBuilder {
        addr: Option<SocketAddr>,
    }

    impl EndpointTcpClientBuilder {
        pub fn set_addr(&mut self, addr: SocketAddr) -> &mut Self {
            self.addr = Some(addr);
            self
        }
    }

    impl EndpointBuilder for EndpointTcpClientBuilder {
        type Endpoint = EndpointTcpClient;

        fn build(&self) -> Self::Endpoint {
            let stream = TcpStream::connect(self.addr.unwrap()).unwrap();

            EndpointTcpClient {
                reader: BufReader::new(stream.try_clone().unwrap()),
                writer: BufWriter::new(stream),
            }
        }
    }

    impl Iterator for EndpointTcpClient {
        type Item = MavLinkFrame;

        fn next(&mut self) -> Option<Self::Item> {
            todo!()
        }
    }

    impl Endpoint for EndpointTcpClient {
        fn recv(&self) -> MavLinkFrame {
            todo!()
        }

        fn send(&self, frame: MavLinkFrame) {
            todo!()
        }
    }

    impl EndpointTcpClient {
        pub fn from_builder() -> EndpointTcpClientBuilder {
            EndpointTcpClientBuilder::default()
        }
    }
}

pub mod node {
    //! # MAVLink node

    use crate::api::endpoint::Endpoint;
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use mavlib_core::MavLinkFrame;

    pub struct Node {
        endpoints: Vec<Box<dyn Endpoint<Item = MavLinkFrame>>>,
        system_id: u8,
        component_id: u8,
        /* keys and other configurations... */
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use endpoint::{EndpointBuilder, EndpointTcpClient};

    #[test]
    fn basic() {
        let endpoint = EndpointTcpClient::from_builder()
            .set_addr("0.0.0.0:5600".parse().unwrap())
            .build();
    }
}
