#[doc(hidden)]
pub trait ArgType<'any> {
    type Item: Clone;
}

#[cfg(doc)]
use crate::BinRead;

/// A macro for defining the type of [`BinRead::Args`] when manually implementing the [`BinRead`]
/// triat.
///
/// ## Example
///
/// ```
/// # use binrw::io::{Read, Seek};
/// use binrw::{BinRead, arg_type, BinResult, ReadOptions};
///
/// struct Test;
///
/// impl BinRead for Test {
///     type Args = arg_type!(u32);
///
///     fn read_options<R: Read + Seek>(
///        reader: &mut R,
///        options: &ReadOptions,
///        args: u32,
///     ) -> BinResult<Self> {
///         Ok(Test)
///     }
/// }
/// ```
#[doc(inline)]
pub use binrw_derive::arg_type;

/// A macro for specifying `T::Args` when manually implementing the [`BinRead`] trait.
///
/// Common usage patterns:
/// * `args_of!(Self)` - get the argument type of the current type
/// * `args_of!(u32 as BinRead)` - get the argument type of a specific type, manually specifying
/// that the arguments should come from the [`BinRead`] trait.
/// * `args_of!(B)` - specify the arguments come from a generic bound (`B: BinRead`)
#[macro_export]
macro_rules! args_of {
    ($ty:ty $(as $trt:path)?) => {
        <<$ty $(as $trt)?>::Args as $crate::ArgType<'_>>::Item
    };
}
