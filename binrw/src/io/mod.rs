//! A swappable version of [std::io](std::io) that works in `no_std + alloc` environments.
//! If the feature flag `std` is enabled (as it is by default), this will just re-export types from `std::io`.
pub mod prelude;
pub mod error;

#[cfg(not(feature = "std"))]
pub mod cursor;

#[cfg(feature = "std")]
pub use std::io::{Error, ErrorKind};

#[cfg(not(feature = "std"))]
pub use error::{Error, ErrorKind};

#[cfg(feature = "std")]
pub use std::io::Result;

#[cfg(not(feature = "std"))]
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(feature = "std")]
pub use std::io::Read;

/// A simplified version of [std::io::Read](std::io::Read) for use in no_std environments
#[cfg(not(feature = "std"))]
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

    fn bytes(&mut self) -> Bytes<Self>
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
pub use std::io::Write;

#[cfg(not(feature = "std"))]
pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(Error::new(ErrorKind::WriteZero, "failed to write whole buffer"));
                }
                Ok(n) => buf = &buf[n..],
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn write_fmt(&mut self, fmt: core::fmt::Arguments<'_>) -> Result<()> {
        use core::fmt;
        // Create a shim which translates a Write to a fmt::Write and saves
        // off I/O errors. instead of discarding them
        struct Adaptor<'a, T: ?Sized + 'a> {
            inner: &'a mut T,
            error: Result<()>,
        }

        impl<T: Write + ?Sized> fmt::Write for Adaptor<'_, T> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.error = Err(e);
                        Err(fmt::Error)
                    }
                }
            }
        }

        let mut output = Adaptor { inner: self, error: Ok(()) };
        match fmt::write(&mut output, fmt) {
            Ok(()) => Ok(()),
            Err(..) => {
                // check if the error came from the underlying `Write` or not
                if output.error.is_err() {
                    output.error
                } else {
                    Err(Error::new(ErrorKind::Other, "formatter error"))
                }
            }
        }
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
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

#[cfg(feature = "std")]
pub use std::io::Seek;

#[cfg(not(feature = "std"))]
pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
}

#[cfg(feature = "std")]
pub use std::io::Cursor;

#[cfg(not(feature = "std"))]
pub use cursor::Cursor;
