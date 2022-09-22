use binrw::io::{Cursor, Read, Seek, SeekFrom, TakeSeekExt};

#[test]
fn take_seek() {
    let data = &mut Cursor::new(b"hello world".to_vec());
    let mut buf = [0; 5];
    let mut take = data.take_seek(6);
    assert_eq!(take.get_ref().position(), 0);
    take.get_mut().get_mut()[10] = b'b';
    assert_eq!(take.limit(), 6);
    assert_eq!(take.read(&mut buf).unwrap(), 5);
    assert_eq!(take.get_ref().position(), 5);
    assert_eq!(take.stream_position().unwrap(), 5);
    assert_eq!(take.limit(), 1);
    assert_eq!(&buf, b"hello");
    assert_eq!(take.seek(SeekFrom::Current(1)).unwrap(), 6);
    assert_eq!(take.stream_position().unwrap(), 6);
    assert_eq!(take.limit(), 0);
    assert_eq!(take.read(&mut buf).unwrap(), 0);
    assert_eq!(take.limit(), 0);
    assert_eq!(take.stream_position().unwrap(), 6);
    take.set_limit(5);
    assert_eq!(take.read(&mut buf).unwrap(), 5);
    assert_eq!(take.read(&mut buf).unwrap(), 0);
    assert_eq!(&buf, b"worlb");
    assert_eq!(take.seek(SeekFrom::Start(0)).unwrap(), 0);
    assert_eq!(take.stream_position().unwrap(), 0);
    assert_eq!(take.read(&mut buf).unwrap(), 5);
    assert_eq!(take.read(&mut buf).unwrap(), 5);
    assert_eq!(take.read(&mut buf).unwrap(), 1);
    assert_eq!(take.seek(SeekFrom::Start(0)).unwrap(), 0);
    take.set_limit(3);
    assert_eq!(take.read(&mut buf).unwrap(), 3);
    assert_eq!(&buf, b"helrl");
    assert_eq!(take.seek(SeekFrom::End(-5)).unwrap(), 6);
    assert_eq!(take.stream_position().unwrap(), 6);
    assert_eq!(take.read(&mut buf).unwrap(), 0);
    assert_eq!(take.seek(SeekFrom::End(-10)).unwrap(), 1);
    assert_eq!(take.read(&mut buf).unwrap(), 2);
    assert_eq!(take.into_inner().position(), 3);
}

#[test]
fn take_seek_ref() {
    let data = &mut Cursor::new(b"hello world");
    let mut buf = [0; 5];
    assert_eq!(data.take_seek(5).read(&mut buf).unwrap(), 5);
    assert_eq!(&buf, b"hello");
    assert_eq!(data.take_seek(5).read(&mut buf).unwrap(), 5);
    assert_eq!(&buf, b" worl");
    assert_eq!(data.take_seek(5).read(&mut buf).unwrap(), 1);
    assert_eq!(&buf, b"dworl");
}
