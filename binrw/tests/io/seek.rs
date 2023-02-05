#![allow(clippy::seek_to_start_instead_of_rewind)]
use binrw::io::{NoSeek, Read, Seek, SeekFrom, Write};

#[test]
fn read() {
    let mut stream = NoSeek::new(b"helloworld".as_slice());
    let mut buf = [0; 5];

    assert_eq!(stream.stream_position().unwrap(), 0);
    assert_eq!(stream.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(stream.stream_position().unwrap(), 0);
    stream.seek(SeekFrom::Start(1)).unwrap_err();
    stream.seek(SeekFrom::Current(1)).unwrap_err();
    stream.seek(SeekFrom::Current(-1)).unwrap_err();
    stream.seek(SeekFrom::End(0)).unwrap_err();

    stream.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"hello");
    assert_eq!(stream.stream_position().unwrap(), 5);
    assert_eq!(stream.seek(SeekFrom::Start(5)).unwrap(), 5);
    assert_eq!(stream.stream_position().unwrap(), 5);
    stream.seek(SeekFrom::Start(0)).unwrap_err();

    assert_eq!(stream.read(&mut buf).unwrap(), 5);
    assert_eq!(&buf, b"world");
    assert_eq!(stream.stream_position().unwrap(), 10);
    assert_eq!(stream.read(&mut buf).unwrap(), 0);

    let mut stream = NoSeek::new(b"string".as_slice());
    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();
    assert_eq!(buf, "string");
    assert_eq!(stream.stream_position().unwrap(), 6);

    let mut stream = NoSeek::new(b"abcd".as_slice());
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"abcd");
    assert_eq!(stream.stream_position().unwrap(), 4);

    assert_eq!(stream.into_inner(), &[]);
}

#[test]
#[cfg(feature = "std")]
fn read_vectored() {
    let mut buf = [0; 10];
    let mut stream = NoSeek::new(b"helloworld".as_slice());
    let bufs = buf.split_at_mut(5);
    assert_eq!(
        stream
            .read_vectored(&mut [
                std::io::IoSliceMut::new(bufs.0),
                std::io::IoSliceMut::new(bufs.1),
            ])
            .unwrap(),
        10
    );
    assert_eq!(&buf, b"helloworld");
    assert_eq!(stream.stream_position().unwrap(), 10);
}

#[test]
fn write() {
    struct MockWriter {
        flushed: bool,
    }
    impl Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> binrw::io::Result<usize> {
            Ok(buf.len())
        }

        fn flush(&mut self) -> binrw::io::Result<()> {
            self.flushed = true;
            Ok(())
        }
    }

    let mut stream = NoSeek::new(Vec::new());
    assert_eq!(stream.write(b"helloworld").unwrap(), 10);
    assert_eq!(stream.stream_position().unwrap(), 10);
    assert_eq!(stream.stream_position().unwrap(), 10);
    assert_eq!(stream.seek(SeekFrom::Start(10)).unwrap(), 10);
    assert_eq!(stream.get_ref(), b"helloworld");
    stream.seek(SeekFrom::Start(1)).unwrap_err();
    stream.seek(SeekFrom::Current(1)).unwrap_err();
    stream.seek(SeekFrom::Current(-1)).unwrap_err();
    stream.seek(SeekFrom::End(0)).unwrap_err();

    stream.get_mut()[0] = b'j';
    assert_eq!(stream.get_ref(), b"jelloworld");

    stream.write_all(b"industries").unwrap();
    assert_eq!(stream.stream_position().unwrap(), 20);
    assert_eq!(stream.get_ref(), b"jelloworldindustries");

    let mut stream = NoSeek::new(MockWriter { flushed: false });
    stream.flush().unwrap();
    assert!(stream.get_ref().flushed);
}

#[test]
#[cfg(feature = "std")]
fn write_vectored() {
    let buf = [b'a'; 10];
    let mut stream = NoSeek::new(Vec::new());
    let bufs = buf.split_at(5);
    assert_eq!(
        stream
            .write_vectored(&[std::io::IoSlice::new(bufs.0), std::io::IoSlice::new(bufs.1),])
            .unwrap(),
        10
    );
    assert_eq!(stream.get_ref(), b"aaaaaaaaaa");
    assert_eq!(stream.stream_position().unwrap(), 10);
}
