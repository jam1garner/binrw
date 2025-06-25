use crate::{
    error::{Backtrace, BacktraceFrame, CustomError},
    io::{self, Read, Seek, SeekFrom, Write},
    BinRead, BinResult, BinWrite, Endian, Error,
};
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String};

pub use crate::named_args::{
    builder_helper, passthrough_helper, Needed, Optional, Satisfied, SatisfiedOrOptional,
};

// This is some nonsense to improve the diagnostic output for types that require
// arguments so that the emitted output is clearer about this fact. Because this
// is implemented for any `Default`, and used as the constraint for shorthand
// functions, it should result in any invalid shorthand call to emit
// `Self::Args: Required` which is hopefully a clear enough hint.
pub trait Required: MissingArgsDirective {
    fn args() -> Self;
}

impl<T: Default> Required for T {
    fn args() -> Self {
        <Self as Default>::default()
    }
}

// This extra trait exists only to give a stronger hint in compiler errors about
// what to do. Without it, the compiler will point to the `Default` bound, which
// is misleading about what the programmer must do to fulfil the requirement of
// the type they are trying to use.
pub trait MissingArgsDirective {}
impl<T: Default> MissingArgsDirective for T {}

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
// compiler will emit the diagnostic on the `#[derive]`d attribute instead of
// the return statement of the map function. The simpler approach of assigning
// the map function to a variable with a function pointer type does not work for
// capturing closures since they are not compatible with that type.
pub fn coerce_fn<R, T, F>(f: F) -> F
where
    F: FnMut(T) -> R,
{
    f
}

pub fn magic<R, B>(reader: &mut R, expected: B, endian: Endian) -> BinResult<()>
where
    B: for<'a> BinRead<Args<'a> = ()>
        + core::fmt::Debug
        + PartialEq
        + Sync
        + Send
        + Clone
        + Copy
        + 'static,
    R: Read + Seek,
{
    let pos = reader.stream_position()?;
    let val = B::read_options(reader, endian, ())?;
    if val == expected {
        Ok(())
    } else {
        Err(Error::BadMagic {
            pos,
            found: Box::new(val) as _,
        })
    }
}

#[must_use]
pub fn not_enough_bytes() -> Error {
    Error::Io(io::Error::new(
        io::ErrorKind::UnexpectedEof,
        "not enough bytes in reader",
    ))
}

pub fn parse_fn_type_hint<Ret, ParseFn, R, Args>(f: ParseFn) -> ParseFn
where
    R: Read + Seek,
    ParseFn: FnOnce(&mut R, Endian, Args) -> BinResult<Ret>,
{
    f
}

pub fn parse_function_args_type_hint<R, Res, Args, F>(_: &F, a: Args) -> Args
where
    R: Read + Seek,
    F: FnOnce(&mut R, Endian, Args) -> BinResult<Res>,
{
    a
}

pub fn write_function_args_type_hint<T, W, Args, F>(_: &F, a: Args) -> Args
where
    W: Write + Seek,
    F: FnOnce(&T, &mut W, Endian, Args) -> BinResult<()>,
{
    a
}

pub fn map_args_type_hint<'a, Input, Output, MapFn, Args>(_: &MapFn, args: Args) -> Args
where
    MapFn: FnOnce(Input) -> Output,
    Input: BinRead<Args<'a> = Args>,
{
    args
}

pub fn map_reader_type_hint<'a, Reader, MapFn, Output>(x: MapFn) -> MapFn
where
    Reader: Read + Seek + 'a,
    MapFn: Fn(&'a mut Reader) -> Output,
    Output: Read + Seek + 'a,
{
    x
}

pub fn map_writer_type_hint<'a, Writer, MapFn, Output>(x: MapFn) -> MapFn
where
    Writer: Write + Seek + 'a,
    MapFn: Fn(&'a mut Writer) -> Output,
    Output: Write + Seek + 'a,
{
    x
}

pub fn write_fn_type_hint<T, WriterFn, Writer, Args>(x: WriterFn) -> WriterFn
where
    Writer: Write + Seek,
    WriterFn: FnOnce(&T, &mut Writer, Endian, Args) -> BinResult<()>,
{
    x
}

pub fn write_map_args_type_hint<'a, Input, Output, MapFn, Args>(_: &MapFn, args: Args) -> Args
where
    MapFn: FnOnce(Input) -> Output,
    Output: BinWrite<Args<'a> = Args>,
{
    args
}

pub fn restore_position<E: Into<Error>, S: Seek>(
    stream: &mut S,
    pos: u64,
) -> impl FnOnce(E) -> Error + '_ {
    move |error| match stream.seek(SeekFrom::Start(pos)) {
        Ok(_) => error.into(),
        Err(seek_error) => restore_position_err(error.into(), seek_error.into()),
    }
}

fn restore_position_err(error: Error, mut seek_error: Error) -> Error {
    let reason = BacktraceFrame::Message("rewinding after a failure".into());
    match error {
        Error::Backtrace(mut bt) => {
            core::mem::swap(&mut seek_error, &mut *bt.error);
            bt.frames.insert(0, seek_error.into());
            bt.frames.insert(0, reason);
            Error::Backtrace(bt)
        }
        error => Error::Backtrace(Backtrace::new(
            seek_error,
            alloc::vec![reason, error.into()],
        )),
    }
}

pub fn restore_position_variant<S: Seek>(
    stream: &mut S,
    pos: u64,
    error: Error,
) -> BinResult<Error> {
    match stream.seek(SeekFrom::Start(pos)) {
        Ok(_) => Ok(error),
        Err(seek_error) => Err(restore_position_err(error, seek_error.into())),
    }
}

pub fn write_try_map_args_type_hint<'a, Input, Output, Error, MapFn, Args>(
    _: &MapFn,
    args: Args,
) -> Args
where
    Error: CustomError,
    MapFn: FnOnce(Input) -> Result<Output, Error>,
    Output: BinWrite<Args<'a> = Args>,
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
    WriteFn: Fn(&Output, &mut Writer, Endian, Args) -> BinResult<()>,
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
    WriteFn: Fn(&Output, &mut Writer, Endian, Args) -> BinResult<()>,
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

#[cfg(feature = "std")]
pub use std::eprintln;

#[cfg(not(feature = "std"))]
#[doc(hidden)]
#[macro_export]
macro_rules! eprintln {
    ($($tt:tt)*) => {
        compile_error!("dbg requires feature `std`")
    };
}

#[cfg(not(feature = "std"))]
pub use crate::eprintln;
