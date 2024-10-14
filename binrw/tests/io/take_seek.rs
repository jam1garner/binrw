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
        0,
        "`read` read incorrect amount at end of stream"
    );
    assert_eq!(&buf, b"worlb", "`read` read wrong data");

    assert_eq!(
        take.seek(SeekFrom::Start(0)).unwrap(),
        0,
        "`SeekFrom::Start` returned wrong position at end of stream"
    );

    // Rewind the underlying cursor so we can start at the beginning of the
    // buffer when `set_limit` is called.
    take.get_mut().rewind().unwrap();

    take.set_limit(3);
    assert_eq!(
        take.read(&mut buf).unwrap(),
        3,
        "`set_limit` caused too-large partial-limit read"
    );
    assert_eq!(&buf, b"hellb", "`read` read wrong data");

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

#[test]
fn test_seek_start() {
    let mut buf = [0; 8];

    let mut data = Cursor::new("\x00\x01\x02\x03\x04\x05\x06\x07\x08");
    data.seek(SeekFrom::Start(1)).unwrap();

    let mut section = data.take_seek(6);

    assert_eq!(section.get_mut().stream_position().unwrap(), 1);
    assert_eq!(section.stream_position().unwrap(), 0);
    assert_eq!(section.limit(), 6);
    assert_eq!(section.read(&mut buf).unwrap(), 6);
    assert_eq!(&buf, b"\x01\x02\x03\x04\x05\x06\x00\x00");
    assert_eq!(section.get_mut().stream_position().unwrap(), 7);
    assert_eq!(section.stream_position().unwrap(), 6);

    let mut buf = [0; 8]; // clear buff to ensure read works.

    section.rewind().unwrap();
    assert_eq!(section.get_mut().stream_position().unwrap(), 1);
    assert_eq!(section.stream_position().unwrap(), 0);
    assert_eq!(section.limit(), 6);
    assert_eq!(section.read(&mut buf).unwrap(), 6);
    assert_eq!(&buf, b"\x01\x02\x03\x04\x05\x06\x00\x00");
    assert_eq!(section.get_mut().stream_position().unwrap(), 7);
    assert_eq!(section.stream_position().unwrap(), 6);
}

#[test]
fn test_seek_relative() {
    let mut buf = [0; 8];

    let mut data = Cursor::new("\x00\x01\x02\x03\x04\x05\x06\x07\x08");
    data.seek(SeekFrom::Start(1)).unwrap();

    let mut section = data.take_seek(6);

    section
        .seek_relative(-1)
        .expect_err("out-of-range `SeekFrom::Current` backward seek should fail");
    assert_eq!(section.get_mut().stream_position().unwrap(), 1);
    assert_eq!(section.stream_position().unwrap(), 0);
    assert_eq!(section.limit(), 6);

    section.seek_relative(2).unwrap();
    assert_eq!(section.get_mut().stream_position().unwrap(), 3);
    assert_eq!(section.stream_position().unwrap(), 2);
    assert_eq!(section.limit(), 4);
    assert_eq!(section.read(&mut buf).unwrap(), 4);
    assert_eq!(&buf, b"\x03\x04\x05\x06\x00\x00\x00\x00");
    assert_eq!(section.get_mut().stream_position().unwrap(), 7);
    assert_eq!(section.stream_position().unwrap(), 6);

    section.seek_relative(-2).unwrap();
    assert_eq!(section.get_mut().stream_position().unwrap(), 5);
    assert_eq!(section.stream_position().unwrap(), 4);
    assert_eq!(section.limit(), 2);
    assert_eq!(section.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf, b"\x05\x06\x05\x06\x00\x00\x00\x00");
    assert_eq!(section.get_mut().stream_position().unwrap(), 7);
    assert_eq!(section.stream_position().unwrap(), 6);

    // According to `std::io::Seek.seek`, seeking past the stream is valid,
    // but behavior is defined by the implementation. In our case we don't
    // allow reading any additional data.
    section.seek_relative(2).unwrap();
    assert_eq!(section.get_mut().stream_position().unwrap(), 9);
    assert_eq!(section.stream_position().unwrap(), 8);
    assert_eq!(section.limit(), 0);
    assert_eq!(section.read(&mut buf).unwrap(), 0);
    assert_eq!(&buf, b"\x05\x06\x05\x06\x00\x00\x00\x00");
    assert_eq!(section.get_mut().stream_position().unwrap(), 9);
    assert_eq!(section.stream_position().unwrap(), 8);
}

