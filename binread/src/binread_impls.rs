use super::*;

/// Internal macro for quickly implementing binread for types supporting from_bytes api
macro_rules! binread_impl {
    ($($type_name:ty),*$(,)?) => {
        $(
            impl BinRead for $type_name {
                type Args = ();
                
                fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, _: Self::Args) -> BinResult<Self> {
                    let mut val = [0; core::mem::size_of::<$type_name>()];
                    
                    #[cfg(feature = "debug_template")]
                    {
                        if !options.dont_output_to_template {
                            let pos = reader.seek(SeekFrom::Current(0))?;
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
                    
                    reader.read_exact(&mut val)?;
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

impl<B: BinRead> BinRead for Vec<B> {
    type Args = B::Args;

    fn read_options<R: Read + Seek>(reader: &mut R, options: &ReadOptions, args: Self::Args) -> BinResult<Self> {
        let mut options = options.clone();
        let count = match options.count.take() {
            Some(x) => x,
            None => panic!("Missing count for Vec"),
        };

        #[cfg(feature = "debug_template")]
        {
            let pos = reader.seek(SeekFrom::Current(0))?;
            let type_name = core::any::type_name::<B>().rsplitn(1, "::").nth(0).unwrap();

            // this is a massive hack. I'm so sorry
            let type_name = if type_name.starts_with("binread::file_ptr::FilePtr<") {
                // Extract the backing type name from file pointers
                type_name.trim_start_matches("binread::file_ptr::FilePtr<")
                        .split(",").nth(0).unwrap()
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

    fn after_parse<R>(&mut self, reader: &mut R, ro: &ReadOptions, args: Self::Args, ao: &AfterParseOptions)-> BinResult<()>
        where R: Read + Seek,
    {
        for val in self.iter_mut() {
            val.after_parse(reader, ro, args, ao)?;
        }

        Ok(())
    }
}

macro_rules! binread_array_impl {
    ($($size:literal),*$(,)?) => {
        $(
            impl<B: BinRead + Default + Copy> BinRead for [B; $size] {
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

                    let mut arr = [B::default(); $size];
                    for i in 0..$size {
                        arr[i] = BinRead::read_options(reader, options, args)?;
                    }
                    Ok(arr)
                }

                fn after_parse<R>(&mut self, reader: &mut R, ro: &ReadOptions, args: B::Args, ao: &AfterParseOptions)-> BinResult<()>
                    where R: Read + Seek,
                {
                    for val in self.iter_mut() {
                        val.after_parse(reader, ro, args, ao)?;
                    }

                    Ok(())
                }
            }
        )*
    }
}

binread_array_impl!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20);

/// Internal macro to recursively implement BinRead for every size tuple 0 to 20
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

            // TODO: Add after_parse impl using paste::item
        }

        binread_tuple_impl!($($types),*);
    };

    () => {
        impl BinRead for () {
            type Args = ();

            fn read_options<R: Read + Seek>(_: &mut R, _: &ReadOptions, _: Self::Args) -> BinResult<Self> {
                Ok(())
            }
        }
    };
}

binread_tuple_impl!(b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16, b17, b18, b19, b20);
