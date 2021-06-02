use core::marker::PhantomData;

#[doc(hidden)]
pub struct Satisfied;

#[doc(hidden)]
pub struct Optional;

#[doc(hidden)]
pub struct Needed;

// TODO: seal?
/// Indicates that a requirement for a typed builder has been met, either by
/// the user providing one, or by a default being given.
#[doc(hidden)]
pub trait SatisfiedOrOptional {}

impl SatisfiedOrOptional for Satisfied {}
impl SatisfiedOrOptional for Optional {}

#[doc(hidden)]
pub fn passthrough_helper<T>(_a: PhantomData<T>) -> T {
    panic!("This is a type system hack and should never be called!");
}

#[doc(hidden)]
pub fn builder_helper<T: BinrwNamedArgs>(_: PhantomData<T>) -> T::Builder {
    <T as BinrwNamedArgs>::builder()
}

/// A macro for creating a binrw argument type
#[macro_export]
macro_rules! args {
    (@ifn { $value:expr } $name:ident) => { $value };
    (@ifn {} $name:ident) => { $name };
    ($($name:ident $(: $value:expr)?),*) => {
        {
            let __args_ty = ::core::marker::PhantomData::<_>;
            if false {
                 $crate::passthrough_helper(__args_ty)
            } else {
                let __builder = $crate::builder_helper(__args_ty);
                $(let __builder = __builder.$name($crate::args!(@ifn { $($value)? } $name));)*
                __builder.finalize()
            }
        }
    };
}

/// A trait indicating a struct can be constructured using a binrw named arguments builder.
pub trait BinrwNamedArgs {
    /// The initial builder type from which this type can be constructed
    type Builder;

    /// A method for creating a new builder to construct this type from
    fn builder() -> Self::Builder;
}
