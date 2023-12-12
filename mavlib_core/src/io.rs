//! # Basic MAVLink I/O
//!
//! This module includes basic MAVLink I/O utils for reading and writing frames
//! ([`MavLinkFrame`](crate::Frame)).
//!
//! # Targets
//!
//! For `std` environments [`mavlib_core`](crate) uses
//! [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html)
//! and [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html) reader and writer.
//!
//! For `no_std` [`mavlib_core`](crate) uses custom `Read` and `Write` traits:
//!
//! ```rust
//! use mavlib_core::errors::Result;
//!
//! trait Read {
//!     fn read_exact(&mut self, buf: &mut [u8]) -> Result<()>;
//! }
//!
//! trait Write {
//!     fn write(&mut self, buf: &[u8]) -> Result<usize>;
//! }
//! ```
//!
//! In addition, the following `IoError` error is defined for `no_std`:
//!
//! ```rust
//! #[derive(Clone, Debug)]
//! pub enum IoError {
//!     /// Operation was interrupted.
//!     ///
//!     /// In most cases this means that operation can be retried.
//!     Interrupted,
//!     /// Invalid data received.
//!     InvalidData,
//!     /// This operation is unsupported.
//!     Unsupported,
//!     /// Unexpected end-of-file.
//!     ///
//!     /// In most cases this means that smaller amount of bytes are available.
//!     UnexpectedEof,
//!     /// Other error.
//!     Other(String),
//! }
//! ```
//!
//! This error will be wrapped with `no_std` version of [`CoreError`](crate::errors::CoreError).

#[cfg(not(feature = "std"))]
pub use no_std::{Read, Write};
#[cfg(feature = "std")]
#[doc(hidden)]
pub use std::io::{Read, Write};

#[cfg(not(feature = "std"))]
pub mod no_std {
    //! # `no_std` interfaces for [`mavlib_core`](crate).
    //!
    //! These interfaces replace [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html)
    //! and [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html) from Rust `std`
    //! package. They define only a handful of methods required by [`mavlib_core`](crate).
    //!
    //! In addition to [`Read`] and [`Write`], [`mavlib_core`](crate) defines a `no_std` version of
    //! [`Result`] and a special type of error [`CoreError::Io`] which will be returned by
    //! [`no_std`](self) I/O interfaces. The kinds of returned errors are defined in [`IoError`].

    use crate::errors::{CoreError, Result};

    /// `no_std` I/O errors.
    ///
    /// Errors returned by `no_std` [`mavlib_core`](crate) I/O.
    ///
    /// These errors will be wrapped with [`CoreError::Io`] upon
    /// returning to client.
    ///
    /// See:
    ///  * [`CoreError::Io`].
    ///  * [`Result`].
    #[derive(Clone, Debug)]
    pub enum IoError {
        /// Operation was interrupted.
        ///
        /// In most cases this means that operation can be retried.
        Interrupted,
        /// Invalid data received.
        InvalidData,
        /// This operation is unsupported.
        Unsupported,
        /// Unexpected end-of-file.
        ///
        /// In most cases this means that smaller amount of bytes are available.
        UnexpectedEof,
        /// Other error.
        Other(String),
    }

    #[cfg(not(feature = "std"))]
    impl From<IoError> for CoreError {
        /// Wraps [`IoError`] with [`CoreError::Io`].
        ///
        /// > **Note!** In case of `std` targets, [`IoError`] will be wrapped with [`CoreError::NoStdIo`]
        /// > instead of [`CoreError::Io`].
        fn from(value: IoError) -> Self {
            Self::Io(value)
        }
    }

    /// `no_std` replacement for [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html).
    ///
    /// Since [`mavlib_core`](crate) required only [`read_exact`](Read::read_exact), only this
    /// method is defined here.
    pub trait Read {
        /// Read the exact number of bytes required to fill buf.
        ///
        /// Mimics the corresponding method from
        /// [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html).
        ///
        /// # Errors
        ///
        /// Returns [`CoreError::Io`] / [`CoreError::NoStdIo`] in case of an error.
        fn read_exact(&mut self, buf: &mut [u8]) -> Result<()>;
    }

    /// `no_std` replacement for [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html).
    ///
    /// Since [`mavlib_core`](crate) required only [`write`](Read::write), only this method is
    /// defined here.
    pub trait Write {
        /// Writes the contents from buffer.
        ///
        /// Returns the number of bytes written.
        ///
        /// Mimics the corresponding method from
        /// [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html).
        ///
        /// # Errors
        ///
        /// Returns [`CoreError::Io`] / [`CoreError::NoStdIo`] in case of an error.
        fn write(&mut self, buf: &[u8]) -> Result<usize>;
    }
}

#[cfg(test)]
#[cfg(not(feature = "std"))]
mod tests {
    use crate::consts::{STX_V1, STX_V2};
    use crate::header::Header;
    use crate::utils::SliceReader;
    use mavlib_spec::MavLinkVersion;

    #[test]
    fn read_v1_header() {
        let content = [
            12,     // \
            24,     //  |Junk bytes
            240,    // /
            STX_V1, // magic byte
            8,      // payload_length
            1,      // sequence
            10,     // system ID
            255,    // component ID
            0,      // message ID
        ];
        let mut buffer = SliceReader::new(&content);

        let header = Header::recv(&mut buffer).unwrap();

        assert!(matches!(header.mavlink_version(), MavLinkVersion::V1));
        assert_eq!(header.payload_length(), 8u8);
        assert_eq!(header.sequence(), 1u8);
        assert_eq!(header.system_id(), 10u8);
        assert_eq!(header.component_id(), 255u8);
        assert_eq!(header.message_id(), 0u32);
        assert!(header.mavlink_v2_fields().is_none());
    }

    #[test]
    fn read_v2_header() {
        let content = [
            12,     // \
            24,     //  |Junk bytes
            240,    // /
            STX_V2, // magic byte
            8,      // payload_length
            1,      // incompatibility flags
            0,      // compatibility flags
            1,      // sequence
            10,     // system ID
            255,    // component ID
            0,      // \
            0,      //  | message ID
            0,      // /
        ];
        let mut reader = SliceReader::new(&content);

        let header = Header::recv(&mut reader).unwrap();

        assert!(matches!(header.mavlink_version(), MavLinkVersion::V2));
        assert_eq!(header.payload_length(), 8u8);
        assert_eq!(header.mavlink_v2_fields().unwrap().incompat_flags, 1u8);
        assert_eq!(header.mavlink_v2_fields().unwrap().compat_flags, 0u8);
        assert_eq!(header.sequence(), 1u8);
        assert_eq!(header.system_id(), 10u8);
        assert_eq!(header.component_id(), 255u8);
        assert_eq!(header.message_id(), 0u32);
    }
}
