use super::{Error, ErrorKind, Read, Result, Seek, SeekFrom};

/// A `Cursor` wraps an in-memory buffer and provides it with a
/// [`Seek`] implementation.
#[derive(Clone, Debug, Default)]
pub struct Cursor<T: AsRef<[u8]>> {
    inner: T,
    pos: u64,
}

impl<T: AsRef<[u8]>> Cursor<T> {
    /// Gets a mutable reference to the underlying value in this cursor.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Gets a reference to the underlying value in this cursor.
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Consumes this cursor, returning the underlying value.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Creates a new cursor wrapping the provided underlying in-memory buffer.
    pub fn new(inner: T) -> Self {
        Self { inner, pos: 0 }
    }

    /// Returns the current position of this cursor.
    pub fn position(&self) -> u64 {
        self.pos
    }

    /// Sets the position of this cursor.
    pub fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

impl<T: AsRef<[u8]>> Read for Cursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let slice = self.inner.as_ref();
        if self.pos > slice.len() as u64 {
            return Ok(0);
        }
        let amt = u64::min(slice.len() as u64 - self.pos, buf.len() as u64);
        buf[..amt as usize].copy_from_slice(&slice[self.pos as usize..(self.pos + amt) as usize]);
        self.pos += amt;
        Ok(amt as usize)
    }
}

impl<T: AsRef<[u8]>> Seek for Cursor<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::Current(x) => {
                if (self.pos as i64) + x < 0 {
                    Err(Error::new(
                        ErrorKind::InvalidInput,
                        "invalid seek to a negative or overflowing position",
                    ))
                } else {
                    self.pos = ((self.pos as i64) + x) as u64;
                    Ok(self.pos)
                }
            }
            SeekFrom::Start(x) => {
                self.pos = x;
                Ok(self.pos)
            }
            SeekFrom::End(x) => {
                let end = self.inner.as_ref().len() as i64;
                if (self.pos as i64) + end + x < 0 {
                    Err(Error::new(
                        ErrorKind::InvalidInput,
                        "invalid seek to a negative or overflowing position",
                    ))
                } else {
                    Ok(self.pos)
                }
            }
        }
    }
}
