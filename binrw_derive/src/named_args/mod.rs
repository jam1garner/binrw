use crate::codegen::typed_builder::{Builder, BuilderField, BuilderFieldKind};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    DeriveInput, Expr, Token,
};

pub(crate) enum NamedArgAttr {
    Default(Box<Expr>),
    TryOptional,
}

impl Parse for NamedArgAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::try_optional) {
            input.parse::<kw::try_optional>()?;
            Ok(NamedArgAttr::TryOptional)
        } else if lookahead.peek(kw::default) {
            input.parse::<kw::default>()?;
            input.parse::<Token![=]>()?;
            Ok(NamedArgAttr::Default(Box::new(input.parse()?)))
        } else {
            Err(lookahead.error())
        }
    }
}

pub(super) fn derive_from_attribute(input: DeriveInput) -> proc_macro2::TokenStream {
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
                        NamedArgAttr::TryOptional => {
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
        _ => return syn::Error::new(input.span(), "only structs are supported").to_compile_error(),
    } {
        Ok(fields) => fields,
        Err(err) => return err.into_compile_error(),
    };

    let generics: Vec<_> = input.generics.params.iter().cloned().collect();

    Builder {
        owner_name: None,
        is_write: false,
        result_name: &input.ident,
        builder_name: &quote::format_ident!("{}Builder", input.ident),
        fields: &fields,
        generics: &generics,
        vis: &input.vis,
    }
    .generate(false)
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
    let file = std::fs::File::open("../binrw/tests/builder.rs").unwrap();
    emulate_derive_expansion_fallible(file, "BinrwNamedArgs", |input| derive_from_attribute(input))
        .unwrap();
}
