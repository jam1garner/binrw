use core::any::Any;
use core::marker::PhantomData;

use crate::alloc::boxed::Box;
use crate::alloc::vec::Vec;
use crate::{
    arg_type, args_of,
    io::{Seek, Write},
    BinResult, BinWrite, Endian, WriteOptions,
};

// ============================= nums =============================

macro_rules! binwrite_num_impl {
    ($($type_name:ty),*$(,)?) => {
        $(
            impl BinWrite for $type_name {
                type Args = arg_type!(());

                fn write_options<W: Write + Seek>(
                    &self,
                    writer: &mut W,
                    options: &WriteOptions,
                    _: (),
                ) -> BinResult<()> {
                    writer.write_all(&match options.endian() {
                        Endian::Big => self.to_be_bytes(),
                        Endian::Little => self.to_le_bytes(),
                        Endian::Native => self.to_ne_bytes(),
                    }).map_err(Into::into)
                }
            }
        )*
    };
}

binwrite_num_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

// =========================== end nums ===========================

// =========================== array/vec ===========================

impl<T: BinWrite + 'static, const N: usize> BinWrite for [T; N] {
    type Args = T::Args;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: args_of!(Self),
    ) -> BinResult<()> {
        if let Some(this) = <dyn Any>::downcast_ref::<[u8; N]>(self) {
            writer.write_all(&this[..])?;
        } else {
            for item in self {
                T::write_options(item, writer, options, args.clone())?;
            }
        }

        Ok(())
    }
}

impl<T: BinWrite> BinWrite for [T] {
    type Args = T::Args;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: args_of!(Self),
    ) -> BinResult<()> {
        for item in self {
            T::write_options(item, writer, options, args.clone())?;
        }

        Ok(())
    }
}

impl<T: BinWrite + 'static> BinWrite for Vec<T> {
    type Args = T::Args;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: args_of!(Self),
    ) -> BinResult<()> {
        if let Some(this) = <dyn Any>::downcast_ref::<Vec<u8>>(self) {
            writer.write_all(this)?;
        } else {
            for item in self {
                T::write_options(item, writer, options, args.clone())?;
            }
        }

        Ok(())
    }
}

// ========================= end array/vec =========================

// ========================= std types =========================

impl<T: BinWrite + ?Sized> BinWrite for &T {
    type Args = T::Args;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: args_of!(Self),
    ) -> BinResult<()> {
        (**self).write_options(writer, options, args)
    }
}

impl<T: BinWrite + 'static> BinWrite for Box<T> {
    type Args = T::Args;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: args_of!(Self),
    ) -> BinResult<()> {
        if let Some(this) = <dyn Any>::downcast_ref::<Box<[u8]>>(self) {
            writer.write_all(this)?;
        } else {
            (**self).write_options(writer, options, args)?;
        }

        Ok(())
    }
}

impl<T: BinWrite> BinWrite for Option<T> {
    type Args = T::Args;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: args_of!(Self),
    ) -> BinResult<()> {
        match self {
            Some(inner) => inner.write_options(writer, options, args),
            None => Ok(()),
        }
    }
}

impl<T: BinWrite> BinWrite for PhantomData<T> {
    type Args = T::Args;

    fn write_options<W: Write + Seek>(
        &self,
        _: &mut W,
        _: &WriteOptions,
        _: args_of!(Self),
    ) -> BinResult<()> {
        Ok(())
    }
}

// ======================= end std types =======================

// =========================== tuples ===========================

impl BinWrite for () {
    type Args = arg_type!(());

    fn write_options<W: Write + Seek>(&self, _: &mut W, _: &WriteOptions, _: ()) -> BinResult<()> {
        Ok(())
    }
}

macro_rules! binwrite_tuple_impl {
    ($type1:ident $(, $types:ident)*) => {
        #[allow(non_camel_case_types)]
        impl<
            $type1: BinWrite<Args=arg_type!(())>, $($types: BinWrite<Args=arg_type!(())>),*
        > BinWrite for ($type1, $($types),*) {
            type Args = arg_type!(());

            fn write_options<W: Write + Seek>(
                &self,
                writer: &mut W,
                options: &WriteOptions,
                _: (),
            ) -> BinResult<()> {
                let ($type1, $(
                    $types
                ),*) = self;

                $type1.write_options(writer, options, ())?;
                $(
                    $types.write_options(writer, options, ())?;
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

// ========================= end tuples =========================
