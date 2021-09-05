use proc_macro2::TokenStream;
use quote::quote;

#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::Endian;

impl Endian {
    pub(crate) fn as_binrw_endian(&self) -> TokenStream {
        match self {
            Self::Big => quote!{ #ENDIAN_ENUM::Big },
            Self::Little => quote!{ #ENDIAN_ENUM::Little },
        }
    }

    pub(crate) fn flipped(&self) -> Self {
        match self {
            Self::Big => Self::Little,
            Self::Little => Self::Big,
        }
    }
}
