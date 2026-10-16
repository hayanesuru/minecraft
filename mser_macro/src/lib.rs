#![recursion_limit = "128"]
#![no_std]
#![warn(clippy::shadow_reuse, clippy::use_self)]

extern crate alloc;

mod deserialize;
mod serialize;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};

const V21MAX: usize = 0x1FFFFF;
const V7MAX: usize = 0x7F;

mod kw {
    syn::custom_keyword!(varint);
    syn::custom_keyword!(filter);
    syn::custom_keyword!(header);
    syn::custom_keyword!(PascalCase);
    syn::custom_keyword!(name);
}

#[derive(Default)]
struct Attrs {
    varint: bool,
    header: Option<syn::Path>,
    pascal_case: bool,
    filter: Option<syn::Path>,
    name: Option<Ident>,
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
            } else if lookahead.peek(kw::PascalCase) {
                let _: kw::PascalCase = input.parse()?;
                attr.pascal_case = true;
            } else if lookahead.peek(kw::filter) {
                let _: kw::filter = input.parse()?;
                let _: syn::Token![=] = input.parse()?;
                attr.filter = Some(input.parse::<syn::Path>()?);
            } else if lookahead.peek(kw::name) {
                let _: kw::name = input.parse()?;
                let _: syn::Token![=] = input.parse()?;
                attr.name = Some(input.parse::<syn::Ident>()?);
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
    for attr in input.attrs.iter() {
        if attr.path().is_ident("mser") {
            let Attrs {
                varint,
                header,
                pascal_case: camel_case,
                filter,
                name,
            } = attr.parse_args()?;
            if varint {
                find.varint = true;
            }
            if let Some(h) = header {
                find.header = Some(h);
            }
            if camel_case {
                find.pascal_case = true;
            }
            if let Some(f) = filter {
                find.filter = Some(f);
            }
            if let Some(n) = name {
                find.name = Some(n);
            }
        }
    }
    Ok((
        find,
        syn::Ident::new("mser", proc_macro2::Span::call_site()).into(),
    ))
}

#[proc_macro_derive(Serialize, attributes(mser))]
pub fn serialize(input: TokenStream) -> TokenStream {
    let i = parse_macro_input!(input as syn::DeriveInput);
    let (attr, cratename) = match crate_name(&i) {
        Ok(cratename) => cratename,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let x = match &i.data {
        syn::Data::Struct(_) => serialize::serialize_struct(i, cratename),
        syn::Data::Enum(_) => serialize::serialize_enum(i, cratename, attr),
        syn::Data::Union(_) => Err(syn::Error::new_spanned(i, "unions are not supported")),
    };
    match x {
        Ok(token) => token.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_derive(Deserialize, attributes(mser))]
pub fn deserialize(input: TokenStream) -> TokenStream {
    let i = parse_macro_input!(input as syn::DeriveInput);
    let (attrs, cratename) = match crate_name(&i) {
        Ok(cratename) => cratename,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let x = match &i.data {
        syn::Data::Struct(_) => deserialize::deserialize_struct(i, cratename, attrs),
        syn::Data::Enum(_) => deserialize::deserialize_enum(i, cratename, attrs),
        syn::Data::Union(_) => Err(syn::Error::new_spanned(i, "unions are not supported")),
    };
    match x {
        Ok(token) => token.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

struct FieldAttrs {
    filter: Option<syn::Path>,
    varint: bool,
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
        Ok(Self { filter, varint })
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

fn ident_case(a: &Attrs, ident: &syn::Ident) -> String {
    let s = ident.to_string();
    if a.pascal_case {
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

#[proc_macro_derive(Name, attributes(mser))]
pub fn enum_info(input: TokenStream) -> TokenStream {
    let i = parse_macro_input!(input as DeriveInput);
    let name = &i.ident;
    let (attrs, c) = match crate_name(&i) {
        Ok(x) => x,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };
    let cratename = &c;
    let fn_name = match &attrs.name {
        Some(x) => x.clone(),
        None => Ident::new("name", proc_macro2::Span::call_site()),
    };

    let Data::Enum(data) = &i.data else {
        return syn::Error::new_spanned(name, "not impl")
            .to_compile_error()
            .into();
    };

    let arms = data.variants.iter().map(|v| {
        let id = &v.ident;
        let s = ident_case(&attrs, id);
        let pat = match &v.fields {
            Fields::Unit => quote!(),
            Fields::Unnamed(_) => quote!((..)),
            Fields::Named(_) => quote!({ .. }),
        };
        quote!(Self::#id #pat => #s,)
    });
    let from_arms = data
        .variants
        .iter()
        .filter(|v| matches!(v.fields, Fields::Unit))
        .map(|v| {
            let id = &v.ident;
            let s = ident_case(&attrs, id);
            quote!(#s => ::core::result::Result::Ok(Self::#id),)
        });

    let (ig, tg, wc) = i.generics.split_for_impl();
    quote! {
        impl #ig #name #tg #wc {
            pub const fn #fn_name(self) -> &'static str {
                match self { #(#arms)* }
            }
        }
        impl #ig ::core::str::FromStr for #name #tg #wc {
            type Err = #cratename::Error;
            fn from_str(__s: &str) -> ::core::result::Result<Self, #cratename::Error> {
                match __s {
                    #(#from_arms)*
                    _ => ::core::result::Result::Err(#cratename::Error),
                }
            }
        }
    }
    .into()
}
