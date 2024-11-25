use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self, Token},
    Expr, Ident, Lit, Token, Type,
};

pub(crate) trait KeywordToken {
    type Token: Token;

    fn display() -> &'static str {
        <Self::Token as Token>::display()
    }

    fn dyn_display(&self) -> &'static str {
        Self::display()
    }

    fn keyword_span(&self) -> Span;
}

type Fields<T> = Punctuated<T, Token![,]>;

/// `MetaExpr` represents a key/expr pair
/// Takes two forms:
/// * ident(expr)
/// * ident = expr
///
/// both are always allowed
pub(crate) type MetaExpr<Keyword> = MetaValue<Keyword, Expr>;

/// `MetaType` represents a key/ty pair
/// Takes two forms:
/// * ident(ty)
/// * ident = ty
///
/// both are always allowed
pub(crate) type MetaType<Keyword> = MetaValue<Keyword, Type>;

/// `MetaIdent` represents a key/ident pair
/// Takes two forms:
/// * ident(ident)
/// * ident = ident
///
/// both are always allowed
pub(crate) type MetaIdent<Keyword> = MetaValue<Keyword, Ident>;

/// `MetaLit` represents a key/lit pair
/// Takes two forms:
/// * ident(lit)
/// * ident = lit
///
/// both are always allowed
pub(crate) type MetaLit<Keyword> = MetaValue<Keyword, Lit>;

#[derive(Debug, Clone)]
pub(crate) struct MetaValue<Keyword, Value> {
    pub(crate) ident: Keyword,
    pub(crate) value: Value,
}

impl<Keyword: Token + Spanned> KeywordToken for MetaVoid<Keyword> {
    type Token = Keyword;

    fn keyword_span(&self) -> Span {
        self.ident.span()
    }
}

impl<Keyword: Parse, Value: Parse> Parse for MetaValue<Keyword, Value> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        let value = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            content.parse()?
        } else {
            input.parse::<Token![=]>()?;
            input.parse()?
        };

        Ok(MetaValue { ident, value })
    }
}

impl<Keyword, Value: ToTokens> From<MetaValue<Keyword, Value>> for TokenStream {
    fn from(value: MetaValue<Keyword, Value>) -> Self {
        value.value.into_token_stream()
    }
}

impl<Keyword> From<MetaValue<Keyword, Ident>> for Ident {
    fn from(value: MetaValue<Keyword, Ident>) -> Self {
        value.value
    }
}

impl<Keyword, Value: ToTokens> ToTokens for MetaValue<Keyword, Value> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.value.to_tokens(tokens);
    }
}

impl<Keyword: Token + Spanned, Value> KeywordToken for MetaValue<Keyword, Value> {
    type Token = Keyword;

    fn keyword_span(&self) -> Span {
        self.ident.span()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MetaVoid<Keyword> {
    pub(crate) ident: Keyword,
}

impl<Keyword: Parse> Parse for MetaVoid<Keyword> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(MetaVoid {
            ident: input.parse()?,
        })
    }
}

impl<Keyword> From<MetaVoid<Keyword>> for () {
    fn from(_: MetaVoid<Keyword>) -> Self {}
}

#[derive(Debug, Clone)]
pub(crate) struct MetaList<Keyword, ItemType> {
    pub(crate) ident: Keyword,
    pub(crate) fields: Fields<ItemType>,
}

impl<Keyword: Parse, ItemType: Parse> Parse for MetaList<Keyword, ItemType> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        let content;
        parenthesized!(content in input);
        Ok(MetaList {
            ident,
            fields: content.parse_terminated(ItemType::parse, Token![,])?,
        })
    }
}

impl<Keyword: Token + Spanned, ItemType> KeywordToken for MetaList<Keyword, ItemType> {
    type Token = Keyword;

