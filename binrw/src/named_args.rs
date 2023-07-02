//! Types and macros for generating named arguments builders.

use core::marker::PhantomData;

/// A convenience macro for constructing
/// [named arguments](crate::docs::attribute#named-arguments).
///
/// This macro uses the [`builder()`](NamedArgs::builder) function of a
/// [named arguments type](NamedArgs), and can only be used in positions
/// where the type can be inferred by the compiler (i.e. as a function argument
/// or an assignment to a variable with an explicit type).
///
/// # Examples
///
/// ```
/// use binrw::BinRead;
/// # use binrw::io::Cursor;
///
/// #[derive(BinRead)]
/// #[br(import { a: i32, b: i32 })]
/// struct Foo;
///
/// let mut reader = Cursor::new(b"");
/// let a = 1;
/// Foo::read_args(&mut reader, binrw::args! {
///     a,
///     b: { a * 2 },
/// }).unwrap();
/// ```
#[macro_export]
macro_rules! args {
    (@ifn { $value:expr } $name:ident) => { $value };
    (@ifn {} $name:ident) => { $name };
    ($($name:ident $(: $value:expr)?),* $(,)?) => {
        {
            // I'll use Ret to represent the type of the block
            // token representing the type of the block. we request that the compiler infer it.
            let args_ty = ::core::marker::PhantomData::<_>;
            if false {
                // this statement will never be run, but will be used for type resolution.
                // since this helper is of PhantomData<T> -> T,
                // and the compiler knows that the type Ret should be returned,
                // it infers that args_ty should be of type PhantomData<Ret>.
                $crate::__private::passthrough_helper(args_ty)
            } else {
                // we now pass the PhantomData<Ret> to a helper of PhantomData<T> -> T::Builder
                // to obtain a builder for the type of the block.
                let builder = $crate::__private::builder_helper(args_ty);

                $(let builder = builder.$name($crate::args!(@ifn { $($value)? } $name));)*

                // since the builder returns the type that we used to obtain the builder,
                // the type unifies across the if expression
                builder.finalize()
            }
        }
    };
}

/// The `NamedArgs` trait allows
/// [named arguments](crate::docs::attribute#named-arguments) objects
/// to be constructed using a builder that checks for correctness at compile
/// time.
///
/// See [`#[derive(NamedArgs)]`](derive@crate::NamedArgs) for information on deriving
/// custom named arguments types.
pub trait NamedArgs {
    /// The builder type for this type.
    type Builder;

    /// Creates a new builder for this type.
    fn builder() -> Self::Builder;
}

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
pub fn builder_helper<T: NamedArgs>(_: PhantomData<T>) -> T::Builder {
    <T as NamedArgs>::builder()
}
