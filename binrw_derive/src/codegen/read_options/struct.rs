use super::{get_assertions, get_magic, PreludeGenerator, ReadOptionsGenerator};
#[allow(clippy::wildcard_imports)]
use crate::codegen::sanitization::*;
use crate::parser::{Input, Map, PassedArgs, ReadMode, Struct, StructField};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(super) fn generate_unit_struct(
    input: &Input,
    name: Option<&Ident>,
    variant_ident: Option<&Ident>,
) -> TokenStream {
    let prelude = get_prelude(input, name);
    let return_type = get_return_type(variant_ident);
    quote! {
        #prelude
        Ok(#return_type)
    }
}

pub(super) fn generate_struct(input: &Input, name: Option<&Ident>, st: &Struct) -> TokenStream {
    StructGenerator::new(input, st)
        .read_fields(name)
        .add_assertions(core::iter::empty())
        .return_value(None)
        .finish()
}

pub(super) struct StructGenerator<'input> {
    input: &'input Input,
    st: &'input Struct,
    out: TokenStream,
}

impl<'input> StructGenerator<'input> {
    pub(super) fn new(input: &'input Input, st: &'input Struct) -> Self {
        Self {
            input,
            st,
            out: TokenStream::new(),
        }
    }

    pub(super) fn finish(self) -> TokenStream {
        self.out
    }

    pub(super) fn add_assertions(
        mut self,
        extra_assertions: impl Iterator<Item = TokenStream>,
    ) -> Self {
        let assertions = get_assertions(&self.st.assertions).chain(extra_assertions);
        let head = self.out;
        self.out = quote! {
            #head
            #(#assertions)*
        };

        self
    }

