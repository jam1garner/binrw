use std::{collections::HashMap, ops::Range};

use crate::parser::read::StructField;
use owo_colors::XtermColors;
use syn::visit::{self, /*visit_expr,*/ visit_type, Visit};

#[derive(Default)]
pub(crate) struct SyntaxInfo {
    pub(crate) lines: HashMap<usize, LineSyntax>,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Color {
    String,   // yellow
    Char,     // purple
    Number,   // purple
    Keyword,  // red
    Function, // green
}

impl Color {
    pub(crate) fn into_owo(self) -> owo_colors::XtermColors {
        match self {
            Self::String => XtermColors::DollyYellow,
            Self::Char | Self::Number => XtermColors::Heliotrope,
            Self::Keyword => XtermColors::DarkRose,
            Self::Function => XtermColors::RioGrandeGreen,
        }
    }
}

#[derive(Default)]
pub(crate) struct LineSyntax {
    pub(crate) highlights: Vec<(Range<usize>, Color)>,
}

#[derive(Default)]
struct Visitor {
    syntax_info: SyntaxInfo,
}

pub(super) fn get_syntax_highlights(field: &StructField) -> SyntaxInfo {
    let mut visit = Visitor::default();

    visit_type(&mut visit, &field.ty);

    let Visitor { syntax_info } = visit;

    syntax_info
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_lit(&mut self, lit: &'ast syn::Lit) {
        let start = lit.span().start();
        let end = lit.span().end();

        // syntax highlighting for multi-line spans isn't supported yet (sorry)
        if start.line == end.line {
            #[allow(clippy::enum_glob_use)]
            use syn::Lit::*;

            let lines = self
                .syntax_info
                .lines
                .entry(start.line)
                .or_insert_with(LineSyntax::default);

            lines.highlights.push((
                start.column..end.column,
                match lit {
                    Str(_) | ByteStr(_) => Color::String,
                    Byte(_) | Char(_) => Color::Char,
                    Int(_) | Float(_) | Bool(_) => Color::Number,
                    Verbatim(_) => return,
                },
            ));
        }
    }

    fn visit_ident(&mut self, ident: &'ast proc_macro2::Ident) {
        if is_keyword_ident(ident) {
            let start = ident.span().start();
            let end = ident.span().end();

            self.syntax_info
                .lines
                .entry(start.line)
                .or_insert_with(LineSyntax::default)
                .highlights
                .push((start.column..end.column, Color::Keyword));
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let ident = &call.method;
        let start = ident.span().start();
        let end = ident.span().end();

        self.syntax_info
            .lines
            .entry(start.line)
            .or_insert_with(LineSyntax::default)
            .highlights
            .push((start.column..end.column, Color::Function));

        // continue walking ast
        for attr in &call.attrs {
            visit::visit_attribute(self, attr);
        }

        visit::visit_expr(self, &*call.receiver);

        if let Some(turbofish) = call.turbofish.as_ref() {
            visit::visit_method_turbofish(self, turbofish);
        }

        for arg in call.args.iter() {
            visit::visit_expr(self, arg);
        }
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*call.func {
            if let Some(ident) = path.path.get_ident() {
                let start = ident.span().start();
                let end = ident.span().end();

                self.syntax_info
                    .lines
                    .entry(start.line)
                    .or_insert_with(LineSyntax::default)
                    .highlights
                    .push((start.column..end.column, Color::Function));
            }
        }

        // continue walking ast
        for attr in &call.attrs {
            visit::visit_attribute(self, attr);
        }

        visit::visit_expr(self, &*call.func);

        for arg in call.args.iter() {
            visit::visit_expr(self, arg);
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

        // binrw 'keywords'
        align_after, align_before, args, args_raw, assert, big, binread, br, brw, binwrite,
        bw, calc, count, default, deref_now, ignore, import, import_raw, is_big, is_little,
        little, magic, map, offset, offset_after, pad_after, pad_before, pad_size_to, parse_with,
        postprocess_now, pre_assert, repr, restore_position, return_all_errors,
        return_unexpected_error, seek_before, temp, try_map, write_with
    );

    is_keyword
}
