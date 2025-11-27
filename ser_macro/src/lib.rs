#![recursion_limit = "128"]
#![no_std]

mod deserialize;
mod serialize;

#[cfg(feature = "nbt")]
use alloc::string::String;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

extern crate alloc;

mod kw {
    syn::custom_keyword!(varint);
}

#[cfg(feature = "nbt")]
#[proc_macro]
pub fn compound(token: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut data = alloc::format!("{{{token}}}");
    let output = mser::nbt::StringifyCompound::decode(&data)
        .expect("Invalid SNBT compound")
        .0;
    data.clear();

    let mut data = data.into_bytes();
    mser::write_exact(&mut data, &output);
    let mut i = itoa::Buffer::new();
    let mut o = String::new();
    o += "&[";
    for &x in &data {
        o += i.format(x);
        o += ", ";
    }
    if !data.is_empty() {
        o.pop();
        o.pop();
    }
    o += "]";
    core::str::FromStr::from_str(&o).unwrap()
}

fn crate_name(input: &DeriveInput) -> Result<syn::Path, syn::Error> {
    let mut find = None;
    for attr in input.attrs.iter() {
        if attr.path().is_ident("mser") {
            if find.is_some() {
                return Err(syn::Error::new_spanned(attr, "multiple `mser` attributes"));
            }
            find = Some(attr);
        }
    }

    Ok(syn::Ident::new("mser", proc_macro2::Span::call_site()).into())
}

#[proc_macro_derive(Serialize, attributes(mser))]
pub fn serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let cratename = match crate_name(&input) {
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
    let cratename = match crate_name(&input) {
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
