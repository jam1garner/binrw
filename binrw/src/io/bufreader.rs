//! Wrapper type to add buffering to read streams.

use super::SeekFrom;
use core::convert::TryFrom;

/// A wrapper to add buffering to a read stream.
///
/// Unlike [`std::io::BufReader`], this wrapper does not invalidate the read
/// buffer every time a [`Seek`](super::Seek) method is called. It also caches
/// the underlying stream position to avoid unnecessary system calls.
///
/// # Limitations
///
/// Reading or seeking the wrapped stream object directly will cause an
/// inconsistency in the internal state of the `BufReader`. Calling
/// [`BufReader::seek_invalidate`] will clear the read buffer and reset the
/// internal state to be consistent with the wrapped stream.
pub struct BufReader<T> {
    inner: std::io::BufReader<T>,
    pos: Option<u64>,
}

impl<T: super::Read> BufReader<T> {
    /// Creates a new `BufReader<T>` with a default buffer capacity.
    pub fn new(inner: T) -> BufReader<T> {
        BufReader {
            inner: std::io::BufReader::new(inner),
            pos: None,
        }
    }

    /// Creates a new `BufReader<T>` with the specified buffer capacity.
    pub fn with_capacity(capacity: usize, inner: T) -> BufReader<T> {
        BufReader {
            inner: std::io::BufReader::with_capacity(capacity, inner),
            pos: None,
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
        let n = super::Seek::seek(&mut self.inner, pos)?;
        self.pos = Some(n);
        Ok(n)
    }
}

impl<T: super::Read> super::Read for BufReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> super::Result<usize> {
        let n = self.inner.read(buf)?;
        if let Some(pos) = &mut self.pos {
            *pos += n as u64;
        }
        Ok(n)
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> super::Result<usize> {
        let n = self.inner.read_vectored(bufs)?;
        if let Some(pos) = &mut self.pos {
            *pos += n as u64;
        }
        Ok(n)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> super::Result<usize> {
        let n = self.inner.read_to_end(buf)?;
        if let Some(pos) = &mut self.pos {
            *pos += n as u64;
        }
        Ok(n)
    }

    fn read_to_string(&mut self, buf: &mut String) -> super::Result<usize> {
        let n = self.inner.read_to_string(buf)?;
        if let Some(pos) = &mut self.pos {
            *pos += n as u64;
        }
        Ok(n)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> super::Result<()> {
        self.inner.read_exact(buf)?;
        if let Some(pos) = &mut self.pos {
            *pos += buf.len() as u64;
        }
        Ok(())
    }
}

impl<T: super::Seek> super::Seek for BufReader<T> {
    fn seek(&mut self, pos: SeekFrom) -> super::Result<u64> {
        let old = self.stream_position()?;

        match pos {
            SeekFrom::Start(n) => {
                if old != n {
                    let rel_n = if n >= old {
                        i64::try_from(n - old)
                    } else {
                        i64::try_from(old - n).map(|n| -n)
                    };

                    let n = if let Ok(rel_n) = rel_n {
                        self.seek(SeekFrom::Current(rel_n))?
                    } else {
                        self.inner.seek(pos)?
                    };

                    self.pos = Some(n);
                    Ok(n)
                } else {
                    Ok(old)
                }
            }
            SeekFrom::End(_) => {
                let n = self.inner.seek(pos)?;
                self.pos = Some(n);
                Ok(n)
            }
            SeekFrom::Current(rel_n) => {
                if rel_n != 0 {
                    // https://github.com/rust-lang/rust/issues/87840
                    let n = if rel_n >= 0 {
                        old.checked_add(rel_n as u64)
                    } else {
                        old.checked_sub(rel_n.unsigned_abs())
                    };

                    if let Some(n) = n {
                        self.inner.seek_relative(rel_n)?;
                        self.pos = Some(n);
                        Ok(n)
                    } else {
                        Err(super::Error::new(
                            super::ErrorKind::InvalidInput,
                            "invalid seek to a negative or overflowing position",
                        ))
                    }
                } else {
                    Ok(old)
                }
            }
        }
    }

    fn stream_position(&mut self) -> super::Result<u64> {
        Ok(match self.pos {
            None => {
                let pos = self.inner.stream_position()?;
                self.pos = Some(pos);
                pos
            }
            Some(pos) => pos,
        })
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
