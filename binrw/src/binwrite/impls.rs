use crate::io::{Seek, Write};
use crate::{BinResult, BinWrite, Endian, WriteOptions};

// ============================= nums =============================

macro_rules! binwrite_num_impl {
    ($($type_name:ty),*$(,)?) => {
        $(
            impl BinWrite for $type_name {
                type Args = ();

                fn write_options<W: Write + Seek>(
                    &self,
                    writer: &mut W,
                    options: &WriteOptions,
                    _: Self::Args,
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

// =========================== arrays ===========================

impl<T: BinWrite, const N: usize> BinWrite for [T; N] {
    type Args = T::Args;

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: Self::Args,
    ) -> BinResult<()> {
        for item in self {
            T::write_options(item, writer, options, args.clone())?;
        }

        Ok(())
    }
}

// ========================= end arrays =========================

// =========================== tuples ===========================

// TODO

// ========================= end tuples =========================
