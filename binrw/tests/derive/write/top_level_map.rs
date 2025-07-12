extern crate binrw;
use super::t;

#[test]
fn round_trip_top_level_map() {
    #[binrw::binrw]
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

    let test = <Test as binrw::BinRead>::read_be(&mut binrw::io::Cursor::new(data)).unwrap();
    let mut x = binrw::io::Cursor::new(t::Vec::new());
    binrw::BinWrite::write(&test, &mut x).unwrap();

    t::assert_eq!(x.into_inner(), data);
}
