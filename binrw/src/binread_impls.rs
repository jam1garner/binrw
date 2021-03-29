use crate::{
    io::{self, Read, Seek, SeekFrom},
    BinRead, BinResult, Endian, Error, ReadOptions,
};
use core::any::Any;
use core::convert::TryInto;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};

macro_rules! binread_impl {
    ($($type_name:ty),*$(,)?) => {
        $(
            impl BinRead for $type_name {
                type Args = ();

                fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, _: Self::Args) -> BinResult<Self> {
                    let mut val = [0; core::mem::size_of::<$type_name>()];
                    let pos = reader.seek(SeekFrom::Current(0))?;

                    reader.read_exact(&mut val).or_else(|e| {
                        reader.seek(SeekFrom::Start(pos))?;
                        Err(e)
                    })?;
                    Ok(match options.endian {
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

impl BinRead for char {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        // TODO: somehow do proper unicode handling?
        Ok(<u8>::read_options(reader, options, ())? as char)
    }
}

binread_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

fn not_enough_bytes<T>(_: T) -> Error {
    Error::Io(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "not enough bytes in reader",
    ))
}

impl<B: BinRead> BinRead for Vec<B> {
    type Args = B::Args;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        args: Self::Args,
    ) -> BinResult<Self> {
        let mut options = *options;
        let count = match options.count.take() {
            Some(x) => x,
            None => panic!("Missing count for Vec"),
        };

        let mut list = Self::with_capacity(count);

        if let Some(bytes) = <dyn Any>::downcast_mut::<Vec<u8>>(&mut list) {
            let byte_count = reader
                .take(count.try_into().map_err(not_enough_bytes)?)
                .read_to_end(bytes)?;

            if byte_count == count {
                Ok(list)
            } else {
                Err(not_enough_bytes(()))
            }
        } else {
            for _ in 0..count {
                list.push(B::read_options(reader, &options, args.clone())?);
            }
            Ok(list)
        }
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
            val.after_parse(reader, ro, args.clone())?;
        }

        Ok(())
    }
}

impl<C: Clone + 'static, B: BinRead<Args = C>, const N: usize> BinRead for [B; N] {
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
