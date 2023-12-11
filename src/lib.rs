//! **MAVSpec** is a library for parsing MAVLink
//! [message definitions](https://github.com/mavlink/mavlink/tree/master/message_definitions/v1.0).
//!
//! Repository: [https://gitlab.com/mavka/libs/mavspec](https://gitlab.com/mavka/libs/mavspec).
//!
//! > ### WARNING!!!
//! >
//! > This project is intended to be used with other [Mavka](https://gitlab.com/mavka) tools. For
//! > now its API is still unstable. Once the library will be successfully consumed by these
//! > projects, API will be eventually stabilised.
//!
//! # Usage
//!
//! ```rust
//! use std::env;
//!
//! use mavspec::parser::XMLInspector;
//!
//! // Instantiate inspector and load list of XML definitions
//! let inspector = XMLInspector::builder()
//!     .set_sources(vec![
//!         "./message_definitions/standard".to_string(),
//!         "./message_definitions/extra".to_string(),
//!     ])
//!     .build()
//!     .unwrap();
//!   
//! // Parse all XML definitions
//! let protocol = inspector.parse().unwrap();
//!   
//! // Get `crazyflight` custom dialect
//! let crazyflight = protocol.dialects().get("crazyflight").unwrap();
//!
//! // Get `DUMMYFLIGHT_OUTCRY` message
//! let outcry_message = crazyflight.messages().get(&54000u32).unwrap();
//! assert_eq!(outcry_message.name(), "CRAZYFLIGHT_OUTCRY");
//! println!("\n`CRAZYFLIGHT_OUTCRY` message: {:#?}", outcry_message);
//! ```
//!
//! # Features
//!
//! * `serde` — add [Serde](https://serde.rs) support.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(
    html_logo_url = "https://gitlab.com/mavka/libs/mavspec/-/raw/main/avatar.png?ref_type=heads",
    html_favicon_url = "https://gitlab.com/mavka/libs/mavspec/-/raw/main/avatar.png?ref_type=heads"
)]

// MAVLink protocol parser.
pub mod parser;

// MAVLink protocol entities.
pub mod protocol;

// Common utils
pub mod utils;
