# binrw

[![crates](https://img.shields.io/crates/v/binrw.svg)](https://crates.io/crates/binrw)
[![tests](https://github.com/jam1garner/binrw/actions/workflows/main.yml/badge.svg)](https://github.com/jam1garner/binrw/actions/workflows/main.yml)
[![docs.rs](https://docs.rs/binrw/badge.svg)](https://docs.rs/binrw)
[![codecov](https://codecov.io/gh/jam1garner/binrw/branch/master/graph/badge.svg?token=UREOWI2KAY)](https://codecov.io/gh/jam1garner/binrw) 
[![discord](https://img.shields.io/discord/818723403871551509?color=gray&label=%20&logo=discord)](https://discord.gg/ABy4Qh549j)
[![matrix: #binrw:matrix.org](https://img.shields.io/badge/style-%23binrw:matrix.org-blue.svg?style=flat&label=[m])](https://matrix.to/#/#binrw:matrix.org)

binrw helps you write maintainable & easy-to-read declarative binary data
readers and writers using ✨macro magic✨.

## Features

* Generates efficient data parsers and serialisers for structs and enums using
  `#[derive]`
* Reads and writes data from any source using standard `io::Read` and
  `io::Write` streams
* [Directives in attributes](https://docs.rs/binrw/latest/binrw/docs/attribute)
  handle common binary parsing tasks like matching magic numbers, byte ordering,
  padding & alignment, data validation, and more
* Includes reusable types for common data structures like
  [null-terminated strings](https://docs.rs/binrw/latest/binrw/struct.NullString.html) and
  [data indirection using offsets](https://docs.rs/binrw/latest/binrw/struct.FilePtr.html)
* Parses types from third-party crates using
  [free functions](https://docs.rs/binrw/latest/binrw/docs/attribute#custom-parsers)
  or [value maps](https://docs.rs/binrw/latest/binrw/docs/attribute#map)
* Uses efficient in-memory representations (does not require `#[repr(C)]` or
  `#[repr(packed)]`)
* Code in attributes is written as code, not as strings, for improved ergonomics
  and first-class IDE support
* Supports no_std

## Usage

```rust
#[derive(BinRead)]
#[br(magic = b"DOG", assert(name.len() != 0))]
struct Dog {
    bone_pile_count: u8,

    #[br(big, count = bone_pile_count)]
    bone_piles: Vec<u16>,

    #[br(align_before = 0xA)]
    name: NullString
}

let mut reader = Cursor::new(b"DOG\x02\x00\x01\x00\x12\0\0Rudy\0");
let dog: Dog = reader.read_ne().unwrap();
assert_eq!(dog.bone_piles, &[0x1, 0x12]);
assert_eq!(dog.name.to_string(), "Rudy")
```

For more information, including a more detailed overview of binrw,
[visit the documentation](https://docs.rs/binrw).