#[test]
fn test_seek_end() {
    let mut buf = [0; 8];

    let mut data = Cursor::new("\x00\x01\x02\x03\x04\x05\x06\x07\x08");
    data.seek(SeekFrom::Start(1)).unwrap();

    let mut section = data.take_seek(6);

    section.seek(SeekFrom::End(0)).unwrap();
    assert_eq!(section.get_mut().stream_position().unwrap(), 7);
    assert_eq!(section.stream_position().unwrap(), 6);
    assert_eq!(section.limit(), 0);
    assert_eq!(section.read(&mut buf).unwrap(), 0);
    assert_eq!(&buf, b"\x00\x00\x00\x00\x00\x00\x00\x00");
    assert_eq!(section.get_mut().stream_position().unwrap(), 7);
    assert_eq!(section.stream_position().unwrap(), 6);

    section.seek(SeekFrom::End(-2)).unwrap();
    assert_eq!(section.get_mut().stream_position().unwrap(), 5);
    assert_eq!(section.stream_position().unwrap(), 4);
    assert_eq!(section.limit(), 2);
    assert_eq!(section.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf, b"\x05\x06\x00\x00\x00\x00\x00\x00");
    assert_eq!(section.get_mut().stream_position().unwrap(), 7);
    assert_eq!(section.stream_position().unwrap(), 6);

    // According to `std::io::Seek.seek`, seeking past the stream is valid,
    // but behavior is defined by the implementation. In our case we don't
    // allow reading any additional data.
    section.seek(SeekFrom::End(2)).unwrap();
    assert_eq!(section.get_mut().stream_position().unwrap(), 9);
    assert_eq!(section.stream_position().unwrap(), 8);
    assert_eq!(section.limit(), 0);
    assert_eq!(section.read(&mut buf).unwrap(), 0);
    assert_eq!(&buf, b"\x05\x06\x00\x00\x00\x00\x00\x00");
    assert_eq!(section.get_mut().stream_position().unwrap(), 9);
    assert_eq!(section.stream_position().unwrap(), 8);

    section
        .seek(SeekFrom::End(-10))
        .expect_err("out-of-range `SeekFrom::End` backward seek should fail");
    assert_eq!(section.get_mut().stream_position().unwrap(), 9);
    assert_eq!(section.stream_position().unwrap(), 8);
    assert_eq!(section.limit(), 0);
}

#[test]
fn test_seek_nested() {
    let mut buf = [0; 8];

    let mut data = Cursor::new("\x00\x01\x02\x03\x04\x05\x06\x07\x08");
    data.seek(SeekFrom::Start(1)).unwrap();

    let mut outer_section = data.take_seek(6);
    outer_section.seek_relative(2).unwrap();
    assert_eq!(outer_section.get_mut().stream_position().unwrap(), 3);
    assert_eq!(outer_section.stream_position().unwrap(), 2);
    assert_eq!(outer_section.limit(), 4);

    // Will only allow reading data[3..5].
    let mut inner_section = outer_section.take_seek(2);
    assert_eq!(
        inner_section.get_mut().get_mut().stream_position().unwrap(),
        3
    );
    assert_eq!(inner_section.get_mut().stream_position().unwrap(), 2);
    assert_eq!(inner_section.stream_position().unwrap(), 0);
    assert_eq!(inner_section.limit(), 2);
    assert_eq!(inner_section.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf, b"\x03\x04\x00\x00\x00\x00\x00\x00");
    assert_eq!(
        inner_section.get_mut().get_mut().stream_position().unwrap(),
        5
    );
    assert_eq!(inner_section.get_mut().stream_position().unwrap(), 4);
    assert_eq!(inner_section.stream_position().unwrap(), 2);

    inner_section.rewind().unwrap();
    assert_eq!(
        inner_section.get_mut().get_mut().stream_position().unwrap(),
        3
    );
    assert_eq!(inner_section.get_mut().stream_position().unwrap(), 2);
    assert_eq!(inner_section.stream_position().unwrap(), 0);
    assert_eq!(inner_section.get_mut().limit(), 4);
    assert_eq!(inner_section.limit(), 2);

    inner_section.seek_relative(1).unwrap();
    assert_eq!(
        inner_section.get_mut().get_mut().stream_position().unwrap(),
        4
    );
    assert_eq!(inner_section.get_mut().stream_position().unwrap(), 3);
    assert_eq!(inner_section.stream_position().unwrap(), 1);
    assert_eq!(inner_section.get_mut().limit(), 3);
    assert_eq!(inner_section.limit(), 1);

    inner_section.seek(SeekFrom::End(0)).unwrap();
    assert_eq!(
        inner_section.get_mut().get_mut().stream_position().unwrap(),
        5
    );
    assert_eq!(inner_section.get_mut().stream_position().unwrap(), 4);
    assert_eq!(inner_section.stream_position().unwrap(), 2);
    assert_eq!(inner_section.get_mut().limit(), 2);
    assert_eq!(inner_section.limit(), 0);

    // Seek past the end of the stream, it should seek `outer_section` to the end.
    inner_section.seek(SeekFrom::End(2)).unwrap();
    assert_eq!(
        inner_section.get_mut().get_mut().stream_position().unwrap(),
        7
    );
    assert_eq!(inner_section.get_mut().stream_position().unwrap(), 6);
    assert_eq!(inner_section.stream_position().unwrap(), 4);
    assert_eq!(inner_section.get_mut().limit(), 0);
    assert_eq!(inner_section.limit(), 0);
}

