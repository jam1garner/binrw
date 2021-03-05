use crate::{BinRead, BinResult, Error, ReadOptions, io};

pub enum AssertErrorFn<M, E> {
    Message(M),
    Error(E),
}

pub fn assert<MsgFn, Msg, ErrorFn, Err>(test: bool, pos: u64, error_fn: AssertErrorFn<MsgFn, ErrorFn>) -> BinResult<()>
where
    MsgFn: Fn() -> Msg,
    Msg: Into<String> + Sized,
    ErrorFn: Fn() -> Err,
    Err: core::any::Any + Sync + Send + 'static,
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
