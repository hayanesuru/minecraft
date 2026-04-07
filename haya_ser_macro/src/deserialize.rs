use crate::{Attrs, Ty, V21MAX, parse_fields, ty};
use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, quote};

pub fn deserialize_struct(
    input: syn::DeriveInput,
    cratename: syn::Path,
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
        syn::Data::Struct(data) => &data.fields,
        _ => unreachable!(),
    };
    let read = parse_fields(fields).map(|(field, attrs, member)| {
        let attrs = attrs.unwrap();
        let v = if attrs.varint {
            match ty(&field.ty) {
                Ty::I32 => quote!(::#cratename::V32::read(r)?.0 as i32),
                Ty::U32 => quote!(::#cratename::V32::read(r)?.0),
                Ty::I64 => quote!(::#cratename::V64::read(r)?.0 as i64),
                _ => quote!(::#cratename::V64::read(r)?.0),
            }
        } else {
            quote!(::#cratename::Read::read(r)?)
        };
        match attrs.filter {
            Some(x) => quote! {
                #member: {
                    let __v = #v;
                    if #x(&__v) {
                        __v
                    } else {
                        return ::core::result::Result::Err(::#cratename::Error);
                    }
                }
            },
            None => quote!(#member: #v),
        }
    });
    Ok(quote! {
        #[automatically_derived]
        impl <'__a #impl_generics ::#cratename::Read<'__a> for #name #tok {
            #[inline]
            fn read(r: &mut ::#cratename::Reader<'__a>) -> ::core::result::Result<Self, ::mser::Error> {
                ::core::result::Result::Ok(Self {
                    #(#read),*
                })
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
    for ele in variants.iter() {
        if ele.discriminant.is_some() {
            return Err(syn::Error::new_spanned(
                ele,
                "expected enum variants to not have discriminants",
            ));
        }
        if !matches!(ele.fields, syn::Fields::Unit) {
            return Err(syn::Error::new_spanned(
                ele,
                "expected enum variants to have unit fields",
            ));
        }
    }
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                let path = meta.path.get_ident().unwrap();
                let lit = syn::LitInt::new(itoa::Buffer::new().format(len), proc_macro2::Span::call_site());
                read = Some(if !varint {
                    quote! {
                        let __x = <#path as ::#cratename::Read>::read(r)?.0;
                        if __x < #lit {
                            unsafe { ::core::result::Result::Ok(::core::mem::transmute::<#path, Self>(__x as #path) ) }
                        } else {
                             unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u8, Self>(0) ) }
                        }
                    }
                } else if path == "u64" {
                        quote! {
                            let __x = <::#cratename::V64 as ::#cratename::Read>::read(r)?.0;
                            if __x < #lit {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(__x) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(0) ) }
                            }
                        }
                } else if len > V21MAX {
                    quote! {
                        let __x = <::#cratename::V32 as ::#cratename::Read>::read(r)?.0;
                        if __x < #lit {
                            unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u32, Self>(__x) ) }
                        } else {
                            unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u32, Self>(0) ) }
                        }
                    }
                } else {
                    quote! {
                        let __x = <::#cratename::V21 as ::#cratename::Read>::read(r)?.0;
                        if __x < #lit {
                            unsafe { ::core::result::Result::Ok(::core::mem::transmute::<#path, Self>(__x as #path) ) }
                        } else {
                            unsafe { ::core::result::Result::Ok(::core::mem::transmute::<#path, Self>(0) ) }
                        }
                    }
                });
                Ok(())
            })?;
        }
    }
    let read =
        read.ok_or_else(|| syn::Error::new_spanned(&input, "expected `#[repr(...)]` attribute"))?;

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
            fn read(r: &mut ::#cratename::Reader<'__a>) -> ::core::result::Result<Self, ::mser::Error> {
                #read
            }
        }
    })
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
                syn::GenericParam::Type(param) => {
                    if param.bounds.len() == 1
                        && let Some(syn::TypeParamBound::Trait(t)) = param.bounds.first()
                        && let Some(ident) = t.path.get_ident()
                        && ident == "Allocator"
                    {
                        continue; // todo
                    }

                    // Leave off the type parameter defaults
                    tokens.append_all(
                        param
                            .attrs
                            .iter()
                            .filter(|x| matches!(x.style, syn::AttrStyle::Outer)),
                    );
                    param.ident.to_tokens(tokens);
                    if !param.bounds.is_empty() {
                        TokensOrDefault(&param.colon_token).to_tokens(tokens);
                        param.bounds.to_tokens(tokens);
                    }
                }
                syn::GenericParam::Const(param) => {
                    // Leave off the const parameter defaults
                    tokens.append_all(
                        param
                            .attrs
                            .iter()
                            .filter(|x| matches!(x.style, syn::AttrStyle::Outer)),
                    );
                    param.const_token.to_tokens(tokens);
                    param.ident.to_tokens(tokens);
                    param.colon_token.to_tokens(tokens);
                    param.ty.to_tokens(tokens);
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
                syn::GenericParam::Type(param) => {
                    if param.bounds.len() == 1
                        && let Some(syn::TypeParamBound::Trait(t)) = param.bounds.first()
                        && let Some(ident) = t.path.get_ident()
                        && ident == "Allocator"
                    {
                        continue; // todo
                    }

                    // Leave off the type parameter defaults
                    param.ident.to_tokens(tokens);
                }
                syn::GenericParam::Const(param) => {
                    // Leave off the const parameter defaults
                    param.ident.to_tokens(tokens);
                }
            }
            param.punct().to_tokens(tokens);
        }

        TokensOrDefault(&self.0.gt_token).to_tokens(tokens);
    }
}
