<style>
.intro { display: flex; flex-direction: column-reverse; }
.ql { display: table; border-bottom: thin solid var(--color-border, #ddd); margin-bottom: .75em; padding-bottom: .75em; }
.ql_h.ql_h.ql_h { border: initial; font: inherit; font-size: 1em; font-weight: bold; padding: 0 .75em 0 0; white-space: nowrap; width: 0; }
.ql > * { display: table-row; }
.ql > *     > * { display: table-cell; }
.ql > * + * > *,
.ql > * + * > .ql_h.ql_h { padding-top: .25em; }
.ql p { display: inline; margin: 0 .75em 0 0; }
.ql p:last-child { margin-right: 0; }
.ql code { white-space: nowrap; }
</style>

<div class="intro">

binrw helps you write maintainable & easy-to-read declarative binary data
readers and writers using ✨macro magic✨.

<div class="ql">

<nav>

<h2 class="ql_h">Quick links</h2>

<div>

[`#[br]`/`#[bw]`/`#[brw]`](docs::attribute)

[`BinReaderExt`]

[`BinWriterExt`]

[`helpers`]
</div>
</nav>

<nav>

<h2 class="ql_h">Need help?</h2>

<div>

[GitHub]

[Discord]

[Matrix]

[More documentation](docs)
</div>
</nav>
</div>

[GitHub]: https://github.com/jam1garner/binrw/discussions/categories/q-a
[Discord]: https://discord.gg/ABy4Qh549j
[Matrix]: https://matrix.to/#/%23binrw:matrix.org
</div>

Adding [`#[binrw]`](macro@binrw) (or `#[derive(BinRead, BinWrite)]`) to a
struct or enum generates a parser that can read that type from raw data and a
serialiser that can write it back:

```
use binrw::{
    binrw,    // #[binrw] attribute
    BinRead,  // trait for reading
    BinWrite, // trait for writing
};
# use binrw::io::Cursor;

#[binrw]
# #[derive(Debug, PartialEq)]
#[brw(little)]
struct Point(i16, i16);

// Read a point from bytes
let point = Point::read(&mut Cursor::new(b"\x80\x02\xe0\x01")).unwrap();
assert_eq!(point, Point(640, 480));

// Write the point back to bytes
let mut writer = Cursor::new(Vec::new());
point.write(&mut writer).unwrap();
assert_eq!(writer.into_inner(), b"\x80\x02\xe0\x01");
```

binrw types are composable and nestable, so everything just works as expected
without any special logic or glue code:

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

Types that can’t implement binrw traits directly (e.g. types from third party
crates) can also be read and written using
[free parser functions](docs::attribute#custom-parserswriters) or by
[mapping values](docs::attribute#map).

Unlike “zero-copy” libraries, the in-memory representation of binrw structs
doesn’t need to match the raw data. This can allow for better memory
performance, especially on architectures where unaligned memory access is
slow. Also, because data is never [transmuted](core::mem::transmute), there
is no risk of undefined behaviour.

# Input and output

binrw reads data from any object that implements [`io::Read`] + [`io::Seek`],
and writes data to any object that implements [`io::Write`] + [`io::Seek`].
(Unseekable streams are also supported, but require a [wrapper](io::NoSeek).)
This means that data can come from memory, network, disk, or any other streaming
source. It also means that low-level data operations like
[buffering](io::BufReader) and compression are efficient and easy to
implement.

binrw also includes extension traits for conveniently [reading](BinReaderExt)
and [writing](BinWriterExt) directly on the stream objects:

```
use binrw::{BinReaderExt, BinWriterExt};
# use binrw::io::Cursor;

let mut stream = Cursor::new(b"\x00\x0a".to_vec());
let val: u16 = stream.read_be().unwrap();
assert_eq!(val, 0xa);

let val = val + 0x10;
stream.write_be(&val).unwrap();
assert_eq!(stream.into_inner(), b"\x00\x0a\x00\x1a");
```

# Directives

Handling things like magic numbers, byte ordering, and padding & alignment
is typical when working with binary data, so binrw includes a variety of
[built-in directives](docs::attribute) for these common cases that can be applied
using the `#[br]`, `#[bw]`, and `#[brw]` attributes:

```
# use binrw::{prelude::*, io::Cursor, NullString};
#
#[binrw]
#[brw(big, magic = b"DOG", assert(name.len() != 0))]
struct Dog {
    #[bw(try_calc(u8::try_from(bone_piles.len())))]
    bone_pile_count: u8,

    #[br(count = bone_pile_count)]
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

See the [`BinRead`](BinRead#foreign-impls) and
[`BinWrite`](BinWrite#foreign-impls) traits for the full list of built-in
implementations.

# no_std support

binrw supports no_std and includes a compatible subset of [`io`]
functionality. The [`alloc`] crate is required.
