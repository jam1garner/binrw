#![allow(dead_code)]

use binrw::io::{Read, Seek, SeekFrom};

pub struct Fill {
    value: u8,
}

impl Fill {
    pub fn new(value: u8) -> Self {
        Self { value }
    }
}

impl Read for Fill {
    fn read(&mut self, buf: &mut [u8]) -> binrw::io::Result<usize> {
        buf.fill(self.value);
        Ok(buf.len())
    }
}

impl Seek for Fill {
    fn seek(&mut self, _: SeekFrom) -> binrw::io::Result<u64> {
        Ok(0)
    }
}

pub struct Counter<T> {
    pub inner: T,
    pub reads: usize,
}

impl<T> Counter<T> {
    pub fn new(inner: T) -> Self {
        Counter { inner, reads: 0 }
    }
}

impl<T: Read> Read for Counter<T> {
    fn read(&mut self, buf: &mut [u8]) -> binrw::io::Result<usize> {
        self.reads += 1;
        self.inner.read(buf)
    }
}

impl<T: Seek> Seek for Counter<T> {
    fn seek(&mut self, pos: SeekFrom) -> binrw::io::Result<u64> {
        self.inner.seek(pos)
    }
}
