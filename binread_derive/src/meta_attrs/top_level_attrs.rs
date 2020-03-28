use super::*;
use super::parser::{TopLevelAttr, MetaAttrList, BinreadAttribute};
use crate::CompileError;

#[derive(FromDeriveInput, FromVariant, Debug, Clone)]
#[darling(attributes(br, binread))]
pub struct TopLevelAttrs {
    // ======================
    //  Top-Only Attributes
    // ======================
    #[darling(default)]
    pub import: Imports, // Vec<Ident>, Vec<Type>

    #[darling(default)]
    pub return_all_errors: SpannedValue<bool>,

    #[darling(default)]
    pub return_unexpected_error: SpannedValue<bool>,

    // ======================
    //  All-level attributes
    // ======================
    // endian
    #[darling(default)]
    pub little: SpannedValue<bool>,
    #[darling(default)]
    pub big: SpannedValue<bool>,
    
    // assertions/error handling
    #[darling(multiple, map = "to_assert")]
    pub assert: Vec<Assert>,
    
    #[darling(default, map = "to_tokens")]
    pub magic: Option<TokenStream>,
}

impl TopLevelAttrs {
    pub fn finalize(self) -> Result<Self, SpanError> {
        if *self.big && *self.little {
            SpanError::err(
                self.big.span().join(self.little.span()).unwrap(),
                "Cannot set endian to both big and little endian"
            )?;
        }

        Ok(self)
    }

    pub fn from_derive_input(input: &syn::DeriveInput) -> Result<Self, CompileError> {
        let attrs: Vec<TopLevelAttr> =
            input.attrs
                .iter()
                .map(tlas_from_attribute)
                .collect::<Result<Vec<TlaList>, CompileError>>()?
                .into_iter()
                .map(|x| x.0.into_iter())
                .flatten()
                .collect();

        verify(&attrs)?;
    }
}

type TlaList = MetaAttrList<TopLevelAttr>;

fn tlas_from_attribute(attr: &syn::Attribute) -> Result<TlaList, CompileError> {
    Ok(syn::parse2(attr.tokens.clone())?)
}

macro_rules! unwrap_tla_list {
    () => {
        
    };
}

fn verify(attrs: &[TopLevelAttr]) -> Result<(), CompileError> {
    Ok(())
}
