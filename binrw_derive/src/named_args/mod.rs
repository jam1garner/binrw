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
                let attrs: Vec<NamedArgAttr> = field
                    .attrs
                    .iter()
                    .filter_map(|attr| {
                        let is_named_args = attr
                            .path
                            .get_ident()
                            .map_or(false, |ident| ident == "named_args");
                        if is_named_args {
                            attr.parse_args().ok()
                        } else {
                            None
                        }
                    })
                    .collect();
                let kind = if attrs
                    .iter()
                    .any(|attr| matches!(attr, NamedArgAttr::TryOptional))
                {
                    BuilderFieldKind::TryOptional
                } else if let Some(NamedArgAttr::Default(default)) = attrs
                    .iter()
                    .find(|attr| matches!(attr, NamedArgAttr::Default(_)))
                {
                    BuilderFieldKind::Optional {
                        default: default.clone(),
                    }
                } else {
                    BuilderFieldKind::Required
                };
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
