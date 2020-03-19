use super::*;

#[non_exhaustive]
pub enum Error {
    BadMagic {
        pos: usize,
        found: Box<dyn Any>
    },
    AssertFail {
        pos: usize,
        message: String
    },
    Io(io::Error),
    Custom {
        pos: usize,
        err: Box<dyn core::fmt::Debug>
    },
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

pub fn magic<R, B>(reader: &mut R, b: B, options: &ReadOptions) -> BinResult<()>
    where B: BinRead<Args=()> + PartialEq + 'static,
          R: io::Read + io::Seek
{
    let pos = reader.seek(SeekFrom::Current(0))?;
    #[cfg(feature = "debug_template")]
    let mut options = options.clone();
    #[cfg(feature = "debug_template")] {
        options.variable_name = Some("magic");
    }
    let val = B::read_options(reader, &options, ())?;
    if val == b {
        Ok(())
    } else {
        Err(Error::BadMagic {
            pos: pos as usize,
            found: Box::new(val) as _
        })
    }
}

pub fn assert_eq<R, B, E, A>(reader: &mut R, b: B, error: Option<E>) -> BinResult<()>
    where B: BinRead<Args=()> + std::fmt::Debug + PartialEq,
          R: io::Read + io::Seek,
          A: core::fmt::Debug + 'static,
          E: Fn() -> A,
{
    let pos = reader.seek(SeekFrom::Current(0))? as usize;
    let val = B::read(reader)?;
    if val == b {
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
                message: "Assertion failed".into()
            })
        })
    }
}

pub fn assert<R, E, A>(reader: &mut R, test: bool, message: &str, error: Option<E>) -> BinResult<()>
    where R: io::Read + io::Seek,
          A: core::fmt::Debug + 'static,
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

pub fn nop3<T1, R: Read + Seek>(_: &mut R, _: &ReadOptions, _: T1) -> BinResult<()> {
    Ok(())
}

pub fn nop3_default<T1, R: Read + Seek, D: Default>(_: &mut R, _: &ReadOptions, _: T1) -> BinResult<D> {
    Ok(D::default())
}

pub fn nop5<T1, T2, R: Read + Seek>(_: &mut T1, _: &mut R, _: &ReadOptions, _: T2, _: &AfterParseOptions) -> BinResult<()> {
    Ok(())
}

pub fn identity_after_parse<PostprocessFn, Reader, ValueType, ArgType>(
    after_parse_fn: PostprocessFn,
    mut item: ValueType,
    reader: &mut Reader,
    ro: &ReadOptions,
    args: ArgType,
    ao: &AfterParseOptions
) -> BinResult<ValueType>
    where Reader: Read + Seek,
          PostprocessFn: Fn(&mut ValueType, &mut Reader, &ReadOptions, ArgType, &AfterParseOptions) -> BinResult<()>, 
{
    after_parse_fn(&mut item, reader, ro, args, ao)?;
    Ok(item)
}
