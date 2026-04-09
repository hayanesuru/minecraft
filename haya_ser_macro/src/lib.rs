#![recursion_limit = "128"]
#![no_std]

extern crate alloc;

mod deserialize;
mod serialize;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use proc_macro::TokenStream;
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::{DeriveInput, parse_macro_input};

const V21MAX: usize = 0x1FFFFF;
const V7MAX: usize = 0x7F;

mod kw {
    syn::custom_keyword!(varint);
    syn::custom_keyword!(filter);
    syn::custom_keyword!(header);
    syn::custom_keyword!(camel_case);
}

#[derive(Default)]
struct Attrs {
    varint: bool,
    header: Option<syn::Path>,
    camel_case: bool,
}

impl syn::parse::Parse for Attrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attr = Self::default();
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::varint) {
                let _: kw::varint = input.parse()?;
                attr.varint = true;
            } else if lookahead.peek(kw::header) {
                let _: kw::header = input.parse()?;
                let _: syn::Token![=] = input.parse()?;
                attr.header = Some(input.parse::<syn::Path>()?);
            } else if lookahead.peek(kw::camel_case) {
                let _: kw::camel_case = input.parse()?;
                attr.camel_case = true;
            } else {
                return Err(lookahead.error());
            }

            if !input.is_empty() {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(attr)
    }
}

fn crate_name(input: &DeriveInput) -> Result<(Attrs, syn::Path), syn::Error> {
    let mut find = Attrs::default();
    let mut flag = false;
    for attr in input.attrs.iter() {
        if attr.path().is_ident("mser") {
            if flag {
                return Err(syn::Error::new_spanned(attr, "multiple `mser` attributes"));
            }
            flag = true;
            find = attr.parse_args()?;
        }
    }
    Ok((
        find,
        syn::Ident::new("mser", proc_macro2::Span::call_site()).into(),
    ))
}

#[proc_macro_derive(Serialize, attributes(mser))]
pub fn serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let (attr, cratename) = match crate_name(&input) {
        Ok(cratename) => cratename,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let x = match input.data {
        syn::Data::Struct(_) => serialize::serialize_struct(input, cratename),
        syn::Data::Enum(_) => serialize::serialize_enum(input, cratename, attr),
        syn::Data::Union(_) => Err(syn::Error::new_spanned(input, "unions are not supported")),
    };
    match x {
        Ok(x) => x.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Deserialize, attributes(mser))]
pub fn deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let (attr, cratename) = match crate_name(&input) {
        Ok(cratename) => cratename,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let x = match input.data {
        syn::Data::Struct(_) => deserialize::deserialize_struct(input, cratename),
        syn::Data::Enum(_) => deserialize::deserialize_enum(input, cratename, attr),
        syn::Data::Union(_) => Err(syn::Error::new_spanned(input, "unions are not supported")),
    };
    match x {
        Ok(x) => x.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

struct FieldAttrs {
    pub filter: Option<syn::Path>,
    pub varint: bool,
}

impl syn::parse::Parse for FieldAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut filter = None;
        let mut varint = false;
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::filter) {
                let _: kw::filter = input.parse()?;
                let _: syn::Token![=] = input.parse()?;
                filter = Some(input.parse::<syn::Path>()?);
            } else if lookahead.peek(kw::varint) {
                let _: kw::varint = input.parse()?;
                varint = true;
            } else {
                return Err(lookahead.error());
            }

            if !input.is_empty() {
                let _: syn::Token![,] = input.parse()?;
            }
        }
        Ok(FieldAttrs { filter, varint })
    }
}

#[allow(clippy::type_complexity)]
fn parse_fields(fields: &syn::Fields) -> syn::Result<Vec<(&syn::Field, FieldAttrs, syn::Member)>> {
    let mut vec = Vec::with_capacity(fields.len());
    for (idx, field) in fields.iter().enumerate() {
        vec.push(match &field.ident {
            Some(ident) => (
                field,
                parse_field_attrs(field)?,
                syn::Member::Named(ident.clone()),
            ),
            None => (
                field,
                parse_field_attrs(field)?,
                syn::Member::Unnamed(syn::Index {
                    index: idx as u32,
                    span: field.span(),
                }),
            ),
        });
    }
    Ok(vec)
}

fn parse_field_attrs(field: &syn::Field) -> syn::Result<FieldAttrs> {
    let mut find = FieldAttrs {
        filter: None,
        varint: false,
    };
    let mut flag = false;
    for attr in field.attrs.iter() {
        if attr.path().is_ident("mser") {
            if flag {
                return Err(syn::Error::new_spanned(attr, "multiple `mser` attributes"));
            }
            flag = true;
            find = attr.parse_args()?;
        }
    }
    Ok(find)
}

#[derive(Clone, Copy)]
enum Ty {
    I32,
    U32,
    U64,
    I64,
    U8Array,
    Other,
}

fn ty(ty: &syn::Type) -> Ty {
    match ty {
        syn::Type::Path(path) => match path.path.get_ident() {
            Some(x) => {
                if x == "i32" {
                    Ty::I32
                } else if x == "u32" {
                    Ty::U32
                } else if x == "u64" {
                    Ty::U64
                } else if x == "i64" {
                    Ty::I64
                } else {
                    Ty::Other
                }
            }
            None => Ty::Other,
        },
        syn::Type::Array(arr) => match &*arr.elem {
            syn::Type::Path(x) => {
                if x.path.is_ident("u8") {
                    Ty::U8Array
                } else {
                    Ty::Other
                }
            }
            _ => Ty::Other,
        },
        _ => Ty::Other,
    }
}

fn ident_case(a: &Attrs, s: &syn::Ident) -> String {
    let s = s.to_string();
    if a.camel_case {
        return s;
    }
    let mut result = String::with_capacity(s.len());
    let mut last_end = 0;
    for (start, part) in s.match_indices(|x: char| x.is_ascii_uppercase()) {
        result.push_str(unsafe { s.get_unchecked(last_end..start) });
        if last_end != 0 {
            result.push('_');
        }
        result.push(part.chars().next().unwrap().to_ascii_lowercase());
        last_end = start + part.len();
    }
    result.push_str(unsafe { s.get_unchecked(last_end..s.len()) });
    result
}
