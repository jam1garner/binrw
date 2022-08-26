use crate::parser::{
    AssertionError, CondEndian, Condition, ErrContext, FieldMode, Map, PassedArgs, StructField,
};
use core::{
    fmt::{Display, Formatter},
    ops::Range,
};
use owo_colors::{styles::BoldDisplay, XtermColors};
use proc_macro2::{Span, TokenTree};
use quote::ToTokens;
use std::collections::HashMap;
use syn::{
    parse::Parse,
    punctuated::Punctuated,
    spanned::Spanned,
    visit::{self, visit_type, Visit},
    Lit,
};

#[derive(Default)]
pub(crate) struct SyntaxInfo {
    pub(crate) lines: HashMap<usize, LineSyntax>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Color {
    String,   // yellow
    Char,     // purple
    Number,   // purple
    Keyword,  // red
    Function, // green
    Unary,    // blue
}

impl Color {
    pub(crate) fn into_owo(self) -> owo_colors::XtermColors {
        match self {
            Self::String => XtermColors::DollyYellow,
            Self::Char | Self::Number => XtermColors::Heliotrope,
            Self::Keyword => XtermColors::DarkRose,
            Self::Function => XtermColors::RioGrandeGreen,
            Self::Unary => XtermColors::MalibuBlue,
        }
    }
}

pub(crate) fn conditional_bold<D>(item: &D, apply: bool) -> CondOwo<BoldDisplay<'_, D>, &'_ D>
where
    D: Display + Sized,
{
    if apply {
        CondOwo::Applied(BoldDisplay(item))
    } else {
        CondOwo::NotApplied(item)
    }
}

pub(crate) enum CondOwo<A, N> {
    Applied(A),
    NotApplied(N),
}

impl<A: Display, N: Display> Display for CondOwo<A, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            CondOwo::Applied(a) => a.fmt(f),
            CondOwo::NotApplied(n) => n.fmt(f),
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

impl SyntaxInfo {
    fn highlight_color(&mut self, span: Span, color: Color) {
        let start = span.start();
        let end = span.end();

        let line = self
            .lines
            .entry(start.line)
            .or_insert_with(LineSyntax::default);

        assert_eq!(start.line, end.line);
        line.highlights.push((start.column..end.column, color));
    }
}

pub(super) fn get_syntax_highlights(field: &StructField) -> SyntaxInfo {
    let mut visit = Visitor::default();

    visit_type(&mut visit, &field.ty);
    visit_expr_attributes(field, &mut visit);
    highlight_attributes(&field.field.attrs, &mut visit);

    let Visitor { mut syntax_info } = visit;

    for keyword_span in &field.keyword_spans {
        let start = keyword_span.start();
        let end = keyword_span.end();
        let line = syntax_info
            .lines
            .entry(start.line)
            .or_insert_with(LineSyntax::default);

        line.highlights
            .push((start.column..end.column, Color::Keyword));
    }

    // ensure highlights are sorted in-order
    syntax_info
        .lines
        .values_mut()
        .for_each(|line| line.highlights.sort_by_key(|x| x.0.start));

    syntax_info
        .lines
        .values_mut()
        .for_each(|line| line.highlights.dedup_by_key(|line| line.0.clone()));

    syntax_info
}

fn highlight_attributes(attrs: &[syn::Attribute], visit: &mut Visitor) {
    let syntax_info = &mut visit.syntax_info;
    for attr in attrs {
        // #[path ...]
        // ^ ^^^^
        // |____|______ path and pound_token
        //
        syntax_info.highlight_color(attr.pound_token.span(), Color::Keyword);
        syntax_info.highlight_color(attr.path.span(), Color::Keyword);

        // #[...]
        //  ^   ^
        //  |___|___ brackets
        //
        let span = attr.bracket_token.span;
        let start = span.start();
        let end = span.end();

        let line = syntax_info
            .lines
            .entry(start.line)
            .or_insert_with(LineSyntax::default);

        // Lint: Makes code less clear.
        #[allow(clippy::range_plus_one)]
        line.highlights
            .push((start.column..start.column + 1, Color::Keyword));
        line.highlights
            .push((end.column - 1..end.column, Color::Keyword));

        // #[path(...)]
        //       ^   ^
        //       |___|___ parens
        //
        if let Some(TokenTree::Group(group)) = attr.tokens.clone().into_iter().next() {
            syntax_info.highlight_color(group.span_open(), Color::Keyword);
            syntax_info.highlight_color(group.span_close(), Color::Keyword);
        }
    }
}

fn visit_expr_attributes(field: &StructField, visitor: &mut Visitor) {
    macro_rules! visit {
        ($expr:expr) => {
            if let Ok(expr) = syn::parse2::<syn::Expr>($expr) {
                visit::visit_expr(visitor, &expr);
            }
        };
    }

    macro_rules! spans_from_exprs {
        ($($field:ident),*) => {
            $(
                if let Some(tokens) = field.$field.clone() {
                    visit!(tokens);
                }
            )*
        };
    }

    spans_from_exprs!(
        count,
        offset,
        pad_before,
        pad_after,
        align_before,
        align_after,
        seek_before,
        pad_size_to
    );

    if let Some(tokens) = field.offset_after.clone() {
        visit!((*tokens).clone());
    }

    if let Some(condition) = field.if_cond.clone() {
        let Condition {
            condition,
            alternate,
        } = condition;

        visit!(condition);
        if let Some(alternate) = alternate {
            visit!(alternate);
        }
    }

    if let Some(magic) = field.magic.clone() {
        visit!(magic.into_value().into_match_value());
    }

    if let CondEndian::Cond(_, expr) = &field.endian {
        visit!(expr.clone());
    }

    if let Map::Map(expr) | Map::Try(expr) = field.map.clone() {
        visit!(expr);
    }

    match &field.args {
        PassedArgs::List(args) => {
            for arg in args.as_ref() {
                visit!(arg.clone());
            }
        }
        PassedArgs::Tuple(expr) => {
            visit!(expr.as_ref().clone());
        }
        PassedArgs::Named(args) => {
            for arg in args.as_ref() {
                if let Ok(args) = syn::parse2::<ArgList>(arg.clone()) {
                    for arg in args.0 {
                        if let Some(expr) = arg.expr {
                            visit::visit_expr(visitor, &expr);
                        }
                    }
                }
            }
        }
        PassedArgs::None => (),
    }

    if let FieldMode::Calc(expr) | FieldMode::Function(expr) = &field.read_mode {
        visit!(expr.clone());
    }

    for assert in &field.assertions {
        visit!(assert.condition.clone());

        if let Some(AssertionError::Message(err) | AssertionError::Error(err)) =
            assert.consequent.clone()
        {
            visit!(err);
        }
    }

    for context_expr in &field.err_context {
        match context_expr {
            ErrContext::Context(expr) => visit!(expr.to_token_stream()),
            ErrContext::Format(fmt, exprs) => {
                visit!(fmt.to_token_stream());
                for expr in exprs {
                    visit!(expr.to_token_stream());
                }
            }
        }
    }
}

struct ArgList(Punctuated<FieldValue, syn::token::Comma>);

impl Parse for ArgList {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        Punctuated::parse_terminated(input).map(Self)
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_lit(&mut self, lit: &'ast syn::Lit) {
        let start = lit.span().start();
        let end = lit.span().end();

        // syntax highlighting for multi-line spans isn't supported yet (sorry)
        if start.line == end.line {
            let lines = self
                .syntax_info
                .lines
                .entry(start.line)
                .or_insert_with(LineSyntax::default);

            lines.highlights.push((
                start.column..end.column,
                match lit {
                    Lit::Str(_) | Lit::ByteStr(_) => Color::String,
                    Lit::Byte(_) | Lit::Char(_) => Color::Char,
                    Lit::Int(_) | Lit::Float(_) | Lit::Bool(_) => Color::Number,
                    Lit::Verbatim(_) => return,
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
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*call.func {
            if let Some(ident) = path.path.segments.last() {
                let ident = &ident.ident;
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
        visit::visit_expr_call(self, call);
    }

    fn visit_bin_op(&mut self, binop: &'ast syn::BinOp) {
        self.syntax_info
            .highlight_color(binop.span(), Color::Keyword);
    }

    fn visit_un_op(&mut self, unop: &'ast syn::UnOp) {
        self.syntax_info.highlight_color(unop.span(), Color::Unary);
    }

    fn visit_member(&mut self, member: &'ast syn::Member) {
        if let syn::Member::Unnamed(index) = member {
            self.syntax_info.highlight_color(index.span, Color::Number);
        }
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path.segments.len() > 1 {
            if let Some(first_segment) = path.segments.iter().next() {
                self.syntax_info
                    .highlight_color(first_segment.ident.span(), Color::Keyword);
            }
        }

        visit::visit_path(self, path);
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
        bw, calc, count, default, deref_now, ignore, import, import_raw, is_big, is_little,
        little, magic, map, offset, offset_after, pad_after, pad_before, pad_size_to, parse_with,
        postprocess_now, pre_assert, repr, restore_position, return_all_errors,
        return_unexpected_error, seek_before, temp, try_map, write_with
    );

    is_keyword
}

#[derive(Debug, Clone)]
struct FieldValue {
    ident: syn::Ident,
    expr: Option<syn::Expr>,
}

impl From<FieldValue> for (syn::Ident, Option<syn::Expr>) {
    fn from(x: FieldValue) -> Self {
        let FieldValue { ident, expr, .. } = x;

        (ident, expr)
    }
}

impl Parse for FieldValue {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        let expr = if input.lookahead1().peek(syn::Token![:]) {
            input.parse::<syn::Token![:]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self { ident, expr })
    }
}
