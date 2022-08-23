//! Wrapper type that provides a fake [`Seek`](crate::io::Seek) implementation.

use super::{Error, ErrorKind, SeekFrom};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// A wrapper that provides a limited implementation of
/// [`Seek`](crate::io::Seek) for unseekable [`Read`](crate::io::Read) and
/// [`Write`](crate::io::Write) streams.
///
/// This is useful when reading or writing from unseekable streams where binrw
/// does not *actually* need to seek to successfully parse or write the data.
pub struct NoSeek<T> {
    /// The original stream.
    inner: T,
    /// The virtual position of the seekable stream.
    pos: u64,
}

impl<T> NoSeek<T> {
    /// Creates a new seekable wrapper for the given value.
    pub fn new(inner: T) -> Self {
        NoSeek { inner, pos: 0 }
    }

    /// Gets a mutable reference to the underlying value.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Gets a reference to the underlying value.
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Consumes this wrapper, returning the underlying value.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> super::Seek for NoSeek<T> {
    fn seek(&mut self, pos: SeekFrom) -> super::Result<u64> {
        match pos {
            SeekFrom::Start(n) if self.pos == n => Ok(n),
            SeekFrom::Current(n) if n == 0 => Ok(self.pos),
            // https://github.com/rust-lang/rust/issues/86442
            _ => Err(Error::new(ErrorKind::Other, "seek on unseekable file")),
        }
    }

    fn stream_position(&mut self) -> super::Result<u64> {
        Ok(self.pos)
    }
}

impl<T: super::Read> super::Read for NoSeek<T> {
    fn read(&mut self, buf: &mut [u8]) -> super::Result<usize> {
        let n = self.inner.read(buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    #[cfg(feature = "std")]
    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> super::Result<usize> {
        let n = self.inner.read_vectored(bufs)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> super::Result<usize> {
        let n = self.inner.read_to_end(buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn read_to_string(&mut self, buf: &mut String) -> super::Result<usize> {
        let n = self.inner.read_to_string(buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> super::Result<()> {
        self.inner.read_exact(buf)?;
        self.pos += buf.len() as u64;
        Ok(())
    }
}

impl<T: super::Write> super::Write for NoSeek<T> {
    fn write(&mut self, buf: &[u8]) -> super::Result<usize> {
        let n = self.inner.write(buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> super::Result<()> {
        self.inner.flush()
    }

    #[cfg(feature = "std")]
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> super::Result<usize> {
        let n = self.inner.write_vectored(bufs)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn write_all(&mut self, buf: &[u8]) -> super::Result<()> {
        self.inner.write_all(buf)?;
        self.pos += buf.len() as u64;
        Ok(())
    }
}
