use crate::{
    io::{self, Read, Seek},
    BinRead, BinResult, Endian, Error, NamedArgs,
};
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
use core::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};

macro_rules! binread_impl {
    // `$(some_lit $(__unused $is_lit_present:tt))?` allows us to match on the present of a literal
    // using `$($($is_list_present)? fn do_whatever() {})?`
    ($($type_name:ty $(as int $(__unused $is_int:tt)?)?),*$(,)?) => {
        $(
            impl BinRead for $type_name {
                type Args<'a> = ();

                fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, (): Self::Args<'_>) -> BinResult<Self> {
                    let mut val = [0; core::mem::size_of::<$type_name>()];
                    let pos = reader.stream_position()?;

                    reader.read_exact(&mut val).or_else(crate::__private::restore_position(reader, pos))?;
                    Ok(match endian {
                        Endian::Big => {
                            <$type_name>::from_be_bytes(val)
                        }
                        Endian::Little => {
                            <$type_name>::from_le_bytes(val)
                        }
                    })
                }

                $(
                    $($is_int)?
                    fn read_options_count<'a, R>(
                        reader: &mut R,
                        endian: Endian,
                        _args: Self::Args<'a>,
                        count: usize,
                    ) -> BinResult<Vec<Self>>
                    where
                        R: Read + Seek,
                        Self::Args<'a>: Clone,
                    {
                        let mut list = Vec::<Self>::new();
                        let mut start = 0;
                        let mut remaining = count;
                        // Allocating and reading from the source in chunks is done to keep
                        // a bad `count` from causing huge memory allocations that are
                        // doomed to fail
                        while remaining != 0 {
                            // Using a similar strategy as std `default_read_to_end` to
                            // leverage the memory growth strategy of the underlying Vec
                            // implementation (in std this will be exponential) using a
                            // minimum byte allocation
                            let growth: usize = 32 / core::mem::size_of::<Self>();
                            list.reserve(remaining.min(growth.max(1)));

                            let items_to_read = remaining.min(list.capacity() - start);
                            let end = start + items_to_read;

                            // In benchmarks, this resize decreases performance by 27–40%
                            // relative to using `unsafe` to write directly to uninitialised
                            // memory, but nobody ever got fired for buying IBM
                            list.resize(end, 0);
                            reader.read_exact(&mut bytemuck::cast_slice_mut::<_, u8>(
                                &mut list[start..end],
                            ))?;

                            remaining -= items_to_read;
                            start += items_to_read;
                        }

                        if core::mem::size_of::<Self>() != 1
                            && ((cfg!(target_endian = "big") && endian == crate::Endian::Little)
                                || (cfg!(target_endian = "little") && endian == crate::Endian::Big))
                        {
                            for value in list.iter_mut() {
                                *value = value.swap_bytes();
                            }
                        }
                        Ok(list)
                    }
                )?
            }
        )*
    }
}

binread_impl!(
    u16 as int,
    u32 as int,
    u64 as int,
    u128 as int,
    i8 as int,
    i16 as int,
    i32 as int,
    i64 as int,
    i128 as int,
    f32,
    f64
);

impl BinRead for u8 {
    type Args<'a> = ();
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut val = 0u8;
        let pos = reader.stream_position()?;
        reader
            .read_exact(core::slice::from_mut(&mut val))
            .or_else(crate::__private::restore_position(reader, pos))?;

        Ok(val)
    }

    // This extra impl for `u8` makes it faster than
    // `binread_impl`, but *only* because `binread_impl` is not allowed
    // to use unsafe code to eliminate the unnecessary zero-fill.
    // Otherwise, performance would be identical and it could be
    // deleted.
    fn read_options_count<'a, R>(
        reader: &mut R,
        _endian: Endian,
        _args: Self::Args<'a>,
        count: usize,
    ) -> BinResult<Vec<Self>>
    where
        R: Read + Seek,
        Self::Args<'a>: Clone,
    {
        let mut container = Vec::<u8>::new();
        container.reserve_exact(count);
        let byte_count = reader
            .take(count.try_into().map_err(crate::helpers::not_enough_bytes)?)
            .read_to_end(&mut container)?;

        if byte_count == count {
            Ok(container)
        } else {
            Err(crate::helpers::not_enough_bytes(()))
        }
    }
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
                type Args<'a> = ();

                fn read_options<R: Read + Seek>(
                    reader: &mut R,
                    endian: Endian,
                    (): Self::Args<'_>,
                ) -> BinResult<Self> {
                    match <$Ty>::new(<$Int>::read_options(reader, endian, ())?) {
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

impl<B> BinRead for Vec<B>
where
    B: BinRead,
    for<'a> B::Args<'a>: Clone,
{
    type Args<'a> = VecArgs<B::Args<'a>>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        B::read_options_count(reader, endian, args.inner, args.count)
    }
}

impl<B, const N: usize> BinRead for [B; N]
where
    B: BinRead,
    for<'a> B::Args<'a>: Clone,
    [B; N]: for<'b> TryFrom<&'b [B]>,
{
    type Args<'a> = B::Args<'a>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        // Not a fan of this. Is there a better signature for read_options_count
        // that could handle arrays in a nicer fashion, without compromising
        // ergonomics too much?
        let pos = reader.stream_position()?;
        BinRead::read_options_count(reader, endian, args, N).and_then(|v| {
            Self::try_from(&v).map_err(|_e| Error::AssertFail {
                pos,
                message: "Unable to turn slice into array".to_string(),
            })
        })
        //array_init::try_array_init(|_| BinRead::read_options(reader, endian, args.clone()))
    }
}

macro_rules! binread_tuple_impl {
    ($type1:ident $(, $types:ident)*) => {
        #[allow(non_camel_case_types)]
        impl<Args: Clone, $type1: for<'a> BinRead<Args<'a> = Args>, $($types: for<'a> BinRead<Args<'a> = Args>),*> BinRead for ($type1, $($types),*) {
            type Args<'a> = Args;

            fn read_options<R: Read + Seek>(reader: &mut R, endian: Endian, args: Self::Args<'_>) -> BinResult<Self> {
                Ok((
                    BinRead::read_options(reader, endian, args.clone())?,
                    $(
                        <$types>::read_options(reader, endian, args.clone())?
                    ),*
                ))
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
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(_: &mut R, _: Endian, (): Self::Args<'_>) -> BinResult<Self> {
        Ok(())
    }
}

impl<T: BinRead> BinRead for Box<T> {
    type Args<'a> = T::Args<'a>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        Ok(Box::new(T::read_options(reader, endian, args)?))
    }
}

impl<T: BinRead> BinRead for Option<T> {
    type Args<'a> = T::Args<'a>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        Ok(Some(T::read_options(reader, endian, args)?))
    }
}

impl<T> BinRead for core::marker::PhantomData<T> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(_: &mut R, _: Endian, (): Self::Args<'_>) -> BinResult<Self> {
        Ok(core::marker::PhantomData)
    }
}
