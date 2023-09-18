mod codegen;

use crate::meta_types::IdentTypeMaybeDefault;
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

pub(crate) fn derive_from_imports(
    ty_name: &Ident,
    is_write: bool,
    result_name: &Ident,
    vis: &Visibility,
    lifetime: Option<syn::Lifetime>,
    args: impl Iterator<Item = IdentTypeMaybeDefault>,
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
        generics: lifetime
            .map(|lifetime| [syn::GenericParam::Lifetime(syn::LifetimeDef::new(lifetime))])
            .as_ref()
            .map_or(&[], |generics| generics.as_slice()),
        vis,
    }
    .generate(true)
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub(crate) fn derive_from_input(input: DeriveInput) -> TokenStream {
    from_input(input).unwrap_or_else(syn::Error::into_compile_error)
}

fn from_input(input: DeriveInput) -> syn::Result<TokenStream> {
    if let syn::Data::Struct(s) = input.data {
        let mut has_try_optional = false;
        let fields = s
            .fields
            .into_iter()
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
                    name: match field.ident {
                        Some(ident) => ident,
                        None => {
                            return Err(syn::Error::new(
                                field.span(),
                                "tuple structs are not supported",
                            ))
                        }
                    },
                    ty: field.ty.clone(),
                })
            })
            .collect::<Result<Vec<_>, syn::Error>>()?;

        Ok(Builder {
            owner_name: None,
            is_write: false,
            result_name: &input.ident,
            builder_name: &quote::format_ident!("{}Builder", input.ident),
            fields: &fields,
            generics: &input.generics.params.iter().cloned().collect::<Vec<_>>(),
            vis: &input.vis,
        }
        .generate(false))
    } else {
        Err(syn::Error::new(input.span(), "only structs are supported"))
    }
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
#[cfg_attr(coverage_nightly, coverage(off))]
#[test]
fn derive_named_args_code_coverage_for_tool() {
    use runtime_macros_derive::emulate_derive_expansion_fallible;
    let file = std::fs::File::open("../binrw/tests/named_args.rs").unwrap();
    emulate_derive_expansion_fallible(file, "NamedArgs", |input| derive_from_input(input)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn try_input(input: TokenStream) {
        from_input(syn::parse2::<DeriveInput>(input).unwrap()).unwrap();
    }

    macro_rules! try_error (
        ($name:ident: $message:literal $tt:tt) => {
            #[test]
            #[cfg_attr(coverage_nightly, coverage(off))]
            #[should_panic(expected = $message)]
            fn $name() {
                try_input(quote::quote! $tt);
            }
        };
    );

    try_error!(invalid_attr_name: "expected `try_optional` or `default`" {
        struct Foo<A> {
            #[named_args(invalid)]
            a: A,
        }
    });

    try_error!(invalid_attr_syntax: "unexpected token" {
        struct Foo<A> {
            #[named_args(try_optional, invalid)]
            a: A,
        }
    });

    try_error!(invalid_enum: "only structs" {
        enum Foo {}
    });

    try_error!(invalid_tuple: "tuple structs are not supported" {
        struct Foo<A>(A);
    });

    try_error!(invalid_union: "only structs" {
        union Foo {}
    });

    try_error!(missing_default_eq_value: "expected `=`" {
        struct Foo<A> {
            #[named_args(default)]
            a: A,
        }
    });

    try_error!(missing_default_value: "expected expression" {
        struct Foo<A> {
            #[named_args(default = )]
            a: A,
        }
    });

    try_error!(multiple_try_optional: "more than one `try_optional`" {
        struct Foo<A, B> {
            #[named_args(try_optional)]
            a: A,
            #[named_args(try_optional)]
            b: B,
        }
    });
}
