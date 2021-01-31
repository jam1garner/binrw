use super::*;

#[derive(Clone, Debug, Default)]
pub struct Cursor<T: AsRef<[u8]>> {
    inner: T,
    pos: u64
}

impl<T: AsRef<[u8]>> Cursor<T> {
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn new(inner: T) -> Self {
        Self {
            inner,
            pos: 0
        }
    }

    pub fn position(&self) -> u64 {
        self.pos
    }

    pub fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

impl<T: AsRef<[u8]>> Read for Cursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let slice = self.inner.as_ref();
        if self.pos > slice.len() as u64 {
            return Ok(0)
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
                    Err(Error::new(ErrorKind::InvalidInput, "invalid seek to a negative or overflowing position"))
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
                    Err(Error::new(ErrorKind::InvalidInput, "invalid seek to a negative or overflowing position"))
                } else {
                    Ok(self.pos)
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg(std)]
mod test {
    use super::*;

    #[test]
    fn cursor_test() {
        let data = &[1, 2, 3, 4, 5];
        let mut test = Cursor::new(data);
        let mut test2 = std::io::Cursor::new(data);

        assert_eq!(test.get_ref(), test2.get_ref());
        assert_eq!(test.get_mut(), test2.get_mut());
        assert_eq!(test.position(), test2.position());
        assert_eq!(test.position(), test2.position());
        test.set_position(5); test2.set_position(5);
        assert_eq!(test.position(), test2.position());
        test.set_position(5000); test2.set_position(5000);
        assert_eq!(test.position(), test2.position());
        assert_eq!(
            test.seek(SeekFrom::Start(0)).unwrap(),
            test2.seek(std::io::SeekFrom::Start(0)).unwrap(),
        );
        let mut buf = [0u8; 4]; let mut buf2 = [0u8; 4];
        assert_eq!(
            test.read(&mut buf).unwrap(),
            test2.read(&mut buf2).unwrap()
        );
        assert_eq!(buf, buf2);
        assert_eq!(
            test.read(&mut buf).unwrap(),
            test2.read(&mut buf2).unwrap()
        );
        assert_eq!(buf, buf2);
    }
}
