//! # Utils
//!
//! Utility functions, structs and traits which does not fall into any category.

use crate::io::Read;

/// Reads the contents of a predefined slice.
///
/// Receives a pre-defined slice and reads it's contents while moving internal
/// cursor position.
///
/// Works both for `std` and `no_std` targets.
///
/// [`SliceReader`] created mainly for testing purposes. In most cases
/// [`std::io::Cursor`](https://doc.rust-lang.org/std/io/struct.Cursor.html) will be a better
/// alternative. However, since it may have a limited potential use, we've decide to include this
/// struct into `mavlib_core` API.
#[derive(Debug, Default)]
pub struct SliceReader<'a> {
    content: &'a [u8],
    pos: usize,
}

impl<'a> SliceReader<'a> {
    /// Creates [`SliceReader`] from slice.
    pub fn new(content: &'a [u8]) -> Self {
        Self { content, pos: 0 }
    }

    /// Slice content.
    pub fn content(&self) -> &[u8] {
        self.content
    }

    /// Cursor position.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Number of remaining bytes.
    pub fn num_remaining_bytes(&self) -> usize {
        self.content.len() - self.pos
    }

    fn read_internal(&mut self, buf: &mut [u8]) -> usize {
        let num_bytes_requested = buf.len();
        let num_bytes = core::cmp::min(self.content.len() - self.pos, num_bytes_requested);

        buf.copy_from_slice(&self.content[self.pos..self.pos + num_bytes]);
        self.pos += num_bytes;

        num_bytes
    }
}

#[cfg(not(feature = "std"))]
impl<'a> Read for SliceReader<'a> {
    /// Read the exact number of bytes required to fill `buf`.
    ///
    /// # Errors
    ///
    /// Returns [`IoError::UnexpectedEof`](no_std::IoError::UnexpectedEof) if buffer does not ave
    /// enough content.
    fn read_exact(&mut self, buf: &mut [u8]) -> crate::errors::Result<()> {
        // Return error if buffer contains insufficient data
        if self.num_remaining_bytes() < buf.len() {
            return Err(crate::io::no_std::IoError::UnexpectedEof.into());
        }

        self.read_internal(buf);
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<'a> Read for SliceReader<'a> {
    /// Tries to fill `buf` with the remaining [`content`](Self::content).
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let num_bytes = self.read_internal(buf);
        Ok(num_bytes)
    }

    /// Read the exact number of bytes required to fill `buf`.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::UnexpectedEof`](std::io::ErrorKind::UnexpectedEof) if buffer does not
    /// have enough content.
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        // Return error if buffer contains insufficient data
        if self.num_remaining_bytes() < buf.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                format!(
                    "buffer contains only {} bytes but {} requested",
                    self.content.len() - self.pos,
                    buf.len()
                ),
            ));
        }

        self.read_internal(buf);
        Ok(())
    }
}
