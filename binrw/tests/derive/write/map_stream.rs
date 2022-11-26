use binrw::{
    io::{Cursor, Seek, SeekFrom, Write},
    BinWrite,
};

struct BadCrypt<T> {
    inner: T,
    key: u8,
}

impl<T> BadCrypt<T> {
    fn new(inner: T) -> Self {
        Self { inner, key: 0 }
    }
}

impl<T: Write> Write for BadCrypt<T> {
    fn write(&mut self, buf: &[u8]) -> binrw::io::Result<usize> {
        let mut w = 0;
        for b in buf {
            self.key ^= b;
            w += self.inner.write(&[self.key])?;
        }
        Ok(w)
    }

    fn flush(&mut self) -> binrw::io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for BadCrypt<T> {
    fn seek(&mut self, pos: SeekFrom) -> binrw::io::Result<u64> {
        self.inner.seek(pos)
    }
}

#[test]
fn map_stream() {
    #[derive(BinWrite, Debug, PartialEq)]
    #[bw(big, map_stream = |inner| BadCrypt { inner, key: 0x80 })]
    struct Test(Vec<u8>);

    let mut out = Cursor::new(vec![]);
    Test(vec![0, 1, 2, 3]).write(&mut out).unwrap();

    assert_eq!(out.into_inner(), &[0x80, 0x81, 0x83, 0x80],);
}

#[test]
fn map_stream_field() {
    #[derive(BinWrite, Debug, PartialEq)]
    #[bw(big)]
    struct Test {
        #[bw(map_stream = BadCrypt::new)]
        a: Vec<u8>,
        #[bw(map_stream = |inner| BadCrypt { inner, key: 0x80 })]
        b: Vec<u8>,
    }

    let mut out = Cursor::new(vec![]);
    Test {
        a: vec![0, 1, 2, 3],
        b: vec![4, 5, 6, 7],
    }
    .write(&mut out)
    .unwrap();

    assert_eq!(out.into_inner(), &[0, 1, 3, 0, 132, 129, 135, 128],);
}
