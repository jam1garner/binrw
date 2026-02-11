#[test]
fn modular_bitfield() {
    use binrw_derive::binread;
    use modular_bitfield::prelude::{bitfield, B8};

    #[allow(dead_code)]
    #[bitfield]
    #[binread]
    struct Foo {
        bits: B8,
    }
}