#[test]
fn test_empty() {
    let mut data = Cursor::new("\x00\x01\x02\x03\x04\x05\x06\x07\x08");
    data.seek(SeekFrom::Start(1)).unwrap();

    let mut section = data.take_seek(0);
    assert_eq!(section.get_mut().stream_position().unwrap(), 1);
    assert_eq!(section.stream_position().unwrap(), 0);
    assert_eq!(section.limit(), 0);
}

#[test]
fn test_set_limit() {
    let mut data = Cursor::new("\x00\x01\x02\x03\x04\x05\x06\x07\x08");
    data.seek(SeekFrom::Start(1)).unwrap();

    let mut buf = [0; 8];

    let mut section = data.take_seek(6);
    section.seek(SeekFrom::End(-2)).unwrap();
    section.set_limit(4);

    assert_eq!(section.limit(), 4);
    assert_eq!(section.read(&mut buf).unwrap(), 4);
    assert_eq!(&buf, b"\x05\x06\x07\x08\x00\x00\x00\x00");
}

#[test]
fn test_early_eof() {
    let mut data = Cursor::new("\x00\x01\x02\x03\x04\x05\x06\x07\x08");
    data.seek(SeekFrom::Start(6)).unwrap();

    let mut buf = [0; 8];

    let mut section = data.take_seek(10);

    assert_eq!(section.limit(), 10);
    assert_eq!(section.read(&mut buf).unwrap(), 3);
    assert_eq!(&buf, b"\x06\x07\x08\x00\x00\x00\x00\x00");
    assert_eq!(section.get_mut().stream_position().unwrap(), 9);
    assert_eq!(section.stream_position().unwrap(), 3);
    assert_eq!(section.limit(), 7);
}

#[test]
fn test_corrupt_position() {
    let mut data = Cursor::new("\x00\x01\x02\x03\x04\x05\x06\x07\x08");
    data.seek(SeekFrom::Start(1)).unwrap();

    let mut section = data.take_seek(2);
    assert_eq!(section.get_mut().stream_position().unwrap(), 1);
    assert_eq!(section.stream_position().unwrap(), 0);
    assert_eq!(section.limit(), 2);

    // Move the underlying cursor before the start of the section. This
    // is an invalid state.
    section.get_mut().rewind().unwrap();
    assert_eq!(section.get_mut().stream_position().unwrap(), 0);
    section
        .stream_position()
        .expect_err("invalid stream position");

    // Fix the cursor by resetting the cursor position.
    section.rewind().unwrap();
    assert_eq!(section.get_mut().stream_position().unwrap(), 1);
    assert_eq!(section.stream_position().unwrap(), 0);
    assert_eq!(section.limit(), 2);
}
