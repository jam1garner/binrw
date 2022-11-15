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
            highlight_line: field.ty.span().start().line,
            syntax_info: syntax_highlighting::get_syntax_highlights(field),
        }
    }

    fn iter_lines(&self) -> impl Iterator<Item = Line> + '_ {
        // Calling `unwrap` will cause a panic during code coverage analysis
        // since in that case proc_macro is being emulated so there is no
        // underlying Span; this code therefore must only run when the
        // proc_macro condition is set
        #[cfg(all(feature = "verbose-backtrace", nightly, proc_macro))]
        let source_text = self.span.unwrap().source_text();
        #[cfg(not(all(feature = "verbose-backtrace", nightly, proc_macro)))]
        let source_text = None::<String>;

        if let Some(text) = source_text {
            let start_col = self.span.start().column - 1;
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
                (self.span.start().line..)
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
        let line = self.span.end().line;

        if line != 0 {
            let max_digits =
                core::iter::successors(Some(line), |n| Some(n / 10).filter(|n| *n != 0)).count();

            let bars = "─".repeat(max_digits);

            writeln!(f, "  ┄{}─╮", bars)?;
            for line in self.iter_lines() {
                self.write_line(line, max_digits, f)?;
            }
            writeln!(f, "  ┄{}─╯", bars)?;
        }

        Ok(())
    }
}
