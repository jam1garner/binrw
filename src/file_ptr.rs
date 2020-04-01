//! A wrapper type for representing a layer of indirection within a file.
//! 
//! A `FilePtr<P, T>` is composed of two types: a pointer type `P` and a value type `T` where
//! the pointer type describes and offset to read the value type from. Once read from the file
//! it can be dereferenced to yeild the value it points to.
//! 
//! ## Example
//! ```rust
//! use binread::{prelude::*, io::Cursor, FilePtr};
//!
//! #[derive(BinRead)]
//! struct Test {
//!     pointer: FilePtr<u32, u8> 
//! }
//! 
//! fn main() {
//!     let test: Test = Cursor::new(b"\0\0\0\x08\0\0\0\0\xff").read_be().unwrap();
//!     assert_eq!(test.pointer.ptr, 8);
//!     assert_eq!(*test.pointer, 0xFF);
//! }
//! ```
//! 
//! Example data mapped out:
//! ```hex
//!           [pointer]           [value]
//! 00000000: 0000 0008 0000 0000 ff                   ............
//! ```
//! 
//! Use `offset` to change what the pointer is relative to (default: beginning of reader).
use super::*;
use std::fmt;
use core::ops::{Deref, DerefMut};

/// A wrapper type for representing a layer of indirection within a file.
/// 
/// A `FilePtr<P, T>` is composed of two types: a pointer type `P` and a value type `T` where
/// the pointer type describes and offset to read the value type from. Once read from the file
/// it can be dereferenced to yeild the value it points to.
/// 
/// ## Example
/// ```rust
/// use binread::{prelude::*, io::Cursor, FilePtr};
///
/// #[derive(BinRead)]
/// struct Test {
///     pointer: FilePtr<u32, u8> 
/// }
/// 
/// fn main() {
///     let test: Test = Cursor::new(b"\0\0\0\x08\0\0\0\0\xff").read_be().unwrap();
///     assert_eq!(test.pointer.ptr, 8);
///     assert_eq!(*test.pointer, 0xFF);
/// }
/// ```
/// 
/// Example data mapped out:
/// ```hex
///           [pointer]           [value]
/// 00000000: 0000 0008 0000 0000 ff                   ............
/// ```
/// 
/// Use `offset` to change what the pointer is relative to (default: beginning of reader).
pub struct FilePtr<Ptr: IntoSeekFrom, BR: BinRead> {
    pub ptr: Ptr,
    pub value: Option<BR>
}

impl<Ptr: BinRead<Args = ()> + IntoSeekFrom, BR: BinRead> BinRead for FilePtr<Ptr, BR> {
    type Args = BR::Args;
    
    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, _: Self::Args) -> BinResult<Self> {
        #[cfg(feature = "debug_template")]
        let options = &{
            let mut options = options.clone();

            let pos = reader.seek(SeekFrom::Current(0)).unwrap();
            let type_name = &core::any::type_name::<Ptr>();
            if let Some(name) = options.variable_name {
                binary_template::write_named(
                    options.endian,
                    pos,
                    type_name,
                    &format!("ptr_to_{}", name)
                );
            } else {
                binary_template::write(
                    options.endian,
                    pos,
                    type_name,
                );
            }
            options.dont_output_to_template = true;

            options
        };

        Ok(FilePtr{
            ptr: Ptr::read_options(reader, options, ())?,
            value: None
        })
    }

    fn after_parse<R>(&mut self, reader: &mut R, ro: &ReadOptions, args: BR::Args, ao: &AfterParseOptions)-> BinResult<()>
        where R: Read + Seek,
    {
        let relative_to = ao.offset;
        let before = reader.seek(SeekFrom::Current(0))?;
        reader.seek(SeekFrom::Start(relative_to))?;
        reader.seek(self.ptr.into_seek_from())?;

        self.value = Some(BinRead::read_options(reader, ro, args)?);

        reader.seek(SeekFrom::Start(before))?;
        Ok(())
    }
}

impl<Ptr: IntoSeekFrom, BR: BinRead> FilePtr<Ptr, BR> {
    pub fn read<R: Read + Seek>(&mut self, reader: &mut R) -> BinResult<()> {
        self.value = Some(BR::read(reader)?);
        Ok(())
    }
}

/// Used to allow any convert any type castable to i64 into a [`SeekFrom::Current`](io::SeekFrom::Current)
pub trait IntoSeekFrom: Copy {
    fn into_seek_from(self) -> SeekFrom;
}

macro_rules! impl_into_seek_from {
    ($($t:ty),*) => {
        $(
            impl IntoSeekFrom for $t {
                fn into_seek_from(self) -> SeekFrom {
                    SeekFrom::Current(self as i64)
                }
            }
        )*
    };
}

impl_into_seek_from!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

/// ## Panics
/// Will panic if the FilePtr has not been read yet using [`BinRead::after_parse`](BinRead::after_parse)
impl<Ptr: IntoSeekFrom, BR: BinRead> Deref for FilePtr<Ptr, BR> {
    type Target = BR;

    fn deref(&self) -> &Self::Target {
        match self.value.as_ref() {
            Some(x) => x,
            None => panic!("Deref'd FilePtr before reading (make sure to use FilePtr::after_parse first)")
        }
    }
}

/// ## Panics
/// Will panic if the FilePtr has not been read yet using [`BinRead::after_parse`](BinRead::after_parse)
impl<Ptr: IntoSeekFrom, BR: BinRead> DerefMut for FilePtr<Ptr, BR> {
    fn deref_mut(&mut self) -> &mut BR {
        match self.value.as_mut() {
            Some(x) => x,
            None => panic!("Deref'd FilePtr before reading (make sure to use FilePtr::after_parse first)")
        }
    }
}

impl<Ptr, BR> fmt::Debug for FilePtr<Ptr, BR>
    where Ptr: BinRead<Args = ()> + IntoSeekFrom,
          BR: BinRead + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref value) = self.value {
            fmt::Debug::fmt(value, f)
        } else {
            write!(f, "UnreadPointer")
        }
    }
}
