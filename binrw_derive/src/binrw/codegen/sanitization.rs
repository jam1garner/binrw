//! Utilities for helping sanitize macro
use crate::util::{from_crate, ident_str};
use proc_macro2::Ident;
use quote::format_ident;

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

pub(crate) const ARGS_LIFETIME: &str = "__binrw_generated_args_lifetime";

ident_str! {
    pub(crate) BINREAD_TRAIT = from_read_trait!();
    pub(crate) BINWRITE_TRAIT = from_write_trait!();
    pub(crate) BIN_ERROR = from_crate!(Error);
    pub(crate) READ_TRAIT = from_crate!(io::Read);
    pub(crate) WRITE_TRAIT = from_crate!(io::Write);
    pub(crate) SEEK_TRAIT = from_crate!(io::Seek);
    pub(crate) SEEK_FROM = from_crate!(io::SeekFrom);
    pub(crate) BIN_RESULT = from_crate!(BinResult);
    pub(crate) ENDIAN_ENUM = from_crate!(Endian);
    pub(crate) READ_METHOD = from_read_trait!(read_options);
    pub(crate) WRITE_METHOD = from_write_trait!(write_options);
    pub(crate) READER = "__binrw_generated_var_reader";
    pub(crate) WRITER = "__binrw_generated_var_writer";
    pub(crate) OPT = "__binrw_generated_var_endian";
    pub(crate) ARGS = "__binrw_generated_var_arguments";
    pub(crate) SAVED_POSITION = "__binrw_generated_saved_position";
    pub(crate) NOT_ENOUGH_BYTES = from_crate!(__private::not_enough_bytes);
    pub(crate) ASSERT_MAGIC = from_crate!(__private::magic);
    pub(crate) ASSERT = from_crate!(__private::assert);
    pub(crate) ASSERT_ERROR_FN = from_crate!(__private::AssertErrorFn);
    pub(crate) COERCE_FN = from_crate!(__private::coerce_fn);
    pub(crate) ARGS_TYPE_HINT = from_crate!(__private::parse_function_args_type_hint);
    pub(crate) MAP_ARGS_TYPE_HINT = from_crate!(__private::map_args_type_hint);
    pub(crate) REQUIRED_ARG_TRAIT = from_crate!(__private::Required);
    pub(crate) MAP_READER_TYPE_HINT = from_crate!(__private::map_reader_type_hint);
    pub(crate) MAP_WRITER_TYPE_HINT = from_crate!(__private::map_writer_type_hint);
    pub(crate) PARSE_FN_TYPE_HINT = from_crate!(__private::parse_fn_type_hint);
    pub(crate) WRITE_FN_TYPE_HINT = from_crate!(__private::write_fn_type_hint);
    pub(crate) WRITE_ARGS_TYPE_HINT = from_crate!(__private::write_function_args_type_hint);
    pub(crate) WRITE_MAP_ARGS_TYPE_HINT = from_crate!(__private::write_map_args_type_hint);
    pub(crate) WRITE_TRY_MAP_ARGS_TYPE_HINT = from_crate!(__private::write_try_map_args_type_hint);
    pub(crate) WRITE_MAP_INPUT_TYPE_HINT = from_crate!(__private::write_map_fn_input_type_hint);
    pub(crate) WRITE_FN_MAP_OUTPUT_TYPE_HINT = from_crate!(__private::write_fn_map_output_type_hint);
    pub(crate) WRITE_FN_TRY_MAP_OUTPUT_TYPE_HINT = from_crate!(__private::write_fn_try_map_output_type_hint);
    pub(crate) RESTORE_POSITION = from_crate!(__private::restore_position);
    pub(crate) RESTORE_POSITION_VARIANT = from_crate!(__private::restore_position_variant);
    pub(crate) WRITE_ZEROES = from_crate!(__private::write_zeroes);
    pub(crate) ARGS_MACRO = from_crate!(args);
    pub(crate) META_ENDIAN_KIND = from_crate!(meta::EndianKind);
    pub(crate) READ_ENDIAN = from_crate!(meta::ReadEndian);
    pub(crate) READ_MAGIC = from_crate!(meta::ReadMagic);
    pub(crate) WRITE_ENDIAN = from_crate!(meta::WriteEndian);
    pub(crate) WRITE_MAGIC = from_crate!(meta::WriteMagic);
    pub(crate) WITH_CONTEXT = from_crate!(error::ContextExt::with_context);
    pub(crate) BACKTRACE_FRAME = from_crate!(error::BacktraceFrame);
    pub(crate) TEMP = "__binrw_temp";
    pub(crate) THIS = "__binrw_this";
    pub(crate) POS = "__binrw_generated_position_temp";
    pub(crate) ERROR_BASKET = "__binrw_generated_error_basket";
    pub(crate) READ_FUNCTION = "__binrw_generated_read_function";
    pub(crate) WRITE_FUNCTION = "__binrw_generated_write_function";
    pub(crate) BEFORE_POS = "__binrw_generated_before_pos";
    pub(crate) ALL_EOF = "__binrw_generated_all_eof";
    pub(crate) DBG_EPRINTLN = from_crate!(__private::eprintln);
}

pub(crate) fn make_ident(ident: &Ident, kind: &str) -> Ident {
    format_ident!("__binrw_generated_{}_{}", kind, ident)
}
