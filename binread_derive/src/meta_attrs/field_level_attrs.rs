use super::*;
use super::parser::FieldLevelAttr;

#[derive(FromField, Debug)]
#[darling(attributes(br, binread))]
pub struct FieldLevelAttrs {
    // ======================
    //    Field-level only
    // ======================
    #[darling(default)]
    pub args: PassedValues,
    #[darling(default, map = "to_tokens")]
    pub map: Option<TokenStream>,
    #[darling(default)]
    pub ignore: bool,
    #[darling(default)]
    pub default: bool,
    #[darling(default, map = "to_tokens")]
    pub calc: Option<TokenStream>,
    #[darling(default, map = "to_tokens")]
    pub count: Option<TokenStream>,
    #[darling(default, map = "to_tokens")]
    pub offset: Option<TokenStream>,
    #[darling(default, map = "to_tokens", rename = "if")]
    pub if_cond: Option<TokenStream>,
    #[darling(default)]
    pub deref_now: bool,
    #[darling(default)]
    pub postprocess_now: bool,
    #[darling(default)]
    pub restore_position: bool,

    // ======================
    //  All-level attributes
    // ======================
    // endian
    #[darling(default)]
    pub little: SpannedValue<bool>,
    #[darling(default)]
    pub big: SpannedValue<bool>,
    #[darling(default, map = "to_tokens")]
    pub is_big: Option<TokenStream>,
    #[darling(default, map = "to_tokens")]
    pub is_little: Option<TokenStream>,
    
    // assertions/error handling
    #[darling(multiple, map = "to_assert")]
    pub assert: Vec<Assert>,
    
    // TODO: this
    #[darling(default)]
    pub magic: Option<Lit>,

    #[darling(default, map = "to_tokens")]
    pub pad_before: Option<TokenStream>,

    #[darling(default, map = "to_tokens")]
    pub pad_after: Option<TokenStream>,

    #[darling(default, map = "to_tokens")]
    pub align_before: Option<TokenStream>,

    #[darling(default, map = "to_tokens")]
    pub align_after: Option<TokenStream>,

    #[darling(default, map = "to_tokens")]
    pub seek_before: Option<TokenStream>,

    #[darling(default, map = "to_tokens")]
    pub pad_size_to: Option<TokenStream>,

    // parsing
    #[darling(default)]
    pub parse_with: Option<Path>
}
