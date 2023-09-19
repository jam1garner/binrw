Tips for high performance parsing and serialisation.

# Use buffered inputs

During parsing, binrw regularly [queries the position](crate::io::Seek::stream_position)
of the input stream to provide byte- and field-accurate error reporting. It will
also rewind the stream when an enum variant fails to parse to try the next one,
or when unrecoverable errors occur to leave the input stream in a consistent
state.

In-memory streams like [`std::io::Cursor`] have no problem with this access
pattern, but others like [`std::fs::File`] may be unexpectedly slow due to
system call overhead. To improve performance of these stream types, simply wrap
them with [`BufReader`](crate::io::BufReader).

**[`std::io::BufReader`] from the standard library invalidates its buffer every
time a seek occurs so may make performance worse.** Use only the binrw wrapper
or some other alternative that does not invalidate its internal buffer on seek.

# Most common enum variants first

binrw parsing starts at the top of an enum and works its way down the list of
variants until one of them parses successfully. Putting the most common variants
first will reduce the amount of work binrw needs to do to parse an enum.

# Discard enum errors

When parsing an enum, binrw generates an [`Error`](crate::Error) for each
variant that fails parsing. By default, these errors are all collected in a
[`Vec`] until one variant parses successfully (at which point they are
discarded) or until parsing fails (at which point they are passed to the
caller).

The [`#[br(return_unexpected_error)]`](crate::docs::attribute#enum-errors)
directive stops binrw from collecting any variant parsing errors, which
eliminates extra work spent allocating and freeing memory for storing errors.
The downside is that [only the stream position](crate::Error::NoVariantMatch)
can be provided when an enum fails to parse because the errors were thrown away.

# Use specific types for faster block I/O

To improve performance when reading or writing large blocks of data, binrw uses
a fake specialisation technique to generate optimised I/O calls for certain
types:

| Type                 | Read | Write |
|----------------------|------|-------|
| `Vec<u8>`            | yes  | yes   |
| `Vec<i8>`            | yes  | yes   |
| `Vec<u16>`           | yes  | no    |
| `Vec<i16>`           | yes  | no    |
| `Vec<u32>`           | yes  | no    |
| `Vec<i32>`           | yes  | no    |
| `Vec<u64>`           | yes  | no    |
| `Vec<i64>`           | yes  | no    |
| `Vec<u128>`          | yes  | no    |
| `Vec<i128>`          | yes  | no    |
| `[u8; N]`            | no   | yes   |
| `Box<[u8]>`          | no   | yes   |

# Avoid random access patterns

Reading data non-sequentially may reduce the effectiveness of hardware
prefetching and cause read buffers to be flushed prematurely and excessively.

See the [`file_ptr`](binrw::file_ptr) documentation for details on how to
improve performance by avoiding extra seeking when parsing offset tables.
