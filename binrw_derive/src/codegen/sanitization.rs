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

macro_rules! from_read_trait {
    () => {
        from_crate!(BinRead)
    };
    ($path:path) => {
        concat!("binrw::BinRead::", stringify!($path))
    };
}
macro_rules! from_write_trait {
    () => {
        from_crate!(BinWrite)
    };
    ($path:path) => {
        concat!("binrw::BinWrite::", stringify!($path))
    };
}

ident_str! {
    pub(crate) BINREAD_TRAIT = from_read_trait!();
    pub(crate) BINWRITE_TRAIT = from_write_trait!();
    pub(crate) BIN_ERROR = from_crate!(Error);
    pub(crate) READ_OPTIONS = from_crate!(ReadOptions);
    pub(crate) WRITE_OPTIONS = from_crate!(WriteOptions);
    pub(crate) READ_TRAIT = from_crate!(io::Read);
    pub(crate) WRITE_TRAIT = from_crate!(io::Write);
    pub(crate) SEEK_TRAIT = from_crate!(io::Seek);
    pub(crate) SEEK_FROM = from_crate!(io::SeekFrom);
    pub(crate) BIN_RESULT = from_crate!(BinResult);
    pub(crate) ENDIAN_ENUM = from_crate!(Endian);
    pub(crate) READ_METHOD = from_read_trait!(read_options);
    pub(crate) AFTER_PARSE = from_read_trait!(after_parse);
    pub(crate) WRITE_METHOD = from_write_trait!(write_options);
    pub(crate) READER = "__binrw_generated_var_reader";
    pub(crate) WRITER = "__binrw_generated_var_writer";
    pub(crate) OPT = "__binrw_generated_var_options";
    pub(crate) ARGS = "__binrw_generated_var_arguments";
    pub(crate) SAVED_POSITION = "__binrw_generated_saved_position";
    pub(crate) ASSERT_MAGIC = from_crate!(__private::magic);
    pub(crate) ASSERT = from_crate!(__private::assert);
    pub(crate) ASSERT_ERROR_FN = from_crate!(__private::AssertErrorFn);
    pub(crate) COERCE_FN = from_crate!(__private::coerce_fn);
    pub(crate) ARGS_TYPE_HINT = from_crate!(__private::parse_function_args_type_hint);
    pub(crate) MAP_ARGS_TYPE_HINT = from_crate!(__private::map_args_type_hint);
    pub(crate) WRITE_FN_TYPE_HINT = from_crate!(__private::write_fn_type_hint);
    pub(crate) WRITE_WITH_ARGS_TYPE_HINT = from_crate!(__private::write_function_args_type_hint);
    pub(crate) WRITE_MAP_ARGS_TYPE_HINT = from_crate!(__private::write_map_args_type_hint);
    pub(crate) WRITE_TRY_MAP_ARGS_TYPE_HINT = from_crate!(__private::write_try_map_args_type_hint);
    pub(crate) WRITE_MAP_INPUT_TYPE_HINT = from_crate!(__private::write_map_fn_input_type_hint);
    pub(crate) WRITE_FN_MAP_OUTPUT_TYPE_HINT = from_crate!(__private::write_fn_map_output_type_hint);
    pub(crate) WRITE_FN_TRY_MAP_OUTPUT_TYPE_HINT = from_crate!(__private::write_fn_try_map_output_type_hint);
    pub(crate) WRITE_ZEROES = from_crate!(__private::write_zeroes);
    pub(crate) SATISFIED_OR_OPTIONAL = from_crate!(SatisfiedOrOptional);
    pub(crate) SATISFIED = from_crate!(Satisfied);
    pub(crate) NEEDED = from_crate!(Needed);
    pub(crate) OPTIONAL = from_crate!(Optional);
    pub(crate) BINRW_NAMED_ARGS = from_crate!(BinrwNamedArgs);
    pub(crate) ARGS_MACRO = from_crate!(args);
    pub(crate) HAS_MAGIC = from_crate!(HasMagic);
    pub(crate) WITH_CONTEXT = from_crate!(error::ContextExt::with_context);
    pub(crate) BACKTRACE_FRAME = from_crate!(error::BacktraceFrame);
    pub(crate) TEMP = "__binrw_temp";
    pub(crate) POS = "__binrw_generated_position_temp";
    pub(crate) ERROR_BASKET = "__binrw_generated_error_basket";
    pub(crate) READ_FUNCTION = "__binrw_generated_read_function";
    pub(crate) WRITE_FUNCTION = "__binrw_generated_write_function";
    pub(crate) BEFORE_POS = "__binrw_generated_before_pos";
}

pub(crate) fn make_ident(ident: &Ident, kind: &str) -> Ident {
    let ident_string = ident.to_string();
    let ident_string = ident_string.strip_prefix("r#").unwrap_or(&ident_string);
    format_ident!("__binrw_generated_{}_{}", kind, ident_string)
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
