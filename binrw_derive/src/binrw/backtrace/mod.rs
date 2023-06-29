mod syntax_highlighting;

use super::parser::StructField;
use core::fmt::{self, Display, Formatter};
use owo_colors::OwoColorize;
use proc_macro2::Span;
use syn::spanned::Spanned;
use syntax_highlighting::{conditional_bold, CondOwo, SyntaxInfo};

pub(crate) struct BacktraceFrame {
    span: Span,
    highlight_line: usize,
    syntax_info: SyntaxInfo,
}

struct Line {
    line_num: usize,
    start_col: usize,
    line: String,
}

impl BacktraceFrame {
    pub(crate) fn from_field(field: &StructField) -> Self {
        Self {
            span: field.field.span(),
            highlight_line: start(field.ty.span()).line(),
            syntax_info: syntax_highlighting::get_syntax_highlights(field),
        }
    }

    fn iter_lines(&self) -> impl Iterator<Item = Line> + '_ {
        if let Some(text) = self.span.source_text() {
            let start_col = start(self.span).column() - 1;
            let mut min_whitespace = start_col;
            for line in text.lines().skip(1) {
                for (i, c) in line.chars().enumerate() {
                    if !c.is_whitespace() {
                        min_whitespace = min_whitespace.min(i);
                        break;
                    }
                }
            }

            either::Left(
                (start(self.span).line()..)
                    .zip(text.lines().enumerate().map(|(i, line)| {
                        let line = if i == 0 {
                            let spaces_to_add = start_col - min_whitespace;
                            if spaces_to_add == 0 {
                                line.to_owned()
                            } else {
                                format!("{}{}", " ".repeat(spaces_to_add), line)
                            }
                        } else {
                            line[min_whitespace..].to_owned()
                        };

                        (min_whitespace + 1, line)
                    }))
                    .map(|(line_num, (start_col, line))| Line {
                        line_num,
                        start_col,
                        line,
                    })
                    .collect::<Vec<_>>()
                    .into_iter(),
            )
        } else {
            either::Right(core::iter::empty())
        }
    }

    fn write_line(
        &self,
        Line {
            line_num,
            start_col,
            line,
        }: Line,
        max_digits: usize,
        f: &mut Formatter<'_>,
    ) -> fmt::Result {
        let should_highlight = line_num == self.highlight_line;

        let bar = if should_highlight {
            CondOwo::Applied("⎬".bold())
        } else {
            CondOwo::NotApplied("|")
        };
        write!(
            f,
            "   {:2$} {}  ",
            conditional_bold(&line_num, should_highlight),
            bar,
            max_digits
        )?;

        if line.trim().starts_with("//") {
            return writeln!(f, "{}", line.color(owo_colors::XtermColors::Boulder));
        }

        if let Some(line_highlights) = self.syntax_info.lines.get(&line_num) {
            let line_len = line.len() + start_col;

            // syntax highlighting on this line
            let highlights = &line_highlights.highlights;
            let highlights = highlights
                .iter()
                .enumerate()
                .filter(|&(i, highlight)| {
                    i == 0 || !highlights[i - 1].0.contains(&highlight.0.start)
                })
                .map(|(_, (range, color))| {
                    (range.start.min(line_len)..range.end.min(line_len), color)
                });
            let highlights_next_start = line_highlights
                .highlights
                .iter()
                .skip(1)
                .map(|x| x.0.start)
                .chain(core::iter::once(start_col + line.len()));

            if let Some((first_range, _)) = line_highlights.highlights.get(0) {
                let component = &line[..first_range.start - start_col];

                write!(f, "{}", conditional_bold(&component, should_highlight))?;
            } else {
                write!(f, "{}", conditional_bold(&line, should_highlight))?;
            }

            for ((range, color), next_start) in highlights.zip(highlights_next_start) {
                let range = (range.start - start_col)..(range.end - start_col);
                let next_start = next_start - start_col;
                let uncolored_range = range.end..next_start;

                // write colored portion
                if !range.is_empty() {
                    write!(
                        f,
                        "{}",
                        conditional_bold(&(&line[range]).color(color.into_owo()), should_highlight)
                    )?;
                }

                if !uncolored_range.is_empty() {
                    // write next uncolored portion
                    write!(
                        f,
                        "{}",
                        conditional_bold(&&line[uncolored_range], should_highlight)
                    )?;
                }
            }

            writeln!(f)
        } else {
            writeln!(f, "{}", conditional_bold(&line, should_highlight))
        }
    }
}

impl Display for BacktraceFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let line = end(self.span).line();

        if line != 0 {
            let max_digits =
                core::iter::successors(Some(line), |n| Some(n / 10).filter(|n| *n != 0)).count();

            let bars = "─".repeat(max_digits);

            writeln!(f, "  ┄{bars}─╮")?;
            for line in self.iter_lines() {
                self.write_line(line, max_digits, f)?;
            }
            writeln!(f, "  ┄{bars}─╯")?;
        }

        Ok(())
    }
}

// Unwrapping the proc-macro2 Span is undesirable but necessary until its API
// is updated to allow retrieving line/column again. Using a separate function
// to unwrap just to make it clearer what needs to be undone later.
// <https://github.com/dtolnay/proc-macro2/pull/383>
struct LineColumn {
    line: usize,
    column: usize,
}

impl LineColumn {
    fn line(&self) -> usize {
        self.line
    }

    fn column(&self) -> usize {
        self.column
    }
}

#[cfg(all(feature = "verbose-backtrace", nightly, proc_macro))]
fn start(span: Span) -> LineColumn {
    let span = span.unwrap().start();
    LineColumn {
        line: span.line(),
        column: span.column(),
    }
}
#[cfg(all(feature = "verbose-backtrace", nightly, proc_macro))]
fn end(span: Span) -> LineColumn {
    let span = span.unwrap().end();
    LineColumn {
        line: span.line(),
        column: span.column(),
    }
}
#[cfg(not(all(feature = "verbose-backtrace", nightly, proc_macro)))]
fn start(_: Span) -> LineColumn {
    LineColumn { line: 0, column: 0 }
}
#[cfg(not(all(feature = "verbose-backtrace", nightly, proc_macro)))]
fn end(_: Span) -> LineColumn {
    LineColumn { line: 0, column: 0 }
}
