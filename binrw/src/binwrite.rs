use crate::{
    io::{Write, Seek},
    BinResult,
    WriteOptions
};

use core::any::Any;

/// The `BinWrite` trait takes the implementing type and writes it to a stream.
///
/// This trait is usually derived, but can also be manually implemented by
/// writing an appropriate [`Args`] type and [`write_options`] function.
///
/// [`Args`]: Self::Args
/// [`write_options`]: BinWrite::write_options
pub trait BinWrite {
    /// The type of arguments needed to be supplied in order to write this type, usually a tuple.
    ///
    /// **NOTE:** For types that don't require any arguments, use the unit (`()`) type. This will
    /// allow [`write`](BinWrite::write) to be used.
    type Args: Any + Copy;

    /// Get the number of bytes that will be written by [`write_options`]
    ///
    /// [`write_options`]: BinWrite::write_options
    fn write_size(
        &self,
        options: &mut WriteOptions,
        args: Self::Args,
        pos: u64,
    ) -> BinResult<u64>;

    /// Get the number of bytes that will be written by [`write_additional`]
    ///
    /// [`write_additional`]: BinWrite::write_additional
    fn additional_write_size(
        &self,
        options: &mut WriteOptions,
        args: Self::Args,
        pos: u64,
    ) -> BinResult<u64>;

    /// Write a type to a writer while assuming no arguments are needed.
    fn write<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()>
        where Self::Args: Default
    {
        self.write_options(writer, &mut WriteOptions::default(), Self::Args::default())
    }

    /// Write the type to a writer while providing the default [`WriteOptions`]
    fn write_args<W: Write + Seek>(&self, writer: &mut W, args: Self::Args) -> BinResult<()> {
        self.write_options(writer, &mut WriteOptions::default(), args)
    }

    /// Write the type to a writer, given the options on how to write it and the type-specific
    /// arguments
    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &mut WriteOptions,
        args: Self::Args,
    ) -> BinResult<()>;

    /// Write any additional data that need to be pointed to.
    ///
    /// An example being
    fn write_additional<W: Write + Seek>(
        &self,
        writer: &mut W,
        options: &mut WriteOptions,
        args: Self::Args,
    ) -> BinResult<()>;
}
