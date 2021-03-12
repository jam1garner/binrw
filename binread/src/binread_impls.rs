use super::*;

/// Internal macro for quickly implementing binread for types supporting from_bytes api
macro_rules! binread_impl {
    ($($type_name:ty),*$(,)?) => {
        $(
            impl BinRead for $type_name {
                type Args = ();

                fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, _: Self::Args) -> BinResult<Self> {
                    let mut val = [0; core::mem::size_of::<$type_name>()];
                    let pos = reader.seek(SeekFrom::Current(0))?;

                    #[cfg(feature = "debug_template")]
                    {
                        if !options.dont_output_to_template {
                            if let Some(name) = options.variable_name {
                                binary_template::write_named(
                                    options.endian,
                                    pos,
                                    stringify!($type_name),
                                    name
                                );
                            } else {
                                binary_template::write(
                                    options.endian,
                                    pos,
                                    stringify!($type_name)
                                );
                            }
                        }
                    }

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

const DEFAULT_ARGS: () = ();

impl BinRead for char {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, _: Self::Args) -> BinResult<Self> {
        // TODO: somehow do proper unicode handling?
        Ok(<u8>::read_options(reader, options, DEFAULT_ARGS)? as char)
    }
}

binread_impl!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl<C: Copy + 'static, B: BinRead<Args = C>> BinRead for Vec<B> {
    type Args = B::Args;

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self> {
        let mut options = *options;
        let count = match options.count.take() {
            Some(x) => x,
            None => panic!("Missing count for Vec"),
        };

        #[cfg(feature = "debug_template")]
        {
            let pos = reader.seek(SeekFrom::Current(0))?;
            let type_name = core::any::type_name::<B>().rsplitn(1, "::").next().unwrap();

            // this is a massive hack. I'm so sorry
            let type_name = if type_name.starts_with("binread::file_ptr::FilePtr<") {
                // Extract the backing type name from file pointers
                type_name.trim_start_matches("binread::file_ptr::FilePtr<")
                        .split(',').next().unwrap()
            } else {
                type_name
            };

            binary_template::write_vec(options.endian, pos, type_name, count);

            options.dont_output_to_template = true;
        }


        (0..count)
            .map(|_| {
                B::read_options(reader, &options, args)
            })
            .collect()
    }

    fn after_parse<R>(&mut self, reader: &mut R, ro: &ReadOptions, args: Self::Args)-> BinResult<()>
        where R: Read + Seek,
    {
        for val in self.iter_mut() {
            val.after_parse(reader, ro, args)?;
        }

        Ok(())
    }
}

macro_rules! binread_array_impl {
    ($($size:literal),*$(,)?) => {
        $(
            impl<C: Copy + 'static, B: BinRead<Args = C> + Default> BinRead for [B; $size] {
                type Args = B::Args;

                fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self> {
                    #[cfg(feature = "debug_template")]
                    {
                        let pos = reader.seek(SeekFrom::Current(0))?;
                        let type_name = core::any::type_name::<B>().rsplitn(1, "::").nth(0).unwrap();

                        if let Some(name) = options.variable_name {
                            binary_template::write_vec_named(
                                options.endian, pos, type_name, $size, name
                            );
                        } else {
                            binary_template::write_vec(options.endian, pos, type_name, $size);
                        }
                    }

                    #[cfg(feature = "debug_template")]
                    let options = &ReadOptions {
                        dont_output_to_template: true,
                        ..*options
                    };

                    let mut arr: [B; $size] = Default::default();
                    for elem in arr.iter_mut() {
                        *elem = BinRead::read_options(reader, options, args)?;
                    }
                    Ok(arr)
                }

                fn after_parse<R>(&mut self, reader: &mut R, ro: &ReadOptions, args: B::Args)-> BinResult<()>
                    where R: Read + Seek,
                {
                    for val in self.iter_mut() {
                        val.after_parse(reader, ro, args)?;
                    }

                    Ok(())
                }
            }
        )*
    }
}

binread_array_impl!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32);

/// Internal macro to recursively implement BinRead for every size tuple given
/// in the invocation
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

binread_tuple_impl!(b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19, b20, b21, b22, b23, b24, b25, b26, b27, b28, b29, b30, b31, b32);

impl BinRead for () {
    type Args = ();

    fn read_options<R: Read + Seek>(_: &mut R, _: &ReadOptions, _: Self::Args) -> BinResult<Self> {
        Ok(())
    }
}

impl<T: BinRead> BinRead for Box<T> {
    type Args = T::Args;

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self> {
        Ok(Box::new(T::read_options(reader, options, args)?))
    }
}

impl<T: BinRead> BinRead for Option<T> {
    type Args = T::Args;

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self> {
        Ok(Some(T::read_options(reader, options, args)?))
    }

    fn after_parse<R>(&mut self, reader: &mut R, ro: &ReadOptions, args: Self::Args)-> BinResult<()>
        where R: Read + Seek,
    {
        match self {
            Some(val) => val.after_parse(reader, ro, args),
            None => Ok(())
        }
    }
}

impl<T> BinRead for core::marker::PhantomData<T> {
    type Args = ();

    fn read_options<R: Read + Seek>(_: &mut R, _: &ReadOptions, _: Self::Args) -> BinResult<Self> {
        Ok(core::marker::PhantomData)
    }
}