    pub(super) fn read_fields(mut self, name: Option<&Ident>) -> Self {
        let prelude = get_prelude(self.input, name);
        let read_fields = self.st.fields.iter().map(|field| generate_field(field));
        let after_parse = {
            let after_parse = self
                .st
                .fields
                .iter()
                .map(|field| generate_after_parse(field));
            wrap_save_restore(quote!(#(#after_parse)*))
        };
        self.out = quote! {
            #prelude
            #(#read_fields)*
            #after_parse
        };

        self
    }

    pub(super) fn return_value(mut self, variant_ident: Option<&Ident>) -> Self {
        let out_names = self.st.iter_permanent_idents();
        let return_type = get_return_type(variant_ident);
        let return_value = if self.st.is_tuple() {
            quote! { #return_type(#(#out_names),*) }
        } else {
            quote! { #return_type { #(#out_names),* } }
        };

        let head = self.out;
        self.out = quote! {
            #head
            Ok(#return_value)
        };

        self
    }
}

fn generate_after_parse(field: &StructField) -> Option<TokenStream> {
    get_after_parse_handler(&field).map(|after_parse_fn| {
        let (args_var, options_var) = make_field_vars(field);
        AfterParseCallGenerator::new(field)
            .get_value_from_ident()
            .call_after_parse(after_parse_fn, &options_var, &args_var)
            .prefix_offset_options(&options_var)
            .finish()
    })
}

fn generate_field(field: &StructField) -> TokenStream {
    FieldGenerator::new(&field)
        .read_value()
        .try_conversion()
        .map_value()
        .deref_now()
        .wrap_seek()
        .wrap_condition()
        .assign_to_var()
        .append_assertions()
        .wrap_restore_position()
        .prefix_magic()
        .prefix_args_and_options()
        .finish()
}

struct AfterParseCallGenerator<'field> {
    field: &'field StructField,
    out: TokenStream,
}

impl<'field> AfterParseCallGenerator<'field> {
    fn new(field: &'field StructField) -> Self {
        Self {
            field,
            out: TokenStream::new(),
        }
    }

    fn call_after_parse(
        mut self,
        after_parse_fn: IdentStr,
        options_var: &Option<Ident>,
        args_var: &Option<Ident>,
    ) -> Self {
        let value = self.out;
        let options_var = options_var.as_ref().expect(
            "called `AfterParseCallGenerator::call_after_parse` but no `options_var` was generated",
        );
        let args_arg = get_args_argument(args_var);
        self.out = quote! {
            #after_parse_fn(#value, #READER, #options_var, #args_arg)?;
        };

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }

    fn get_value_from_ident(mut self) -> Self {
        let ident = &self.field.ident;
        self.out = quote! { &mut #ident };

        self
    }

    fn get_value_from_temp(mut self) -> Self {
        self.out = quote! { &mut #TEMP };

        self
    }

    fn prefix_offset_options(mut self, options_var: &Option<Ident>) -> Self {
        if let (Some(options_var), Some(offset)) = (options_var, &self.field.offset_after) {
            let tail = self.out;
            let offset = offset.as_ref();
            self.out = quote! {
                let #options_var = &{
                    let mut #TEMP = *#options_var;
                    #TEMP.offset = #offset;
                    #TEMP
                };
                #tail
            };
        }

        self
    }
}

struct FieldGenerator<'field> {
    field: &'field StructField,
    out: TokenStream,
    args_var: Option<Ident>,
    options_var: Option<Ident>,
}

impl<'field> FieldGenerator<'field> {
    fn new(field: &'field StructField) -> Self {
        let (args_var, options_var) = make_field_vars(field);

        Self {
            field,
            out: TokenStream::new(),
            args_var,
            options_var,
        }
    }

    fn append_assertions(mut self) -> Self {
        let assertions = get_assertions(&self.field.assertions);
        let head = self.out;
        self.out = quote! {
            #head
            #(#assertions)*
        };

        self
    }

    fn assign_to_var(mut self) -> Self {
        let ident = &self.field.ident;
        let ty = &self.field.ty;
        let value = self.out;
        self.out = quote! { let mut #ident: #ty = #value; };

        self
    }

    fn deref_now(mut self) -> Self {
        if self.field.should_use_after_parse() {
            return self;
        }

        if let Some(after_parse) = get_after_parse_handler(&self.field) {
            let after_parse = AfterParseCallGenerator::new(self.field)
                .get_value_from_temp()
                .call_after_parse(after_parse, &self.options_var, &self.args_var)
                .finish();

            let value = self.out;
            self.out = quote! {{
                let mut #TEMP = #value;
                #after_parse
                #TEMP
            }};
        }

        self
    }

    fn finish(self) -> TokenStream {
        self.out
    }

    fn map_value(mut self) -> Self {
        let ty = &self.field.ty;

        self.out = match &self.field.map {
            Map::None => return self,
            Map::Map(map) => {
                let value = self.out;
                quote! { (#COERCE_FN::<#ty, _, _>(#map))(#value) }
            }
            Map::Try(try_map) => {
                // TODO: Position should always just be saved once for a field if used
                let value = self.out;
                let map_err = super::get_map_err(SAVED_POSITION);
                quote! {{
                    let #SAVED_POSITION = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))?;

                    (#COERCE_FN::<::core::result::Result<#ty, _>, _, _>(#try_map))(#value)#map_err?
                }}
            }
        };

        self
    }

    fn prefix_args_and_options(mut self) -> Self {
        let args = self.args_var.as_ref().map(|args_var| {
            let args = get_passed_args(&self.field);
            quote! {
                let #args_var = #args;
            }
        });

        let options = self.options_var.as_ref().map(|options_var| {
            ReadOptionsGenerator::new(options_var)
                .endian(&self.field.endian)
                .offset(&self.field.offset)
                .count(&self.field.count)
                .finish()
        });

        let tail = self.out;

        self.out = quote! {
            #args
            #options
            #tail
        };

        self
    }

    fn prefix_magic(mut self) -> Self {
        if let Some(options_var) = &self.options_var {
            if let Some(magic) = get_magic(&self.field.magic, options_var) {
                let tail = self.out;
                self.out = quote! {
                    #magic
                    #tail
                };
            }
        }

        self
    }

    fn read_value(mut self) -> Self {
        self.out = match &self.field.read_mode {
            ReadMode::Default => quote! { <_>::default() },
            ReadMode::Calc(calc) => quote! { #calc },
            ReadMode::Normal | ReadMode::ParseWith(_) => {
                let read_method = if let ReadMode::ParseWith(parser) = &self.field.read_mode {
                    parser.clone()
                } else {
                    quote! { #READ_METHOD }
                };

                let args_arg = get_args_argument(&self.args_var);
                let options_var = &self.options_var;

                quote! {
                    #read_method(#READER, #options_var, #args_arg)
                }
            }
        };

        self
    }

    fn try_conversion(mut self) -> Self {
        if self.field.generated_value() {
            if self.field.do_try {
                let value = self.out;
                self.out = quote! { Some(#value) };
            }
        } else {
            let result = self.out;
            self.out = if self.field.do_try {
                quote! { #result.ok() }
            } else {
                quote! { #result? }
            };
        }

        self
    }

    fn wrap_condition(mut self) -> Self {
        if let Some(cond) = &self.field.if_cond {
            let condition = &cond.condition;
            let consequent = self.out;
            let alternate = &cond.alternate;
            self.out = quote! {
                if #condition {
                    #consequent
                } else {
                    #alternate
                }
            };
        }

        self
    }

    fn wrap_restore_position(mut self) -> Self {
        if self.field.restore_position {
            self.out = wrap_save_restore(self.out);
        }

        self
    }

    fn wrap_seek(mut self) -> Self {
        let seek_before = generate_seek_before(self.field);
        let seek_after = generate_seek_after(self.field);
        if !seek_before.is_empty() || !seek_after.is_empty() {
            let value = self.out;
            self.out = quote! {{
                #seek_before
                let #TEMP = #value;
                #seek_after
                #TEMP
            }};
        }

        self
    }
}

fn get_args_argument(args_var: &Option<Ident>) -> TokenStream {
    args_var.as_ref().map_or_else(
        || quote! { <_>::default() },
        |args_var| quote! { #args_var.clone() },
    )
}

fn get_passed_args(field: &StructField) -> Option<TokenStream> {
    let args = &field.args;
    match args {
        PassedArgs::Named(fields) => Some({
            if fields.is_empty() {
                return None;
            }
            let ty = &field.ty;
            let added_fields = fields.iter().map(|(name, expr)| quote!( .#name( #expr ) ));
            quote!(
                <#ty as #TRAIT_NAME>::Args::builder()
                    #(
                        #added_fields
                     )*
                    .finalize()
            )
        }),
        PassedArgs::List(list) => Some(quote! { (#(#list,)*) }),
        PassedArgs::Tuple(tuple) => Some(tuple.clone()),
        PassedArgs::None => None,
    }
}

fn get_prelude(input: &Input, name: Option<&Ident>) -> TokenStream {
    PreludeGenerator::new(input)
        .add_imports(name)
        .add_options()
        .add_magic_pre_assertion()
        .finish()
}

fn generate_seek_after(field: &StructField) -> TokenStream {
    let pad_size_to = field.pad_size_to.as_ref().map(|pad| {
        quote! {{
            let pad = (#pad) as i64;
            let size = (#SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))? - #POS) as i64;
            if size < pad {
                #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(pad - size))?;
            }
        }}
    });
    let pad_after = field.pad_after.as_ref().map(map_pad);
    let align_after = field.align_after.as_ref().map(map_align);

    quote! {
        #pad_size_to
        #pad_after
        #align_after
    }
}

fn generate_seek_before(field: &StructField) -> TokenStream {
    let seek_before = field.seek_before.as_ref().map(|seek| {
        quote! {
            #SEEK_TRAIT::seek(#READER, #seek)?;
        }
    });
    let pad_before = field.pad_before.as_ref().map(map_pad);
    let align_before = field.align_before.as_ref().map(map_align);
    let pad_size_to_before = field.pad_size_to.as_ref().map(|_| {
        quote! {
            let #POS = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))?;
        }
    });

    quote! {
        #seek_before
        #pad_before
        #align_before
        #pad_size_to_before
    }
}

fn get_after_parse_handler(field: &StructField) -> Option<IdentStr> {
    if !field.can_call_after_parse() {
        None
    } else if field.do_try {
        Some(TRY_AFTER_PARSE)
    } else {
        Some(AFTER_PARSE)
    }
}

fn get_return_type(variant_ident: Option<&Ident>) -> TokenStream {
    variant_ident.map_or_else(|| quote! { Self }, |ident| quote! { Self::#ident })
}

fn make_field_vars(field: &StructField) -> (Option<Ident>, Option<Ident>) {
    let args_var = if field.args.is_some() {
        Some(make_ident(&field.ident, "args"))
    } else {
        None
    };

    let options_var = if field.needs_options() {
        Some(make_ident(&field.ident, "options"))
    } else {
        None
    };

    (args_var, options_var)
}

fn map_align(align: &TokenStream) -> TokenStream {
    quote! {{
        let align = (#align) as i64;
        let pos = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))? as i64;
        #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current((align - (pos % align)) % align))?;
    }}
}

fn map_pad(pad: &TokenStream) -> TokenStream {
    quote! {
        #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(#pad))?;
    }
}

fn wrap_save_restore(value: TokenStream) -> TokenStream {
    if value.is_empty() {
        value
    } else {
        quote! {
            let #SAVED_POSITION = #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Current(0))?;
            #value
            #SEEK_TRAIT::seek(#READER, #SEEK_FROM::Start(#SAVED_POSITION))?;
        }
    }
}
