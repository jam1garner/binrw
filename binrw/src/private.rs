use crate::{
    error::CustomError,
    io::{self, Seek, Write},
    BinRead, BinResult, Error, ReadOptions, WriteOptions,
};
use alloc::{boxed::Box, string::String};

pub enum AssertErrorFn<M, E> {
    Message(M),
    Error(E),
}

pub fn assert<MsgFn, Msg, ErrorFn, Err>(
    test: bool,
    pos: u64,
    error_fn: AssertErrorFn<MsgFn, ErrorFn>,
) -> BinResult<()>
where
    MsgFn: Fn() -> Msg,
    Msg: Into<String> + Sized,
    ErrorFn: Fn() -> Err,
    Err: CustomError + 'static,
{
    if test {
        Ok(())
    } else {
        Err(match error_fn {
            AssertErrorFn::Message(error_fn) => Error::AssertFail {
                pos,
                message: error_fn().into(),
            },
            AssertErrorFn::Error(error_fn) => Error::Custom {
                pos,
                err: Box::new(error_fn()),
            },
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
pub fn coerce_fn<R, T, F>(f: F) -> F
where
    F: Fn(T) -> R,
{
    f
}

pub fn magic<R, B>(reader: &mut R, expected: B, options: &ReadOptions) -> BinResult<()>
where
    B: BinRead<Args = ()> + core::fmt::Debug + PartialEq + Sync + Send + Clone + Copy + 'static,
    R: io::Read + io::Seek,
{
    let pos = reader.stream_position()?;
    let val = B::read_options(reader, options, ())?;
    if val == expected {
        Ok(())
    } else {
        Err(Error::BadMagic {
            pos,
            found: Box::new(val) as _,
        })
    }
}

pub fn parse_function_args_type_hint<R, Res, Args, F>(_: F, a: Args) -> Args
where
    R: crate::io::Read + Seek,
    F: FnOnce(&mut R, &crate::ReadOptions, Args) -> crate::BinResult<Res>,
{
    a
}

pub fn write_function_args_type_hint<T, W, Args, F>(_: F, a: Args) -> Args
where
    W: Write + Seek,
    F: FnOnce(&T, &mut W, &crate::WriteOptions, Args) -> crate::BinResult<()>,
{
    a
}

pub fn map_args_type_hint<Input, Output, MapFn, Args>(_: &MapFn, args: Args) -> Args
where
    MapFn: FnOnce(Input) -> Output,
    Input: BinRead<Args = Args>,
{
    args
}

pub fn write_fn_type_hint<T, WriterFn, Writer, Args>(x: WriterFn) -> WriterFn
where
    Args: Clone,
    Writer: Write + Seek,
    WriterFn: Fn(&T, &mut Writer, &WriteOptions, Args) -> BinResult<()>,
{
    x
}

pub fn write_map_args_type_hint<Input, Output, MapFn, Args>(_: &MapFn, args: Args) -> Args
where
    MapFn: FnOnce(Input) -> Output,
    Output: crate::BinWrite<Args = Args>,
{
    args
}

pub fn write_try_map_args_type_hint<Input, Output, Error, MapFn, Args>(
    _: &MapFn,
    args: Args,
) -> Args
where
    Error: CustomError,
    MapFn: FnOnce(Input) -> Result<Output, Error>,
    Output: crate::BinWrite<Args = Args>,
{
    args
}

pub fn write_map_fn_input_type_hint<Input, Output, MapFn>(func: MapFn) -> MapFn
where
    MapFn: FnOnce(Input) -> Output,
{
    func
}

pub fn write_fn_map_output_type_hint<Input, Output, MapFn, Writer, WriteFn, Args>(
    _: &MapFn,
    func: WriteFn,
) -> WriteFn
where
    MapFn: FnOnce(Input) -> Output,
    Args: Clone,
    Writer: Write + Seek,
    WriteFn: Fn(&Output, &mut Writer, &WriteOptions, Args) -> BinResult<()>,
{
    func
}

pub fn write_fn_try_map_output_type_hint<Input, Output, Error, MapFn, Writer, WriteFn, Args>(
    _: &MapFn,
    func: WriteFn,
) -> WriteFn
where
    Error: CustomError,
    MapFn: FnOnce(Input) -> Result<Output, Error>,
    Args: Clone,
    Writer: Write + Seek,
    WriteFn: Fn(&Output, &mut Writer, &WriteOptions, Args) -> BinResult<()>,
{
    func
}

pub fn write_zeroes<W: Write>(writer: &mut W, count: u64) -> BinResult<()> {
    const BUF_SIZE: u16 = 0x20;
    const ZEROES: [u8; BUF_SIZE as usize] = [0u8; BUF_SIZE as usize];

    if count <= BUF_SIZE.into() {
        // Lint: `count` is guaranteed to be <= BUF_SIZE
        #[allow(clippy::cast_possible_truncation)]
        writer.write_all(&ZEROES[..count as usize])?;
    } else {
        let full_chunks = count / u64::from(BUF_SIZE);
        let remaining = count % u64::from(BUF_SIZE);

        for _ in 0..full_chunks {
            writer.write_all(&ZEROES)?;
        }

        // Lint: `remaining` is guaranteed to be < BUF_SIZE
        #[allow(clippy::cast_possible_truncation)]
        writer.write_all(&ZEROES[..remaining as usize])?;
    }

    Ok(())
}
