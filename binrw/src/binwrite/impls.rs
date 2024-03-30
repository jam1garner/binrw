use crate::{
    io::{Seek, Write},
    BinResult, BinWrite, Endian,
};
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};
use core::{
    any::Any,
    marker::PhantomData,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU8,
    },
};

macro_rules! binwrite_num_impl {
    ($($type_name:ty),*$(,)?) => {
        $(
            impl BinWrite for $type_name {
                type Args<'a> = ();

                fn write_options<W: Write + Seek>(
                    &self,
                    writer: &mut W,
                    endian: Endian,
                    (): Self::Args<'_>,
                ) -> BinResult<()> {
                    writer.write_all(&match endian {
                        Endian::Big => self.to_be_bytes(),
                        Endian::Little => self.to_le_bytes(),
                    }).map_err(Into::into)
                }
            }
        )*
    };
}

binwrite_num_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

macro_rules! binwrite_nonzero_num_impl {
    ($($non_zero_type:ty => $type_name:ty),*$(,)?) => {
        $(
            impl BinWrite for $non_zero_type {
                type Args<'a> = ();

                fn write_options<W: Write + Seek>(
                    &self,
                    writer: &mut W,
                    endian: Endian,
                    (): Self::Args<'_>,
                ) -> BinResult<()> {
                    let num = <$type_name>::from(*self);

                    writer.write_all(&match endian {
                        Endian::Big => num.to_be_bytes(),
                        Endian::Little => num.to_le_bytes(),
                    }).map_err(Into::into)
                }
            }
        )*
    };
}

binwrite_nonzero_num_impl!(
    NonZeroU8   => u8,
    NonZeroU16  => u16,
    NonZeroU32  => u32,
    NonZeroU64  => u64,
    NonZeroU128 => u128,
    NonZeroI8   => i8,
    NonZeroI16  => i16,
    NonZeroI32  => i32,
    NonZeroI64  => i64,
    NonZeroI128 => i128,
);

impl<T, const N: usize> BinWrite for [T; N]
where
    T: BinWrite + 'static,
    for<'a> T::Args<'a>: Clone,
{
    type Args<'a> = T::Args<'a>;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        if let Some(this) = <dyn Any>::downcast_ref::<[u8; N]>(self) {
            writer.write_all(&this[..])?;
        } else {
            for item in self {
                T::write_options(item, writer, endian, args.clone())?;
            }
        }

        Ok(())
    }
}

impl<T> BinWrite for [T]
where
    T: BinWrite,
    for<'a> T::Args<'a>: Clone,
{
    type Args<'a> = T::Args<'a>;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        for item in self {
            T::write_options(item, writer, endian, args.clone())?;
        }

        Ok(())
    }
}

impl<T> BinWrite for Vec<T>
where
    T: BinWrite + 'static,
    for<'a> T::Args<'a>: Clone,
{
    type Args<'a> = T::Args<'a>;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        if let Some(this) = <dyn Any>::downcast_ref::<Vec<u8>>(self) {
            writer.write_all(this)?;
        } else if let Some(this) = <dyn Any>::downcast_ref::<Vec<i8>>(self) {
            writer.write_all(bytemuck::cast_slice(this.as_slice()))?;
        } else {
            for item in self {
                T::write_options(item, writer, endian, args.clone())?;
            }
        }

        Ok(())
    }
}

impl<T: BinWrite + ?Sized> BinWrite for &T {
    type Args<'a> = T::Args<'a>;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        (**self).write_options(writer, endian, args)
    }
}

impl<T: BinWrite + ?Sized + 'static> BinWrite for Box<T> {
    type Args<'a> = T::Args<'a>;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        if let Some(this) = <dyn Any>::downcast_ref::<Box<[u8]>>(self) {
            writer.write_all(this)?;
        } else {
            (**self).write_options(writer, endian, args)?;
        }

        Ok(())
    }
}

impl<T: BinWrite> BinWrite for Option<T> {
    type Args<'a> = T::Args<'a>;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<()> {
        match self {
            Some(inner) => inner.write_options(writer, endian, args),
            None => Ok(()),
        }
    }
}

impl<T> BinWrite for PhantomData<T> {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        _: &mut W,
        _: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<()> {
        Ok(())
    }
}

impl BinWrite for () {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        _: &mut W,
        _: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<()> {
        Ok(())
    }
}

macro_rules! binwrite_tuple_impl {
    ($type1:ident $(, $types:ident)*) => {
        #[allow(non_camel_case_types)]
        impl<Args: Clone,
            $type1: for<'a> BinWrite<Args<'a> = Args>, $($types: for<'a> BinWrite<Args<'a> = Args>),*
        > BinWrite for ($type1, $($types),*) {
            type Args<'a> = Args;

            fn write_options<W: Write + Seek>(
                &self,
                writer: &mut W,
                endian: Endian,
                args: Self::Args<'_>,
            ) -> BinResult<()> {
                let ($type1, $(
                    $types
                ),*) = self;

                $type1.write_options(writer, endian, args.clone())?;
                $(
                    $types.write_options(writer, endian, args.clone())?;
                )*

                Ok(())
            }
        }

        binwrite_tuple_impl!($($types),*);
    };

    () => {};
}

binwrite_tuple_impl!(
    b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19, b20, b21,
    b22, b23, b24, b25, b26, b27, b28, b29, b30, b31, b32
);