    fn keyword_span(&self) -> Span {
        self.ident.span()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Enclosure<ParenType, BraceType> {
    Paren { fields: Fields<ParenType> },
    Brace { fields: Fields<BraceType> },
}

#[derive(Debug, Clone)]
pub(crate) struct MetaEnclosedList<Keyword, ParenItemType, BraceItemType> {
    pub(crate) ident: Keyword,
    pub(crate) list: Enclosure<ParenItemType, BraceItemType>,
}

impl<Keyword, ParenItemType, BraceItemType> Parse
    for MetaEnclosedList<Keyword, ParenItemType, BraceItemType>
where
    Keyword: Parse,
    ParenItemType: Parse,
    BraceItemType: Parse,
{
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        let content;
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Paren) {
            parenthesized!(content in input);
            Ok(Self {
                ident,
                list: Enclosure::Paren {
                    fields: content.parse_terminated(ParenItemType::parse, Token![,])?,
                },
            })
        } else if lookahead.peek(token::Brace) {
            braced!(content in input);
            Ok(Self {
                ident,
                list: Enclosure::Brace {
                    fields: content.parse_terminated(BraceItemType::parse, Token![,])?,
                },
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl<Keyword: Token + Spanned, ParenItemType, BraceItemType> KeywordToken
    for MetaEnclosedList<Keyword, ParenItemType, BraceItemType>
{
    type Token = Keyword;

    fn keyword_span(&self) -> Span {
        self.ident.span()
    }
}

// This is like `syn::PatType` except:
// (1) Implements `Parse`;
// (2) No attributes;
// (3) Only allows an ident on the LHS instead of any `syn::Pat`.
#[derive(Debug, Clone)]
pub(crate) struct IdentPatType {
    pub(crate) ident: syn::Ident,
    pub(crate) ty: syn::Type,
}

impl Parse for IdentPatType {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Self { ident, ty })
    }
}

// This is like `syn::PatType` except:
// (1) Implements `Parse`;
// (2) No attributes;
// (3) Only allows an ident on the LHS instead of any `syn::Pat`.
// (4) Optionally allows a `= $expr` following the type
#[derive(Debug, Clone)]
pub(crate) struct IdentTypeMaybeDefault {
    pub(crate) ident: syn::Ident,
    pub(crate) ty: syn::Type,
    pub(crate) default: Option<Box<syn::Expr>>,
}

impl Parse for IdentTypeMaybeDefault {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let default = if input.lookahead1().peek(Token![=]) {
            input.parse::<Token![=]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self { ident, ty, default })
    }
}

pub(crate) struct MetaAttrList<P>(Fields<P>);

impl<P> MetaAttrList<P> {
    pub(crate) fn into_iter(self) -> impl Iterator<Item = P> {
        self.0.into_iter()
    }
}

impl<P: Parse> Parse for MetaAttrList<P> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(MetaAttrList(Fields::parse_terminated(input)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod kw {
        syn::custom_keyword!(test);
        syn::custom_keyword!(test_list);
        syn::custom_keyword!(test_enclosed_list);
    }

    type MetaValueTest = MetaValue<kw::test, Lit>;
    type MetaListTest = MetaList<kw::test_list, Lit>;
    type MetaAttrListTest = MetaAttrList<Lit>;
    type MetaEnclosedListTest = MetaEnclosedList<kw::test_enclosed_list, Lit, Lit>;

    macro_rules! try_parse {
        ($name:ident, $ty:ty, $tt:tt) => {
            #[test]
            #[cfg_attr(coverage_nightly, coverage(off))]
            fn $name() {
                syn::parse2::<$ty>(quote::quote! $tt).unwrap();
            }
        }
    }

    macro_rules! try_parse_fail {
        ($name:ident, $message:literal, $ty:ty, $tt:tt) => {
            #[test]
            #[cfg_attr(coverage_nightly, coverage(off))]
            #[should_panic = $message]
            fn $name() {
                syn::parse2::<$ty>(quote::quote! $tt).unwrap();
            }
        }
    }

    try_parse!(meta_keyword, kw::test, { test });

    try_parse!(meta_value_assign, MetaValueTest, { test = 3u8 });
    try_parse!(meta_value_paren, MetaValueTest, { test(b"TEST") });
    try_parse_fail!(meta_value_missing_keyword, "expected `test`", MetaValueTest, { = 3u8 });
    try_parse_fail!(meta_value_missing_value, "expected `=`", MetaValueTest, {
        test
    });
    try_parse_fail!(
        meta_value_wrong_keyword,
        "expected `test`",
        MetaValueTest,
        { wrong = 3u8 }
    );
    try_parse_fail!(
        meta_value_wrong_value_type,
        "expected literal",
        MetaValueTest,
        { test = u8 }
    );
    try_parse_fail!(
        meta_value_confused_as_list,
        "unexpected token",
        MetaValueTest,
        { test(3u8, 3u8) }
    );

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn meta_value_into_tokenstream() {
        let expected = quote::quote! { 0u8 };
        let value = syn::parse2::<MetaValueTest>(quote::quote! { test = #expected }).unwrap();
        assert_eq!(expected.to_string(), TokenStream::from(value).to_string());
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn meta_value_to_tokens() {
        let expected = quote::quote! { 0u8 };
        let value = syn::parse2::<MetaValueTest>(quote::quote! { test = #expected }).unwrap();
        let mut actual = TokenStream::new();
        value.to_tokens(&mut actual);
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn meta_value_keyword_token() {
        use syn::spanned::Spanned;
        let keyword = quote::quote! { test };
        let value = syn::parse2::<MetaValueTest>(quote::quote! { #keyword = 0u8 }).unwrap();
        assert_eq!(
            format!("{:?}", keyword.span()),
            format!("{:?}", value.keyword_span())
        );
    }

    try_parse!(meta_list, MetaListTest, { test_list(3u8, 3u8) });
    try_parse!(meta_list_empty, MetaListTest, { test_list() });
    try_parse_fail!(
        meta_list_missing_keyword,
        "expected `test_list`",
        MetaListTest,
        { (3u8, 3u8) }
    );
    try_parse_fail!(
        meta_list_missing_value,
        "unexpected end of input",
        MetaListTest,
        { test_list }
    );
    try_parse_fail!(
        meta_list_wrong_delimiter,
        "expected parentheses",
        MetaListTest,
        { test_list = (3u8, 3u8) }
    );
    try_parse_fail!(
        meta_list_wrong_keyword,
        "expected `test_list`",
        MetaListTest,
        { wrong }
    );
    try_parse_fail!(
        meta_list_wrong_item_type,
        "expected literal",
        MetaListTest,
        { test_list(i32) }
    );

    try_parse!(meta_enclosed_list_paren, MetaEnclosedListTest, {
        test_enclosed_list(3u8, 3u8)
    });
    try_parse!(meta_enclosed_list_paren_empty, MetaEnclosedListTest, {
        test_enclosed_list()
    });
    try_parse!(meta_enclosed_list_brace, MetaEnclosedListTest, { test_enclosed_list { 3u8, 3u8 } });
    try_parse!(meta_enclosed_list_brace_empty, MetaEnclosedListTest, {
        test_enclosed_list {}
    });
    try_parse_fail!(
        meta_enclosed_list_wrong_keyword,
        "expected `test_enclosed_list`",
        MetaEnclosedListTest,
        { wrong }
    );
    try_parse_fail!(
        meta_enclosed_list_wrong_delimiter,
        "expected parentheses or curly braces",
        MetaEnclosedListTest,
        { test_enclosed_list = (3u8, 3u8) }
    );
    try_parse_fail!(
        meta_enclosed_list_wrong_bracket_kind,
        "expected parentheses or curly braces",
        MetaEnclosedListTest,
        { test_enclosed_list [] }
    );
    try_parse_fail!(
        meta_enclosed_list_wrong_item_type,
        "expected literal",
        MetaEnclosedListTest,
        { test_enclosed_list(i32) }
    );

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn meta_list_keyword_token() {
        use syn::spanned::Spanned;
        let keyword = quote::quote! { test_list };
        let value = syn::parse2::<MetaListTest>(quote::quote! { #keyword(0u8, 0u8) }).unwrap();
        assert_eq!(
            format!("{:?}", keyword.span()),
            format!("{:?}", value.keyword_span())
        );
    }

    try_parse!(ident_pat_type, IdentPatType, { foo: u8 });
    try_parse_fail!(ident_pat_type_missing_ident, "expected identifier", IdentPatType, { : 3u8 });
    try_parse_fail!(ident_pat_type_missing_ty, "unexpected end of input", IdentPatType, { foo: });
    try_parse_fail!(ident_pat_type_wrong_ty_type, "expected one of", IdentPatType, { foo: 3u8 });

    try_parse!(ident_type_default, IdentTypeMaybeDefault, { foo: u8 = 1 });
    try_parse!(ident_type_no_default, IdentTypeMaybeDefault, { foo: u8 });
    try_parse_fail!(ident_type_missing_type, "unexpected end of input", IdentTypeMaybeDefault, { foo: });
    try_parse_fail!(ident_type_missing_colon, "expected `:`", IdentTypeMaybeDefault, { foo u8 });
    try_parse_fail!(ident_type_missing_ident, "expected identifier", IdentTypeMaybeDefault, { :u8 });

    try_parse!(meta_attr_list, MetaAttrListTest, { 1u8, 2u8, 3u8 });
    try_parse!(meta_attr_list_empty, MetaAttrListTest, {});
    try_parse_fail!(
        meta_attr_list_wrong_type,
        "expected literal",
        MetaAttrListTest,
        { i32 }
    );

    #[test]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn meta_attr_list_into_iter() {
        let expected = [
            Lit::new(proc_macro2::Literal::u8_suffixed(1)),
            Lit::new(proc_macro2::Literal::u8_suffixed(2)),
            Lit::new(proc_macro2::Literal::u8_suffixed(3)),
        ];

        let value = syn::parse2::<MetaAttrListTest>(quote::quote! { 1u8, 2u8, 3u8 }).unwrap();
        assert_eq!(expected, value.into_iter().collect::<Vec<_>>()[..]);
    }
}
