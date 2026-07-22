use super::parser::StructField;
use core::fmt::{self, Display, Write as _};
use owo_colors::{OwoColorize as _, XtermColors};
use proc_macro2::{Delimiter, LineColumn, Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{Lit, spanned::Spanned};

/// Generates a basic syntax-highlighted copy of the original source text for
/// the given `field`.
pub(crate) fn source_text(field: &StructField) -> Result<String, fmt::Error> {
    let field_span = field.field.span();
    let Some(text) = field_span.source_text() else {
        return Ok(<_>::default());
    };

    let first_column = field_span.start().column;
    let indent = text
        .lines()
        // The first line starts after the indent, at `first_column`, so needs
        // to be ignored here or else indent will always be 0
        .skip(1)
        // If a whole line is whitespace (`None`), it can be excluded from
        // consideration
        .filter_map(|text| text.find(|c: char| !c.is_ascii_whitespace()))
        .fold(first_column, core::cmp::Ord::min);

    // On at least ≤1.97 (maybe because `Span::join` is not stable),
    // `field.span.end()` is wrong, and so is `source_text()`. At least we know
    // what we want to highlight is the line with the ident on it (or the line
    // with the type, if it’s an unnamed field).
    let bold_line = if field.generated_ident {
        field.ty.span().start().line
    } else {
        field.ident.span().start().line
    };
    // Similarly, the last line should be the one where the field type ends
    let digits = usize::try_from(field.ty.span().end().line.ilog10() + 1).unwrap();
    let bars = "─".repeat(digits);

    let mut f = String::new();
    write!(f, "  ┄{bars}─╮")?;
    Highlighter::new(&mut f, digits, indent, first_column, bold_line)
        .highlight(field.field.to_token_stream())?;
    writeln!(f)?;
    writeln!(f, "  ┄{bars}─╯")?;
    Ok(f)
}

/// Highlighter tracking state.
struct Highlighter<'a> {
    /// The current 0-indexed column.
    column: usize,
    /// The maximum number of digits in the line number.
    digits: usize,
    /// The whitespace common to all lines of the field.
    indent: usize,
    /// The line which should be shown with emphasis in the output.
    bold_line: usize,
    /// The current 1-indexed line (or 0 if nothing has been emitted yet).
    line: usize,
    /// The output string for the highlighter.
    out: &'a mut String,
}

impl<'a> Highlighter<'a> {
    /// Creates a new `Highlighter` with the given number of line `digits`,
    /// common whitespace `indent` size, first token column `column`, and
    /// emphasis on `last_line`.
    fn new(
        out: &'a mut String,
        digits: usize,
        indent: usize,
        column: usize,
        bold_line: usize,
    ) -> Self {
        Self {
            column,
            digits,
            indent,
            bold_line,
            line: 0,
            out,
        }
    }

    /// Writes basic highlighted source code for the given `tokens` to
    /// [`Self::out`].
    fn highlight(&mut self, tokens: TokenStream) -> fmt::Result {
        for token in tokens {
            match token {
                TokenTree::Group(group) => {
                    // The span ranges on these delimiters are fucked
                    let (open, close) = match group.delimiter() {
                        Delimiter::Parenthesis => ("(", ")"),
                        Delimiter::Brace => ("{", "}"),
                        Delimiter::Bracket => ("[", "]"),
                        Delimiter::None => ("", ""),
                    };
                    self.write_delim(group.span_open().start(), open)?;
                    self.highlight(group.stream())?;
                    self.write_delim(group.span_close().start(), close)?;
                }
                TokenTree::Ident(ident) => {
                    let color = if syn::parse2::<syn::LitBool>(ident.to_token_stream()).is_ok() {
                        Some(Color::Unary)
                    } else if is_keyword_ident(&ident) {
                        Some(Color::Keyword)
                    } else {
                        None
                    };
                    self.write_color_span(ident.span(), color)?;
                }
                TokenTree::Punct(punct) => {
                    self.write_color_span(punct.span(), None)?;
                }
                TokenTree::Literal(literal) => {
                    let color = match syn::parse2::<syn::Lit>(literal.to_token_stream()) {
                        Ok(Lit::Str(_) | Lit::ByteStr(_)) => Some(Color::String),
                        Ok(Lit::Byte(_) | Lit::Char(_)) => Some(Color::Char),
                        Ok(Lit::Int(_) | Lit::Float(_) | Lit::Bool(_)) => Some(Color::Number),
                        _ => None,
                    };
                    self.write_color_span(literal.span(), color)?;
                }
            }
        }
        Ok(())
    }

