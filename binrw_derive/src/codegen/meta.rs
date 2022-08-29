use super::sanitization::{READ_ENDIAN, READ_MAGIC, WRITE_ENDIAN, WRITE_MAGIC};
use crate::{
    codegen::sanitization::META_ENDIAN_KIND,
    parser::{Input, Map},
};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn generate<const WRITE: bool>(
    input: &Input,
    derive_input: &syn::DeriveInput,
) -> TokenStream {
    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();

    let magic = input.magic().as_ref().map(|magic| {
        let magic_meta = if WRITE { WRITE_MAGIC } else { READ_MAGIC };
        let ty = TokenStream::from(magic.kind());
        let val = magic.deref_value();
        quote! {
            impl #impl_generics #magic_meta for #name #ty_generics #where_clause {
                type MagicType = #ty;
                const MAGIC: Self::MagicType = #val;
            }
        }
    });

    let endian_meta = if WRITE { WRITE_ENDIAN } else { READ_ENDIAN };

    let endian = match input.endian() {
        crate::parser::CondEndian::Inherited => match input.map() {
            Map::None => input.is_empty().then(|| {
                quote! {
                    #META_ENDIAN_KIND::None
                }
            }),
            Map::Map(_) | Map::Try(_) => Some(quote! {
                #META_ENDIAN_KIND::None
            }),
            Map::Repr(repr) => ["i8", "u8"].contains(&repr.to_string().as_str()).then(|| {
                quote! { <(#repr) as #endian_meta>::ENDIAN }
            }),
        },
        crate::parser::CondEndian::Fixed(endian) => Some(quote! {
            #META_ENDIAN_KIND::Endian(#endian)
        }),
        crate::parser::CondEndian::Cond(..) => Some(quote! {
            #META_ENDIAN_KIND::Runtime
        }),
    }
    .map(|endian| {
        quote! {
            impl #impl_generics #endian_meta for #name #ty_generics #where_clause {
                const ENDIAN: #META_ENDIAN_KIND = #endian;
            }
        }
    });

    quote! {
        #magic
        #endian
    }
}
