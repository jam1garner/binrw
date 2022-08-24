use binrw::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::io::BufRead;

#[test]
fn bufreader() {
    struct Counter<T> {
        inner: T,
        reads: usize,
    }

    impl<T> Counter<T> {
        fn new(inner: T) -> Self {
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

    let mut stream =
        BufReader::with_capacity(10, Counter::new(Cursor::new(b"helloworld".to_vec())));
    let mut buf = [0; 5];
    assert_eq!(stream.capacity(), 10);

    // Multiple reads
    stream.read_exact(&mut buf).unwrap();
    assert_eq!(stream.stream_position().unwrap(), 5);
    assert_eq!(&buf, b"hello");
    stream.read_exact(&mut buf).unwrap();
    assert_eq!(stream.stream_position().unwrap(), 10);
    assert_eq!(&buf, b"world");
    assert_eq!(stream.get_ref().reads, 1);

    // Forward absolute seek
    assert_eq!(stream.seek(SeekFrom::Start(5)).unwrap(), 5);
    assert_eq!(stream.stream_position().unwrap(), 5);
    assert_eq!(stream.read(&mut buf).unwrap(), 5);
    assert_eq!(&buf, b"world");
    assert_eq!(stream.get_ref().reads, 1);

    // Backward relative seek
    assert_eq!(stream.seek(SeekFrom::Current(-8)).unwrap(), 2);
    assert_eq!(stream.stream_position().unwrap(), 2);
    assert_eq!(stream.read(&mut buf).unwrap(), 5);
    assert_eq!(&buf, b"llowo");
    assert_eq!(stream.get_ref().reads, 1);

    // Null seek
    assert_eq!(stream.seek(SeekFrom::Current(0)).unwrap(), 7);
    assert_eq!(stream.stream_position().unwrap(), 7);
    assert_eq!(stream.read(&mut buf).unwrap(), 3);
    assert_eq!(&buf, b"rldwo");
    assert_eq!(stream.get_ref().reads, 1);

    // Backward absolute seek
    assert_eq!(stream.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(stream.stream_position().unwrap(), 0);
    assert_eq!(stream.read(&mut buf).unwrap(), 5);
    assert_eq!(&buf, b"hello");
    assert_eq!(stream.get_ref().reads, 1);

    // Forward relative seek
    assert_eq!(stream.seek(SeekFrom::Current(1)).unwrap(), 6);
    assert_eq!(stream.stream_position().unwrap(), 6);
    assert_eq!(stream.read(&mut buf).unwrap(), 4);
    assert_eq!(&buf, b"orldo");
    assert_eq!(stream.get_ref().reads, 1);

    // Explicitly invalidating seek
    assert_eq!(stream.seek_invalidate(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(stream.stream_position().unwrap(), 0);
    assert_eq!(stream.read(&mut buf).unwrap(), 5);
    assert_eq!(&buf, b"hello");
    assert_eq!(stream.get_ref().reads, 2);

    // Invalid seek maintains correct stream position
    stream.seek(SeekFrom::Current(-9001)).unwrap_err();
    assert_eq!(stream.stream_position().unwrap(), 5);

    // get_ref/get_mut
    stream
        .get_mut()
        .inner
        .get_mut()
        .extend_from_slice(b"tell my wife hello");
    assert_eq!(stream.get_ref().inner.get_ref().len(), 28);

    // SeekFrom::End/read_to_string
    assert_eq!(stream.seek(SeekFrom::End(-5)).unwrap(), 23);
    let mut str = String::new();
    assert_eq!(stream.read_to_string(&mut str).unwrap(), 5);
    assert_eq!(stream.stream_position().unwrap(), 28);
    assert_eq!(str, "hello");
    assert_eq!(stream.get_ref().reads, 4);

    // read_to_end
    let mut buf = Vec::new();
    assert_eq!(stream.seek(SeekFrom::End(-18)).unwrap(), 10);
    assert_eq!(stream.read_to_end(&mut buf).unwrap(), 18);
    assert_eq!(stream.stream_position().unwrap(), 28);
    assert_eq!(buf, b"tell my wife hello");
    assert_eq!(stream.get_ref().reads, 6);

    // Very large absolute position seek
    assert_eq!(
        stream.seek(SeekFrom::Start(u64::MAX - 1)).unwrap(),
        u64::MAX - 1
    );
    assert_eq!(stream.seek(SeekFrom::Start(0)).unwrap(), 0);

    // fill_buf/consume
    assert_eq!(stream.fill_buf().unwrap(), b"helloworld");
    stream.consume(5);
    assert_eq!(stream.buffer(), b"world");
    assert_eq!(stream.stream_position().unwrap(), 0);

    // into_inner
    let mut buf = Vec::new();
    let mut cursor = stream.into_inner();
    cursor.read_to_end(&mut buf).unwrap();
    assert_eq!(buf, b"tell my wife hello");

    // read_vectored
    let mut stream = BufReader::new(Cursor::new(b"if i don't survive"));
    let mut buf = [0; 18];
    let bufs = buf.split_at_mut(9);
    assert_eq!(
        stream
            .read_vectored(&mut [
                std::io::IoSliceMut::new(bufs.0),
                std::io::IoSliceMut::new(bufs.1)
            ])
            .unwrap(),
        18
    );
    assert_eq!(stream.stream_position().unwrap(), 18);
    assert_eq!(&buf, b"if i don't survive");
}
