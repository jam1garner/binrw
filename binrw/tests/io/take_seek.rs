#![allow(clippy::seek_to_start_instead_of_rewind)]
use binrw::io::{Cursor, Read, Seek, SeekFrom, TakeSeekExt};

#[test]
fn take_seek() {
    let data = &mut Cursor::new(b"hello world".to_vec());
    let mut buf = [0; 5];
    let mut take = data.take_seek(6);

    assert_eq!(take.get_ref().position(), 0, "`get_ref` seems broken");
    take.get_mut().get_mut()[10] = b'b';
    assert_eq!(take.get_ref().get_ref()[10], b'b', "`get_mut` seems broken");
    assert_eq!(take.limit(), 6, "wrong initial limit");
    assert_eq!(take.read(&mut buf).unwrap(), 5, "`read` seems broken");
    assert_eq!(take.get_ref().position(), 5, "`read` from a mystery source");
    assert_eq!(take.stream_position().unwrap(), 5, "bad stream position");
    assert_eq!(take.limit(), 1, "limit did not update after a read");
    assert_eq!(&buf, b"hello", "`read` read wrong data");

    assert_eq!(
        take.seek(SeekFrom::Current(1)).unwrap(),
        6,
        "`SeekFrom::Current` returned wrong position"
    );
    assert_eq!(
        take.stream_position().unwrap(),
        6,
        "stream position did not update after `SeekFrom::Current`"
    );
    assert_eq!(take.limit(), 0, "limit did not update after `seek`");
    assert_eq!(
        take.read(&mut buf).unwrap(),
        0,
        "`read` did something even though limit was 0"
    );
    assert_eq!(
        take.limit(),
        0,
        "`read` changed the limit even though it should not have done anything"
    );
    assert_eq!(
        take.stream_position().unwrap(),
        6,
        "`read` updated the stream position even though it should not have done anything"
    );

    take.set_limit(5);
    assert_eq!(
        take.read(&mut buf).unwrap(),
        5,
        "`set_limit` caused too-small read"
    );
    assert_eq!(
        take.read(&mut buf).unwrap(),
        0,
        "`set_limit` caused too-large zero-limit read"
    );
    assert_eq!(&buf, b"worlb", "`read` read wrong data");

    assert_eq!(
        take.seek(SeekFrom::Start(0)).unwrap(),
        0,
        "`SeekFrom::Start` returned wrong position"
    );
    assert_eq!(
        take.stream_position().unwrap(),
        0,
        "stream position did not update after `SeekFrom::Start`"
    );
    assert_eq!(
        take.read(&mut buf).unwrap(),
        5,
        "`read` did not read enough after `SeemFrom::Start`"
    );
    assert_eq!(
        take.read(&mut buf).unwrap(),
        5,
        "`read` did not read enough after `SeemFrom::Start`"
    );
    assert_eq!(
        take.read(&mut buf).unwrap(),
        1,
        "`read` read incorrect amount at end of stream"
    );
    assert_eq!(
        take.seek(SeekFrom::Start(0)).unwrap(),
        0,
        "`SeekFrom::Start` returned wrong position at end of stream"
    );

    take.set_limit(3);
    assert_eq!(
        take.read(&mut buf).unwrap(),
        3,
        "`set_limit` caused too-large partial-limit read"
    );
    assert_eq!(&buf, b"helrl", "`read` read wrong data");

    take.seek(SeekFrom::End(-5))
        .expect_err("out-of-range `SeekFrom::End` backward seek should fail");

    take.seek(SeekFrom::Start(0)).unwrap();
    take.set_limit(10);
    assert_eq!(
        take.seek(SeekFrom::End(-6)).unwrap(),
        4,
        "`SeekFrom::End` returned wrong position"
    );
    assert_eq!(
        take.stream_position().unwrap(),
        4,
        "stream position did not update after `SeekFrom::End`"
    );
    assert_eq!(
        take.read(&mut buf).unwrap(),
        5,
        "`read` after `SeekFrom::End` read wrong number of bytes"
    );
    assert_eq!(&buf, b"o wor", "`read` read wrong data");
    assert_eq!(take.into_inner().position(), 9, "`into_inner` seems broken");
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
