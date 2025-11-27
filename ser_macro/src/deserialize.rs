use crate::kw;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use quote::TokenStreamExt;

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
    let read = fields
        .members()
        .map(|field| quote!(#field: ::#cratename::Read::read(r)?,));
    Ok(quote! {
        #[automatically_derived]
        impl <'__a #impl_generics ::#cratename::Read<'__a> for #name #tok {
            #[inline]
            fn read(r: &mut &'__a [u8]) -> ::core::result::Result<Self, ::mser::Error> {
                ::core::result::Result::Ok(Self {
                    #(#read)*
                })
            }
        }
    })
}

pub fn deserialize_enum(
    input: syn::DeriveInput,
    cratename: syn::Path,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let mut read = None;
    let mut varint = false;
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
        if attr.path().is_ident("mser") {
            attr.meta.require_list()?.parse_args::<kw::varint>()?;
            varint = true;
        }
    }
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("u8") {
                    if varint && len > mser::V7MAX {
                        let len = len as u32;
                        read = Some(quote! {
                            let x = <::#cratename::V21 as ::#cratename::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u8, Self>(x.0 as u8) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u8, Self>(0) ) }
                            }
                        });
                    } else {
                        let len = len as u8;
                        read = Some(quote! {
                            let x = <u8 as ::#cratename::Read>::read(r)?;
                            if x < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u8, Self>(x) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u8, Self>(0) ) }
                            }
                        });
                    }
                } else if meta.path.is_ident("u16") {
                    if varint && len > mser::V7MAX {
                        let len = len as u32;
                        read = Some(quote! {
                            let x = <::#cratename::V21 as ::#cratename::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u16, Self>(x.0 as u16) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u16, Self>(0) ) }
                            }
                        });
                    } else if varint {
                        let len = len as u8;
                        read = Some(quote! {
                            let x = <u8 as ::#cratename::Read>::read(r)?;
                            if x < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u16, Self>(x as u16) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u16, Self>(0) ) }
                            }
                        });
                    } else {
                        let len = len as u16;
                        read = Some(quote! {
                            let x = <u16 as ::#cratename::Read>::read(r)?;
                            if x < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u16, Self>(x) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u16, Self>(0) ) }
                            }
                        });
                    }
                } else if meta.path.is_ident("u32") {
                    if varint && len > mser::V21MAX {
                        let len = len as u32;
                        read = Some(quote! {
                            let x = <::#cratename::V32 as ::#cratename::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u32, Self>(x.0) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u32, Self>(0) ) }
                            }
                        });
                    } else if varint && len > mser::V7MAX {
                        let len = len as u32;
                        read = Some(quote! {
                            let x = <::#cratename::V21 as ::#cratename::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u32, Self>(x.0) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u32, Self>(0) ) }
                            }
                        });
                    } else {
                        let len = len as u32;
                        read = Some(quote! {
                            let x = <u32 as ::#cratename::Read>::read(r)?;
                            if x < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u32, Self>(x) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u32, Self>(0) ) }
                            }
                        });
                    }
                } else if meta.path.is_ident("u64") {
                    if varint && len > u32::MAX as usize {
                        let len = len as u64;
                        read = Some(quote! {
                            let x = <::#cratename::V64 as ::#cratename::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(x.0) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(0) ) }
                            }
                        });
                    } else if varint && len > mser::V21MAX {
                        let len = len as u32;
                        read = Some(quote! {
                            let x = <::#cratename::V32 as ::#cratename::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(x.0 as u64) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(0) ) }
                            }
                        });
                    } else if varint && len > mser::V7MAX {
                        let len = len as u32;
                        read = Some(quote! {
                            let x = <::#cratename::V21 as ::#cratename::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(x.0 as u64) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(0) ) }
                            }
                        });
                    } else if varint {
                        let len = len as u8;
                        read = Some(quote! {
                            let x = <u8 as ::#cratename::Read>::read(r)?;
                            if x < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(x as u64) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(0) ) }
                            }
                        });
                    } else {
                        let len = len as u64;
                        read = Some(quote! {
                            let x = <u64 as ::#cratename::Read>::read(r)?;
                            if x < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(x) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(0) ) }
                            }
                        });
                    }
                }
                Ok(())
            })?;
        }
        if attr.path().is_ident("mser") {
            attr.parse_nested_meta(|meta| {
                let lookahead = meta.input.lookahead1();
                if lookahead.peek(kw::varint) {
                    varint = true;
                }
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
            fn read(r: &mut &'__a [u8]) -> ::core::result::Result<Self, ::mser::Error> {
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
                    if param.bounds.len() == 1 {
                        if let Some(syn::TypeParamBound::Trait(t)) = param.bounds.first() {
                            if let Some(ident) = t.path.get_ident() {
                                if ident == "Allocator" {
                                    continue; // todo
                                }
                            }
                        }
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
                    if param.bounds.len() == 1 {
                        if let Some(syn::TypeParamBound::Trait(t)) = param.bounds.first() {
                            if let Some(ident) = t.path.get_ident() {
                                if ident == "Allocator" {
                                    continue; // todo
                                }
                            }
                        }
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
