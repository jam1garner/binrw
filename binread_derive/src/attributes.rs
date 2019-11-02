use syn::*;
use proc_macro2::*;
use quote::quote;
use crate::binwrite_endian::Endian;
use std::result::Result;

#[derive(Clone, Debug)]
pub enum AttrSetting {
    Ignore,
    Endian(Endian),
    With(TokenStream),
    Preprocessor(TokenStream),
    AlignBefore(usize),
    AlignAfter(usize),
    PadBefore(usize),
    PadAfter(usize),
}

#[derive(Debug, Clone)]
pub struct SpanError {
    pub span: Span,
    pub error: String,
}

impl SpanError {
    pub fn new(span: Span, error: String) -> Self {
        SpanError {
            span,
            error: error
        }
    }
}

fn get_literal_from_token(span: Span, token: Option<TokenTree>) -> Result<usize, SpanError> {
    match token {
        Some(TokenTree::Literal(lit)) => {
            match Lit::new(lit.clone()) {
                Lit::Int(lit) =>
                    usize::from_str_radix(
                        lit.base10_digits(),
                        10
                    ).or(
                        Err(SpanError::new(
                            lit.span(),
                            "Invalid digit".into()
                        ))
                    ),
                _ => Err(SpanError::new(
                        lit.span(),
                        "Invalid literal type, expected Integer".into()
                    ))
            }
        }
        _ => Err(SpanError::new(
                span,
                "Invalid contents of pad".into()
            ))
    }
}

impl AttrSetting {
    fn try_from_attr_type(attr: &AttrType) -> Result<Self, SpanError> {
        match attr {
            AttrType::EnableDisable {
                id, enable
            } => {
                if *enable {
                    match &id.to_string()[..] {
                        "big" => {
                            Ok(Self::Endian(Endian::Big))
                        }
                        "little" => {
                            Ok(Self::Endian(Endian::Little))
                        }
                        "native" => {
                            Ok(Self::Endian(Endian::Native))
                        }
                        "ignore" => {
                            Ok(Self::Ignore)
                        }
                        func @ _ => {
                            Ok(
                                Self::With(
                                    Self::get_function_path(func)
                                        .ok_or(
                                            SpanError::new(
                                                id.span(),
                                                format!("Property not supported")
                                            ))?
                                )
                            )
                        }
                    }
                } else {
                    Err(SpanError::new(
                        id.span(),
                        format!("Disabling of {} not supported", id.to_string())
                    ))
                }
            }
            AttrType::Function {
                id, stream
            } => {
                match &id.to_string()[..] {
                    "with" => {
                        Ok(Self::With(stream.clone()))
                    }
                    "preprocessor" => {
                        Ok(Self::Preprocessor(stream.clone()))
                    }
                    name @ "pad" | name @ "pad_after" => {
                        let token = stream.clone().into_iter().nth(0);

                        let pad = get_literal_from_token(id.span(), token)?;

                        Ok(match name {
                            "pad" => Self::PadBefore(pad),
                            "pad_after" => Self::PadAfter(pad),
                            _ => unreachable!()
                        })
                    }
                    name @ "align" | name @ "align_after" => {
                        let token = stream.clone().into_iter().nth(0);

                        let pad = match token {
                            Some(TokenTree::Literal(lit)) => {
                                match Lit::new(lit.clone()) {
                                    Lit::Int(lit) =>
                                        usize::from_str_radix(
                                            lit.base10_digits(),
                                            10
                                        ).or(
                                            Err(SpanError::new(
                                                lit.span(),
                                                "Invalid digit".into()
                                            ))
                                        )?,
                                    _ => Err(SpanError::new(
                                            lit.span(),
                                            "Invalid literal type, expected Integer".into()
                                        ))?
                                }
                            }
                            _ => Err(SpanError::new(
                                    id.span(),
                                    "Invalid contents of pad".into()
                                ))?
                        };

                        Ok(match name {
                            "align" => Self::AlignBefore(pad),
                            "align_after" => Self::AlignAfter(pad),
                            _ => unreachable!()
                        })
                    }
                    name @ _ => Err(SpanError::new(
                            id.span(),
                            format!("Function \"{}\" not supported", name)
                        ))?
                }
            }
            AttrType::Assignment {
                id, ..
            } => {
                Err(SpanError::new(
                    id.span(),
                    format!("Setting \"{}\" not supported", id.to_string())
                ))?
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
    type Item = Result<(Ident, Vec<AttrSetting>), SpanError>;

    fn next(&mut self) -> Option<Self::Item> {
        let (id, attrs) = self.items.get(self.current)?;
        self.current += 1;


        let attrs: Result<(Ident, Vec<AttrSetting>), SpanError> = try {
            (
                id.clone(), 
                attrs
                    .iter()
                    .map(|attr| parse_attr_setting_group(attr.tokens.clone()))
                    .collect::<Result<Vec<_>, SpanError>>()?
                    .into_iter()
                    .flatten()
                    .collect()
            )
        };


        Some(attrs)
    }
}

pub fn parse_attr_setting_group(attr: TokenStream) -> Result<Vec<AttrSetting>, SpanError> {
    let attr = attr.into_iter().nth(0).unwrap();
    match attr {
        TokenTree::Group(group) => {
            if let Delimiter::Parenthesis = group.delimiter() {
                comma_split_token_stream(group.stream())
                    .iter()
                    .map(AttrType::try_parse)
                    .collect::<Option<Vec<_>>>()
                    .expect("Failed to convert to AttrType")
                    .iter()
                    .map(AttrSetting::try_from_attr_type)
                    .collect::<Result<Vec<_>, SpanError>>()
            } else {
                Err(SpanError::new(
                    group.span(),
                    "Unsupported delimeter. Use parenthesis.".into()
                ))
            }
        },
        _ => {
            Err(SpanError::new(
                attr.span(),
                "Unsupported attribute formatting.".into()
            ))
        }
    }

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

pub fn filter_single_attrs(attrs: &Vec<Attribute>) -> Option<Vec<Attribute>> {
    let attrs = attrs
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
    if attrs.is_empty() {
        None
    } else {
        Some(attrs)
    }
}

fn filter_attrs(fields: Fields) -> Vec<(Ident, Vec<Attribute>)> {
    fields
        .iter()
        .filter_map(|field|{
            Some(
                (
                    field.ident.clone()?,
                    filter_single_attrs(&field.attrs).unwrap_or_default()
                )
            )
        })
        .collect::<Vec<(Ident, Vec<Attribute>)>>()
}
