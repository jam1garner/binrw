extern crate binrw;
use super::t;

#[test]
fn writer_var() {
    struct Checksum<T> {
        inner: T,
        check: ::core::num::Wrapping<u8>,
    }

    impl<T> Checksum<T> {
        fn new(inner: T) -> Self {
            Self {
                inner,
                check: ::core::num::Wrapping(0),
            }
        }

        fn check(&self) -> u8 {
            self.check.0
        }
    }

    impl<T: binrw::io::Write> binrw::io::Write for Checksum<T> {
        fn write(&mut self, buf: &[u8]) -> binrw::io::Result<usize> {
            for b in buf {
                self.check += b;
            }
            self.inner.write(buf)
        }

        fn flush(&mut self) -> binrw::io::Result<()> {
            self.inner.flush()
        }
    }

    impl<T: binrw::io::Seek> binrw::io::Seek for Checksum<T> {
        fn seek(&mut self, pos: binrw::io::SeekFrom) -> binrw::io::Result<u64> {
            self.inner.seek(pos)
        }
    }

    #[binrw::binwrite]
    #[bw(little, stream = w, map_stream = Checksum::new)]
    struct Test {
        a: u16,
        b: u16,
        #[bw(calc(w.check()))]
        c: u8,
    }

    let mut out = binrw::io::Cursor::new(t::vec![]);
    binrw::BinWrite::write(&Test { a: 0x201, b: 0x403 }, &mut out).unwrap();
    t::assert_eq!(out.into_inner(), b"\x01\x02\x03\x04\x0a");
}
