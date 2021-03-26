///! Utilities for helping sanitize macro
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};

macro_rules! ident_str {
    () => {};

    ($vis:vis $ident:ident = $path:expr; $($tail:tt)*) => {
        ident_str!($vis $ident = $path);
        ident_str!($($tail)*);
    };

    ($vis:vis $ident:ident = $path:expr) => {
        $vis const $ident: $crate::codegen::sanitization::IdentStr =
            $crate::codegen::sanitization::IdentStr::new($path);
    };
}

macro_rules! from_crate {
    ($path:path) => {
        concat!("binrw::", stringify!($path))
    };
}

macro_rules! from_trait {
    () => {
        from_crate!(BinRead)
    };
    ($path:path) => {
        concat!("binrw::BinRead::", stringify!($path))
    };
}

ident_str! {
    pub(super) TRAIT_NAME = from_trait!();
    pub(super) BIN_ERROR = from_crate!(Error);
    pub(super) OPTIONS = from_crate!(ReadOptions);
    pub(super) READ_TRAIT = from_crate!(io::Read);
    pub(super) SEEK_TRAIT = from_crate!(io::Seek);
    pub(super) SEEK_FROM = from_crate!(io::SeekFrom);
    pub(super) BIN_RESULT = from_crate!(BinResult);
    pub(super) ENDIAN_ENUM = from_crate!(Endian);
    pub(super) READ_METHOD = from_trait!(read_options);
    pub(super) AFTER_PARSE = from_trait!(after_parse);
    pub(super) READER = "__binrw_generated_var_reader";
    pub(super) OPT = "__binrw_generated_var_options";
    pub(super) ARGS = "__binrw_generated_var_arguments";
    pub(super) SAVED_POSITION = "__binrw_generated_saved_position";
    pub(super) ASSERT_MAGIC = from_crate!(__private::magic);
    pub(super) ASSERT = from_crate!(__private::assert);
    pub(super) ASSERT_ERROR_FN = from_crate!(__private::AssertErrorFn);
    pub(super) COERCE_FN = from_crate!(__private::coerce_fn);
    pub(super) TRY_AFTER_PARSE = from_crate!(__private::try_after_parse);
    pub(super) SATISFIED_OR_OPTIONAL = from_crate!(SatisfiedOrOptional);
    pub(super) NEEDED = from_crate!(Needed);
    pub(super) OPTIONAL = from_crate!(Optional);
    pub(super) TEMP = "__binrw_temp";
    pub(super) POS = "__binrw_generated_position_temp";
    pub(super) ERROR_BASKET = "__binrw_generated_error_basket";
}

pub(crate) fn make_ident(ident: &Ident, kind: &str) -> Ident {
    format_ident!("__binrw_generated_{}_{}", kind, ident.to_string())
}

/// A string wrapper that converts the str to a $path `TokenStream`, allowing
/// for constant-time idents that can be shared across threads
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct IdentStr(&'static str);

impl IdentStr {
    pub(crate) const fn new(str: &'static str) -> Self {
        IdentStr(str)
    }
}

impl ToTokens for IdentStr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let idents = self
            .0
            .split("::")
            .map(|ident| Ident::new(ident, Span::call_site()));
        tokens.append_separated(idents, quote!(::));
    }
}
