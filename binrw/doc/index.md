Need help or want to contribute? Join us on [discord] or [matrix]! (whichever you prefer, they're
bridged together)

---

|**Quick links**|[`#[br]`/`#[bw]`/`#[brw]`](docs::attribute)|[`BinReaderExt`]|[discord]|[matrix]|
|-|-|-|-|-|

---

[discord]: https://discord.gg/ABy4Qh549j
[matrix]: https://matrix.to/#/#binrw:matrix.org

binrw helps you write maintainable & easy-to-read declarative binary data
readers and writers using ✨macro magic✨.

Adding [`#[binrw]`](macro@binrw) (or `#[derive(BinRead, BinWrite)]`) to a struct or enum
generates a parser that can read that type from raw data and a writer that
can write it back to bytes:

```
use binrw::binrw; // #[binrw] attribute
use binrw::{BinReaderExt, BinWrite, io::Cursor}; // reading/writing utilities

#[binrw]
# #[derive(Debug, PartialEq)]
#[br(little)]
struct Point(i16, i16);

// Read a point from bytes
let point: Point = Cursor::new(b"\x80\x02\xe0\x01").read_le().unwrap();
assert_eq!(point, Point(640, 480));

// Write the point back to bytes
let mut writer = Cursor::new(Vec::new());
point.write_to(&mut writer).unwrap();
assert_eq!(&writer.into_inner()[..], b"\x80\x02\xe0\x01");
```

These types are composable, allowing you to use [`BinRead`]/[`BinWrite`] types within
others without any special logic:

```
# use binrw::{binrw, BinRead, BinWrite, io::Cursor};
# #[binrw]
# #[derive(Debug, PartialEq)]
# #[br(little)]
# struct Point(i16, i16);
#
# #[derive(Debug, PartialEq)]
#[derive(BinRead)]
#[br(big, magic = b"SHAP")]
enum Shape {
    #[br(magic(0u8))] Rect {
        left: i16, top: i16, right: i16, bottom: i16
    },
    #[br(magic(1u8))] Oval { origin: Point, rx: u8, ry: u8 }
}

let oval = Shape::read(&mut Cursor::new(b"SHAP\x01\x80\x02\xe0\x01\x2a\x15")).unwrap();
assert_eq!(oval, Shape::Oval { origin: Point(640, 480), rx: 42, ry: 21 });
```

Types that can’t implement `BinRead` directly (e.g. types from third party
crates) can also be read using
[free parser functions](docs::attribute#custom-parserswriters) or by
[mapping values](docs::attribute#map).

Unlike “zero-copy” libraries, the in-memory representation of binrw structs
doesn’t need to match the raw data. This can allow for better memory
performance, especially on architectures where unaligned memory access is
slow. Also, because data is never [transmuted](core::mem::transmute), there
is no risk of undefined behaviour.

# Input

`BinRead` reads data from any object that implements [`io::Read`] +
[`io::Seek`]. This means that data can come from memory, network, disk, or
any other streaming source. It also means that low-level data operations
like buffering and compression are efficient and easy to implement.

`BinRead` also includes an [extension trait](BinReaderExt) for reading types
directly from input objects:

```
use binrw::{BinReaderExt, io::Cursor};

let mut reader = Cursor::new(b"\x00\x0A");
let val: u16 = reader.read_be().unwrap();
assert_eq!(val, 10);
```

# Directives

Handling things like magic numbers, byte ordering, and padding & alignment
is typical when working with binary data, so binrw includes a variety of
[built-in directives](docs::attribute) for these common cases that can be applied
using the `#[br]` attribute:

```
# use binrw::{prelude::*, io::Cursor, NullString};
#
#[derive(BinRead)]
#[br(magic = b"DOG", assert(name.len() != 0))]
struct Dog {
    bone_pile_count: u8,

    #[br(big, count = bone_pile_count)]
    bone_piles: Vec<u16>,

    #[br(align_before = 0xA)]
    name: NullString
}

let mut data = Cursor::new(b"DOG\x02\x00\x01\x00\x12\0\0Rudy\0");
let dog = Dog::read(&mut data).unwrap();
assert_eq!(dog.bone_piles, &[0x1, 0x12]);
assert_eq!(dog.name.to_string(), "Rudy")
```

Directives can also reference earlier fields by name. For tuple types,
earlier fields are addressable by `self_N`, where `N` is the index of the
field.

See the [attribute documentation](docs::attribute) for the full list of
available directives.

# Built-in implementations

Implementations for all primitive data types, arrays, tuples, and standard
Rust types like [`Vec`] are included, along with parsers for other
frequently used binary data patterns like
[null-terminated strings](NullString) and
[indirect addressing using offsets](FilePtr). Convenient access into
bitfields is possible using crates like
[modular-bitfield](docs::attribute#using-map-on-a-struct-to-create-a-bit-field).

See the [`BinRead` trait](BinRead#foreign-impls) for the full list of
built-in implementations.

# no_std support

binrw supports no_std and includes a compatible subset of [`io`]
functionality. The [`alloc`] crate is required.
