use super::KeywordToken;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Expr, Lit, Token, Type,
};

type Fields<T> = Punctuated<T, Token![,]>;

/// `MetaExpr` represents a key/expr pair
/// Takes two forms:
/// * ident(expr)
/// * ident = expr
/// both are always allowed
pub(crate) type MetaExpr<Keyword> = MetaValue<Keyword, Expr>;

/// `MetaType` represents a key/ty pair
/// Takes two forms:
/// * ident(ty)
/// * ident = ty
/// both are always allowed
pub(crate) type MetaType<Keyword> = MetaValue<Keyword, Type>;

/// `MetaLit` represents a key/lit pair
/// Takes two forms:
/// * ident(lit)
/// * ident = lit
/// both are always allowed
pub(crate) type MetaLit<Keyword> = MetaValue<Keyword, Lit>;

#[derive(Debug, Clone)]
pub(crate) struct MetaValue<Keyword, Value> {
    pub(crate) ident: Keyword,
    pub(crate) value: Value,
}

impl<Keyword: syn::token::Token + syn::spanned::Spanned> KeywordToken for MetaVoid<Keyword> {
    type Token = Keyword;

    fn keyword_span(&self) -> proc_macro2::Span {
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

impl<Keyword, Value: ToTokens> ToTokens for MetaValue<Keyword, Value> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.value.to_tokens(tokens);
    }
}

impl<Keyword: syn::token::Token + syn::spanned::Spanned, Value> KeywordToken
    for MetaValue<Keyword, Value>
{
    type Token = Keyword;

    fn keyword_span(&self) -> proc_macro2::Span {
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
    pub(crate) parens: token::Paren,
    pub(crate) fields: Fields<ItemType>,
}

impl<Keyword: Parse, ItemType: Parse> Parse for MetaList<Keyword, ItemType> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        let content;
        let parens = parenthesized!(content in input);
        Ok(MetaList {
            ident,
            parens,
            fields: content.parse_terminated::<_, Token![,]>(ItemType::parse)?,
        })
    }
}

