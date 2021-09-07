use crate::parser::write::{Input, Struct, StructField};
use crate::parser::{CondEndian, WriteMode};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;

pub(super) fn generate_struct(input: &Input, name: Option<&Ident>, st: &Struct) -> TokenStream {
    StructGenerator::new(input, st, name)
        .write_fields()
        .prefix_endian()
        .prefix_borrow_fields()
        .finish()
}

struct StructGenerator<'input> {
    #[allow(dead_code)]
    input: &'input Input,
    st: &'input Struct,
    name: Option<&'input Ident>,
    out: TokenStream,
}

impl<'input> StructGenerator<'input> {
    fn new(input: &'input Input, st: &'input Struct, name: Option<&'input Ident>) -> Self {
        Self {
            input,
            st,
            name,
            out: TokenStream::new(),
        }
    }

    fn write_fields(mut self) -> Self {
        let write_fields = self.st.fields.iter().map(write_field);

        self.out = quote! {
            #(#write_fields)*
        };

        self
    }

    fn prefix_borrow_fields(mut self) -> Self {
        let borrow_fields = self.name.as_ref().map(|name| {
            let pattern = match &self.input {
                Input::Struct(input) => {
                    let fields = self.st.iter_permanent_idents();

                    if input.is_tuple() {
                        quote! {
                            #name (#(ref #fields),*)
                        }
                    } else {
                        quote! {
                            #name { #(ref #fields),* }
                        }
                    }
                }
                Input::UnitStruct(_) => quote! { _ },
                Input::Enum(_) | Input::UnitOnlyEnum(_) => unreachable!(),
            };

            Some(quote! {
                let #pattern = self;
            })
        });

        let out = self.out;
        self.out = quote! {
            #borrow_fields
            #out
        };

        self
    }

    fn prefix_endian(mut self) -> Self {
        let out = self.out;
        let set_endian = match &self.st.endian {
            CondEndian::Inherited => None,
            CondEndian::Fixed(endian) => Some({
                let endian = endian.as_binrw_endian();
                quote! {
                    let #OPT = #OPT.clone().with_endian(#endian);
                    let #OPT = &#OPT;
                }
            }),
            CondEndian::Cond(endian, cond) => Some({
                let else_endian = endian.flipped().as_binrw_endian();
                let endian = endian.as_binrw_endian();
                quote! {
                    let #OPT = #OPT.clone().with_endian(if #cond { #endian } else { #else_endian });
                    let #OPT = &#OPT;
                }
            }),
        };

        self.out = quote! {
            #set_endian
            #out
        };

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }
}

fn write_field(field: &StructField) -> TokenStream {
    StructFieldGenerator::new(field)
        .write_field()
        .prefix_args()
        .finish()
}

struct StructFieldGenerator<'input> {
    field: &'input StructField,
    out: TokenStream,
}

impl<'a> StructFieldGenerator<'a> {
    fn new(field: &'a StructField) -> Self {
        Self {
            field,
            out: TokenStream::new(),
        }
    }

    fn args_ident(&self) -> Ident {
        make_ident(&self.field.ident, "args")
    }

    fn specify_endian(&self) -> Option<TokenStream> {
        match &self.field.endian {
            CondEndian::Inherited => None,
            CondEndian::Fixed(endian) => Some({
                let endian = endian.as_binrw_endian();
                quote! {
                    .clone().with_endian(#endian)
                }
            }),
            CondEndian::Cond(endian, cond) => Some({
                let else_endian = endian.flipped().as_binrw_endian();
                let endian = endian.as_binrw_endian();
                quote! {
                    .clone().with_endian(if #cond { #endian } else { #else_endian })
                }
            }),
        }
    }

    fn write_field(mut self) -> Self {
        let name = &self.field.ident;
        let args = self.args_ident();
        let specify_endian = self.specify_endian();

        let initialize = if let WriteMode::Calc(expr) = &self.field.write_mode {
            Some({
                let ty = &self.field.ty;
                quote! {
                    let #name: #ty = #expr;
                }
            })
        } else {
            None
        };

        let write_fn = match &self.field.write_mode {
            WriteMode::Normal | WriteMode::Calc(_) => quote! { #WRITE_METHOD },
            WriteMode::Ignore => quote! { |_, _, _, _| { Ok(()) } },
            WriteMode::WriteWith(write_fn) => write_fn.clone(),
        };

        let set_write_fn = quote! {
            let #WRITE_FUNCTION = #WRITE_FN_TYPE_HINT(#write_fn);
        };

        self.out = if let WriteMode::Ignore = &self.field.write_mode {
            quote! {
                #initialize
            }
        } else {
            quote! {
                #initialize
                #set_write_fn

                #WRITE_FUNCTION (
                    &#name,
                    #WRITER,
                    &#OPT#specify_endian,
                    #args
                )?;
            }
        };

        self
    }

    fn prefix_args(mut self) -> Self {
        let args = self.args_ident();

        let out = &self.out;
        self.out = quote! {
            // TODO: add support for passing arguments
            let #args = ();
            #out
        };

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }
}