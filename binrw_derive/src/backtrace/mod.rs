#![allow(clippy::non_ascii_literal)]
#![allow(unused_imports, unused_variables, dead_code)]
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    ops::Range,
};

use crate::parser::read::StructField;
use proc_macro2::Span;
use syn::{spanned::Spanned, Type};

#[derive(Debug)]
pub(crate) struct BacktraceFrame {
    span: Span,
    ty: Type,
    highlight_line: usize,
}

//struct SyntaxInfo {
//    lines: HashMap<usize, LineSyntax>,
//}
//
//enum Color {
//
//}
//
//struct LineSyntax {
//    highlights: Vec<(Range<usize>, Color)>,
//}

impl BacktraceFrame {
    pub(crate) fn from_field(field: &StructField) -> Self {
        Self {
            span: field.span,
            highlight_line: field.ty.span().start().line,
            ty: field.ty.clone(),
        }
    }

    #[cfg(nightly)]
    fn iter_lines(&self) -> impl Iterator<Item = (usize, String)> + '_ {
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
                    if i == 0 {
                        let spaces_to_add = start_col - min_whitespace;
                        if spaces_to_add == 0 {
                            line.to_owned()
                        } else {
                            format!("{}{}", " ".repeat(spaces_to_add), line)
                        }
                    } else {
                        line[min_whitespace..].to_owned()
                    }
                }))
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            Vec::new().into_iter()
        }
    }

    fn write_line(&self, line_num: usize, line: &str, f: &mut Formatter<'_>) -> fmt::Result {
        if line_num == self.highlight_line {
            writeln!(f, "   {} ⎬  {}", line_num, line)
        } else {
            writeln!(f, "   {} │  {}", line_num, line)
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
        for (line_num, line) in self.iter_lines() {
            self.write_line(line_num, &line, f)?;
        }
        writeln!(f, "  ┄{}─╯", bars)?;

        Ok(())
    }

    #[cfg(not(nightly))]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
