use binrw::{binwrite, io::Cursor, BinWrite, Endian, WriteOptions};

#[test]
fn ignore_is_not_written() {
    #[binwrite]
    struct Test {
        #[bw(ignore)]
        x: u32,
    }

    let mut x = Cursor::new(Vec::new());

    Test { x: 1 }
        .write_options(&mut x, &WriteOptions::new(Endian::Big), ())
        .unwrap();

    // Since it's bw(ignore), nothing is written here.
    assert_eq!(&x.into_inner()[..], b"");
}
