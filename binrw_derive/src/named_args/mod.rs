mod codegen;

use crate::parser::IdentTypeMaybeDefault;
use codegen::{Builder, BuilderField, BuilderFieldKind};
use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    DeriveInput, Expr, Ident, Token, Visibility,
};

pub(crate) fn arg_type_name(ty_name: &Ident, is_write: bool) -> Ident {
    if is_write {
        format_ident!("{}BinWriteArgs", ty_name, span = Span::mixed_site())
    } else {
        format_ident!("{}BinReadArgs", ty_name, span = Span::mixed_site())
    }
}

pub(crate) fn derive_from_attribute(input: DeriveInput) -> proc_macro2::TokenStream {
    let mut has_try_optional = false;
    let fields = match match input.data {
        syn::Data::Struct(s) => s
            .fields
            .iter()
            .map(|field| {
                let attrs = field.attrs.iter().filter_map(|attr| {
                    attr.path
                        .get_ident()
                        .filter(|ident| *ident == "named_args")
                        .map(|_| attr.parse_args::<NamedArgAttr>())
                });

                let mut kind = BuilderFieldKind::Required;
                for attr in attrs {
                    match attr? {
                        NamedArgAttr::Default(default) => {
                            kind = BuilderFieldKind::Optional { default }
                        }
                        NamedArgAttr::TryOptional(span) => {
                            if has_try_optional {
                                return Err(syn::Error::new(
                                    span,
                                    "cannot have more than one `try_optional` per struct",
                                ));
                            }
                            has_try_optional = true;
                            kind = BuilderFieldKind::TryOptional;
                            break;
                        }
                    }
                }

                Ok(BuilderField {
                    kind,
                    name: match field.ident.as_ref() {
                        Some(ident) => ident.clone(),
                        None => {
                            return Err(syn::Error::new(
                                field.span(),
                                "must not be a tuple-style field",
                            ))
                        }
                    },
                    ty: field.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, syn::Error>>(),
        _ => Err(syn::Error::new(input.span(), "only structs are supported")),
    } {
        Ok(fields) => fields,
        Err(err) => return err.into_compile_error(),
    };

    Builder {
        owner_name: None,
        is_write: false,
        result_name: &input.ident,
        builder_name: &quote::format_ident!("{}Builder", input.ident),
        fields: &fields,
        generics: &input.generics.params.iter().cloned().collect::<Vec<_>>(),
        vis: &input.vis,
    }
    .generate(false)
}

pub(crate) fn derive_from_imports<'a>(
    ty_name: &Ident,
    is_write: bool,
    result_name: &Ident,
    vis: &Visibility,
    args: impl Iterator<Item = &'a IdentTypeMaybeDefault>,
) -> TokenStream {
    let builder_name = &if is_write {
        format_ident!("{}BinWriteArgBuilder", ty_name, span = Span::mixed_site())
    } else {
        format_ident!("{}BinReadArgBuilder", ty_name, span = Span::mixed_site())
    };

    Builder {
        owner_name: Some(ty_name),
        is_write,
        builder_name,
        result_name,
        fields: &args.map(Into::into).collect::<Vec<_>>(),
        generics: &[],
        vis,
    }
    .generate(true)
}

enum NamedArgAttr {
    Default(Box<Expr>),
    TryOptional(Span),
}

impl Parse for NamedArgAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::try_optional) {
            Ok(NamedArgAttr::TryOptional(
                input.parse::<kw::try_optional>()?.span(),
            ))
        } else if lookahead.peek(kw::default) {
            input.parse::<kw::default>()?;
            input.parse::<Token![=]>()?;
            Ok(NamedArgAttr::Default(Box::new(input.parse()?)))
        } else {
            Err(lookahead.error())
        }
    }
}

mod kw {
    syn::custom_keyword!(default);
    syn::custom_keyword!(try_optional);
}

#[cfg(coverage)]
#[cfg_attr(coverage_nightly, no_coverage)]
#[test]
fn derive_named_args_code_coverage_for_tool() {
    use runtime_macros_derive::emulate_derive_expansion_fallible;
    let file = std::fs::File::open("../binrw/tests/named_args.rs").unwrap();
    emulate_derive_expansion_fallible(file, "NamedArgs", |input| derive_from_attribute(input))
        .unwrap();
}
