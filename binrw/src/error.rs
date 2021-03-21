//! Error types and internal error handling functions
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};
use core::{any::Any, fmt};
use crate::{BinRead, BinResult, ReadOptions, io};

pub trait CustomError: Any + fmt::Display + fmt::Debug + Send + Sync + 'static {
    fn as_any(&self) -> &(dyn Any + Send + Sync);
    fn as_box_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
}
impl <T: Any + fmt::Display + fmt::Debug + Send + Sync + 'static> CustomError for T {
    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
    fn as_box_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }
}
impl dyn CustomError {
    pub fn downcast<T: Any>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if self.is::<T>() {
            unsafe {
                let raw: *mut dyn Any = Box::into_raw(self.as_box_any());
                Ok(Box::from_raw(raw as *mut T))
            }
        } else {
            Err(self)
        }
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }

    pub fn is<T: Any>(&self) -> bool {
        core::any::TypeId::of::<T>() == self.type_id()
    }
}

/// An error while parsing a BinRead type
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// The magic value did not match the provided one
    BadMagic {
        // Position in number of bytes from the start of the reader
        pos: u64,
        // The value found. Use [`Any::downcast_ref`](core::any::Any::downcast_ref) to access
        found: Box<dyn fmt::Debug + Send + Sync>,
    },
    /// The condition of an assertion without a custom type failed
    AssertFail {
        pos: u64,
        message: String
    },
    /// An error that occured while reading from, or seeking within, the reader
    Io(io::Error),
    /// A custom error, most often given from the second value passed into an [`assert`](crate::attribute#assert)
    Custom {
        pos: u64,
        err: Box<dyn CustomError>,
    },
    /// No variant in the enum was successful in parsing the data
    NoVariantMatch {
        pos: u64
    },
    EnumErrors {
        pos: u64,
        variant_errors: Vec<(/*variant name*/ &'static str, Error)>,
    }
}

impl Error {
    /// Gets a custom error of type T from the Error. Returns `None` if the error type is not
    /// custom or if the contained error is not of the desired type.
    pub fn custom_err<T: Any>(&self) -> Option<&T> {
        if let Error::Custom { err, .. } = self {
            err.downcast_ref()
        } else {
            None
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadMagic { pos, found } => write!(f, "bad magic at 0x{:x}: {:?}", pos, found),
            Error::AssertFail { pos, message } => write!(f, "{} at 0x{:x}", message, pos),
            Error::Io(err) => fmt::Display::fmt(err, f),
            Error::Custom { pos, err } => write!(f, "{} at 0x{:x}", err, pos),
            Error::NoVariantMatch { pos } => write!(f, "no variants matched at 0x{:x}", pos),
            Error::EnumErrors { pos, variant_errors } => {
                write!(f, "no variants matched at 0x{:x}:", pos)?;
                for (name, err) in variant_errors {
                    write!(f, "\n  {}: {}", name, err)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Read a value then check if it is the expected value
pub fn magic<R, B>(reader: &mut R, expected: B, options: &ReadOptions) -> BinResult<()>
where
    B: BinRead<Args = ()> + fmt::Debug + PartialEq + Sync + Send + 'static,
    R: io::Read + io::Seek,
{
    let pos = reader.seek(io::SeekFrom::Current(0))?;
    let val = B::read_options(reader, &options, ())?;
    if val == expected {
        Ok(())
    } else {
        Err(Error::BadMagic {
            pos,
            found: Box::new(val) as _
        })
    }
}

pub fn read_options_then_after_parse<Args, T, R>(
    reader: &mut R,
    ro: &ReadOptions,
    args: T::Args,
) -> BinResult<T>
    where Args: Copy + 'static,
          T: BinRead<Args = Args>,
          R: io::Read + io::Seek,
{
    let mut val = T::read_options(reader, ro, args)?;
    val.after_parse(reader, ro, args)?;
    Ok(val)
}
