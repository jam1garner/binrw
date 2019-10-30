use syn::*;
use proc_macro2::*;
use quote::quote;
use binwrite::Endian;

pub enum AttrSetting {
    Endian(Endian),
    With(TokenStream),
    PadBefore(usize),
    PadAfter(usize),
}

impl AttrSetting {
    fn try_from_attr_type(attr: &AttrType) -> Option<Self> {
        match attr {
            AttrType::EnableDisable {
                id, enable
            } => {
                if *enable {
                    match &id.to_string()[..] {
                        "big" => {
                            Some(Self::Endian(Endian::Big))
                        }
                        "little" => {
                            Some(Self::Endian(Endian::Little))
                        }
                        "native" => {
                            Some(Self::Endian(Endian::Native))
                        }
                        func @ _ => {
                            Some(
                                Self::With(
                                    Self::get_function_path(func)?
                                )
                            )
                        }
                    }
                } else {
                    None
                }
            }
            AttrType::Function {
                id, stream
            } => {
                match &id.to_string()[..] {
                    "with" => {
                        Some(Self::With(stream.clone()))
                    }
                    name @ "pad" | name @ "pad_after" => {
                        let token = stream.clone().into_iter().nth(0);

                        let pad = match token {
                            Some(TokenTree::Literal(lit)) => {
                                match Lit::new(lit) {
                                    Lit::Int(lit) => usize::from_str_radix(
                                                        lit.base10_digits(),
                                                        10
                                                    ).ok()?,
                                    _ => None?
                                }
                            }
                            _ => None?
                        };

                        Some(match name {
                            "pad" => Self::PadBefore(pad),
                            "pad_after" => Self::PadAfter(pad),
                            _ => panic!()
                        })
                    }
                    _ => None
                }
            }
            _ => {
                None
            }
        }
    }

    fn get_function_path(name: &str) -> Option<TokenStream> {
        Some(
            match name {
                "cstr" => quote!{::binwrite::writers::null_terminated_string},
                "utf16" => quote!{::binwrite::writers::utf16_null_string},
                "utf16_null" => quote!{::binwrite::writers::utf16_string},
                _ => None?
            }
        )
    }
}

pub struct Attributes {
    items: Vec<(Ident, Vec<Attribute>)>,
    current: usize,
}

impl Attributes {
    pub fn from_fields(fields: Fields) -> Self {
        Self {
            items: filter_attrs(fields),
            current: 0
        }
    }
}

impl Iterator for Attributes {
    type Item = (Ident, Vec<AttrSetting>);

    fn next(&mut self) -> Option<Self::Item> {
        let (id, attrs) = self.items.get(self.current)?;
        self.current += 1;


        let attrs = attrs
                    .iter()
                    .map(|attr|{
                        let attr = attr.tokens.clone();

                        parse_attr_setting_group(attr)
                    })
                    .flatten()
                    .collect();


        Some((id.clone(), attrs))
    }
}

fn parse_attr_setting_group(attr: TokenStream) -> Vec<AttrSetting> {
    let attr = attr.into_iter().nth(0).unwrap();
    match attr {
        TokenTree::Group(group) => {
            if let Delimiter::Parenthesis = group.delimiter() {
                return comma_split_token_stream(group.stream())
                        .iter()
                        .map(AttrType::try_parse)
                        .collect::<Option<Vec<_>>>()
                        .expect("Failed to convert to AttrType")
                        .iter()
                        .map(AttrSetting::try_from_attr_type)
                        .collect::<Option<Vec<_>>>()
                        .expect("Failed to convert to AttrSetting")
            }
        },
        _ => {}
    }

    vec![]
}

fn comma_split_token_stream(tokens: TokenStream) -> Vec<Vec<TokenTree>> {
    tokens.clone()
          .into_iter()
          .collect::<Vec<_>>()[..]
          .split(|token| {
              match token {
                  TokenTree::Punct(punct) => punct.as_char() == ',',
                  _ => false
              }
          })
          .map(Vec::from)
          .collect()
}

#[derive(Debug)]
enum AttrType {
    Function {
        id: Ident,
        stream: TokenStream,
    },
    Assignment {
        id: Ident,
        value: Literal,
    },
    EnableDisable {
        id: Ident,
        enable: bool,
    }
}

impl AttrType {
    pub fn try_parse(tokens: &Vec<TokenTree>) -> Option<Self> {
        try_parse_func_attr(tokens)
            .or_else(||try_parse_assign_attr(tokens))
            .or_else(||try_parse_enable_attr(tokens))
    }
}

fn try_parse_func_attr(input: &Vec<TokenTree>) -> Option<AttrType> {
    if input.len() == 2 {
        if let TokenTree::Ident(id) = &input[0] {
            if let TokenTree::Group(group) = &input[1] {
                if let Delimiter::Parenthesis = group.delimiter() {
                    return Some(AttrType::Function {
                        id: id.clone(),
                        stream: group.stream().clone()
                    });
                }
            }
        }
    }

    None
}

fn try_parse_assign_attr(input: &Vec<TokenTree>) -> Option<AttrType> {
    if input.len() == 3 {
        if let TokenTree::Ident(id) = &input[0] {
            if let TokenTree::Punct(punct) = &input[1] {
                if let TokenTree::Literal(lit) = &input[2] {
                    if punct.as_char() == '=' {
                        return Some(AttrType::Assignment{
                            id: id.clone(),
                            value: lit.clone()
                        });
                    }
                }
            }
        }
    }

    None
}

fn try_parse_enable_attr(input: &Vec<TokenTree>) -> Option<AttrType> {
    if input.len() == 2 {
        if let TokenTree::Punct(punct) = &input[0] {
            if let TokenTree::Ident(id) = &input[1] {
                if punct.as_char() == '!' {
                    return Some(AttrType::EnableDisable{
                        id: id.clone(),
                        enable: false
                    });
                }
            }
        }
    } else if input.len() == 1 {
        if let TokenTree::Ident(id) = &input[0] {
            return Some(AttrType::EnableDisable{
                id: id.clone(),
                enable: true
            });
        }
    }

    None
}


fn filter_attrs(fields: Fields) -> Vec<(Ident, Vec<Attribute>)> {
    fields
        .iter()
        .filter_map(|field|{
            Some(
                (
                    field.ident.clone()?,
                    {
                        let a = field.attrs
                            .iter()
                            .filter_map(|attr|
                                if let Some(ident) = attr.path.get_ident() {
                                    if ident.to_string() == "binwrite" {
                                        Some(attr.clone())
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                             )
                             .collect::<Vec<_>>();
                        if a.is_empty() {
                            None
                        } else {
                            Some(a)
                        }
                    }.unwrap_or_default()
                )
            )
        })
        .collect::<Vec<(Ident, Vec<Attribute>)>>()
}
