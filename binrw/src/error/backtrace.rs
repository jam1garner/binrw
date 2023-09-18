use super::{ContextExt, CustomError, Error};
use alloc::{borrow::Cow, boxed::Box, format, string::ToString, vec::Vec};
use core::fmt::{self, Write};

#[cfg(feature = "verbose-backtrace")]
const BOLD_OPEN: &str = "\x1b[1m";
#[cfg(feature = "verbose-backtrace")]
const BOLD_CLOSE: &str = "\x1b[22m";
#[cfg(not(feature = "verbose-backtrace"))]
const BOLD_OPEN: &str = "";
#[cfg(not(feature = "verbose-backtrace"))]
const BOLD_CLOSE: &str = "";

/// An error backtrace.
#[non_exhaustive]
#[derive(Debug)]
pub struct Backtrace {
    /// The source error which caused this backtrace.
    ///
    /// This is guaranteed to not itself be a backtrace.
    pub error: Box<Error>,

    /// The frames which lead to the given error.
    ///
    /// The first frame is the innermost frame.
    pub frames: Vec<BacktraceFrame>,
}

impl Backtrace {
    /// Creates a new backtrace from a source error and a set of frames.
    ///
    /// If the source error is an [`Error::Backtrace`], the given frames are
    /// appended to that object and it is unwrapped and used instead of creating
    /// a new backtrace. This ensures that [`Backtrace::error`] is never a
    /// `Backtrace` and avoids recursion.
    #[must_use]
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
                &format!(
                    "{BOLD_OPEN}Error: {}{BOLD_CLOSE}\n    {}{BOLD_OPEN}{}{BOLD_CLOSE}",
                    FirstErrorFmt(&self.error),
                    if matches!(self.error.as_ref(), Error::EnumErrors { .. }) {
                        "..."
                    } else {
                        "       "
                    },
                    first_frame.message(),
                ),
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
    fn with_context<Frame: Into<BacktraceFrame>>(mut self, frame: Frame) -> Self {
        self.frames.push(frame.into());
        self
    }

    #[track_caller]
    fn with_message(self, message: impl Into<Cow<'static, str>>) -> Self {
        let caller = core::panic::Location::caller();
        self.with_context(BacktraceFrame::Full {
            code: None,
            message: message.into(),
            file: caller.file(),
            line: caller.line(),
        })
    }
}

impl fmt::Display for Backtrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if cfg!(feature = "verbose-backtrace") {
            writeln!(
                f,
                "\n ╺━━━━━━━━━━━━━━━━━━━━┅ Backtrace ┅━━━━━━━━━━━━━━━━━━━━╸\n"
            )?;
        }

        self.fmt_no_bars(f)?;

        if cfg!(feature = "verbose-backtrace") {
            writeln!(
                f,
                "\n ╺━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╸\n"
            )?;
        }

        Ok(())
    }
}

/// A captured backtrace frame.
#[derive(Debug)]
pub enum BacktraceFrame {
    /// A standard frame.
    Full {
        /// The code at the location where the frame was generated.
        code: Option<&'static str>,

        /// The context message. This may be overridden by the error itself when
        /// full backtraces are enabled.
        message: Cow<'static, str>,

        /// The origin filename.
        file: &'static str,

        /// The origin line number.
        line: u32,
    },

    /// A message-only frame.
    Message(Cow<'static, str>),

    /// A user-specified custom error context.
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
            } => {
                writeln!(
                    f,
                    " {index}: {BOLD_OPEN}{message}{BOLD_CLOSE}\n     at {file}:{line}",
                )?;
                if let Some(code) = code {
                    write!(f, "{code}")?;
                }
                Ok(())
            }
            BacktraceFrame::Message(_) | BacktraceFrame::Custom(_) => {
                writeln!(f, " {index}: {BOLD_OPEN}{message}{BOLD_CLOSE}")
            }
        }
    }

    fn display(&self, f: &mut fmt::Formatter<'_>, index: usize) -> fmt::Result {
        self.display_with_message(f, &self.message(), index)
    }

    fn message(&self) -> Cow<'_, str> {
        match self {
            BacktraceFrame::Full { message: msg, .. } | BacktraceFrame::Message(msg) => msg.clone(),
            BacktraceFrame::Custom(context) => context.to_string().into(),
        }
    }
}

impl<T: CustomError + 'static> From<T> for BacktraceFrame {
    fn from(err: T) -> Self {
        Self::Custom(Box::new(err) as _)
    }
}

struct FirstErrorFmt<'a>(&'a Error);

impl fmt::Display for FirstErrorFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Error::EnumErrors {
                pos,
                variant_errors,
            } => {
                writeln!(f, "no variants matched at {pos:#x?}...{BOLD_CLOSE}")?;

                for (i, (name, err)) in variant_errors.iter().enumerate() {
                    if i != 0 {
                        writeln!(f)?;
                    }

                    writeln!(
                        f,
                        "   ╭───────────────────────┄ {name} ┄────────────────────┄"
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

struct Indenter<'a, 'b>(&'a mut fmt::Formatter<'b>);

impl Write for Indenter<'_, '_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut is_first = true;
        for line in s.split_inclusive('\n') {
            if is_first {
                is_first = false;
            } else {
                self.0.write_str("   ┆")?;
            }
            self.0.write_str(line)?;
        }

        if s.ends_with('\n') {
            self.0.write_str("   ┆")
        } else {
            Ok(())
        }
    }
}

struct NoBars<'a>(&'a Error);

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
    #[cfg_attr(coverage_nightly, coverage(off))]
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
                message: Cow::Borrowed(ERR1),
                file: file!(),
                line: l1,
            }, BacktraceFrame::Full {
                code: None,
                message: Cow::Borrowed(ERR2),
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
