use binrw::io::{Cursor, Read, Seek, SeekFrom};
use std::io::{Read as StdReadExt, Seek as StdSeekExt};

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