impl<Keyword: syn::token::Token + syn::spanned::Spanned, ItemType> KeywordToken
    for MetaList<Keyword, ItemType>
{
    type Token = Keyword;

    fn keyword_span(&self) -> proc_macro2::Span {
        self.ident.span()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Enclosure<ParenType, BraceType> {
    Paren {
        parens: token::Paren,
        fields: Fields<ParenType>,
    },
    Brace {
        braces: token::Brace,
        fields: Fields<BraceType>,
    },
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
            let parens = parenthesized!(content in input);
            Ok(Self {
                ident,
                list: Enclosure::Paren {
                    parens,
                    fields: content.parse_terminated::<_, Token![,]>(ParenItemType::parse)?,
                },
            })
        } else if lookahead.peek(token::Brace) {
            let braces = braced!(content in input);
            Ok(Self {
                ident,
                list: Enclosure::Brace {
                    braces,
                    fields: content.parse_terminated::<_, Token![,]>(BraceItemType::parse)?,
                },
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl<Keyword: syn::token::Token + syn::spanned::Spanned, ParenItemType, BraceItemType> KeywordToken
    for MetaEnclosedList<Keyword, ParenItemType, BraceItemType>
{
    type Token = Keyword;

    fn keyword_span(&self) -> proc_macro2::Span {
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
    pub(crate) colon_token: Token![:],
    pub(crate) ty: syn::Type,
}

impl Parse for IdentPatType {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
        })
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
    pub(crate) colon_token: Token![:],
    pub(crate) ty: syn::Type,
    pub(crate) eq: Option<Token![=]>,
    pub(crate) default: Option<Box<syn::Expr>>,
}

impl Parse for IdentTypeMaybeDefault {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        let colon_token = input.parse()?;
        let ty = input.parse()?;
        let lookahead = input.lookahead1();
        let (eq, default) = if lookahead.peek(Token![=]) {
            (Some(input.parse()?), Some(input.parse()?))
        } else {
            (None, None)
        };

        Ok(Self {
            ident,
            colon_token,
            ty,
            eq,
            default,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FieldValue {
    pub(crate) ident: syn::Ident,
    pub(crate) colon_token: Option<Token![:]>,
    pub(crate) expr: Option<syn::Expr>,
}

impl From<FieldValue> for (syn::Ident, Option<syn::Expr>) {
    fn from(x: FieldValue) -> Self {
        let FieldValue { ident, expr, .. } = x;

        (ident, expr)
    }
}

impl Parse for FieldValue {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;
        let lookahead = input.lookahead1();

        let expr;
        let colon_token;

        if lookahead.peek(Token![:]) {
            colon_token = Some(input.parse()?);
            expr = Some(input.parse()?);
        } else {
            colon_token = None;
            expr = None;
        }

        Ok(Self {
            ident,
            colon_token,
            expr,
        })
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
        let content;
        parenthesized!(content in input);
        Ok(MetaAttrList(Fields::parse_terminated(&content)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod kw {
        syn::custom_keyword!(test);
        syn::custom_keyword!(test_list);
    }

    type MetaValueTest = MetaValue<kw::test, Lit>;
    type MetaListTest = MetaList<kw::test_list, Lit>;
    type MetaAttrListTest = MetaAttrList<Lit>;

    macro_rules! try_parse {
        ($name:ident, $ty:ty, $tt:tt) => {
            #[test]
            fn $name() {
                syn::parse2::<$ty>(quote::quote! $tt).unwrap();
            }
        }
    }

    macro_rules! try_parse_fail {
        ($name:ident, $ty:ty, $tt:tt) => {
            #[test]
            #[should_panic]
            fn $name() {
                syn::parse2::<$ty>(quote::quote! $tt).unwrap();
            }
        }
    }

    try_parse!(meta_keyword, kw::test, { test });

    try_parse!(meta_value_assign, MetaValueTest, { test = 3u8 });
    try_parse!(meta_value_paren, MetaValueTest, { test(b"TEST") });
    try_parse_fail!(meta_value_missing_keyword, MetaValueTest, { = 3u8 });
    try_parse_fail!(meta_value_missing_value, MetaValueTest, { test });
    try_parse_fail!(meta_value_wrong_keyword, MetaValueTest, { wrong = 3u8 });
    try_parse_fail!(meta_value_wrong_value_type, MetaValueTest, { test = u8 });
    try_parse_fail!(meta_value_confused_as_list, MetaValueTest, {
        test(3u8, 3u8)
    });

    #[test]
    fn meta_value_into_tokenstream() {
        let expected = quote::quote! { 0u8 };
        let value = syn::parse2::<MetaValueTest>(quote::quote! { test = #expected }).unwrap();
        assert_eq!(expected.to_string(), TokenStream::from(value).to_string());
    }

    #[test]
    fn meta_value_to_tokens() {
        let expected = quote::quote! { 0u8 };
        let value = syn::parse2::<MetaValueTest>(quote::quote! { test = #expected }).unwrap();
        let mut actual = TokenStream::new();
        value.to_tokens(&mut actual);
        assert_eq!(expected.to_string(), actual.to_string());
    }

    #[test]
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
    try_parse_fail!(meta_list_missing_keyword, MetaListTest, { (3u8, 3u8) });
    try_parse_fail!(meta_list_missing_value, MetaListTest, { test_list });
    try_parse_fail!(meta_list_wrong_delimiter, MetaListTest, {
        test_list = (3u8, 3u8)
    });
    try_parse_fail!(meta_list_wrong_keyword, MetaListTest, { wrong });
    try_parse_fail!(meta_list_wrong_item_type, MetaListTest, { test_list(i32) });

    #[test]
    fn meta_list_keyword_token() {
        use syn::spanned::Spanned;
        let keyword = quote::quote! { test_list };
        let value = syn::parse2::<MetaListTest>(quote::quote! { #keyword(0u8, 0u8) }).unwrap();
        assert_eq!(
            format!("{:?}", keyword.span()),
            format!("{:?}", value.keyword_span())
        );
    }

    try_parse!(ident_type_default, IdentTypeMaybeDefault, { foo: u8 = 1 });
    try_parse!(ident_type_no_default, IdentTypeMaybeDefault, { foo: u8 });
    try_parse_fail!(ident_type_missing_type, IdentTypeMaybeDefault, { foo: });
    try_parse_fail!(ident_type_missing_colon, IdentTypeMaybeDefault, { foo u8 });
    try_parse_fail!(ident_type_missing_ident, IdentTypeMaybeDefault, { :u8 });

    try_parse!(ident_pat_type, IdentPatType, { foo: u8 });
    try_parse_fail!(ident_pat_type_missing_ident, IdentPatType, { : 3u8 });
    try_parse_fail!(ident_pat_type_missing_ty, IdentPatType, { foo: });
    try_parse_fail!(ident_pat_type_wrong_ty_type, IdentPatType, { foo: 3u8 });

    try_parse!(meta_attr_list, MetaAttrListTest, { (1u8, 2u8, 3u8) });
    try_parse!(meta_attr_list_empty, MetaAttrListTest, { () });
    try_parse_fail!(meta_attr_list_wrong_type, MetaAttrListTest, { (i32) });
    try_parse_fail!(meta_attr_list_confused_as_list, MetaAttrListTest, {
        wrong(i32)
    });

    #[test]
    fn meta_attr_list_into_iter() {
        let expected = [
            Lit::new(proc_macro2::Literal::u8_suffixed(1)),
            Lit::new(proc_macro2::Literal::u8_suffixed(2)),
            Lit::new(proc_macro2::Literal::u8_suffixed(3)),
        ];

        let value = syn::parse2::<MetaAttrListTest>(quote::quote! { (1u8, 2u8, 3u8) }).unwrap();
        assert_eq!(expected, value.into_iter().collect::<Vec<_>>()[..]);
    }
}
