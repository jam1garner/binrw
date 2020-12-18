//! Error types and internal error handling functions
use super::*;

/// An error while parsing a BinRead type
#[non_exhaustive]
pub enum Error {
    /// The magic value did not match the provided one
    BadMagic {
        // Position in number of bytes from the start of the reader
        pos: usize,
        // The value found. Use [`Any::downcast_ref`](core::any::Any::downcast_ref) to access
        found: Box<dyn Any + Sync + Send>,
    },
    /// The condition of an assertion without a custom type failed
    AssertFail {
        pos: usize,
        message: String
    },
    /// An error that occured while reading from, or seeking within, the reader
    Io(io::Error),
    /// A custom error, most often given from the second value passed into an [`assert`](attribute#Assert)
    Custom {
        pos: usize,
        err: Box<dyn Any + Sync + Send>,
    },
    /// No variant in the enum was successful in parsing the data
    NoVariantMatch {
        pos: usize
    },
    EnumErrors {
        pos: usize,
        variant_errors: Vec<(/*variant name*/ &'static str, Error)>,
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

use core::fmt;

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMagic { pos, .. } => write!(f, "BadMagic {{ pos: 0x{:X} }}", pos),
            Self::AssertFail { pos, message } => write!(f, "AssertFail at 0x{:X}: \"{}\"", pos, message),
            Self::Io(err) => write!(f, "Io({:?})", err),
            Self::Custom { pos, err } => write!(f, "Custom {{ pos: 0x{:X}, err: {:?} }}", pos, err),
            _ => write!(f, "EnumErrors")
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BadMagic { pos, .. } => write!(f, "BadMagic {{ pos: 0x{:X} }}", pos),
            Self::AssertFail { pos, message } => write!(f, "AssertFail at 0x{:X}: \"{}\"", pos, message),
            Self::Io(err) => write!(f, "Io({:?})", err),
            Self::Custom { pos, err } => write!(f, "Custom {{ pos: 0x{:X}, err: {:?} }}", pos, err),
            _ => write!(f, "EnumErrors")
        }
    }
}

/// Read a value then check if it is the expected value
pub fn magic<R, B>(reader: &mut R, expected: B, options: &ReadOptions) -> BinResult<()>
where
    B: BinRead<Args = ()> + PartialEq + Sync + Send + 'static,
    R: io::Read + io::Seek,
{
    let pos = reader.seek(SeekFrom::Current(0))?;
    #[cfg(feature = "debug_template")]
    let options = {
        let mut options = *options;
        options.variable_name = Some("magic");
        options
    };
    let val = B::read_options(reader, &options, ())?;
    if val == expected {
        Ok(())
    } else {
        Err(Error::BadMagic {
            pos: pos as usize,
            found: Box::new(val) as _
        })
    }
}

// pub fn assert_eq<R, B, E, A>(reader: &mut R, expected: B, error: Option<E>) -> BinResult<()>
//     where B: BinRead<Args=()> + std::fmt::Debug + PartialEq,
//           R: io::Read + io::Seek,
//           A: core::fmt::Debug + 'static,
//           E: Fn() -> A,
// {
//     let pos = reader.seek(SeekFrom::Current(0))? as usize;
//     let val = B::read(reader)?;
//     if val == expected {
//         Ok(())
//     } else {
//         error.map(|err|{
//             Err(Error::Custom {
//                 pos,
//                 err: Box::new(err())
//             })
//         }).unwrap_or_else(||{
//             Err(Error::AssertFail {
//                 pos,
//                 message: "Assertion failed".into()
//             })
//         })
//     }
// }

/// Assert a condition is true and if not optionally apply a function to generate the error
pub fn assert<R, E, A>(reader: &mut R, test: bool, message: &str, error: Option<E>) -> BinResult<()>
where
    R: io::Read + io::Seek,
    A: core::fmt::Debug + Sync + Send + 'static,
    E: Fn() -> A,
{
    let pos = reader.seek(SeekFrom::Current(0))? as usize;
    if test {
        Ok(())
    } else {
        error.map(|err|{
            Err(Error::Custom {
                pos,
                err: Box::new(err())
            })
        }).unwrap_or_else(||{
            Err(Error::AssertFail {
                pos,
                message: message.into()
            })
        })
    }
}

