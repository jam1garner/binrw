use crate::{
    io::{self, Read, Seek, SeekFrom},
    BinRead, BinResult, Endian, Error, ReadOptions,
};
use core::any::Any;
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};

use binrw_derive::BinrwNamedArgs;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};

macro_rules! binread_impl {
    ($($type_name:ty),*$(,)?) => {
        $(
            impl BinRead for $type_name {
                type Args = ();

                fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, _: Self::Args) -> BinResult<Self> {
                    let mut val = [0; core::mem::size_of::<$type_name>()];
                    let pos = reader.stream_position()?;

                    reader.read_exact(&mut val).or_else(|e| {
                        reader.seek(SeekFrom::Start(pos))?;
                        Err(e)
                    })?;
                    Ok(match options.endian() {
                        Endian::Big => {
                            <$type_name>::from_be_bytes(val)
                        }
                        Endian::Little => {
                            <$type_name>::from_le_bytes(val)
                        }
                        Endian::Native => {
                            if cfg!(target_endian = "little") {
                                <$type_name>::from_le_bytes(val)
                            } else {
                                <$type_name>::from_be_bytes(val)
                            }
                        }
                    })
                }
            }
        )*
    }
}

binread_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

fn not_enough_bytes<T>(_: T) -> Error {
    Error::Io(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "not enough bytes in reader",
    ))
}

fn unexpected_zero_num() -> Error {
    Error::Io(io::Error::new(
        io::ErrorKind::InvalidData,
        "unexpected zero found",
    ))
}

macro_rules! binread_nonzero_impl {
    ($($Ty:ty, $Int:ty),* $(,)?) => {
        $(
            impl BinRead for $Ty {
                type Args = ();

                fn read_options<R: Read + Seek>(
                    reader: &mut R,
                    options: &ReadOptions,
                    _: Self::Args,
                ) -> BinResult<Self> {
                    match <$Ty>::new(<$Int>::read_options(reader, options, ())?) {
                        Some(x) => Ok(x),
                        None => Err(unexpected_zero_num()),
                    }
                }
            }
        )+
    }
}

binread_nonzero_impl! {
    NonZeroU8, u8, NonZeroU16, u16, NonZeroU32, u32, NonZeroU64, u64, NonZeroU128, u128,
    NonZeroI8, i8, NonZeroI16, i16, NonZeroI32, i32, NonZeroI64, i64, NonZeroI128, i128,
}

/// Arguments passed to the binread impl for Vec
///
/// # Examples
///
/// ```rust
/// use binrw::{BinRead, io::Cursor};
///
/// #[derive(BinRead, Debug, PartialEq)]
/// struct Collection {
///     count: u32,
///     #[br(args { count: count as usize, inner: ElementBinReadArgs { count: 2 } })]
///     elements: Vec<Element>,
/// }
///
/// #[derive(BinRead, Debug, PartialEq)]
/// #[br(import { count: u32 })]
/// struct Element(#[br(args { count: count as usize, inner: () })] Vec<u8>);
///
/// assert_eq!(
///     Collection::read(&mut Cursor::new(b"\x03\0\0\0\x04\0\x05\0\x06\0")).unwrap(),
///     Collection {
///         count: 3,
///         elements: vec![
///             Element(vec![4, 0]),
///             Element(vec![5, 0]),
///             Element(vec![6, 0])
///         ]
///     }
/// )
/// ```
///
/// Inner types that don't require args take unit args.
///
/// ```rust
/// # use binrw::prelude::*;
/// #[derive(BinRead)]
/// struct Collection {
///     count: u32,
///     #[br(args { count: count as usize, inner: () })]
///     elements: Vec<u32>,
/// }
/// ```
///
/// Unit args for the inner type can be omitted.
/// The [count](crate::docs::attribute#count) attribute also assumes unit args for the inner type.
///
/// ```
/// # use binrw::prelude::*;
/// #[derive(BinRead)]
/// struct Collection {
///     count: u32,
///     #[br(args { count: count as usize })]
///     elements: Vec<u32>,
/// }
/// ```
#[derive(BinrwNamedArgs, Clone)]
pub struct VecArgs<B> {
    /// The number of elements to read.
    pub count: usize,

    /// Arguments to pass to the inner type
    #[named_args(try_optional)]
    pub inner: B,
}

impl<B: BinRead> BinRead for Vec<B> {
    type Args = VecArgs<B::Args>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        let mut list = Self::with_capacity(args.count);

