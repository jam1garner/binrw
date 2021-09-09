use proc_macro2::{Ident, TokenStream};
use crate::parser::write::{Input, UnitOnlyEnum};
//use quote::quote;

pub(crate) fn generate_unit_enum(
    _input: &Input,
    _name: Option<&Ident>,
    _enm: &UnitOnlyEnum
) -> TokenStream {
    todo!()
}
