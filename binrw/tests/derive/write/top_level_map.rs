use binrw::{binrw, io::Cursor, BinReaderExt, BinWrite};

#[test]
fn round_trip_top_level_map() {
    #[binrw]
    #[br(map = Test::from_bytes)]
    #[bw(map = Test::to_bytes)]
    struct Test {
        x: bool,
    }

    impl Test {
        fn to_bytes(&self) -> [u8; 4] {
            if self.x {
                [1, 0, 0, 0]
            } else {
                [0; 4]
            }
        }

        fn from_bytes(bytes: [u8; 4]) -> Self {
            Self { x: bytes[0] == 1 }
        }
    }

    let data = b"\x01\0\0\0";

    let test: Test = Cursor::new(data).read_be().unwrap();
    let mut x = Cursor::new(Vec::new());
    test.write(&mut x).unwrap();

    assert_eq!(x.into_inner(), data);
}
