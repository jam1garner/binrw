use crate::{
    binrw::{
        codegen::sanitization::ENDIAN_ENUM,
        parser::{attrs, TrySet},
    },
    meta_types::KeywordToken,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Endian {
    Big,
    Little,
}

impl Endian {
    pub(crate) fn flipped(self) -> Self {
        match self {
            Self::Big => Self::Little,
            Self::Little => Self::Big,
        }
    }
}

impl ToTokens for Endian {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Endian::Big => tokens.append_all(quote! { #ENDIAN_ENUM::Big }),
            Endian::Little => tokens.append_all(quote! { #ENDIAN_ENUM::Little }),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum CondEndian {
    Inherited,
    Fixed(Endian),
    Cond(Endian, TokenStream),
}

impl Default for CondEndian {
    fn default() -> Self {
        Self::Inherited
    }
}

impl From<attrs::Big> for CondEndian {
    fn from(_: attrs::Big) -> Self {
        Self::Fixed(Endian::Big)
    }
}

impl From<attrs::Little> for CondEndian {
    fn from(_: attrs::Little) -> Self {
        Self::Fixed(Endian::Little)
    }
}

impl From<attrs::IsBig> for CondEndian {
    fn from(is_big: attrs::IsBig) -> Self {
        Self::Cond(Endian::Big, is_big.value.to_token_stream())
    }
}

impl From<attrs::IsLittle> for CondEndian {
    fn from(is_little: attrs::IsLittle) -> Self {
        Self::Cond(Endian::Little, is_little.value.to_token_stream())
    }
}

impl<T: Into<CondEndian> + KeywordToken> TrySet<CondEndian> for T {
    fn try_set(self, to: &mut CondEndian) -> syn::Result<()> {
        if matches!(*to, CondEndian::Inherited) {
            *to = self.into();
            Ok(())
        } else {
            Err(syn::Error::new(
                self.keyword_span(),
                "conflicting endianness keyword",
            ))
        }
    }
}
