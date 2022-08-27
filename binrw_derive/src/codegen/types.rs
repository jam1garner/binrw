use crate::codegen::sanitization::ENDIAN_ENUM;
use crate::parser::Endian;
use proc_macro2::TokenStream;
use quote::quote;

impl Endian {
    pub(crate) fn as_binrw_endian(self) -> TokenStream {
        match self {
            Self::Big => quote! { #ENDIAN_ENUM::Big },
            Self::Little => quote! { #ENDIAN_ENUM::Little },
        }
    }

    pub(crate) fn flipped(self) -> Self {
        match self {
            Self::Big => Self::Little,
            Self::Little => Self::Big,
        }
    }
}
