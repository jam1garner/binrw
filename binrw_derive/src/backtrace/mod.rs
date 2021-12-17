#![allow(unused_imports, unused_variables, dead_code, clippy::non_ascii_literal)]
use owo_colors::OwoColorize;
use proc_macro2::Span;
use std::fmt::{self, Display, Formatter};
use syn::spanned::Spanned;
mod syntax_highlighting;
use syntax_highlighting::{conditional_bold, CondOwo, SyntaxInfo};

use crate::parser::read::StructField;

pub(crate) struct BacktraceFrame {
    span: Span,
    highlight_line: usize,
    syntax_info: SyntaxInfo,
}

#[cfg(nightly)]
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
            //ty: field.ty.clone(),
        }
    }

    #[cfg(nightly)]
    fn iter_lines(&self) -> impl Iterator<Item = Line> + '_ {
        if let Some(text) = self.span.unwrap().source_text() {
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
                .into_iter()
        } else {
            Vec::new().into_iter()
        }
    }

    #[cfg(nightly)]
    fn write_line(
        &self,
        Line {
            line_num,
            start_col,
            line,
        }: Line,
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
            "   {} {}  ",
            conditional_bold(&line_num, should_highlight),
            bar
        )?;

        if line.trim().starts_with("//") {
            return writeln!(f, "{}", line.color(owo_colors::XtermColors::Boulder));
        }

        if should_highlight {
            write!(f, "   {} {}  ", line_num.bold(), "⎬".bold())?;
        } else {
            write!(f, "   {} │  ", line_num)?;
        }

        if let Some(line_highlights) = self.syntax_info.lines.get(&line_num) {
            let line_len = line.len() + start_col;

            // syntax highlighting on this line
            let highlights = line_highlights.highlights.iter().collect::<Vec<_>>();
            let highlights = highlights
                .iter()
                .enumerate()
                .filter(|&(i, highlight)| {
                    i == 0 || !highlights[i - 1].0.contains(&highlight.0.start)
                })
                .map(|(_, (range, color))| {
                    (range.start.min(line_len)..range.end.min(line_len), color)
                })
                .collect::<Vec<_>>()
                .into_iter();
            let highlights_next_start = line_highlights
                .highlights
                .iter()
                .skip(1)
                .map(|x| x.0.start)
                .chain(std::iter::once(start_col + line.len()));

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
                if should_highlight {
                    write!(f, "{}", (&line[range]).bold().color(color.into_owo()))?;
                } else {
                    write!(f, "{}", (&line[range]).color(color.into_owo()))?;
                }

                // write next uncolored portion
                if should_highlight {
                    write!(f, "{}", (&line[uncolored_range]).bold())?;
                } else {
                    write!(f, "{}", &line[uncolored_range])?;
                }
            }

            writeln!(f)
        } else if should_highlight {
            // no syntax highlighting on this line, but  b o l d
            writeln!(f, "{}", line.bold())
        } else {
            // no syntax highlighting on this line
            writeln!(f, "{}", line)
        }
    }
}

impl Display for BacktraceFrame {
    #[cfg(nightly)]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // it's one allocation, we'll live
        let max_digits = self.span.end().line.to_string().len();

        let bars = "─".repeat(max_digits);

        //writeln!(f)?;
        writeln!(f, "  ┄{}─╮", bars)?;
        for line in self.iter_lines() {
            self.write_line(line, f)?;
        }
        writeln!(f, "  ┄{}─╯", bars)?;

        Ok(())
    }
    #[cfg(not(nightly))]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
