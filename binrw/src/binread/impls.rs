use crate::{
    io::{self, Read, Seek, SeekFrom},
    BinRead, BinResult, Endian, Error, NamedArgs, ReadOptions,
};
use alloc::{boxed::Box, vec::Vec};
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};

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
                    })
                }
            }
        )*
    }
}

binread_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

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

/// Named arguments for the [`BinRead::read_options()`] implementation of [`Vec`].
///
/// # Examples
///
/// ```
/// use binrw::{BinRead, io::Cursor};
///
/// #[derive(BinRead)]
/// # #[derive(Debug, PartialEq)]
/// #[br(little)]
/// struct Collection {
///     count: u16,
///     #[br(args {
///         count: count.into(),
///         inner: ElementBinReadArgs { count: 2 },
///     })]
///     elements: Vec<Element>,
/// }
///
/// #[derive(BinRead)]
/// # #[derive(Debug, PartialEq)]
/// #[br(import { count: usize })]
/// struct Element(#[br(args {
///     count,
///     inner: <_>::default(),
/// })] Vec<u8>);
///
/// assert_eq!(
///     Collection::read(&mut Cursor::new(b"\x03\0\x04\0\x05\0\x06\0")).unwrap(),
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
/// The `inner` field can be omitted completely if the inner type doesn’t
/// require arguments, in which case a default value will be used:
///
/// ```
/// # use binrw::prelude::*;
/// #[derive(BinRead)]
/// struct Collection {
///     count: u16,
///     #[br(args { count: count.into() })]
///     elements: Vec<u32>,
/// }
/// ```
#[derive(NamedArgs, Clone)]
pub struct VecArgs<Inner: Clone> {
    /// The number of elements to read.
    pub count: usize,

    /// The [arguments](crate::BinRead::Args) for the inner type.
    #[named_args(try_optional)]
    pub inner: Inner,
}

impl<B: BinRead> BinRead for Vec<B> {
    type Args = VecArgs<B::Args>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        crate::helpers::count_with(args.count, B::read_options)(reader, options, args.inner)
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
        impl<Args: Clone, $type1: BinRead<Args=Args>, $($types: BinRead<Args=Args>),*> BinRead for ($type1, $($types),*) {
            type Args = Args;

            fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self> {
                Ok((
                    BinRead::read_options(reader, options, args.clone())?,
                    $(
                        <$types>::read_options(reader, options, args.clone())?
                    ),*
                ))
            }

            fn after_parse<R: Read + Seek>(&mut self, reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<()> {
                let ($type1, $(
                    $types
                ),*) = self;

                $type1.after_parse(reader, options, args.clone())?;
                $(
                    $types.after_parse(reader, options, args.clone())?;
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
