use crate::{
    io::{Write, Seek},
    BinResult,
    WriteOptions
};

use core::any::Any;

pub trait BinWrite {
    /// The type of arguments needed to be supplied in order to write this type, usually a tuple.
    ///
    /// **NOTE:** For types that don't require any arguments, use the unit (`()`) type. This will allow [`read`](BinRead::read) to be used.
    type Args: Any + Copy;


    /// Write a type to a writer while assuming no arguments are needed.
    ///
    /// # Panics
    /// Panics if there is no [`args_default`](BinWrite::args_default) implementation
    fn write<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()>
        where Self::Args: Default
    {
        self.write_options(writer, &WriteOptions::default(), Self::Args::default())
    }

    /// Write the type to a writer while providing the default options
    fn write_args<W: Write + Seek>(&self, writer: &mut W, args: Self::Args) -> BinResult<()> {
        self.write_options(writer, &WriteOptions::default(), args)
    }

    /// Write the type to a writer, given the options on how to write it and the type-specific
    /// arguments
    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &WriteOptions,
        args: Self::Args,
    ) -> BinResult<()>;
}
