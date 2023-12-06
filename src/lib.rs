//! **MAVSpec** is a library and for parsing MAVLink
//! [message definitions](https://github.com/mavlink/mavlink/tree/master/message_definitions/v1.0).
//!
//! Repository: [https://gitlab.com/mavka/libs/mavspec](https://gitlab.com/mavka/mavspec).
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
