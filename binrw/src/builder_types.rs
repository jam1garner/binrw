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
#[cfg_attr(coverage_nightly, no_coverage)]
#[must_use]
pub fn passthrough_helper<T>(_a: PhantomData<T>) -> T {
    panic!("This is a type system hack and should never be called!");
}

#[doc(hidden)]
#[must_use]
pub fn builder_helper<T: BinrwNamedArgs>(_: PhantomData<T>) -> T::Builder {
    <T as BinrwNamedArgs>::builder()
}

/// A macro for creating a binrw argument type
///
/// This macro avoids taking an explicit type by inferring the type it should create.
/// In general, the result should have an explicit type *immediately*.
/// i.e. being passed to a function or let binding with specific type.
#[macro_export]
macro_rules! args {
    (@ifn { $value:expr } $name:ident) => { $value };
    (@ifn {} $name:ident) => { $name };
    ($($name:ident $(: $value:expr)?),*) => {
        {
            // I'll use Ret to represent the type of the block
            // token representing the type of the block. we request that the compiler infer it.
            let __args_ty = ::core::marker::PhantomData::<_>;
            if false {
                // this statement will never be run, but will be used for type resolution.
                // since this helper is of PhantomData<T> -> T,
                // and the compiler knows that the type Ret should be returned,
                // it infers that __args_ty should be of type PhantomData<Ret>.
                $crate::passthrough_helper(__args_ty)
            } else {
                // we now pass the PhantomData<Ret> to a helper of PhantomData<T> -> T::Builder
                // to obtain a builder for the type of the block.
                let __builder = $crate::builder_helper(__args_ty);

                $(let __builder = __builder.$name($crate::args!(@ifn { $($value)? } $name));)*

                // since the builder returns the type that we used to obtain the builder,
                // the type unifies across the if expression
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
