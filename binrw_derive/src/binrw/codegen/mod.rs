mod meta;
mod read_options;
pub(crate) mod sanitization;
mod write_options;

use std::collections::HashSet;

use crate::{
    binrw::parser::{
        Assert, AssertionError, CondEndian, Imports, Input, ParseResult, PassedArgs, Struct,
        StructField,
    },
    named_args::{arg_type_name, derive_from_imports},
    util::{quote_spanned_any, IdentStr},
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use sanitization::{
    ARGS, ARGS_LIFETIME, ARGS_MACRO, ASSERT, ASSERT_ERROR_FN, BINREAD_TRAIT, BINWRITE_TRAIT,
    BIN_ERROR, BIN_RESULT, ENDIAN_ENUM, OPT, POS, READER, READ_TRAIT, SEEK_TRAIT, TEMP, WRITER,
    WRITE_TRAIT,
};
use syn::{parse_quote, spanned::Spanned, DeriveInput, Ident, Type};

use super::parser::{EnumVariant, FieldMode};

pub(crate) fn generate_impl<const WRITE: bool>(
    derive_input: &DeriveInput,
    binrw_input: &ParseResult<Input>,
) -> TokenStream {
    let (arg_type, arg_type_declaration) = match binrw_input {
        ParseResult::Ok(binrw_input) | ParseResult::Partial(binrw_input, _) => generate_imports(
            binrw_input.imports(),
            &derive_input.ident,
            &derive_input.vis,
            WRITE,
        ),
        ParseResult::Err(_) => (quote! { () }, None),
    };

    let trait_impl = generate_trait_impl::<WRITE>(binrw_input, derive_input, &arg_type);

    let meta_impls = match binrw_input {
        ParseResult::Ok(binrw_input) | ParseResult::Partial(binrw_input, _) => {
            Some(meta::generate::<WRITE>(binrw_input, derive_input))
        }
        ParseResult::Err(_) => None,
    };

    quote! {
        #trait_impl
        #meta_impls
        #arg_type_declaration
    }
}

fn generate_imports(
    imports: &Imports,
    type_name: &Ident,
    ty_vis: &syn::Visibility,
    is_write: bool,
) -> (TokenStream, Option<TokenStream>) {
    use syn::fold::Fold;

    fn has_elided_lifetime(ty: &syn::Type) -> bool {
        use syn::visit::Visit;
        struct Finder(bool);
        impl Visit<'_> for Finder {
            fn visit_lifetime(&mut self, i: &syn::Lifetime) {
                self.0 |= i.ident == "_";
            }

            fn visit_type_reference(&mut self, i: &syn::TypeReference) {
                self.0 |= i.lifetime.is_none();
            }
        }
        let mut finder = Finder(false);
        finder.visit_type(ty);
        finder.0
    }

    struct ExpandLifetimes;
    impl Fold for ExpandLifetimes {
        fn fold_lifetime(&mut self, mut i: syn::Lifetime) -> syn::Lifetime {
            if i.ident == "_" {
                i.ident = syn::Ident::new(ARGS_LIFETIME, i.ident.span());
            }
            i
        }

        fn fold_type_reference(&mut self, mut i: syn::TypeReference) -> syn::TypeReference {
            if i.lifetime.is_none()
                || matches!(&i.lifetime, Some(lifetime) if lifetime.ident == "_")
            {
                i.lifetime = Some(get_args_lifetime(i.and_token.span()));
            }
            i.elem = Box::new(ExpandLifetimes.fold_type(*i.elem));
            i
        }
    }

    match imports {
        Imports::None => (quote! { () }, None),
        Imports::List(_, types) => {
            let types = types.iter().map(|ty| ExpandLifetimes.fold_type(ty.clone()));
            (quote! { (#(#types,)*) }, None)
        }
        Imports::Raw(_, ty) => (
            ExpandLifetimes
                .fold_type(ty.as_ref().clone())
                .into_token_stream(),
            None,
        ),
        Imports::Named(args) => {
            let name = arg_type_name(type_name, is_write);
            let lifetime = args
                .iter()
                .any(|arg| has_elided_lifetime(&arg.ty))
                .then(|| get_args_lifetime(type_name.span()));
            let defs = derive_from_imports(
                type_name,
                is_write,
                &name,
                ty_vis,
                lifetime.clone(),
                args.iter().map(|arg| {
                    let mut arg = arg.clone();
                    arg.ty = ExpandLifetimes.fold_type(arg.ty);
                    arg
                }),
            );
            (
                if let Some(lifetime) = lifetime {
                    quote_spanned! { type_name.span()=> #name<#lifetime> }
                } else {
                    name.into_token_stream()
                },
                Some(defs),
            )
        }
    }
}

fn generate_trait_impl<const WRITE: bool>(
    binrw_input: &ParseResult<Input>,
    derive_input: &DeriveInput,
    arg_type: &TokenStream,
) -> TokenStream {
    let (trait_name, fn_sig) = if WRITE {
        (
            BINWRITE_TRAIT,
            quote! {
                fn write_options<W: #WRITE_TRAIT + #SEEK_TRAIT>(
                    &self,
                    #WRITER: &mut W,
                    #OPT: #ENDIAN_ENUM,
                    #ARGS: Self::Args<'_>
                ) -> #BIN_RESULT<()>
            },
        )
    } else {
        (
            BINREAD_TRAIT,
            quote! {
                fn read_options<R: #READ_TRAIT + #SEEK_TRAIT>
                    (#READER: &mut R, #OPT: #ENDIAN_ENUM, #ARGS: Self::Args<'_>)
                    -> #BIN_RESULT<Self>
            },
        )
    };

    let (fn_impl, generics) = match binrw_input {
        ParseResult::Ok(binrw_input) => (
            if WRITE {
                write_options::generate(binrw_input, derive_input)
            } else {
                read_options::generate(binrw_input, derive_input)
            },
            get_generics::<WRITE>(binrw_input, &derive_input.generics),
        ),
        // If there is a parsing error, an impl for the trait still needs to be
        // generated to avoid misleading errors at all call sites that use the
        // trait, so emit the trait and just stick the errors inside the generated
        // function
        ParseResult::Partial(_, error) | ParseResult::Err(error) => {
            (error.to_compile_error(), derive_input.generics.clone())
        }
    };

    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let args_lifetime = get_args_lifetime(Span::call_site());
    quote! {
        #[automatically_derived]
        #[allow(non_snake_case, unknown_lints)]
        #[allow(clippy::redundant_closure_call)]
        impl #impl_generics #trait_name for #name #ty_generics #where_clause {
            type Args<#args_lifetime> = #arg_type;

            #fn_sig {
                #fn_impl
            }
        }
    }
}

fn get_generics<const WRITE: bool>(binrw_input: &Input, generics: &syn::Generics) -> syn::Generics {
    match binrw_input.bound() {
        None => get_inferred_generics::<WRITE>(binrw_input, generics),
        Some(bound) => {
            let mut generics = generics.clone();
            generics
                .make_where_clause()
                .predicates
                .extend(bound.predicates().iter().cloned());
            generics
        }
    }
}

#[allow(clippy::too_many_lines)]
fn get_inferred_generics<const WRITE: bool>(
    binrw_input: &Input,
    generics: &syn::Generics,
) -> syn::Generics {
    // AST visitor adapted from serde
    // https://github.com/serde-rs/serde/blob/b9de3658ad9ca7850c496e0f990d5241b943eefb/serde_derive/src/bound.rs#L91
    struct FindTyParams<'ast> {
        all_type_params: HashSet<syn::Ident>,
        relevant_type_params: HashSet<syn::Ident>,
        associated_type_usage: Vec<&'ast syn::TypePath>,
    }

    pub fn ungroup(mut ty: &Type) -> &Type {
        while let Type::Group(group) = ty {
            ty = &group.elem;
        }
        ty
    }

    impl<'ast> FindTyParams<'ast> {
        fn new(generics: &syn::Generics) -> Self {
            Self {
                all_type_params: generics
                    .type_params()
                    .map(|param| param.ident.clone())
                    .collect(),
                relevant_type_params: HashSet::new(),
                associated_type_usage: Vec::new(),
            }
        }

        fn visit_field(&mut self, field: &'ast syn::Field) {
            if let syn::Type::Path(ty) = ungroup(&field.ty) {
                if let Some(syn::punctuated::Pair::Punctuated(t, _)) =
                    ty.path.segments.pairs().next()
                {
                    if self.all_type_params.contains(&t.ident) {
                        self.associated_type_usage.push(ty);
                    }
                }
            }
            self.visit_type(&field.ty);
        }

        fn visit_path(&mut self, path: &'ast syn::Path) {
            if let Some(seg) = path.segments.last() {
                // We know PhantomData<T> will always impl BinRead/BinWrite so
                // we don't count this T as a relevant type param.
                if seg.ident == "PhantomData" {
                    return;
                }
            }
            if path.leading_colon.is_none() && path.segments.len() == 1 {
                let id = &path.segments[0].ident;
                if self.all_type_params.contains(id) {
                    self.relevant_type_params.insert(id.clone());
                }
            }
            for segment in &path.segments {
                self.visit_path_segment(segment);
            }
        }

        fn visit_type(&mut self, ty: &'ast syn::Type) {
            match ty {
                syn::Type::Array(ty) => self.visit_type(&ty.elem),
                syn::Type::BareFn(ty) => {
                    for arg in &ty.inputs {
                        self.visit_type(&arg.ty);
                    }
                    self.visit_return_type(&ty.output);
                }
                syn::Type::Group(ty) => self.visit_type(&ty.elem),
                syn::Type::ImplTrait(ty) => {
                    for bound in &ty.bounds {
                        self.visit_type_param_bound(bound);
                    }
                }
                syn::Type::Paren(ty) => self.visit_type(&ty.elem),
                syn::Type::Path(ty) => {
                    if let Some(qself) = &ty.qself {
                        self.visit_type(&qself.ty);
                    }
                    self.visit_path(&ty.path);
                }
                syn::Type::Ptr(ty) => self.visit_type(&ty.elem),
                syn::Type::Reference(ty) => self.visit_type(&ty.elem),
                syn::Type::Slice(ty) => self.visit_type(&ty.elem),
                syn::Type::TraitObject(ty) => {
                    for bound in &ty.bounds {
                        self.visit_type_param_bound(bound);
                    }
                }
                syn::Type::Tuple(ty) => {
                    for elem in &ty.elems {
                        self.visit_type(elem);
                    }
                }
                _ => {}
            }
        }

        fn visit_path_segment(&mut self, segment: &'ast syn::PathSegment) {
            self.visit_path_arguments(&segment.arguments);
        }

        fn visit_path_arguments(&mut self, arguments: &'ast syn::PathArguments) {
            match arguments {
                syn::PathArguments::None => {}
                syn::PathArguments::AngleBracketed(arguments) => {
                    for arg in &arguments.args {
                        match arg {
                            syn::GenericArgument::Type(arg) => self.visit_type(arg),
                            syn::GenericArgument::AssocType(arg) => self.visit_type(&arg.ty),
                            _ => {}
                        }
                    }
                }
                syn::PathArguments::Parenthesized(arguments) => {
                    for argument in &arguments.inputs {
                        self.visit_type(argument);
                    }
                    self.visit_return_type(&arguments.output);
                }
            }
        }

        fn visit_return_type(&mut self, return_type: &'ast syn::ReturnType) {
            match return_type {
                syn::ReturnType::Default => {}
                syn::ReturnType::Type(_, output) => self.visit_type(output),
            }
        }

        fn visit_type_param_bound(&mut self, bound: &'ast syn::TypeParamBound) {
            if let syn::TypeParamBound::Trait(bound) = bound {
                self.visit_path(&bound.path);
            }
        }
    }

    let mut visitor = FindTyParams::new(generics);

    let should_visit = if WRITE {
        |_: &Struct, f: &StructField| !matches!(&f.field_mode, FieldMode::Function(_))
    } else {
        |s: &Struct, f: &StructField| {
            matches!(&f.field_mode, FieldMode::Normal) && f.map.is_none() && s.map.is_none()
        }
    };

    match binrw_input {
        Input::Struct(s) | Input::UnitStruct(s) => {
            for field in s.fields.iter().filter(|f| should_visit(s, f)) {
                visitor.visit_field(&field.field);
            }
        }
        Input::Enum(e) => {
            for variant in &e.variants {
                match variant {
                    EnumVariant::Variant { options, .. } => {
                        let relevant_fields =
                            options.fields.iter().filter(|f| should_visit(options, f));
                        for field in relevant_fields {
                            visitor.visit_field(&field.field);
                        }
                    }
                    EnumVariant::Unit(_) => {}
                }
            }
        }
        Input::UnitOnlyEnum(_) => {}
    }

    let relevant_type_params = visitor.relevant_type_params;
    let associated_type_usage = visitor.associated_type_usage;
    let new_predicates =
        generics
            .type_params()
            .map(|param| param.ident.clone())
            .filter(|ident| relevant_type_params.contains(ident))
            .map(|ident| syn::TypePath {
                qself: None,
                path: ident.into(),
            })
            .chain(associated_type_usage.into_iter().cloned())
            .flat_map(|bounded_ty| {
                let args_lifetime = get_args_lifetime(Span::call_site());

                let binrw_bound = if WRITE { BINWRITE_TRAIT } else { BINREAD_TRAIT };

                let where_predicates: syn::punctuated::Punctuated<
                    syn::WherePredicate,
                    syn::Token![,],
                > = parse_quote! {
                    for<#args_lifetime> #bounded_ty: #binrw_bound + #args_lifetime,
                    for<#args_lifetime> #bounded_ty::Args<#args_lifetime>: ::core::default::Default,
                };

                where_predicates
            });

    let mut generics = generics.clone();
    generics
        .make_where_clause()
        .predicates
        .extend(new_predicates);
    generics
}

fn get_args_lifetime(span: proc_macro2::Span) -> syn::Lifetime {
    syn::Lifetime::new(&format!("'{ARGS_LIFETIME}"), span)
}

fn get_assertions(assertions: &[Assert]) -> impl Iterator<Item = TokenStream> + '_ {
    assertions.iter().map(
        |Assert {
             kw_span,
             condition,
             consequent,
             ..
         }| {
            let error_fn = match &consequent {
                AssertionError::Message(message) => {
                    quote! { #ASSERT_ERROR_FN::<_, fn() -> !>::Message(|| { #message }) }
                }
                AssertionError::Error(error) => {
                    quote! { #ASSERT_ERROR_FN::Error::<fn() -> &'static str, _>(|| { #error }) }
                }
            };

            quote_spanned_any! {*kw_span=>
                #ASSERT(#condition, #POS, #error_fn)?;
            }
        },
    )
}

fn get_destructured_imports(
    imports: &Imports,
    type_name: Option<&Ident>,
    is_write: bool,
) -> Option<TokenStream> {
    match imports {
        Imports::None => None,
        Imports::List(idents, _) => {
            if idents.is_empty() {
                None
            } else {
                let idents = idents.iter();
                Some(quote! {
                    (#(mut #idents,)*)
                })
            }
        }
        Imports::Raw(ident, _) => Some(quote! {
            mut #ident
        }),
        Imports::Named(args) => type_name.map(|type_name| {
            let args_ty_name = arg_type_name(type_name, is_write);
            let idents = args.iter().map(|x| &x.ident);
            quote! {
                #args_ty_name {
                    #(#idents),*
                }
            }
        }),
    }
}

fn get_endian(endian: &CondEndian) -> TokenStream {
    match endian {
        CondEndian::Inherited => OPT.to_token_stream(),
        CondEndian::Fixed(endian) => endian.to_token_stream(),
        CondEndian::Cond(endian, condition) => {
            let (true_cond, false_cond) = (endian, endian.flipped());
            quote! {
                if (#condition) {
                    #true_cond
                } else {
                    #false_cond
                }
            }
        }
    }
}

fn get_map_err(pos: IdentStr, span: Span) -> TokenStream {
    quote_spanned_any! { span=>
        .map_err(|e| {
            #BIN_ERROR::Custom {
                pos: #pos,
                err: Box::new(e) as _,
            }
        })
    }
}

fn get_passed_args(field: &StructField, stream: &TokenStream) -> Option<TokenStream> {
    let args = &field.args;
    let span = args.span().unwrap_or_else(|| field.ty.span());
    match args {
        PassedArgs::Named(fields) => Some({
            let extra_args = directives_to_args(field, stream);
            quote_spanned_any! { span=>
                #ARGS_MACRO! { #extra_args #(#fields, )* }
            }
        }),
        PassedArgs::List(list) => Some(quote_spanned! {span=> (#(#list,)*) }),
        PassedArgs::Tuple(tuple) => Some(tuple.as_ref().clone()),
        PassedArgs::None => {
            let extra_args = directives_to_args(field, stream);
            (!extra_args.is_empty()).then(|| {
                quote_spanned_any! { span=> #ARGS_MACRO! { #extra_args } }
            })
        }
    }
}

fn get_try_calc(pos: IdentStr, ty: &Type, calc: &TokenStream) -> TokenStream {
    let map_err = get_map_err(pos, calc.span());
    quote_spanned! {ty.span()=> {
        let #TEMP: ::core::result::Result<#ty, _> = #calc;
        #TEMP #map_err ?
    }}
}

fn directives_to_args(field: &StructField, stream: &TokenStream) -> TokenStream {
    let args = field
        .count
        .as_ref()
        .map(|count| {
            quote_spanned_any! {count.span()=>
                count: {
                    let #TEMP = #count;
                    #[allow(clippy::useless_conversion, clippy::unnecessary_fallible_conversions)]
                    usize::try_from(#TEMP).map_err(|_| {
                        extern crate alloc;
                        #BIN_ERROR::AssertFail {
                            pos: #SEEK_TRAIT::stream_position(#stream)
                                .unwrap_or_default(),
                            // This is using debug formatting instead of display
                            // formatting to reduce the chance of some
                            // additional confusing error complaining about
                            // Display not being implemented if someone tries
                            // using a bogus type with `count`
                            message: alloc::format!("count {:?} out of range of usize", #TEMP)
                        }
                    })?
                }
            }
        })
        .into_iter()
        .chain(
            field
                .offset
                .as_ref()
                .map(|offset| quote_spanned! { offset.span()=> offset: #offset }),
        );
    quote! { #(#args,)* }
}
