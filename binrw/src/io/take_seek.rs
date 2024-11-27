//! Types for seekable reader adapters which limit the number of bytes read from
//! the underlying reader.

use super::{Read, Result, Seek, SeekFrom};
use core::ops::Range;

/// Read adapter which limits the bytes read from an underlying reader, with
/// seek support.
///
/// This struct is generally created by importing the [`TakeSeekExt`] extension
/// and calling [`take_seek`] on a reader.
///
/// [`take_seek`]: TakeSeekExt::take_seek
#[derive(Debug)]
pub struct TakeSeek<T> {
    inner: T,

    /// The range that is allowed to read from inner.
    inner_range: Range<u64>,
}

impl<T> TakeSeek<T> {
    /// Gets a reference to the underlying reader.
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    ///
    /// Care should be taken to avoid modifying the internal I/O state of the
    /// underlying reader as doing so may corrupt the internal limit of this
    /// `TakeSeek`.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consumes this wrapper, returning the wrapped value.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Seek> TakeSeek<T> {
    /// Returns the number of bytes that can be read before this instance will
    /// return EOF.
    ///
    /// # Note
    ///
    /// This instance may reach EOF after reading fewer bytes than indicated by
    /// this method if the underlying [`Read`] instance reaches EOF.
    ///
    /// # Panics
    ///
    /// Panics if the inner stream returns an error from `stream_position`.
    pub fn limit(&mut self) -> u64 {
        let pos = self
            .stream_position()
            .expect("cannot get position for `limit`");

        let inner_pos = self
            .inner_range
            .start
            .checked_add(pos)
            .expect("start + pos to not overflow");

        if self.inner_range.end <= inner_pos {
            0
        } else {
            self.inner_range
                .end
                .checked_sub(inner_pos)
                .expect("end - pos to not overflow")
        }
    }

    /// Sets the number of bytes that can be read before this instance will
    /// return EOF. This is the same as constructing a new `TakeSeek` instance,
    /// so the amount of bytes read and the previous limit value don’t matter
    /// when calling this method.
    ///
    /// # Panics
    ///
    /// Panics if the inner stream returns an error from `stream_position`.
    pub fn set_limit(&mut self, limit: u64) {
        let inner_pos = self
            .inner
            .stream_position()
            .expect("cannot get position for `set_limit`");
        self.inner_range = inner_pos..inner_pos + limit;
    }
}

impl<T: Read + Seek> Read for TakeSeek<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let limit = self.limit();

        // Don't call into inner reader at all at EOF because it may still block
        if limit == 0 {
            return Ok(0);
        }

        // Lint: It is impossible for this cast to truncate because the value
        // being cast is the minimum of two values, and one of the value types
        // is already `usize`.
        #[allow(clippy::cast_possible_truncation)]
        let max = (buf.len() as u64).min(limit) as usize;
        let n = self.inner.read(&mut buf[0..max])?;
        Ok(n)
    }
}

impl<T: Seek> Seek for TakeSeek<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let inner_pos = match pos {
            SeekFrom::Start(offset) => self.inner_range.start.checked_add(offset),
            SeekFrom::End(offset) => self.inner_range.end.checked_add_signed(offset),
            SeekFrom::Current(offset) => self.inner.stream_position()?.checked_add_signed(offset),
        };

        let Some(inner_pos) = inner_pos else {
            return Err(super::Error::new(
                super::ErrorKind::InvalidData,
                "invalid seek to a negative or overflowing position",
            ));
        };

        if inner_pos < self.inner_range.start {
            return Err(super::Error::new(
                super::ErrorKind::InvalidData,
                "invalid seek to a negative position",
            ));
        }

        let inner_pos = self.inner.seek(SeekFrom::Start(inner_pos))?;

        Ok(inner_pos
            .checked_sub(self.inner_range.start)
            .expect("Can't happen"))
    }

    fn stream_position(&mut self) -> Result<u64> {
        let inner_pos = self.inner.stream_position()?;

        match inner_pos.checked_sub(self.inner_range.start) {
            Some(pos) => Ok(pos),
            None => Err(super::Error::new(
                super::ErrorKind::InvalidData,
                "cursor is out of bounds",
            )),
        }
    }
}

/// An extension trait that implements `take_seek()` for compatible streams.
pub trait TakeSeekExt {
    /// Creates an adapter which will read at most `limit` bytes from the
    /// wrapped stream.
    fn take_seek(self, limit: u64) -> TakeSeek<Self>
    where
        Self: Sized;
}

impl<T: Read + Seek> TakeSeekExt for T {
    fn take_seek(mut self, limit: u64) -> TakeSeek<Self>
    where
        Self: Sized,
    {
        let start = self
            .stream_position()
            .expect("cannot get position for `take_seek`");

        TakeSeek {
            inner: self,
            inner_range: start..start + limit,
        }
    }
}