    /// Advances the highlighter output to the given `line` and `column`,
    /// writing whitespace for positions where there were no tokens. (This
    /// should write whatever content was in the source text, but
    /// `Span::byte_range` is not stable, and there is no way to generate a span
    /// between tokens, so it is not possible to do this on stable compilers.)
    fn next_pos(&mut self, line: usize, column: usize) -> fmt::Result {
        if line == 0 {
            return Ok(());
        }

        if self.line == 0 {
            self.line = line - 1;
        }

        for _ in self.line..line {
            self.out.push('\n');
            self.line += 1;

            let bar = if self.line == self.bold_line {
                "⎬"
            } else {
                "|"
            };
            let line_num = self.line;
            let digits = self.digits;
            self.write(&format_args!("   {line_num:digits$} {bar}  "))?;
            self.column = self.indent;
        }

        for _ in self.column..column {
            self.out.push(' ');
            self.column += 1;
        }

        Ok(())
    }

    /// Writes `text` to [`Self::out`], possibly with emphasis.
    fn write<D: Display + ?Sized>(&mut self, text: &D) -> fmt::Result {
        if self.line == self.bold_line {
            write!(self.out, "{}", text.bold())
        } else {
            write!(self.out, "{text}")
        }
    }

    /// Writes the text of `span` to [`Self::out`], using `color` if specified.
    fn write_color_span(&mut self, span: Span, color: Option<Color>) -> fmt::Result {
        if let Some(text) = span.source_text() {
            let start = span.start();
            let mut column_num = start.column;
            for (line_num, line) in (start.line..).zip(text.lines()) {
                self.next_pos(line_num, column_num)?;
                let line = if column_num == 0 {
                    &line[self.indent..]
                } else {
                    line
                };
                if let Some(color) = color {
                    self.write(&line.color(color.into_owo()))?;
                } else {
                    self.write(line)?;
                }
                column_num = 0;
            }
            self.column = span.end().column;
        }
        Ok(())
    }

    /// Writes the given delimiter `text` at the given source `pos`.
    fn write_delim(&mut self, pos: LineColumn, text: &str) -> fmt::Result {
        self.next_pos(pos.line, pos.column)?;
        self.write(&text)?;
        self.column = pos.column + 1;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Color {
    Char,    // purple
    Keyword, // red
    Number,  // purple
    String,  // yellow
    Unary,   // blue
}

impl Color {
    fn into_owo(self) -> XtermColors {
        match self {
            Self::Char | Self::Number => XtermColors::Heliotrope,
            Self::Keyword => XtermColors::DarkRose,
            Self::String => XtermColors::DollyYellow,
            Self::Unary => XtermColors::MalibuBlue,
        }
    }
}

fn is_keyword_ident(ident: &syn::Ident) -> bool {
    macro_rules! is_any {
        ($($option:ident),*) => {
            $(ident == (stringify!($option)) ||)* false
        }
    }

    #[rustfmt::skip]
    let is_keyword = is_any!(
        // prelude/keywords/primitives
        Vec, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, char, String, Default,
        Self, super, Drop, Send, Sync, Sized, Fn, FnMut, FnOnce, From, Into, Iterator,
        IntoIterator, Ord, Eq, PartialEq, Eq, Box, ToString, usize, isize, f32, f64, str,
        Option,

        // binrw 'keywords'
        align_after, align_before, args, args_raw, assert, big, binread, br, brw, binwrite,
        bw, calc, count, default, ignore, import, import_raw, is_big, is_little,
        little, magic, map, offset, pad_after, pad_before, pad_size_to, parse_with,
        pre_assert, repr, restore_position, return_all_errors,
        return_unexpected_error, seek_before, temp, try_map, write_with
    );

    is_keyword
}
