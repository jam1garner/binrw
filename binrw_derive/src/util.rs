use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

macro_rules! ident_str {
    () => {};

    ($vis:vis $ident:ident = $path:expr; $($tail:tt)*) => {
        ident_str!($vis $ident = $path);
        ident_str!($($tail)*);
    };

    ($vis:vis $ident:ident = $path:expr) => {
        $vis const $ident: $crate::util::IdentStr =
            $crate::util::IdentStr::new($path);
    };
}
pub(crate) use ident_str;

macro_rules! from_crate {
    ($path:path) => {
        concat!("binrw::", stringify!($path))
    };
}
pub(crate) use from_crate;

pub(crate) trait ToSpannedTokens {
    fn to_spanned_tokens(&self, tokens: &mut TokenStream, span: Span);
}

impl<T: ToTokens> ToSpannedTokens for &T {
    fn to_spanned_tokens(&self, tokens: &mut TokenStream, _: Span) {
        self.to_tokens(tokens);
    }
}

/// A string wrapper that converts the str to a $path `TokenStream`, allowing
/// for constant-time idents that can be shared across threads
#[derive(Clone, Copy)]
pub(crate) struct IdentStr(&'static str);

impl IdentStr {
    #[cfg_attr(coverage_nightly, no_coverage)] // const-only function
    pub(crate) const fn new(str: &'static str) -> Self {
        IdentStr(str)
    }

    pub(crate) fn iter(&self, span: Span) -> impl Iterator<Item = Ident> + '_ {
        self.0.split("::").map(move |ident| Ident::new(ident, span))
    }
}

impl ToTokens for IdentStr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_separated(self.iter(Span::call_site()), quote!(::));
    }
}

impl ToSpannedTokens for IdentStr {
    fn to_spanned_tokens(&self, tokens: &mut TokenStream, span: Span) {
        tokens.append_separated(self.iter(span), quote::quote_spanned!(span=> ::));
    }
}

/// Like `quote::quote_spanned!`, except allows interpolations to optionally
/// have their spans overridden by implementing the `ToSpannedTokens` trait.
/// Currently, for laziness/YAGNI reasons, repetitions fall back to
/// `quote::quote_spanned!`, so interpolated tokens inside repetitions will
/// not have overridden spans.
macro_rules! quote_spanned_any {
    (@group $ts:ident $span:ident $delimiter:ident $($tt:tt)*) => {
        let mut _inner_ts = proc_macro2::TokenStream::new();
        $crate::util::quote_spanned_any!(@tt _inner_ts $span $($tt)*);
        quote::TokenStreamExt::append(&mut $ts, {
            let mut group = proc_macro2::Group::new(proc_macro2::Delimiter::$delimiter, _inner_ts);
            group.set_span($span);
            group
        });
    };

    (@tt $ts:ident $span:ident { $($inner:tt)* } $($tt:tt)*) => {
        $crate::util::quote_spanned_any!(@group $ts $span Brace $($inner)*);
        $crate::util::quote_spanned_any!(@tt $ts $span $($tt)*);
    };

    (@tt $ts:ident $span:ident [ $($inner:tt)* ] $($tt:tt)*) => {
        $crate::util::quote_spanned_any!(@group $ts $span Bracket $($inner)*);
        $crate::util::quote_spanned_any!(@tt $ts $span $($tt)*);
    };

    (@tt $ts:ident $span:ident ( $($inner:tt)* ) $($tt:tt)*) => {
        $crate::util::quote_spanned_any!(@group $ts $span Parenthesis $($inner)*);
        $crate::util::quote_spanned_any!(@tt $ts $span $($tt)*);
    };

    (@tt $ts:ident $span:ident # ( $($inner:tt)* ) * * $($tt:tt)*) => {
        $ts.extend(quote::quote_spanned!($span=> #( $($inner)* ) * *));
        $crate::util::quote_spanned_any!(@tt $ts $span $($tt)*);
    };

    (@tt $ts:ident $span:ident # ( $($inner:tt)* ) * $($tt:tt)*) => {
        $ts.extend(quote::quote_spanned!($span=> #( $($inner)* ) *));
        $crate::util::quote_spanned_any!(@tt $ts $span $($tt)*);
    };

    (@tt $ts:ident $span:ident # ( $($inner:tt)* ) $sep:tt * $($tt:tt)*) => {
        $ts.extend(quote::quote_spanned!($span=> #( $($inner)* ) $sep *));
        $crate::util::quote_spanned_any!(@tt $ts $span $($tt)*);
    };

    (@tt $ts:ident $span:ident # $ident:ident $($tt:tt)*) => {
        (&$ident).to_spanned_tokens(&mut $ts, $span);
        $crate::util::quote_spanned_any!(@tt $ts $span $($tt)*);
    };

    (@tt $ts:ident $span:ident $token:tt $($tt:tt)*) => {
        $ts.extend(quote::quote_spanned!($span=> $token));
        $crate::util::quote_spanned_any!(@tt $ts $span $($tt)*);
    };

    (@tt $ts:ident $span:ident) => {};

    ($span:expr => $($tt:tt)*) => { {
        #[allow(unused_imports)]
        use $crate::util::ToSpannedTokens;
        let mut _ts = proc_macro2::TokenStream::new();
        let _span = $span;
        $crate::util::quote_spanned_any!(@tt _ts _span $($tt)*);
        _ts
    } }
}
pub(crate) use quote_spanned_any;
