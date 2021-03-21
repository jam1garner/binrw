#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String};
use crate::{BinRead, BinResult, Error, ReadOptions, error::CustomError, io};

pub enum AssertErrorFn<M, E> {
    Message(M),
    Error(E),
}

pub fn assert<MsgFn, Msg, ErrorFn, Err>(test: bool, pos: u64, error_fn: AssertErrorFn<MsgFn, ErrorFn>) -> BinResult<()>
where
    MsgFn: Fn() -> Msg,
    Msg: Into<String> + Sized,
    ErrorFn: Fn() -> Err,
    Err: CustomError,
{
    if test {
        Ok(())
    } else {
        Err(match error_fn {
            AssertErrorFn::Message(error_fn) => Error::AssertFail { pos, message: error_fn().into() },
            AssertErrorFn::Error(error_fn) => Error::Custom { pos, err: Box::new(error_fn()) },
        })
    }
}

// This validates the map function return value by trying to coerce it into
// a function with the expected return type. If this is not done, the
// compiler will emit the diagnostic on the `#[derive(BinRead)]` attribute
// instead of the return statement of the map function. The simpler approach
// of assigning the map function to a variable with a function pointer type
// does not work for capturing closures since they are not compatible with
// that type.
pub fn coerce_fn<R, T, F>(f: F) -> F where F: Fn(T) -> R { f }

pub fn try_after_parse<Reader, ValueType, ArgType>(
    item: &mut Option<ValueType>,
    reader: &mut Reader,
    ro: &ReadOptions,
    args: ArgType,
) -> BinResult<()>
    where Reader: io::Read + io::Seek,
          ValueType: BinRead<Args = ArgType>,
          ArgType: Copy + 'static,
{
    if let Some(value) = item.as_mut() {
        value.after_parse(reader, ro, args)?;
    }

    Ok(())
}
