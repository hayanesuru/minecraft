#![recursion_limit = "128"]
#![no_std]

mod deserialize;
mod serialize;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

const V21MAX: usize = 0x1FFFFF;
const V7MAX: usize = 0x7F;

mod kw {
    syn::custom_keyword!(varint);
}

fn crate_name(input: &DeriveInput) -> Result<(Option<&syn::Attribute>, syn::Path), syn::Error> {
    let mut find = None;
    for attr in input.attrs.iter() {
        if attr.path().is_ident("mser") {
            if find.is_some() {
                return Err(syn::Error::new_spanned(attr, "multiple `mser` attributes"));
            }
            find = Some(attr);
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
    let (_attr, cratename) = match crate_name(&input) {
        Ok(cratename) => cratename,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let x = match input.data {
        syn::Data::Struct(_) => serialize::serialize_struct(input, cratename),
        syn::Data::Enum(_) => serialize::serialize_enum(input, cratename),
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
    let (_attr, cratename) = match crate_name(&input) {
        Ok(cratename) => cratename,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let x = match input.data {
        syn::Data::Struct(_) => deserialize::deserialize_struct(input, cratename),
        syn::Data::Enum(_) => deserialize::deserialize_enum(input, cratename),
        syn::Data::Union(_) => Err(syn::Error::new_spanned(input, "unions are not supported")),
    };
    match x {
        Ok(x) => x.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
