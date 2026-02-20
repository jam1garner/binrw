extern crate binrw;
use super::t;

macro_rules! derive_binread {
    ($item:item) => {
        #[derive(::core::fmt::Debug, ::core::cmp::PartialEq, ::binrw::BinRead)]
        $item
    };
}

macro_rules! derive_binwrite {
    ($item:item) => {
        #[derive(::core::fmt::Debug, ::core::cmp::PartialEq, ::binrw::BinWrite)]
        $item
    };
}

macro_rules! derive_binrw {
    ($item:item) => {
        #[derive(
            ::core::fmt::Debug, ::core::cmp::PartialEq, ::binrw::BinRead, ::binrw::BinWrite,
        )]
        $item
    };
}

derive_binrw! {
    #[br(import(v: bool))]
    enum PreAssertEnum {
        #[br(pre_assert(v))]
        A(u32),
        #[br(pre_assert(!v))]
        B(u32),
    }
}

derive_binrw! {
    #[br(assert(x == 1))]
    #[bw(assert(*x == 1))]
    struct Asserted {
        x: u8,
    }
}

derive_binread! {
    #[br(import(len: usize))]
    struct Counted {
        #[br(count = len)]
        values: t::Vec<u8>,
    }
}

#[binrw::parser(reader, endian)]
fn parse_one_byte() -> binrw::BinResult<u8> {
    <u8 as ::binrw::BinRead>::read_options(reader, endian, ())
}

derive_binread! {
    struct ParseWithField {
        #[br(parse_with = parse_one_byte)]
        value: u8,
    }
}

derive_binrw! {
    #[br(map_stream = |reader| reader)]
    #[bw(map_stream = |writer| writer)]
    struct TopLevelMapStream {
        value: u8,
    }
}

derive_binrw! {
    struct FieldMapStream {
        #[br(map_stream = |reader| reader)]
        #[bw(map_stream = |writer| writer)]
        value: u8,
    }
}

derive_binread! {
    struct MapRead {
        #[br(map = |x: u8| x + 1)]
        value: u8,
    }
}

derive_binread! {
    struct TryMapRead {
        #[br(try_map = |x: u16| <u8 as ::core::convert::TryFrom<_>>::try_from(x))]
        value: u8,
    }
}

derive_binwrite! {
    struct MapWrite {
        #[bw(map = |&x| x as u16)]
        value: u8,
    }
}

derive_binwrite! {
    struct TryMapWrite {
        #[bw(try_map = |&x| <u8 as ::core::convert::TryFrom<_>>::try_from(x))]
        value: u16,
    }
}

#[test]
fn derive_macro_pre_assert_hygiene() {
    let a = <PreAssertEnum as binrw::BinRead>::read_be_args(
        &mut binrw::io::Cursor::new(b"\0\0\0\x01"),
        (true,),
    )
    .unwrap();
    t::assert_eq!(a, PreAssertEnum::A(1));

    let b = <PreAssertEnum as binrw::BinRead>::read_be_args(
        &mut binrw::io::Cursor::new(b"\0\0\0\x01"),
        (false,),
    )
    .unwrap();
    t::assert_eq!(b, PreAssertEnum::B(1));

    let mut out = binrw::io::Cursor::new(t::vec![]);
    <PreAssertEnum as binrw::BinWrite>::write_be(&PreAssertEnum::A(2), &mut out).unwrap();
    t::assert_eq!(out.into_inner(), b"\0\0\0\x02");
}

#[test]
fn derive_macro_assert_hygiene() {
    let ok = <Asserted as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x01")).unwrap();
    t::assert_eq!(ok, Asserted { x: 1 });

    let err =
        <Asserted as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x00")).unwrap_err();
    t::assert!(t::matches!(err, binrw::Error::AssertFail { .. }));

    let mut out = binrw::io::Cursor::new(t::vec![]);
    <Asserted as binrw::BinWrite>::write_le(&Asserted { x: 1 }, &mut out).unwrap();
    t::assert_eq!(out.into_inner(), b"\x01");

    let err = <Asserted as binrw::BinWrite>::write_le(
        &Asserted { x: 0 },
        &mut binrw::io::Cursor::new(t::vec![]),
    )
    .unwrap_err();
    t::assert!(t::matches!(err, binrw::Error::AssertFail { .. }));
}

#[test]
fn derive_macro_count_hygiene() {
    let counted =
        <Counted as binrw::BinRead>::read_le_args(&mut binrw::io::Cursor::new(b"\x01\x02"), (2,))
            .unwrap();
    t::assert_eq!(counted.values, t::vec![1, 2]);
}

#[test]
fn derive_macro_parse_with_hygiene() {
    let parsed =
        <ParseWithField as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x2a")).unwrap();
    t::assert_eq!(parsed, ParseWithField { value: 0x2a });
}

#[test]
fn derive_macro_map_stream_hygiene() {
    let read_top =
        <TopLevelMapStream as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x11"))
            .unwrap();
    t::assert_eq!(read_top, TopLevelMapStream { value: 0x11 });

    let read_field =
        <FieldMapStream as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x22")).unwrap();
    t::assert_eq!(read_field, FieldMapStream { value: 0x22 });

    let mut out = binrw::io::Cursor::new(t::vec![]);
    <TopLevelMapStream as binrw::BinWrite>::write_le(&TopLevelMapStream { value: 0x33 }, &mut out)
        .unwrap();
    t::assert_eq!(out.into_inner(), b"\x33");

    let mut out = binrw::io::Cursor::new(t::vec![]);
    <FieldMapStream as binrw::BinWrite>::write_le(&FieldMapStream { value: 0x44 }, &mut out)
        .unwrap();
    t::assert_eq!(out.into_inner(), b"\x44");
}

#[test]
fn derive_macro_map_hygiene() {
    let mapped =
        <MapRead as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x01")).unwrap();
    t::assert_eq!(mapped, MapRead { value: 2 });

    let try_mapped_ok =
        <TryMapRead as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\x7f\0")).unwrap();
    t::assert_eq!(try_mapped_ok, TryMapRead { value: 127 });

    let try_mapped_err =
        <TryMapRead as binrw::BinRead>::read_le(&mut binrw::io::Cursor::new(b"\0\x01"))
            .unwrap_err();
    t::assert!(t::matches!(try_mapped_err, binrw::Error::Custom { .. }));

    let mut out = binrw::io::Cursor::new(t::vec![]);
    <MapWrite as binrw::BinWrite>::write_le(&MapWrite { value: 5 }, &mut out).unwrap();
    t::assert_eq!(out.into_inner(), b"\x05\0");

    let mut out = binrw::io::Cursor::new(t::vec![]);
    <TryMapWrite as binrw::BinWrite>::write_le(&TryMapWrite { value: 127 }, &mut out).unwrap();
    t::assert_eq!(out.into_inner(), b"\x7f");

    let err = <TryMapWrite as binrw::BinWrite>::write_le(
        &TryMapWrite { value: 256 },
        &mut binrw::io::Cursor::new(t::vec![]),
    )
    .unwrap_err();
    t::assert!(t::matches!(err, binrw::Error::Custom { .. }));
}
