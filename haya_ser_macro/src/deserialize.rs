use crate::{Attrs, Ty, V21MAX, ident_case, parse_fields, ty};
use alloc::vec::Vec;
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};

pub fn deserialize_struct(
    input: syn::DeriveInput,
    cratename: syn::Path,
    attrs: Attrs,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let has_lifetimes = generics.lifetimes().next().is_some();

    let type_generics = TypeGenerics(generics);
    let tok = if has_lifetimes {
        quote!(<'__a #type_generics)
    } else {
        quote!()
    };

    let impl_generics = ImplGenerics(generics);
    let fields = match &input.data {
        syn::Data::Struct(data) => parse_fields(&data.fields)?,
        _ => unreachable!(),
    };
    let read1 = fields
        .iter()
        .map(|(field, attrs, m)| read_field(&cratename, field, attrs, m));
    let read2 = quote!(Self {
        #(#read1,)*
    });
    let read = if let Some(filter) = attrs.filter {
        quote! {
            let __v = #read2;
            if #filter(&__v) {
                ::core::result::Result::Ok(__v)
            } else {
                ::core::result::Result::Err(::#cratename::Error)
            }
        }
    } else {
        quote!(::core::result::Result::Ok(#read2))
    };
    Ok(quote! {
        #[automatically_derived]
        impl <'__a #impl_generics ::#cratename::Read<'__a> for #name #tok {
            #[inline]
            fn read(__r: &mut ::#cratename::Reader<'__a>) -> ::core::result::Result<Self, ::mser::Error> {
               #read
            }
        }
    })
}

pub fn deserialize_enum(
    input: syn::DeriveInput,
    cratename: syn::Path,
    attrs: Attrs,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let mut read = None;
    let varint = attrs.varint;
    let variants = match &input.data {
        syn::Data::Enum(data) => &data.variants,
        _ => unreachable!(),
    };
    let len = variants.len();
    let is_repr_enum = variants
        .iter()
        .all(|x| matches!(x.fields, syn::Fields::Unit));
    for ele in variants.iter() {
        if ele.discriminant.is_some() {
            return Err(syn::Error::new_spanned(
                ele,
                "expected enum variants to not have discriminants",
            ));
        }
    }
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                let path = meta.path.get_ident().unwrap();
                let lit = syn::LitInt::new(
                    itoa::Buffer::new().format(len),
                    proc_macro2::Span::call_site(),
                );
                read = Some(repr_enum(&cratename, varint, len, path, lit));
                Ok(())
            })?;
        }
    }
    if !is_repr_enum {
        let header = match &attrs.header {
            Some(x) => x,
            None => return Err(syn::Error::new_spanned(input, "expected header")),
        };
        let mut match_arms = Vec::with_capacity(variants.len());
        for variant in variants.iter() {
            let variant_name = &variant.ident;
            let header_variant = syn::Ident::new(
                &ident_case(&attrs, variant_name),
                proc_macro2::Span::call_site(),
            );
            let fields2 = parse_fields(&variant.fields)?;
            let fields3 = fields2
                .iter()
                .map(|(field, attrs, m)| read_field(&cratename, field, attrs, m));
            match_arms.push(quote! {
                #header::#header_variant => Self::#variant_name { #(#fields3,)* }
            });
        }

        read = Some(quote! {
            match <#header as ::#cratename::Read>::read(__r)? {
                #(#match_arms,)*
            }
        });
    }

    let read1 = match read {
        Some(x) => x,
        None => {
            return Err(syn::Error::new_spanned(
                &input,
                "expected `#[repr(...)]` attribute",
            ));
        }
    };
    let read2 = if let Some(filter) = attrs.filter {
        quote! {
            let __v = #read1;
            if #filter(&__v) {
                ::core::result::Result::Ok(__v)
            } else {
                ::core::result::Result::Err(::#cratename::Error)
            }
        }
    } else {
        quote!(::core::result::Result::Ok(#read1))
    };

    let generics = &input.generics;
    let has_lifetimes = generics.lifetimes().next().is_some();

    let type_generics = TypeGenerics(generics);
    let tok = if has_lifetimes {
        quote!(<'__a #type_generics)
    } else {
        quote!()
    };
    let impl_generics = ImplGenerics(generics);
    Ok(quote! {
        #[automatically_derived]
        impl <'__a #impl_generics ::#cratename::Read<'__a> for #name #tok {
            #[inline]
            fn read(__r: &mut ::#cratename::Reader<'__a>) -> ::core::result::Result<Self, ::mser::Error> {
                #read2
            }
        }
    })
}

fn read_field(
    cratename: &syn::Path,
    field: &&syn::Field,
    attrs: &crate::FieldAttrs,
    m: &syn::Member,
) -> TokenStream {
    let v = if attrs.varint {
        match ty(&field.ty) {
            Ty::I32 => quote!(::#cratename::V32::read(__r)?.0 as i32),
            Ty::U32 => quote!(::#cratename::V32::read(__r)?.0),
            Ty::I64 => quote!(::#cratename::V64::read(__r)?.0 as i64),
            _ => quote!(::#cratename::V64::read(__r)?.0),
        }
    } else {
        quote!(::#cratename::Read::read(__r)?)
    };
    match &attrs.filter {
        Some(x) => quote! {
            #m: {
                let __v = #v;
                if #x(&__v) {
                    __v
                } else {
                    return ::core::result::Result::Err(::#cratename::Error);
                }
            }
        },
        None => quote!(#m: #v),
    }
}

fn repr_enum(
    cratename: &syn::Path,
    varint: bool,
    len: usize,
    path: &syn::Ident,
    lit: syn::LitInt,
) -> TokenStream {
    if !varint {
        quote! {{
            let __x = <#path as ::#cratename::Read>::read(__r)?;
            if __x < #lit {
                unsafe { ::core::mem::transmute::<#path, Self>(__x as #path) }
            } else {
                 unsafe { ::core::mem::transmute::<#path, Self>(0) }
            }
        }}
    } else if path == "u64" {
        quote! {{
            let __x = <::#cratename::V64 as ::#cratename::Read>::read(__r)?.0;
            if __x < #lit {
                unsafe { ::core::mem::transmute::<u64, Self>(__x) }
            } else {
                unsafe { ::core::mem::transmute::<u64, Self>(0) }
            }
        }}
    } else if len > V21MAX {
        quote! {{
            let __x = <::#cratename::V32 as ::#cratename::Read>::read(__r)?.0;
            if __x < #lit {
                unsafe { ::core::mem::transmute::<u32, Self>(__x) }
            } else {
                unsafe { ::core::mem::transmute::<u32, Self>(0) }
            }
        }}
    } else {
        quote! {{
            let __x = <::#cratename::V21 as ::#cratename::Read>::read(__r)?.0;
            if __x < #lit {
                unsafe { ::core::mem::transmute::<#path, Self>(__x as #path) }
            } else {
                unsafe { ::core::mem::transmute::<#path, Self>(0) }
            }
        }}
    }
}

pub struct ImplGenerics<'a>(&'a syn::Generics);
impl<'a> ToTokens for ImplGenerics<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.0.params.is_empty() {
            TokensOrDefault(&self.0.gt_token).to_tokens(tokens);
            return;
        }

        // TokensOrDefault(&self.0.lt_token).to_tokens(tokens);

        // Print lifetimes before types and consts, regardless of their
        // order in self.params.
        let mut trailing_or_empty = false;
        // for param in self.0.params.pairs() {
        //     if let syn::GenericParam::Lifetime(_) = **param.value() {
        //         param.to_tokens(tokens);
        //         trailing_or_empty = param.punct().is_some();
        //     }
        // }
        for param in self.0.params.pairs() {
            if let syn::GenericParam::Lifetime(_) = **param.value() {
                continue;
            }
            if !trailing_or_empty {
                <syn::Token![,]>::default().to_tokens(tokens);
                trailing_or_empty = true;
            }
            match param.value() {
                syn::GenericParam::Lifetime(_) => unreachable!(),
                syn::GenericParam::Type(p) => {
                    if p.bounds.len() == 1
                        && let Some(syn::TypeParamBound::Trait(t)) = p.bounds.first()
                        && let Some(ident) = t.path.get_ident()
                        && ident == "Allocator"
                    {
                        continue; // todo
                    }

                    // Leave off the type parameter defaults
                    tokens.append_all(
                        p.attrs
                            .iter()
                            .filter(|x| matches!(x.style, syn::AttrStyle::Outer)),
                    );
                    p.ident.to_tokens(tokens);
                    if !p.bounds.is_empty() {
                        TokensOrDefault(&p.colon_token).to_tokens(tokens);
                        p.bounds.to_tokens(tokens);
                    }
                }
                syn::GenericParam::Const(p) => {
                    // Leave off the const parameter defaults
                    tokens.append_all(
                        p.attrs
                            .iter()
                            .filter(|x| matches!(x.style, syn::AttrStyle::Outer)),
                    );
                    p.const_token.to_tokens(tokens);
                    p.ident.to_tokens(tokens);
                    p.colon_token.to_tokens(tokens);
                    p.ty.to_tokens(tokens);
                }
            }
            param.punct().to_tokens(tokens);
        }

        TokensOrDefault(&self.0.gt_token).to_tokens(tokens);
    }
}

pub(crate) struct TokensOrDefault<'a, T: 'a>(pub &'a Option<T>);

impl<'a, T> ToTokens for TokensOrDefault<'a, T>
where
    T: ToTokens + Default,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.0 {
            Some(t) => t.to_tokens(tokens),
            None => T::default().to_tokens(tokens),
        }
    }
}
pub struct TypeGenerics<'a>(&'a syn::Generics);
impl<'a> ToTokens for TypeGenerics<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.0.params.is_empty() {
            TokensOrDefault(&self.0.gt_token).to_tokens(tokens);
            return;
        }

        // TokensOrDefault(&self.0.lt_token).to_tokens(tokens);

        // Print lifetimes before types and consts, regardless of their
        // order in self.params.
        let mut trailing_or_empty = false;
        // for param in self.0.params.pairs() {
        //     if let syn::GenericParam::Lifetime(def) = *param.value() {
        //         // Leave off the lifetime bounds and attributes
        //         def.lifetime.to_tokens(tokens);
        //         param.punct().to_tokens(tokens);
        //         trailing_or_empty = param.punct().is_some();
        //     }
        // }
        for param in self.0.params.pairs() {
            if let syn::GenericParam::Lifetime(_) = **param.value() {
                continue;
            }
            if !trailing_or_empty {
                <syn::Token![,]>::default().to_tokens(tokens);
                trailing_or_empty = true;
            }
            match param.value() {
                syn::GenericParam::Lifetime(_) => unreachable!(),
                syn::GenericParam::Type(p) => {
                    if p.bounds.len() == 1
                        && let Some(syn::TypeParamBound::Trait(t)) = p.bounds.first()
                        && let Some(ident) = t.path.get_ident()
                        && ident == "Allocator"
                    {
                        continue; // todo
                    }

                    // Leave off the type parameter defaults
                    p.ident.to_tokens(tokens);
                }
                syn::GenericParam::Const(p) => {
                    // Leave off the const parameter defaults
                    p.ident.to_tokens(tokens);
                }
            }
            param.punct().to_tokens(tokens);
        }

        TokensOrDefault(&self.0.gt_token).to_tokens(tokens);
    }
}
