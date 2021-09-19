#![allow(clippy::non_ascii_literal)]
//#![allow(unused_imports, unused_variables, dead_code)]
use std::fmt::{self, Display, Formatter};

mod syntax_highlighting;
use owo_colors::OwoColorize;
use syntax_highlighting::SyntaxInfo;

use crate::parser::read::StructField;
use proc_macro2::Span;
use syn::spanned::Spanned;

pub(crate) struct BacktraceFrame {
    span: Span,
    //ty: Type,
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
            span: field.span,
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

        if should_highlight {
            write!(f, "   {} {}  ", line_num.bold(), "⎬".bold())?;
        } else {
            write!(f, "   {} │  ", line_num)?;
        }

        if let Some(line_highlights) = self.syntax_info.lines.get(&line_num) {
            // syntax highlighting on this line
            let highlights = line_highlights.highlights.iter();
            let highlights_next_start = line_highlights
                .highlights
                .iter()
                .skip(1)
                .map(|x| x.0.start)
                .chain(std::iter::once(start_col + line.len()));

            if let Some((first_range, _)) = line_highlights.highlights.get(0) {
                let component = &line[..first_range.start - start_col];

                if should_highlight {
                    write!(f, "{}", component.bold())?;
                } else {
                    write!(f, "{}", component)?;
                }
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
