use proc_macro2::TokenStream;
use super::{FromAttrs, FromField, FromInput, ParseResult, Struct, TrySet, types::{Assert, CondEndian, Magic, Map, PassedArgs}};

attr_struct! {
    #[from(StructFieldAttr)]
    #[derive(Clone, Debug)]
    pub(crate) struct StructField {
        pub(crate) ident: Option<syn::Ident>,
        pub(crate) ty: syn::Type,
        #[from(Big, Little, IsBig, IsLittle)]
        pub(crate) endian: CondEndian,
        #[from(Map, TryMap)]
        pub(crate) map: Map,
        #[from(Magic)]
        pub(crate) magic: Magic,
        #[from(Args, ArgsTuple)]
        pub(crate) args: PassedArgs,
        #[from(Ignore)]
        pub(crate) ignore: bool,
        #[from(Default)]
        pub(crate) default: bool,
        #[from(Calc)]
        pub(crate) calc: Option<TokenStream>,
        #[from(Count)]
        pub(crate) count: Option<TokenStream>,
        #[from(Offset)]
        pub(crate) offset: Option<TokenStream>,
        #[from(OffsetAfter)]
        pub(crate) offset_after: Option<TokenStream>,
        #[from(If)]
        pub(crate) if_cond: Option<TokenStream>,
        #[from(DerefNow)]
        pub(crate) deref_now: bool,
        #[from(PostProcessNow)]
        pub(crate) postprocess_now: bool,
        #[from(RestorePosition)]
        pub(crate) restore_position: bool,
        #[from(Try)]
        pub(crate) do_try: bool,
        #[from(Temp)]
        pub(crate) temp: bool,
        #[from(Assert)]
        pub(crate) assert: Vec<Assert>,
        #[from(PadBefore)]
        pub(crate) pad_before: Option<TokenStream>,
        #[from(PadAfter)]
        pub(crate) pad_after: Option<TokenStream>,
        #[from(AlignBefore)]
        pub(crate) align_before: Option<TokenStream>,
        #[from(AlignAfter)]
        pub(crate) align_after: Option<TokenStream>,
        #[from(SeekBefore)]
        pub(crate) seek_before: Option<TokenStream>,
        #[from(PadSizeTo)]
        pub(crate) pad_size_to: Option<TokenStream>,
        #[from(ParseWith)]
        pub(crate) parse_with: Option<TokenStream>,
    }
}

impl FromField for StructField {
    type In = syn::Field;

    fn from_field(field: &Self::In) -> ParseResult<Self> {
        Self::set_from_attrs(Self {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            endian: <_>::default(),
            map: <_>::default(),
            magic: <_>::default(),
            args: <_>::default(),
            ignore: <_>::default(),
            default: <_>::default(),
            calc: <_>::default(),
            count: <_>::default(),
            offset: <_>::default(),
            offset_after: <_>::default(),
            if_cond: <_>::default(),
            deref_now: <_>::default(),
            postprocess_now: <_>::default(),
            restore_position: <_>::default(),
            do_try: <_>::default(),
            temp: <_>::default(),
            assert: <_>::default(),
            pad_before: <_>::default(),
            pad_after: <_>::default(),
            align_before: <_>::default(),
            align_after: <_>::default(),
            seek_before: <_>::default(),
            pad_size_to: <_>::default(),
            parse_with: <_>::default(),
        }, &field.attrs)
    }
}

attr_struct! {
    #[from(UnitEnumFieldAttr)]
    #[derive(Clone, Debug)]
    pub(crate) struct UnitEnumField {
        pub(crate) ident: syn::Ident,
        // TODO: Magic and PreAssert seem to be conflicting preconditions, in
        // which case they should both be parsed into the same property instead
        // of being separated.
        #[from(Magic)]
        pub(crate) magic: Magic,
        #[from(PreAssert)]
        pub(crate) pre_assert: Vec<Assert>,
    }
}

impl FromField for UnitEnumField {
    type In = syn::Variant;

    fn from_field(field: &Self::In) -> ParseResult<Self> {
        Self::set_from_attrs(Self {
            ident: field.ident.clone(),
            magic: <_>::default(),
            pre_assert: <_>::default(),
        }, &field.attrs)
    }
}

#[derive(Clone, Debug)]
pub(crate) enum EnumVariant {
    Variant {
        ident: syn::Ident,
        options: Struct,
    },
    Unit(UnitEnumField),
}

impl EnumVariant {
    pub(crate) fn ident(&self) -> &syn::Ident {
        match self {
            EnumVariant::Variant { ident, .. } => &ident,
            EnumVariant::Unit(field) => &field.ident,
        }
    }
}

impl FromField for EnumVariant {
    type In = syn::Variant;

    fn from_field(variant: &Self::In) -> ParseResult<Self> {
        match variant.fields {
            syn::Fields::Named(_) | syn::Fields::Unnamed(_) =>
                Struct::from_input(&variant.attrs, variant.fields.iter())
                    .map(|options| Self::Variant {
                        ident: variant.ident.clone(),
                        options,
                    }),
            syn::Fields::Unit => UnitEnumField::from_field(variant).map(Self::Unit),
        }
    }
}
