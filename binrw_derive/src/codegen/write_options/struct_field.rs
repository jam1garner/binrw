use crate::parser::write::StructField;
use crate::parser::{CondEndian, PassedArgs, WriteMode};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::Ident;

#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;

pub(crate) fn write_field(field: &StructField) -> TokenStream {
    StructFieldGenerator::new(field)
        .write_field()
        .prefix_args()
        .prefix_write_fn()
        .prefix_magic()
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

    fn prefix_write_fn(mut self) -> Self {
        let write_fn = match &self.field.write_mode {
            WriteMode::Normal | WriteMode::Calc(_) => quote! { #WRITE_METHOD },
            WriteMode::Ignore => quote! { |_, _, _, _| { Ok(()) } },
            WriteMode::WriteWith(write_fn) => write_fn.clone(),
        };

        let out = &self.out;
        self.out = quote! {
            let #WRITE_FUNCTION = #WRITE_FN_TYPE_HINT(#write_fn);
            #out
        };

        self
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

        self.out = if let WriteMode::Ignore = &self.field.write_mode {
            quote! {
                #initialize
            }
        } else {
            quote! {
                #initialize

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

        let args_val = if let Some(args) = get_passed_args(self.field) {
            args
        } else {
            quote! { () }
        };

        let out = &self.out;
        self.out = match self.field.write_mode {
            WriteMode::Normal => {
                let ty = &self.field.ty;
                quote! {
                    let #args: <#ty as #BINWRITE_TRAIT>::Args = #args_val;
                    #out
                }
            },
            WriteMode::Ignore => if self.field.args.is_some() {
                let name = &self.field.ident;
                quote_spanned! { self.field.ident.span() =>
                    compile_error!(concat!(
                        "Cannot pass arguments to the field '",
                        stringify!(#name),
                        "'  as it is uses the 'ignore' directive"
                    ));
                    #out
                }
            } else {
                quote! {
                    let #args = ();
                    #out
                }
            },
            WriteMode::Calc(_) => if self.field.args.is_some() {
                let name = &self.field.ident;
                quote_spanned! { self.field.ident.span() =>
                    compile_error!(concat!(
                        "Cannot pass arguments to the field '",
                        stringify!(#name),
                        "'  as it is uses the 'calc' directive"
                    ));
                    #out
                }
            } else {
                quote! {
                    let #args = ();
                    #out
                }
            },
            WriteMode::WriteWith(_) => {
                let ty = &self.field.ty;
                quote! {
                    let #args = #WRITE_WITH_ARGS_TYPE_HINT::<#ty, W, _, _>(
                        #WRITE_FUNCTION, #args_val
                    );
                    #out
                }
            },
        };

        self
    }

    fn prefix_magic(mut self) -> Self {
        if let Some(magic) = &self.field.magic {
            let magic = magic.match_value();
            let specify_endian = self.specify_endian();
            let out = self.out;
            self.out = quote! {
                #WRITE_METHOD (
                    &#magic,
                    #WRITER,
                    &#OPT #specify_endian,
                    ()
                )?;

                #out
            };
        }

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }
}

fn get_passed_args(field: &StructField) -> Option<TokenStream> {
    let args = &field.args;
    match args {
        PassedArgs::Named(fields) => Some(if let Some(count) = &field.count {
            quote! {
                #ARGS_MACRO! { count: ((#count) as usize) #(, #fields)* }
            }
        } else {
            quote! {
                #ARGS_MACRO! { #(#fields),* }
            }
        }),
        PassedArgs::List(list) => Some(quote! { (#(#list,)*) }),
        PassedArgs::Tuple(tuple) => Some(tuple.clone()),
        PassedArgs::None => field
            .count
            .as_ref()
            .map(|count| quote! { #ARGS_MACRO! { count: ((#count) as usize) }}),
    }
}
