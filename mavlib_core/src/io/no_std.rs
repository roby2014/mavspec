//! # `no_std` interfaces for [`mavlib_core`](crate).
//!
//! These interfaces replace [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html)
//! and [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html) from Rust `std`
//! package. They define only a handful of methods required by [`mavlib_core`](crate).
//!
//! In addition to [`Read`] and [`Write`], [`mavlib_core`](crate) defines a `no_std` version of
//! [`std::result::Result`] and a special type of error [`CoreError::Io`] which will be returned by
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
///  * [`std::result::Result`].
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
    /// Pull some bytes from this source into the specified buffer, returning how many bytes were read.
    ///
    /// Mimics the corresponding method from
    /// [`std::io::Read`](https://doc.rust-lang.org/std/io/trait.Read.html).
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] / [`CoreError::NoStdIo`] in case of an error.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

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

    /// Attempts to write an entire buffer into this writer.
    ///
    /// Mimics the corresponding method from
    /// [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html).
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Io`] / [`CoreError::NoStdIo`] in case of an error.
    fn write_all(&mut self, buf: &[u8]) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use crate::consts::{STX_V1, STX_V2};
    use crate::protocol::{Header, MavLinkVersion};
    use crate::utils::SliceReader;

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
        assert!(header.v2_fields().is_none());
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
        assert_eq!(header.v2_fields().unwrap().incompat_flags, 1u8);
        assert_eq!(header.v2_fields().unwrap().compat_flags, 0u8);
        assert_eq!(header.sequence(), 1u8);
        assert_eq!(header.system_id(), 10u8);
        assert_eq!(header.component_id(), 255u8);
        assert_eq!(header.message_id(), 0u32);
    }
}
