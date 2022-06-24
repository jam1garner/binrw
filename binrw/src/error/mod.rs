//! Functions and type definitions for handling errors.

#[cfg(all(doc, not(feature = "std")))]
extern crate std;
use crate::io;
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};
use core::{any::Any, fmt};

use crate::alloc::borrow::Cow;

mod backtrace;
pub use backtrace::*;

/// The `CustomError` trait describes types that are usable as custom errors
/// in a [`BinResult`](crate::BinResult).
///
/// This trait is automatically implemented for any type which implements the
/// same traits as [`std::error::Error`], so anything you would normally use as
/// an error in other code is also a valid `CustomError`, with the additional
/// restriction that it must also be [`Send`] + [`Sync`].
///
/// This trait is Sealed.
pub trait CustomError: fmt::Display + fmt::Debug + Send + Sync + private::Sealed {
    #[doc(hidden)]
    fn as_any(&self) -> &(dyn Any + Send + Sync);

    #[doc(hidden)]
    fn as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync);

    #[doc(hidden)]
    fn as_box_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
}

impl<T: fmt::Display + fmt::Debug + Send + Sync + 'static> CustomError for T {
    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync) {
        self
    }

    fn as_box_any(self: Box<Self>) -> Box<dyn Any + Send + Sync> {
        self
    }
}

/// An extension trait allowing for errors to be given additional frames of context
pub trait ContextExt {
    /// Adds an additional frame of context to the backtrace
    fn with_context<Frame: Into<BacktraceFrame>>(self, frame: Frame) -> Self;

    /// Adds an additional frame of context to the backtrace including a message, file name, and
    /// line number.
    #[track_caller]
    fn with_message(self, message: impl Into<Cow<'static, str>>) -> Self;
}

// The intent here is to allow any object which is compatible with
// `std::error::Error + Send + Sync` to be stored in errors, including no_std
// mode.
impl dyn CustomError {
    /// Attempts to downcast a boxed error to a concrete type.
    pub fn downcast<T: CustomError + 'static>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if self.is::<T>() {
            Ok(self.as_box_any().downcast().unwrap())
        } else {
            Err(self)
        }
    }

    /// Returns some mutable reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    pub fn downcast_mut<T: CustomError + 'static>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut()
    }

    /// Returns some reference to the boxed value if it is of type `T`, or
    /// `None` if it isn’t.
    pub fn downcast_ref<T: CustomError + 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }

    /// Returns `true` if the boxed type is the same as `T`.
    pub fn is<T: CustomError + 'static>(&self) -> bool {
        core::any::TypeId::of::<T>() == self.as_any().type_id()
    }
}

/// The error type used by [`BinRead`](crate::BinRead).
#[non_exhaustive]
pub enum Error {
    /// An expected [magic number](crate::attribute#magic) was not found.
    BadMagic {
        /// The byte position of the unexpected magic in the reader.
        pos: u64,

        /// The value which was actually read.
        found: Box<dyn fmt::Debug + Send + Sync>,
    },

    /// An assertion failed.
    ///
    /// This variant is used for [`assert`] directives which use a string
    /// literal instead of an error object. Assertions that use error objects
    /// are represented by the [`Custom`] variant.
    ///
    /// [`assert`]: crate::attribute#assert
    /// [`Custom`]: Self::Custom
    AssertFail {
        /// The byte position of the start of the field or object that raised
        /// an error.
        pos: u64,

        /// The failure message.
        message: String,
    },

    /// An error occurred in the underlying reader while reading or seeking to
    /// data.
    Io(io::Error),

    /// A user-generated error.
    ///
    /// This variant is used for [`assert`] directives which use an error object
    /// instead of a string literal. Assertions that use string literals are
    /// represented by the [`AssertFail`] variant.
    ///
    /// [`assert`]: crate::attribute#assert
    /// [`AssertFail`]: Self::AssertFail
    Custom {
        /// The byte position of the start of the field or object that raised
        /// an error.
        pos: u64,

        /// The original error.
        err: Box<dyn CustomError>,
    },

    /// None of the variants of an enum could successfully be parsed from the
    /// data in the reader.
    ///
    /// This variant is used when the [`return_unexpected_error`] directive is
    /// set on an enum.
    ///
    /// [`return_unexpected_error`]: crate::attribute#enum-errors
    NoVariantMatch {
        /// The byte position of the unparsable data in the reader.
        pos: u64,
    },

    /// None of the variants of an enum could successfully be parsed from the
    /// data in the reader.
    ///
    /// This variant is used when the [`return_all_errors`] directive is
    /// set on an enum (which is the default).
    ///
    /// [`return_all_errors`]: crate::attribute#enum-errors
    EnumErrors {
        /// The byte position of the unparsable data in the reader.
        pos: u64,

        /// The original errors which occurred when trying to parse each
        /// variant.
        ///
        /// The first field of the tuple is the name of the variant, and the
        /// second field is the error that occurred when parsing that variant.
        variant_errors: Vec<(&'static str, Error)>,
    },

    /// An error with additional frames of context used to construct a backtrace
    Backtrace(Backtrace),
}

impl Error {
    /// Returns the source error. For a Backtrace this is the error that caused it, for every
    /// other error this returns self
    pub fn root_cause(&self) -> &Self {
        match self {
            Self::Backtrace(backtrace) => &*backtrace.error,
            error => error,
        }
    }

    /// Check if the [root cause][`Self::root_cause`] of this error is an [`Error::Io`] and an
    /// [`io::ErrorKind::UnexpectedEof`].
    pub fn is_eof(&self) -> bool {
        if let Error::EnumErrors {
            pos: _,
            variant_errors,
        } = self
        {
            variant_errors.iter().all(|(_, err)| err.is_eof())
        } else {
            matches!(
                self.root_cause(),
                Error::Io(err) if err.kind() == io::ErrorKind::UnexpectedEof,
            )
        }
    }

    /// Returns a reference to the boxed error object if this `Error` is a
    /// custom error of type `T`, or `None` if it isn’t.
    pub fn custom_err<T: CustomError + 'static>(&self) -> Option<&T> {
        if let Error::Custom { err, .. } = self {
            err.downcast_ref()
        } else {
            None
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMagic { pos, found } => write!(f, "bad magic at 0x{:x}: {:?}", pos, found),
            Self::AssertFail { pos, message } => write!(f, "{} at 0x{:x}", message, pos),
            Self::Io(err) => fmt::Display::fmt(err, f),
            Self::Custom { pos, err } => write!(f, "{} at 0x{:x}", err, pos),
            Self::NoVariantMatch { pos } => write!(f, "no variants matched at 0x{:x}", pos),
            Self::EnumErrors {
                pos,
                variant_errors,
            } => {
                write!(f, "no variants matched at 0x{:x}:", pos)?;
                for (name, err) in variant_errors {
                    write!(f, "\n  {}: {}", name, err)?;
                }
                Ok(())
            }
            Self::Backtrace(backtrace) => write!(f, "{}", backtrace),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Error as fmt::Display>::fmt(self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

mod private {
    use core::fmt;
    pub trait Sealed {}
    impl<T: fmt::Display + fmt::Debug + Send + Sync + 'static> Sealed for T {}
}
