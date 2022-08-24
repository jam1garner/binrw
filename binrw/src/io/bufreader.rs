//! Wrapper type for [`std::io::BufReader`] with improved performance
//! characteristics.

use super::SeekFrom;
use core::convert::TryFrom;

/// A wrapper for [`std::io::BufReader`] that does not invalidate the read
/// buffer any time a seek occurs.
pub struct BufReader<T> {
    inner: std::io::BufReader<T>,
    pos: u64,
}

impl<T: super::Read> BufReader<T> {
    /// Creates a new `BufReader<T>` with a default buffer capacity.
    pub fn new(inner: T) -> BufReader<T> {
        BufReader {
            inner: std::io::BufReader::new(inner),
            pos: 0,
        }
    }

    /// Creates a new `BufReader<T>` with the specified buffer capacity.
    pub fn with_capacity(capacity: usize, inner: T) -> BufReader<T> {
        BufReader {
            inner: std::io::BufReader::with_capacity(capacity, inner),
            pos: 0,
        }
    }
}

impl<T> BufReader<T> {
    /// Returns a reference to the internally buffered data.
    pub fn buffer(&self) -> &[u8] {
        self.inner.buffer()
    }

    /// Returns the number of bytes the internal buffer can hold at once.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader as it
    /// will, at the least, break the cached position information.
    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }

    /// Gets a reference to the underlying reader.
    pub fn get_ref(&self) -> &T {
        self.inner.get_ref()
    }

    /// Unwraps this `BufReader<T>`, returning the underlying reader.
    ///
    /// Note that any leftover data in the internal buffer is lost. Therefore,
    /// a following read from the underlying reader may lead to data loss.
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

impl<T: super::Seek> BufReader<T> {
    /// Performs a seek that forces invalidation of the buffer and internal
    /// position state.
    pub fn seek_invalidate(&mut self, pos: SeekFrom) -> super::Result<u64> {
        self.pos = super::Seek::seek(&mut self.inner, pos)?;
        Ok(self.pos)
    }
}

impl<T: super::Read> super::Read for BufReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> super::Result<usize> {
        let n = self.inner.read(buf)?;
        self.pos += n as u64;
        Ok(n)
    }

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

impl<T: super::Seek> super::Seek for BufReader<T> {
    fn seek(&mut self, pos: SeekFrom) -> super::Result<u64> {
        match pos {
            SeekFrom::Start(n) => {
                if self.pos != n {
                    let rel_n = if n >= self.pos {
                        i64::try_from(n - self.pos)
                    } else {
                        i64::try_from(self.pos - n).map(|n| -n)
                    };

                    if let Ok(rel_n) = rel_n {
                        self.pos = self.seek(SeekFrom::Current(rel_n))?;
                    } else {
                        self.pos = self.inner.seek(pos)?;
                    }
                }
            }
            SeekFrom::End(_) => {
                self.pos = self.inner.seek(pos)?;
            }
            SeekFrom::Current(n) => {
                if n != 0 {
                    // https://github.com/rust-lang/rust/issues/87840
                    let pos = if n >= 0 {
                        self.pos.checked_add(n as u64)
                    } else {
                        self.pos.checked_sub(n.unsigned_abs())
                    };

                    if let Some(pos) = pos {
                        self.inner.seek_relative(n)?;
                        self.pos = pos;
                    } else {
                        return Err(super::Error::new(
                            super::ErrorKind::InvalidInput,
                            "invalid seek to a negative or overflowing position",
                        ));
                    }
                }
            }
        }

        Ok(self.pos)
    }
}

impl<T: super::Read> std::io::BufRead for BufReader<T> {
    fn fill_buf(&mut self) -> super::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt)
    }
}
