use crate::{
    combine_error,
    result::PartialResult,
    util::{from_crate, ident_str},
};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Error, FnArg, Ident, ItemFn, Pat, Token,
};

#[cfg_attr(coverage_nightly, no_coverage)]
pub(crate) fn derive_from_attribute<const WRITE: bool>(
    attr: TokenStream,
    input: TokenStream,
) -> TokenStream {
    match generate::<WRITE>(
        parse_macro_input!(attr as Options<WRITE>),
        parse_macro_input!(input as ItemFn),
    ) {
        PartialResult::Ok(func) => func.into_token_stream(),
        PartialResult::Partial(func, err) => {
            let err = err.into_compile_error();
            quote! {
                #func
                #err
            }
        }
        PartialResult::Err(err) => err.into_compile_error(),
    }
    .into()
}

fn generate<const WRITE: bool>(
    Options { stream, endian }: Options<WRITE>,
    mut func: ItemFn,
) -> PartialResult<ItemFn, Error> {
    // Since these functions are written to match the binrw API, args must be
    // passed by value even when they are not consumed, so suppress this lint
    func.attrs
        .push(parse_quote!(#[allow(clippy::needless_pass_by_value)]));

    let raw_args_span = func.sig.variadic.take().map(|variadic| variadic.span());

    func.sig.generics.params.push({
        let stream_trait = if WRITE { WRITE_TRAIT } else { READ_TRAIT };

        parse_quote!(#STREAM_T: #stream_trait + #SEEK_TRAIT)
    });

    let mut args = core::mem::take(&mut func.sig.inputs).into_iter();
    let mut args_pat = Punctuated::<_, Token![,]>::new();
    let mut args_ty = Punctuated::<_, Token![,]>::new();

    if WRITE {
        if let Some(arg) = args.next() {
            func.sig.inputs.push(arg);
        } else {
            let span = func.sig.ident.span();
            return PartialResult::Partial(
                func,
                Error::new(span, "missing required value parameter"),
            );
        }
    }

    func.sig.inputs.push(parse_quote!(#stream: &mut #STREAM_T));
    func.sig.inputs.push(parse_quote!(#endian: #ENDIAN_ENUM));

    if let Some(raw_args_span) = raw_args_span {
        if let Some(arg) = args.next() {
            func.sig.inputs.push(arg);
        } else {
            return PartialResult::Partial(
                func,
                Error::new(raw_args_span, "missing raw arguments parameter"),
            );
        }

        if let Some(arg) = args.next() {
            return PartialResult::Partial(
                func,
                Error::new(arg.span(), "unexpected extra parameter after raw arguments"),
            );
        }
    } else {
        for arg in args {
            match arg {
                FnArg::Receiver(r) => {
                    return PartialResult::Partial(
                        func,
                        Error::new(r.span(), "invalid `self` in free function"),
                    );
                }
                FnArg::Typed(ty) => {
                    args_pat.push(ty.pat);
                    args_ty.push(ty.ty);
                }
            }
        }

        if args_ty.len() == 1 {
            // Add trailing comma so it's a single-element tuple, not a parenthesized item
            args_pat.push_punct(parse_quote!(,));
            args_ty.push_punct(parse_quote!(,));
        }

        func.sig.inputs.push(parse_quote!((#args_pat): (#args_ty)));
    }

    PartialResult::Ok(func)
}

ident_str! {
    STREAM_T = "__BinrwGeneratedStreamT";
    ENDIAN_ENUM = from_crate!(Endian);
    READ_TRAIT = from_crate!(io::Read);
    WRITE_TRAIT = from_crate!(io::Write);
    SEEK_TRAIT = from_crate!(io::Seek);
}

struct Options<const WRITE: bool> {
    stream: Pat,
    endian: Pat,
}

impl<const WRITE: bool> Parse for Options<WRITE> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        fn try_set(
            kw: &str,
            value: Ident,
            out: &mut Option<Ident>,
            all_errors: &mut Option<Error>,
        ) {
            if out.is_none() {
                *out = Some(value);
            } else {
                combine_error(
                    all_errors,
                    Error::new(value.span(), format!("conflicting `{kw}` keyword")),
                );
            }
        }

        let mut stream = None;
        let mut endian = None;

        let mut all_errors = None;

        for arg in Punctuated::<Arg<WRITE>, Token![,]>::parse_terminated(input)? {
            match arg {
                Arg::Stream(ident) => try_set(
                    if WRITE { "writer" } else { "reader" },
                    ident,
                    &mut stream,
                    &mut all_errors,
                ),
                Arg::Endian(ident) => try_set("endian", ident, &mut endian, &mut all_errors),
            }
        }

        if let Some(error) = all_errors {
            Err(error)
        } else {
            Ok(Self {
                stream: stream.map_or_else(|| parse_quote!(_), |ident| parse_quote!(#ident)),
                endian: endian.map_or_else(|| parse_quote!(_), |ident| parse_quote!(#ident)),
            })
        }
    }
}

enum Arg<const WRITE: bool> {
    Stream(Ident),
    Endian(Ident),
}

impl<const WRITE: bool> Parse for Arg<WRITE> {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        fn maybe_ident(default: Ident, input: ParseStream<'_>) -> syn::Result<Ident> {
            if input.is_empty() || input.peek(Token![,]) {
                Ok(default)
            } else {
                let next = input.lookahead1();
                if next.peek(Token![:]) {
                    input.parse::<Token![:]>()?;
                    input.parse()
                } else {
                    Err(next.error())
                }
            }
        }

        let kw = input.lookahead1();
        if (WRITE && kw.peek(kw::writer)) || (!WRITE && kw.peek(kw::reader)) {
            let kw = input.parse::<Ident>()?;
            Ok(Arg::Stream(maybe_ident(kw, input)?))
        } else if kw.peek(kw::endian) {
            let kw = input.parse::<Ident>()?;
            Ok(Arg::Endian(maybe_ident(kw, input)?))
        } else {
            Err(kw.error())
        }
    }
}

mod kw {
    syn::custom_keyword!(endian);
    syn::custom_keyword!(reader);
    syn::custom_keyword!(value);
    syn::custom_keyword!(writer);
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;

    #[cfg_attr(coverage_nightly, no_coverage)]
    fn try_input<const WRITE: bool>(attr: TokenStream, params: &TokenStream) {
        let options = syn::parse2::<Options<WRITE>>(attr).unwrap();
        let func = syn::parse2::<ItemFn>(quote::quote! {
            fn test(#params) -> binrw::BinResult<()> { Ok(()) }
        })
        .unwrap();
        generate::<WRITE>(options, func).unwrap();
    }

    macro_rules! try_error (
        (read $name:ident: $message:literal $opts:tt $params:tt) => {
            #[test]
            #[cfg_attr(coverage_nightly, no_coverage)]
            #[should_panic(expected = $message)]
            fn $name() {
                try_input::<false>(quote::quote! $opts, &quote::quote! $params);
            }
        };

        (write $name:ident: $message:literal $opts:tt $params:tt) => {
            #[test]
            #[cfg_attr(coverage_nightly, no_coverage)]
            #[should_panic(expected = $message)]
            fn $name() {
                try_input::<true>(quote::quote! $opts, &quote::quote! $params);
            }
        };
    );

    try_error!(read fn_helper_invalid_option_value: "expected identifier"
        [reader:] ()
    );

    try_error!(read fn_helper_invalid_option_token: "expected `:`"
        [reader = invalid] ()
    );

    try_error!(read fn_helper_invalid_reader: "expected `reader` or `endian`"
        [invalid] ()
    );

    try_error!(write fn_helper_invalid_writer: "expected `writer` or `endian`"
        [invalid] ()
    );

    try_error!(read fn_helper_conflicting_reader: "conflicting `reader`"
        [reader, reader] ()
    );

    try_error!(write fn_helper_conflicting_writer: "conflicting `writer`"
        [writer, writer] ()
    );

    try_error!(read fn_helper_conflicting_endian: "conflicting `endian`"
        [endian, endian] ()
    );

    try_error!(read fn_helper_invalid_self: "invalid `self`"
        [] (&self)
    );

    try_error!(write fn_helper_missing_object: "missing required value"
        [] ()
    );

    try_error!(read fn_helper_missing_args_reader: "missing raw arguments"
        [] (...)
    );

    try_error!(read fn_helper_extra_args_reader: "unexpected extra parameter"
        [] (arg0: (), arg1: (), ...)
    );

    try_error!(write fn_helper_extra_args_writer: "unexpected extra parameter"
        [] (arg0: &(), arg1: (), arg2: (), ...)
    );

    try_error!(write fn_helper_missing_args_writer: "missing raw arguments"
        [] (obj: &(), ...)
    );
}
