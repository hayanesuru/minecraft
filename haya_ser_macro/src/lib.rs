#![recursion_limit = "128"]
#![no_std]

extern crate alloc;

mod deserialize;
mod serialize;

use proc_macro::TokenStream;
use syn::parse::ParseStream;
use syn::{DeriveInput, parse_macro_input};

const V21MAX: usize = 0x1FFFFF;
const V7MAX: usize = 0x7F;

mod kw {
    syn::custom_keyword!(varint);
    syn::custom_keyword!(filter);
}

struct Attrs {
    varint: bool,
}

fn crate_name(input: &DeriveInput) -> Result<(Attrs, syn::Path), syn::Error> {
    let mut find = None;
    for attr in input.attrs.iter() {
        if attr.path().is_ident("mser") {
            if find.is_some() {
                return Err(syn::Error::new_spanned(attr, "multiple `mser` attributes"));
            };
            find = Some(Attrs {
                varint: attr
                    .meta
                    .require_list()
                    .and_then(|list| list.parse_args::<kw::varint>())
                    .is_ok(),
            });
        }
    }

    Ok((
        match find {
            Some(x) => x,
            None => Attrs { varint: false },
        },
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
fn parse_fields<'a>(
    fields: &'a syn::Fields,
) -> core::iter::Map<
    core::iter::Enumerate<syn::punctuated::Iter<'a, syn::Field>>,
    impl FnMut((usize, &'a syn::Field)) -> (&'a syn::Field, syn::Result<FieldAttrs>, syn::Member),
> {
    fields
        .iter()
        .enumerate()
        .map(|(idx, field)| match field.ident.clone() {
            Some(ident) => (field, parse_field_attrs(field), syn::Member::Named(ident)),
            None => (
                field,
                parse_field_attrs(field),
                syn::Member::Unnamed(syn::Index {
                    index: idx as u32,
                    span: proc_macro2::Span::call_site(),
                }),
            ),
        })
}

fn parse_field_attrs(field: &syn::Field) -> syn::Result<FieldAttrs> {
    let mut find = None;
    for attr in field.attrs.iter() {
        if attr.path().is_ident("mser") {
            if find.is_some() {
                return Err(syn::Error::new_spanned(attr, "multiple `mser` attributes"));
            };
            find = Some(attr.parse_args::<FieldAttrs>()?);
        }
    }

    Ok(match find {
        Some(x) => x,
        None => FieldAttrs {
            filter: None,
            varint: false,
        },
    })
}

#[derive(Clone, Copy)]
enum Ty {
    I32,
    U8Array,
    Other,
}

fn ty(ty: &syn::Type) -> Ty {
    match ty {
        syn::Type::Path(path) => {
            if path.path.is_ident("i32") {
                Ty::I32
            } else {
                Ty::Other
            }
        }
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
