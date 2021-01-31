use super::*;
use super::parser::{FieldLevelAttr, MetaAttrList};
use crate::CompileError;

#[derive(Debug, Default)]
pub(crate) struct FieldLevelAttrs {
    // ======================
    //    Field-level only
    // ======================
    pub args: PassedArgs,
    pub map: Option<TokenStream>,
    pub ignore: bool,
    pub default: bool,
    pub calc: Option<TokenStream>,
    pub count: Option<TokenStream>,
    pub offset: Option<TokenStream>,
    pub offset_after: Option<TokenStream>,
    pub if_cond: Option<TokenStream>,
    pub deref_now: bool,
    pub postprocess_now: bool,
    pub restore_position: bool,
    pub do_try: bool,
    pub temp: bool,

    // ======================
    //  All-level attributes
    // ======================
    // endian
    pub little: SpannedValue<bool>,
    pub big: SpannedValue<bool>,
    pub is_big: Option<TokenStream>,
    pub is_little: Option<TokenStream>,
    
    // assertions/error handling
    pub assert: Vec<Assert>,
    
    // TODO: this
    pub magic: Option<Lit>,
    pub pad_before: Option<TokenStream>,
    pub pad_after: Option<TokenStream>,
    pub align_before: Option<TokenStream>,
    pub align_after: Option<TokenStream>,
    pub seek_before: Option<TokenStream>,
    pub pad_size_to: Option<TokenStream>,

    // parsing
    pub parse_with: Option<TokenStream>
}

macro_rules! get_fla_type {
    ($tla:ident.$variant:ident) => {
        $tla.iter()
            .filter_map(|x|{
                if let FieldLevelAttr::$variant(x) = x {
                    Some(x)
                } else {
                    None
                }
            })
    };
}

type FlaList = MetaAttrList<FieldLevelAttr>;

impl FieldLevelAttrs {
    pub fn from_field(field: &syn::Field) -> Result<Self, CompileError> {
        let attrs: Vec<FieldLevelAttr> =
            field.attrs
                .iter()
                .filter(|x| x.path.is_ident("br") || x.path.is_ident("binread"))
                .map(flas_from_attribute)
                .collect::<Result<Vec<FlaList>, CompileError>>()?
                .into_iter()
                .flat_map(|x| x.0.into_iter())
                .collect();

        // bool type
        let big = first_span_true(get_fla_type!(attrs.Big));
        let little = first_span_true(get_fla_type!(attrs.Little));
        let default = get_fla_type!(attrs.Default).next().is_some();
        let ignore = get_fla_type!(attrs.Ignore).next().is_some();
        let deref_now = get_fla_type!(attrs.DerefNow).next().is_some();
        let restore_position = get_fla_type!(attrs.RestorePosition).next().is_some();
        let postprocess_now = get_fla_type!(attrs.PostProcessNow).next().is_some();
        let do_try = get_fla_type!(attrs.Try).next().is_some();
        let temp = get_fla_type!(attrs.Temp).next().is_some();

        // func assignment type
        let map = get_fla_type!(attrs.Map);
        let parse_with = get_fla_type!(attrs.ParseWith);

        // lit assignment type
        let magic = get_fla_type!(attrs.Magic);

        // args type
        let args = get_fla_type!(attrs.Args);
        let args_tuple = get_fla_type!(attrs.ArgsTuple);
        let asserts = get_fla_type!(attrs.Assert);

        // expr type
        let calc = get_fla_type!(attrs.Calc);
        let count = get_fla_type!(attrs.Count);
        let is_little = get_fla_type!(attrs.IsLittle);
        let is_big = get_fla_type!(attrs.IsBig);
        let offset = get_fla_type!(attrs.Offset);
        let offset_after = get_fla_type!(attrs.OffsetAfter);
        let if_cond = get_fla_type!(attrs.If);

        let pad_before = get_fla_type!(attrs.PadBefore);
        let pad_after = get_fla_type!(attrs.PadAfter);
        let align_before = get_fla_type!(attrs.AlignBefore);
        let align_after = get_fla_type!(attrs.AlignAfter);
        let seek_before = get_fla_type!(attrs.SeekBefore);
        let pad_size_to = get_fla_type!(attrs.PadSizeTo);

        check_mutually_exclusive(args.clone(), args_tuple.clone(), "Conflicting instances of args and args_tuple")?;

        macro_rules! only_first {
            ($($a:ident),*) => {
                $(
                    let $a = get_only_first(
                        $a,
                        concat!("Conflicting instances of ", stringify!($a))
                    )?.map(|x| x.get());
                )*
            }
        }

        only_first!(
            pad_before, pad_after, align_before, align_after, seek_before, pad_size_to,
            calc, count, is_little, is_big, offset, offset_after, if_cond, map, magic,
            parse_with, args, args_tuple
        );

        let args = if let Some(arg) = args_tuple {
            PassedArgs::Tuple(arg)
        } else {
            PassedArgs::List(args.unwrap_or_default())
        };
        
        Ok(Self {
            little,
            big,
            ignore,
            default,
            deref_now,
            postprocess_now,
            restore_position,
            do_try,
            temp,
            
            calc,
            count,
            offset,
            offset_after,
            if_cond,
            is_big,
            is_little,
            pad_before,
            pad_after,
            align_before,
            align_after,
            seek_before,
            pad_size_to,

            parse_with,
            map,
            args,
            assert: asserts.map(convert_assert).collect::<Result<_, _>>()?,
            magic,
        })
    }
}

fn flas_from_attribute(attr: &syn::Attribute) -> Result<FlaList, CompileError> {
    Ok(syn::parse2(attr.tokens.clone())?)
}
