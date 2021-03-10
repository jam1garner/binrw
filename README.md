# binread

[![tests](https://github.com/jam1garner/binread/actions/workflows/cargo_tests.yml/badge.svg)](https://github.com/jam1garner/binread/actions/workflows/cargo_tests.yml)
[![docs.rs](https://docs.rs/binread/badge.svg)](https://docs.rs/binread)
[![codecov](https://codecov.io/gh/jam1garner/binread/branch/master/graph/badge.svg?token=UREOWI2KAY)](https://codecov.io/gh/jam1garner/binread) 
[![discord](https://img.shields.io/discord/818723403871551509)](https://discord.gg/ABy4Qh549j)

A Rust crate for helping parse structs from binary data using ✨macro magic✨


## Usage

BinRead uses a derive macro for declaratively defining binary parsing methods for structs.

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
assert_eq!(dog.name.into_string(), "Rudy")
```

[More documentation can be found here](https://docs.rs/binread/)
