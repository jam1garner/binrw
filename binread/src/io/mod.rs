//! A swappable version of [std::io](std::io) that works in `no_std + alloc` environments.
//! If the feature flag `std` is enabled (as it is by default), this will just re-export types from `std::io`.
pub mod prelude;
pub mod cursor;
pub mod error;

#[cfg(feature = "std")]
pub use std::io::{Error, ErrorKind};

#[cfg(not(feature = "std"))]
pub use error::{Error, ErrorKind};

#[cfg(feature = "std")]
pub use std::io::Result;

#[cfg(not(feature = "std"))]
pub type Result<T> = core::result::Result<T, Error>;

/// A simplified version of [std::io::Read](std::io::Read) for use in no_std environments
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        if let Ok(n) = self.read(buf) {
            if n == buf.len() {
                return Ok(())
            }
        }

        Err(Error::new(ErrorKind::UnexpectedEof, "Out of bytes in reader"))
    }

    fn iter_bytes(&mut self) -> Bytes<'_, Self>
        where Self: Sized,
    {
        Bytes {
            inner: self
        }
    }
}

pub struct Bytes<'a, R: Read> {
    inner: &'a mut R
}

impl<'a, R: Read> Iterator for Bytes<'a, R> {
    type Item = Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut byte = [0u8];
        Some(
            self.inner.read_exact(&mut byte)
                .map(|_| byte[0])
        )
    }
}

#[cfg(feature = "std")]
pub use std::io::SeekFrom;

#[cfg(not(feature = "std"))]
#[derive(Debug, Clone, Copy)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
}

#[cfg(feature = "std")]
impl<R: std::io::Read> Read for R {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.read(buf)
    }
}

#[cfg(feature = "std")]
impl<S: std::io::Seek> Seek for S {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.seek(pos)
    }
}

#[cfg(feature = "std")]
pub use std::io::Cursor;

#[cfg(not(feature = "std"))]
pub use cursor::Cursor;
