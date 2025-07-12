extern crate binrw;
use super::t;

#[test]
fn unit_enum_magic() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum Test {
        #[br(magic(1u16))]
        One,
        #[br(magic(2u16))]
        Two,
    }

    let error = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\0"))
        .expect_err("accepted bad data");
    t::assert!(t::matches!(error, binrw::Error::NoVariantMatch { .. }));
    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\x01")).unwrap(),
        Test::One
    );
    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\x02")).unwrap(),
        Test::Two
    );
}

#[test]
fn unit_enum_magic_different_types() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum Test {
        #[br(magic(b"\0\x01"))]
        One,

        #[br(magic(2u16))]
        Two,

        Zero,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\0")).unwrap(),
        Test::Zero
    );
    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\x01")).unwrap(),
        Test::One
    );
    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\x02")).unwrap(),
        Test::Two
    );
}

#[test]
fn unit_enum_magic_order() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum Test {
        #[br(magic(b"\0\x01"))]
        One,

        EverythingElse,

        #[br(magic(2u16))]
        Two,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\0")).unwrap(),
        Test::EverythingElse
    );
    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\x01")).unwrap(),
        Test::One
    );
    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\x02")).unwrap(),
        Test::EverythingElse
    );
}

#[test]
fn unit_enum_magic_bytes() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum Test {
        #[br(magic(b"zero"))]
        Zero,

        #[br(magic(b"two0"))]
        Two,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"zero")).unwrap(),
        Test::Zero
    );
    let error = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"oops"))
        .expect_err("accepted bad data");
    t::assert!(t::matches!(error, binrw::Error::NoVariantMatch { .. }));
    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"two0")).unwrap(),
        Test::Two
    );
}

#[test]
fn unit_enum_magic_pre_assert() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(big, import { allow_zero: bool, forbid_zero: bool })]
    enum Test {
        #[br(magic(0u16), pre_assert(allow_zero))]
        // This redundant check is intentional and tests that assertions are
        // combining properly
        #[br(pre_assert(!forbid_zero))]
        Zero,
        #[br(magic(0u16))]
        OtherZero,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_args(
            &mut binrw::io::Cursor::new(b"\0\0"),
            <Test as binrw::BinRead>::Args::builder()
                .allow_zero(true)
                .forbid_zero(false)
                .finalize()
        )
        .unwrap(),
        Test::Zero
    );
    // Tests allow_zero condition actually applies
    t::assert_eq!(
        <Test as binrw::BinRead>::read_args(
            &mut binrw::io::Cursor::new(b"\0\0"),
            <Test as binrw::BinRead>::Args::builder()
                .allow_zero(true)
                .forbid_zero(true)
                .finalize()
        )
        .unwrap(),
        Test::OtherZero
    );
    // Tests forbid_zero condition actually applies
    t::assert_eq!(
        <Test as binrw::BinRead>::read_args(
            &mut binrw::io::Cursor::new(b"\0\0"),
            <Test as binrw::BinRead>::Args::builder()
                .allow_zero(false)
                .forbid_zero(true)
                .finalize()
        )
        .unwrap(),
        Test::OtherZero
    );
    let error = <Test as binrw::BinRead>::read_args(
        &mut binrw::io::Cursor::new(b"\0\x01"),
        <Test as binrw::BinRead>::Args::builder()
            .allow_zero(false)
            .forbid_zero(true)
            .finalize(),
    )
    .expect_err("accepted bad data");

    t::assert!(t::matches!(error, binrw::Error::NoVariantMatch { .. }));
}

#[test]
fn unit_enum_pre_assert() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(import(one: bool), repr(u8))]
    enum Test {
        #[br(pre_assert(false))]
        Zero,
        #[br(pre_assert(one))]
        One,
        Two,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read_args(&mut binrw::io::Cursor::new(b"\x01"), (true,)).unwrap(),
        Test::One
    );
    t::assert_eq!(
        <Test as binrw::BinRead>::read_args(&mut binrw::io::Cursor::new(b"\x02"), (false,))
            .unwrap(),
        Test::Two
    );
    <Test as binrw::BinRead>::read_args(&mut binrw::io::Cursor::new(b"\0"), (false,)).unwrap_err();
}

#[test]
fn unit_enum_repr() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(big, repr(i16))]
    enum Test {
        Neg1 = -1,
        Zero,
        Two = 2,
    }

    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\xff\xff")).unwrap(),
        Test::Neg1
    );
    let error = <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\x01"))
        .expect_err("accepted bad data");
    assert!(t::matches!(
        error.root_cause(),
        binrw::Error::NoVariantMatch { .. }
    ));
    t::assert_eq!(
        <Test as binrw::BinRead>::read(&mut binrw::io::Cursor::new(b"\0\x02")).unwrap(),
        Test::Two
    );
}

#[test]
fn unit_enum_rewind_on_eof() {
    #[derive(binrw::BinRead, Debug)]
    #[br(repr(u16))]
    enum Test {
        A,
    }

    let mut data = binrw::io::Cursor::new(b"\0\0");
    let expected = binrw::io::Seek::seek(&mut data, binrw::io::SeekFrom::Start(1)).unwrap();
    <Test as binrw::BinRead>::read_le(&mut data).expect_err("accepted bad data");
    t::assert_eq!(
        expected,
        binrw::io::Seek::stream_position(&mut data).unwrap()
    );
}

#[test]
fn unit_enum_rewind_on_no_variant() {
    #[derive(binrw::BinRead, Debug)]
    #[br(repr(u8))]
    enum Test {
        A = 1,
    }

    let mut data = binrw::io::Cursor::new(b"\0\0");
    let expected = binrw::io::Seek::seek(&mut data, binrw::io::SeekFrom::Start(1)).unwrap();
    <Test as binrw::BinRead>::read(&mut data).expect_err("accepted bad data");
    t::assert_eq!(
        expected,
        binrw::io::Seek::stream_position(&mut data).unwrap()
    );
}

#[test]
fn unit_enum_eof_when_all_magic_eof() {
    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum TestHomogenous {
        #[br(magic(b"ONE"))]
        One,

        #[br(magic(b"TWO"))]
        Two,

        #[br(magic(b"ZER"))]
        Zero,
    }

    assert!(
        <TestHomogenous as binrw::BinRead>::read(&mut binrw::io::Cursor::new(&[]))
            .unwrap_err()
            .is_eof()
    );

    #[derive(binrw::BinRead, Debug, Eq, PartialEq)]
    #[br(big)]
    enum TestHeterogenous {
        #[br(magic(b"O"))]
        One,

        #[br(magic(b"TW"))]
        Two,

        #[br(magic(b"ZERO"))]
        Zero,
    }

    assert!(
        <TestHeterogenous as binrw::BinRead>::read(&mut binrw::io::Cursor::new(&[]))
            .unwrap_err()
            .is_eof()
    );
}
