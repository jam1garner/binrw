pub use super::{cursor::Cursor, error::{Error, ErrorKind}};

pub type Result<T> = core::result::Result<T, Error>;

/// A simplified version of [std::io::Read](std::io::Read) for use in no_std environments
pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        if !buf.is_empty() {
            Err(Error::new(ErrorKind::UnexpectedEof, "failed to fill whole buffer"))
        } else {
            Ok(())
        }
    }

    fn bytes(self) -> Bytes<Self>
    where
        Self: Sized,
    {
        Bytes { inner: self }
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

impl<R: Read + ?Sized> Read for &mut R {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        (**self).read(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        (**self).read_exact(buf)
    }
}

#[derive(Debug)]
pub struct Bytes<R: Read> {
    inner: R
}

impl<R: Read> Iterator for Bytes<R> {
    type Item = Result<u8>;

    fn next(&mut self) -> Option<Result<u8>> {
        let mut byte = 0;
        loop {
            return match self.inner.read(core::slice::from_mut(&mut byte)) {
                Ok(0) => None,
                Ok(..) => Some(Ok(byte)),
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => Some(Err(e)),
            };
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;
}

impl<S: Seek + ?Sized> Seek for &mut S {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        (**self).seek(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_exact() {
        const IN: &[u8] = b"ABCD";
        let mut x = Cursor::new(IN);
        let mut out = [0, 0, 0, 0];
        x.read_exact(&mut out[..]).unwrap();
        assert_eq!(out, IN);
    }

    #[test]
    fn iter_bytes() {
        const IN: &[u8] = b"ABCD";
        let x = Cursor::new(IN);
        let mut x = x.bytes();

        assert_eq!(x.next().unwrap().unwrap(), b'A');
        assert_eq!(x.next().unwrap().unwrap(), b'B');
        assert_eq!(x.next().unwrap().unwrap(), b'C');
        assert_eq!(x.next().unwrap().unwrap(), b'D');
        assert!(x.next().is_none());
        assert!(x.next().is_none());
    }

    #[test]
    fn interupt_once() {
        struct InteruptReader(bool);

        impl Read for InteruptReader {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
                if self.0 {
                    self.0 = false;
                    Err(Error::new(ErrorKind::Interrupted, ()))
                } else {
                    buf.fill(0);
                    Ok(buf.len())
                }
            }
        }

        let mut x = InteruptReader(true);
        let mut out = [1, 2, 3, 4];
        x.read_exact(&mut out).unwrap();

        assert_eq!(out, [0, 0, 0, 0]);

        let mut x = InteruptReader(true).bytes();
        assert_eq!(x.next().unwrap().unwrap(), 0);
        assert_eq!(x.next().unwrap().unwrap(), 0);
        assert_eq!(x.next().unwrap().unwrap(), 0);
        assert_eq!(x.next().unwrap().unwrap(), 0);
    }

    #[test]
    fn return_error() {
        struct ReturnError(Option<Error>);

        impl Read for ReturnError {
            fn read(&mut self, _buf: &mut [u8]) -> Result<usize> {
                Err(self.0.take().unwrap())
            }
        }

        let mut x = ReturnError(Some(Error::new(ErrorKind::ConnectionRefused, ())));
        let mut out = [0, 1, 2, 3];

        assert_eq!(x.read_exact(&mut out).unwrap_err().kind(), ErrorKind::ConnectionRefused);

        let mut x = ReturnError(Some(Error::new(ErrorKind::ConnectionRefused, ()))).bytes();
        assert_eq!(x.next().unwrap().unwrap_err().kind(), ErrorKind::ConnectionRefused);
    }
}
