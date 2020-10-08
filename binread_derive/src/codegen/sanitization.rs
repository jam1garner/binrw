#![allow(dead_code, non_upper_case_globals)]
///! Utilities for helping sanitize macro
use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident, ToTokens};

const CNAME: &str = "::binread";
const TNAME: &str = "BinRead";

macro_rules! from_crate {
    () => { IdentStr(CNAME) };
    ($path:path) => { IdentStr(concat!("::binread::", stringify!($path))) };
}

macro_rules! from_trait {
    () => { from_crate!(BinRead) };
    ($path:path) => { IdentStr(concat!("::binread::BinRead::", stringify!($path))) };
}

pub static CRATE_NAME: IdentStr = from_crate!();
pub static TRAIT_NAME: IdentStr = from_trait!();

pub static BIN_ERROR: IdentStr = from_crate!(Error);
pub static OPTIONS: IdentStr = from_crate!(ReadOptions);
pub static READ_TRAIT: IdentStr = from_crate!(io::Read);
pub static SEEK_TRAIT: IdentStr = from_crate!(io::Seek);
pub static SEEK_FROM: IdentStr = from_crate!(io::SeekFrom);
pub static BIN_RESULT: IdentStr = from_crate!(BinResult);
pub static ENDIAN_ENUM: IdentStr = from_crate!(Endian);

pub static READ_METHOD: IdentStr = from_trait!(read_options);
pub static AFTER_PARSE: IdentStr = from_trait!(after_parse);

pub static READER: IdentStr = IdentStr("__binread_generated_var_reader");
pub static OPT: IdentStr = IdentStr("__binread_generated_var_options");
pub static ARGS: IdentStr = IdentStr("__binread_generated_var_arguments");
pub static AFTER_OPTS: IdentStr = IdentStr("__binread_generated_var_after_options"); 

pub static DEFAULT: IdentStr = IdentStr("core::default::Default::default");

pub static ASSERT_MAGIC: IdentStr = from_crate!(error::magic);
//pub static ASSERT_EQ: IdentStr = from_crate!(error::assert_eq);
pub static ASSERT: IdentStr = from_crate!(error::assert);

pub static WRITE_START_STRUCT: IdentStr = from_crate!(binary_template::write_start_struct);
pub static WRITE_END_STRUCT: IdentStr = from_crate!(binary_template::write_end_struct);
pub static WRITE_COMMENT: IdentStr = from_crate!(binary_template::write_comment);

pub static IDENTITY_FN: IdentStr = IdentStr("core::convert::identity");
pub static READ_METHOD_NOP: IdentStr = from_crate!(error::nop3);
pub static READ_METHOD_DEFAULT: IdentStr = from_crate!(error::nop3_default);
pub static AFTER_PARSE_NOP: IdentStr = from_crate!(error::nop5);
pub static AFTER_PARSE_TRY: IdentStr = from_crate!(error::try_after_parse);
pub static AFTER_PARSE_IDENTITY: IdentStr = from_crate!(error::identity_after_parse);
pub static TRY_CONVERSION: IdentStr = from_crate!(error::try_conversion);

pub static TEMP: IdentStr = IdentStr("__binread__temp");
pub static POS: IdentStr = IdentStr("__binread_generated_position_temp");


pub fn closure_wrap<T: ToTokens>(value: T) -> TokenStream {
    quote!(
        (||{ #value })()
    )
}

pub fn fmt_ident_options<S: ToString>(ident: S) -> Ident {
    format_ident!("__{}_options", ident.to_string())
}

/// A string wrapper that converts the str to a $path TokenStream, allowing for constant-time
/// idents that can be shared across threads
#[derive(Debug, Clone, Copy)]
pub struct IdentStr<'a>(pub &'a str);

use quote::TokenStreamExt;

impl<'a> ToTokens for IdentStr<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let idents: Vec<_> =
            self.0.split("::")
            .map(|id|{
                let id = id.trim();
                if id.is_empty() {
                    None
                } else {
                    Some(format_ident!("{}", id))
                }
            }).collect();
        tokens.append_separated(idents, quote!(::));
    }
}