#[doc(hidden)]
/// A no-op replacement for [`BinRead::read_options`](BinRead::read_options) that returns the unit type
///
/// **Intended for internal use only**
pub fn nop3<T1, R: Read + Seek>(_: &mut R, _: &ReadOptions, _: T1) -> BinResult<()> {
    Ok(())
}

#[doc(hidden)]
/// A no-op replacement for [`BinRead::read_options`](BinRead::read_options) that returns the
/// default value for the given type. Internally used for the `default` attribute.
///
/// **Intended for internal use only**
pub fn nop3_default<T1, R: Read + Seek, D: Default>(_: &mut R, _: &ReadOptions, _: T1) -> BinResult<D> {
    Ok(D::default())
}

#[doc(hidden)]
/// A no-op replacement for [`BinRead::after_parse`](BinRead::after_parse)
///
/// **Intended for internal use only**
pub fn nop5<T1, T2, R: Read + Seek>(_: &mut T1, _: &mut R, _: &ReadOptions, _: T2) -> BinResult<()> {
    Ok(())
}

#[doc(hidden)]
/// A replacement for [`BinRead::after_parse`](BinRead::after_parse) that runs after_parse only if
/// a value is present.
///
/// **Intended for internal use only**
pub fn try_after_parse<Reader, ValueType, ArgType>(
    item: &mut Option<ValueType>,
    reader: &mut Reader,
    ro: &ReadOptions,
    args: ArgType,
) -> BinResult<()>
    where Reader: Read + Seek,
          ValueType: BinRead<Args = ArgType>,
          ArgType: Copy + 'static,
{
    if let Some(value) = item.as_mut() {
        value.after_parse(reader, ro, args)?;
    }

    Ok(())
}

#[doc(hidden)]
/// Functional wrapper to apply a [`BinRead::after_parse`](BinRead::after_parse) stand-in function
/// to a value and then return the value if the [`after_parse`](BinRead::after_parse) function succeeds.
///
/// Used by the derive macro to optionally immediately dereference/postprocess the value when
/// first parsed. In theory should be optimized out in case of no-op.
///
/// **Intended for internal use only**
pub fn identity_after_parse<PostprocessFn, Reader, ValueType, ArgType>(
    after_parse_fn: PostprocessFn,
    mut item: ValueType,
    reader: &mut Reader,
    ro: &ReadOptions,
    args: ArgType,
) -> BinResult<ValueType>
    where Reader: Read + Seek,
          PostprocessFn: Fn(
              &mut ValueType,
              &mut Reader,
              &ReadOptions,
              ArgType,
          ) -> BinResult<()>,
{
    after_parse_fn(&mut item, reader, ro, args)?;
    Ok(item)
}

/// Conversion used internally by the `try` attribute to make parsing failures softer
pub fn try_conversion<T>(result: BinResult<T>) -> BinResult<Option<T>> {
    Ok(result.ok())
}

pub fn read_options_then_after_parse<Args, T, R>(
    reader: &mut R,
    ro: &ReadOptions,
    args: T::Args,
) -> BinResult<T>
    where Args: Copy + 'static,
          T: BinRead<Args = Args>,
          R: Read + Seek,
{
    let mut val = T::read_options(reader, ro, args)?;
    val.after_parse(reader, ro, args)?;
    Ok(val)
}

impl Error {
    /// Gets a custom error of type T from the Error. Returns `None` if the error type is not
    /// custom or if the contained error is not of the desired type.
    pub fn custom_err<T: Any>(&self) -> Option<&T> {
        if let Error::Custom { err, ..} = self {
            err.downcast_ref()
        } else {
            None
        }
    }
}