        vec_fast_int!(try (i8 i16 u16 i32 u32 i64 u64 i128 u128) using (list, reader, options, args) else {
            if let Some(bytes) = <dyn Any>::downcast_mut::<Vec<u8>>(&mut list) {
                let byte_count = reader
                    .take(args.count.try_into().map_err(not_enough_bytes)?)
                    .read_to_end(bytes)?;

                if byte_count == args.count {
                    Ok(list)
                } else {
                    Err(not_enough_bytes(()))
                }
            } else {
                for _ in 0..args.count {
                    list.push(B::read_options(reader, options, args.inner.clone())?);
                }
                Ok(list)
            }
        })
    }

    fn after_parse<R>(
        &mut self,
        reader: &mut R,
        ro: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<()>
    where
        R: Read + Seek,
    {
        for val in self.iter_mut() {
            val.after_parse(reader, ro, args.inner.clone())?;
        }

        Ok(())
    }
}

macro_rules! vec_fast_int {
    (try ($($Ty:ty)+) using ($list:expr, $reader:expr, $options:expr, $args:expr) else { $($else:tt)* }) => {
        $(if let Some(list) = <dyn Any>::downcast_mut::<Vec<$Ty>>(&mut $list) {
            // In benchmarks, this resize decreases performance by
            // 27–40% relative to using `unsafe` to write directly to
            // uninitialised memory, but nobody ever got fired for buying IBM
            list.resize($args.count, 0);
            $reader.read_exact(&mut bytemuck::cast_slice_mut::<_, u8>(list.as_mut_slice()))?;
            if
                core::mem::size_of::<$Ty>() != 1
                && (
                    (cfg!(target_endian = "big") && $options.endian() == Endian::Little)
                    || (cfg!(target_endian = "little") && $options.endian() == Endian::Big)
                )
            {
                for value in list.iter_mut() {
                    *value = value.swap_bytes();
                }
            }
            Ok($list)
        } else)* {
            $($else)*
        }
    }
}

use vec_fast_int;

impl<B: BinRead, const N: usize> BinRead for [B; N] {
    type Args = B::Args;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        array_init::try_array_init(|_| BinRead::read_options(reader, options, args.clone()))
    }

    fn after_parse<R>(&mut self, reader: &mut R, ro: &ReadOptions, args: B::Args) -> BinResult<()>
    where
        R: Read + Seek,
    {
        for val in self.iter_mut() {
            val.after_parse(reader, ro, args.clone())?;
        }

        Ok(())
    }
}

macro_rules! binread_tuple_impl {
    ($type1:ident $(, $types:ident)*) => {
        #[allow(non_camel_case_types)]
        impl<$type1: BinRead<Args=()>, $($types: BinRead<Args=()>),*> BinRead for ($type1, $($types),*) {
            type Args = ();

            fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, _: Self::Args) -> BinResult<Self> {
                Ok((
                    BinRead::read_options(reader, options, ())?,
                    $(
                        <$types>::read_options(reader, options, ())?
                    ),*
                ))
            }

            fn after_parse<R: Read + Seek>(&mut self, reader: &mut R, options: &ReadOptions, _: Self::Args) -> BinResult<()> {
                let ($type1, $(
                    $types
                ),*) = self;

                $type1.after_parse(reader, options, ())?;
                $(
                    $types.after_parse(reader, options, ())?;
                )*

                Ok(())
            }
        }

        binread_tuple_impl!($($types),*);
    };

    () => {};
}

binread_tuple_impl!(
    b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19, b20, b21,
    b22, b23, b24, b25, b26, b27, b28, b29, b30, b31, b32
);

impl BinRead for () {
    type Args = ();

    fn read_options<R: Read + Seek>(_: &mut R, _: &ReadOptions, _: Self::Args) -> BinResult<Self> {
        Ok(())
    }
}

impl<T: BinRead> BinRead for Box<T> {
    type Args = T::Args;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        Ok(Box::new(T::read_options(reader, options, args)?))
    }
}

impl<T: BinRead> BinRead for Option<T> {
    type Args = T::Args;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        Ok(Some(T::read_options(reader, options, args)?))
    }

    fn after_parse<R>(
        &mut self,
        reader: &mut R,
        ro: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<()>
    where
        R: Read + Seek,
    {
        match self {
            Some(val) => val.after_parse(reader, ro, args),
            None => Ok(()),
        }
    }
}

impl<T: 'static> BinRead for core::marker::PhantomData<T> {
    type Args = ();

    fn read_options<R: Read + Seek>(_: &mut R, _: &ReadOptions, _: Self::Args) -> BinResult<Self> {
        Ok(core::marker::PhantomData)
    }
}
