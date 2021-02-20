use crate::parser::attrs;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Lit;

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum Kind {
    Str,
    ByteStr,
    Byte,
    Char,
    Int(String),
    Float,
    Bool,
    Verbatim,
}

impl core::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Kind::Str => "string",
            Kind::ByteStr => "byte string",
            Kind::Byte => "byte",
            Kind::Char => "char",
            Kind::Int(suffix) => suffix,
            Kind::Float => "float",
            Kind::Bool => "bool",
            Kind::Verbatim => "raw token literal",
        })
    }
}

pub(crate) type Magic = Option<(Kind, TokenStream)>;

impl From<attrs::Magic> for (Kind, TokenStream) {
    fn from(magic: attrs::Magic) -> Self {
        let magic = &magic.value;
        (match &magic {
            Lit::Str(_) => Kind::Str,
            Lit::ByteStr(_) => Kind::ByteStr,
            Lit::Byte(_) => Kind::Byte,
            Lit::Char(_) => Kind::Char,
            Lit::Int(i) => Kind::Int(i.suffix().to_owned()),
            Lit::Float(_) => Kind::Float,
            Lit::Bool(_) => Kind::Bool,
            Lit::Verbatim(_) => Kind::Verbatim
        }, {
            if let Lit::Str(_) | Lit::ByteStr(_) = magic {
                quote::quote! {
                    *#magic
                }
            } else {
                magic.to_token_stream()
            }
        })
    }
}
