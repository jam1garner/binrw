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
    pub(crate) TRAIT_NAME = from_trait!();
    pub(crate) BIN_ERROR = from_crate!(Error);
    pub(crate) OPTIONS = from_crate!(ReadOptions);
    pub(crate) READ_TRAIT = from_crate!(io::Read);
    pub(crate) SEEK_TRAIT = from_crate!(io::Seek);
    pub(crate) SEEK_FROM = from_crate!(io::SeekFrom);
    pub(crate) BIN_RESULT = from_crate!(BinResult);
    pub(crate) ENDIAN_ENUM = from_crate!(Endian);
    pub(crate) READ_METHOD = from_trait!(read_options);
    pub(crate) AFTER_PARSE = from_trait!(after_parse);
    pub(crate) READER = "__binrw_generated_var_reader";
    pub(crate) OPT = "__binrw_generated_var_options";
    pub(crate) ARGS = "__binrw_generated_var_arguments";
    pub(crate) SAVED_POSITION = "__binrw_generated_saved_position";
    pub(crate) ASSERT_MAGIC = from_crate!(__private::magic);
    pub(crate) ASSERT = from_crate!(__private::assert);
    pub(crate) ASSERT_ERROR_FN = from_crate!(__private::AssertErrorFn);
    pub(crate) COERCE_FN = from_crate!(__private::coerce_fn);
    pub(crate) TRY_AFTER_PARSE = from_crate!(__private::try_after_parse);
    pub(crate) SATISFIED_OR_OPTIONAL = from_crate!(SatisfiedOrOptional);
    pub(crate) SATISFIED = from_crate!(Satisfied);
    pub(crate) NEEDED = from_crate!(Needed);
    pub(crate) OPTIONAL = from_crate!(Optional);
    pub(crate) TEMP = "__binrw_temp";
    pub(crate) POS = "__binrw_generated_position_temp";
    pub(crate) ERROR_BASKET = "__binrw_generated_error_basket";
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
