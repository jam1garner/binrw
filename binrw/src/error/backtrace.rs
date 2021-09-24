use super::*;
use crate::alloc::borrow::Cow;

/// A backtrace containing a set of frames representing (in order from innermost to outmost code)
#[non_exhaustive]
#[derive(Debug)]
pub struct Backtrace {
    /// The source error which caused this backtrace. This is guaranteed to not itself be a
    /// backtrace.
    pub error: Box<Error>,

    /// The frames which lead to the given error
    pub frames: Vec<BacktraceFrame>,
}

impl Backtrace {
    /// Create a new backtrace from a source error and a set of frames
    ///
    /// If the error itself is a `Backtrace`, the set of frames is appended to the existing
    /// set of frames. This ensures `Backtrace::error` is not itself a `Backtrace`.
    pub fn new(error: Error, frames: Vec<BacktraceFrame>) -> Self {
        let mut frames = frames;
        match error {
            Error::Backtrace(mut backtrace) => {
                backtrace.frames.append(&mut frames);
                backtrace
            }
            error => Self {
                error: Box::new(error),
                frames,
            },
        }
    }
}

impl ContextExt for Backtrace {
    /// Adds an additional frame of context to the backtrace
    fn with_context<Frame: Into<BacktraceFrame>>(mut self, frame: Frame) -> Self {
        self.frames.push(frame.into());
        self
    }

    /// Adds an additional frame of context to the backtrace including a message, file name, and
    /// line number.
    #[track_caller]
    fn with_message(mut self, message: impl Into<Cow<'static, str>>) -> Self {
        let message = message.into();
        let caller = std::panic::Location::caller();

        match message {
            Cow::Owned(message) => {
                self.frames.push(BacktraceFrame::OwnedFull {
                    code: None,
                    message,
                    file: caller.file(),
                    line: caller.line(),
                });
            }
            Cow::Borrowed(message) => {
                self.frames.push(BacktraceFrame::Full {
                    code: None,
                    message,
                    file: caller.file(),
                    line: caller.line(),
                });
            }
        }

        self
    }
}

impl<T> ContextExt for Result<T, Error> {
    /// Adds an additional frame of context to the backtrace
    fn with_context<Frame: Into<BacktraceFrame>>(self, frame: Frame) -> Self {
        self.map_err(|err| match err {
            Error::Backtrace(mut backtrace) => {
                backtrace.frames.push(frame.into());
                Error::Backtrace(backtrace)
            }
            error => Error::Backtrace(Backtrace::new(error, vec![frame.into()])),
        })
    }

    /// Adds an additional frame of context to the backtrace including a message, file name, and
    /// line number.
    #[track_caller]
    fn with_message(self, message: impl Into<Cow<'static, str>>) -> Self {
        self.map_err(|err| match err {
            Error::Backtrace(backtrace) => Error::Backtrace(backtrace.with_message(message)),
            error => {
                let message = message.into();
                let caller = std::panic::Location::caller();
                Error::Backtrace(Backtrace::new(
                    error,
                    vec![match message {
                        Cow::Owned(message) => BacktraceFrame::OwnedFull {
                            code: None,
                            message,
                            file: caller.file(),
                            line: caller.line(),
                        },
                        Cow::Borrowed(message) => BacktraceFrame::Full {
                            code: None,
                            message,
                            file: caller.file(),
                            line: caller.line(),
                        },
                    }],
                ))
            }
        })
    }
}

/// A single frame in the backtrace
#[derive(Debug)]
pub enum BacktraceFrame {
    /// A full backtrace including (optional) codeblocks, a message, a file name, and a line number
    Full {
        /// An optional code block to display only when full backtrace is enabled
        code: Option<&'static str>,

        /// A message explaining the relevance of this current frame. This may be overriden
        /// by the error itself when displaying the full backtrace.
        message: &'static str,

        /// The name of the file this frame comes from
        file: &'static str,

        /// The line number this frame comes from
        line: u32,
    },

    /// A frame which only consists of a static string
    Message(&'static str),

    /// An owned message for use with runtime formatting
    OwnedFull {
        /// An optional code block to display only when full backtrace is enabled
        code: Option<&'static str>,

        /// A message explaining the relevance of this current frame. This may be overriden
        /// by the error itself when displaying the full backtrace.
        message: String,

        /// The name of the file this frame comes from
        file: &'static str,

        /// The line number this frame comes from
        line: u32,
    },

    /// An owned message for use with runtime formatting
    OwnedMessage(String),

    /// A custom error type which doesn't take a specific form
    Custom(Box<dyn CustomError>),
}

impl<T: CustomError + 'static> From<T> for BacktraceFrame {
    fn from(err: T) -> Self {
        Self::Custom(Box::new(err) as _)
    }
}
