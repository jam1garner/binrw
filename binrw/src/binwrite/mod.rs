use crate::{
    io::{Seek, Write},
    BinResult, Endian,
};

mod impls;

/// A trait for writing a given type to a writer
pub trait BinWrite {
    /// The type of arguments needed to be supplied in order to write this type, usually a tuple.
    ///
    /// **Note:** For types that don't require any arguments, use the unit (`()`) type.
    /// This will allow [`write_to`](BinWrite::write_to) to be used.
    type Args: Clone;

    /// Write a type to a writer while assuming no arguments are needed.
    fn write_to<W: Write + Seek>(&self, writer: &mut W) -> BinResult<()>
    where
        Self::Args: Default,
    {
        self.write_options(writer, &WriteOptions::default(), Self::Args::default())
    }

    /// Write the type to a writer while providing the default [`WriteOptions`]
    fn write_with_args<W: Write + Seek>(&self, writer: &mut W, args: Self::Args) -> BinResult<()> {
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

/// Options for how data should be written
///
/// Functionally the purpose of WriteOptions is simple: maintaining context which is implicitly
/// passed throughout all types being written.
#[derive(Default, Clone)]
pub struct WriteOptions {
    endian: Endian,
}

impl WriteOptions {
    /// Create a new `WriteOptions`. Additional fields can be instantiated using `.with_{field}`.
    pub fn new(endian: Endian) -> Self {
        Self { endian }
    }

    /// Retrieves the specified endian
    pub fn endian(&self) -> Endian {
        self.endian
    }

    /// Returns the same `WriteOptions` but with the endian set
    pub fn with_endian(self, endian: Endian) -> Self {
        WriteOptions { endian }
    }
}
