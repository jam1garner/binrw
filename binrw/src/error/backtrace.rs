use super::*;

use crate::alloc::borrow::Cow;
use crate::alloc::string::ToString;
use crate::alloc::{format, vec};
use core::fmt;

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

impl fmt::Display for Backtrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "\n ╺━━━━━━━━━━━━━━━━━━━━┅ Backtrace ┅━━━━━━━━━━━━━━━━━━━━╸\n"
        )?;

        self.fmt_no_bars(f)?;

        #[cfg(not(nightly))]
        writeln!(
            f,
            "\n ╺━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╸\n"
        )?;

        #[cfg(nightly)]
        writeln!(
            f,
            " ╺━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╸\n"
        )?;

        Ok(())
    }
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

    fn fmt_no_bars(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut frames = self.frames.iter();

        if let Some(first_frame) = frames.next() {
            first_frame.display_with_message(
                f,
                &format!("\x1b[1mError: {}\x1b[22m", FirstErrorFmt(&*self.error)),
                0,
            )?;

            for (i, frame) in frames.enumerate() {
                frame.display(f, i + 1)?;
            }
        }

        Ok(())
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
        let caller = core::panic::Location::caller();

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

impl ContextExt for Error {
    fn with_context<Frame: Into<BacktraceFrame>>(self, frame: Frame) -> Self {
        match self {
            Error::Backtrace(mut backtrace) => {
                backtrace.frames.push(frame.into());
                Error::Backtrace(backtrace)
            }
            error => Error::Backtrace(Backtrace::new(error, vec![frame.into()])),
        }
    }

    #[track_caller]
    fn with_message(self, message: impl Into<Cow<'static, str>>) -> Self {
        match self {
            Error::Backtrace(backtrace) => Error::Backtrace(backtrace.with_message(message)),
            error => {
                let message = message.into();
                let caller = core::panic::Location::caller();
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
        }
    }
}

impl<T> ContextExt for Result<T, Error> {
    /// Adds an additional frame of context to the backtrace
    fn with_context<Frame: Into<BacktraceFrame>>(self, frame: Frame) -> Self {
        self.map_err(|err| err.with_context(frame))
    }

    /// Adds an additional frame of context to the backtrace including a message, file name, and
    /// line number.
    #[track_caller]
    fn with_message(self, message: impl Into<Cow<'static, str>>) -> Self {
        match self {
            Err(err) => {
                let caller = core::panic::Location::caller();
                Err(match err {
                    Error::Backtrace(backtrace) => {
                        Error::Backtrace(backtrace.with_message(message))
                    }
                    error => {
                        let message = message.into();
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
            ok => ok,
        }
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

impl BacktraceFrame {
    fn display_with_message(
        &self,
        f: &mut fmt::Formatter<'_>,
        message: &impl fmt::Display,
        index: usize,
    ) -> fmt::Result {
        match self {
            BacktraceFrame::Full {
                code, file, line, ..
            }
            | BacktraceFrame::OwnedFull {
                code, file, line, ..
            } => {
                writeln!(f, " {}: {}\n     at {}:{}", index, message, file, line)?;
                if let Some(code) = code {
                    writeln!(f, "{}", code.trim_end())?;
                }
                Ok(())
            }
            BacktraceFrame::Message(_)
            | BacktraceFrame::OwnedMessage(_)
            | BacktraceFrame::Custom(_) => {
                writeln!(f, " {}: {}", index, message)
            }
        }
    }

    fn display(&self, f: &mut fmt::Formatter<'_>, index: usize) -> fmt::Result {
        let msg;
        let message: &str = match self {
            BacktraceFrame::Full { message: msg, .. } | BacktraceFrame::Message(msg) => msg,
            BacktraceFrame::OwnedFull { message: msg, .. } | BacktraceFrame::OwnedMessage(msg) => {
                msg
            }
            BacktraceFrame::Custom(context) => {
                msg = context.to_string();
                &msg
            }
        };

        self.display_with_message(f, &message, index)
    }
}

impl<T: CustomError + 'static> From<T> for BacktraceFrame {
    fn from(err: T) -> Self {
        Self::Custom(Box::new(err) as _)
    }
}

struct NoBars<'a>(&'a Error);
struct FirstErrorFmt<'a>(&'a Error);
struct Indenter<'a, 'b>(&'a mut fmt::Formatter<'b>);

use fmt::Write;

impl fmt::Display for FirstErrorFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Error::EnumErrors {
                pos,
                variant_errors,
            } => {
                writeln!(f, "no variants matched at {:#x?}\x1b[22m", pos)?;

                for (name, err) in variant_errors {
                    writeln!(
                        f,
                        "   ╭───────────────────────┄ {} ┄────────────────────┄",
                        name
                    )?;
                    writeln!(f, "   ┆")?;
                    write!(f, "   ┆")?;
                    write!(Indenter(f), "{}", NoBars(err))?;
                    write!(
                        f,
                        "\n   ╰─────────────────────────{}──────────────────────┄",
                        "─".repeat(name.len())
                    )?;
                }

                Ok(())
            }
            error => <Error as fmt::Display>::fmt(error, f),
        }
    }
}

impl fmt::Write for Indenter<'_, '_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !s.contains('\n') {
            self.0.write_str(s)
        } else {
            let mut last_ended_in_newline = false;
            let mut is_first = true;
            for line in s.split_inclusive('\n') {
                if !is_first {
                    self.0.write_str("   ┆")?;
                }
                is_first = false;
                self.0.write_str(line)?;

                last_ended_in_newline = line.ends_with('\n');
            }

            if last_ended_in_newline {
                self.0.write_str("   ┆")
            } else {
                Ok(())
            }
        }
    }
}

impl fmt::Display for NoBars<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Error::Backtrace(backtrace) => backtrace.fmt_no_bars(f),
            error => <Error as fmt::Display>::fmt(error, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backtrace() {
        const ERR0: &str = "assert_failed";
        const ERR1: &str = "while doing something";
        const ERR2: &str = "then this happened";
        const ERR3: &str = "and lastly this happened";

        let error = Error::AssertFail {
            pos: 4,
            message: ERR0.to_string(),
        };

        let (line1, error) = (line!(), Err::<(), _>(error.with_message(ERR1)));
        let (line2, error) = (line!(), error.with_message(ERR2));
        let error = error.with_context(ERR3);

        if let Error::Backtrace(backtrace) = error.unwrap_err() {
            if let Error::AssertFail { pos: 4, message } = &*backtrace.error {
                assert_eq!(message, ERR0);
            } else {
                panic!("Not AssertFail")
            }

            if let [BacktraceFrame::Full {
                code: None,
                message: ERR1,
                file: file!(),
                line: l1,
            }, BacktraceFrame::Full {
                code: None,
                message: ERR2,
                file: file!(),
                line: l2,
            }, BacktraceFrame::Custom(last)] = &backtrace.frames[..]
            {
                assert_eq!(line1, *l1);
                assert_eq!(line2, *l2);
                assert_eq!(last.to_string(), ERR3);
            } else {
                panic!("Backtrace incorrect: {:?}", &backtrace.frames)
            }
        } else {
            panic!("Not a backtrace")
        }
    }
}
