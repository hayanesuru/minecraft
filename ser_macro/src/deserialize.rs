use crate::kw;
use quote::quote;

pub fn deserialize_struct(
    input: syn::DeriveInput,
    mser: syn::Path,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let has_lifetimes = generics.lifetimes().next().is_some();
    let tok = if has_lifetimes {
        quote!(<'__a>)
    } else {
        quote!()
    };
    let type_params1 = generics.lifetimes();
    let type_params2 = generics.type_params();
    let type_params3 = generics.const_params();
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => unreachable!(),
    };
    let read = fields
        .members()
        .map(|field| quote!(#field: #mser::Read::read(r)?,));
    Ok(quote! {
        #[automatically_derived]
        impl <'__a, #(#type_params1),* #(#type_params2),* #(#type_params3),*> #mser::Read<'__a> for #name #tok {
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
    mser: syn::Path,
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
                            let x = <#mser::V21 as #mser::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u8, Self>(x.0 as u8) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u8, Self>(0) ) }
                            }
                        });
                    } else {
                        let len = len as u8;
                        read = Some(quote! {
                            let x = <u8 as #mser::Read>::read(r)?;
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
                            let x = <#mser::V21 as #mser::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u16, Self>(x.0 as u16) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u16, Self>(0) ) }
                            }
                        });
                    } else if varint {
                        let len = len as u8;
                        read = Some(quote! {
                            let x = <u8 as #mser::Read>::read(r)?;
                            if x < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u16, Self>(x as u16) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u16, Self>(0) ) }
                            }
                        });
                    } else {
                        let len = len as u16;
                        read = Some(quote! {
                            let x = <u16 as #mser::Read>::read(r)?;
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
                            let x = <#mser::V32 as #mser::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u32, Self>(x.0) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u32, Self>(0) ) }
                            }
                        });
                    } else if varint && len > mser::V7MAX {
                        let len = len as u32;
                        read = Some(quote! {
                            let x = <#mser::V21 as #mser::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u32, Self>(x.0) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u32, Self>(0) ) }
                            }
                        });
                    } else {
                        let len = len as u32;
                        read = Some(quote! {
                            let x = <u32 as #mser::Read>::read(r)?;
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
                            let x = <#mser::V64 as #mser::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(x.0) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(0) ) }
                            }
                        });
                    } else if varint && len > mser::V21MAX {
                        let len = len as u32;
                        read = Some(quote! {
                            let x = <#mser::V32 as #mser::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(x.0 as u64) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(0) ) }
                            }
                        });
                    } else if varint && len > mser::V7MAX {
                        let len = len as u32;
                        read = Some(quote! {
                            let x = <#mser::V21 as #mser::Read>::read(r)?;
                            if x.0 < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(x.0 as u64) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(0) ) }
                            }
                        });
                    } else if varint {
                        let len = len as u8;
                        read = Some(quote! {
                            let x = <u8 as #mser::Read>::read(r)?;
                            if x < #len {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(x as u64) ) }
                            } else {
                                unsafe { ::core::result::Result::Ok(::core::mem::transmute::<u64, Self>(0) ) }
                            }
                        });
                    } else {
                        let len = len as u64;
                        read = Some(quote! {
                            let x = <u64 as #mser::Read>::read(r)?;
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
    let tok = if has_lifetimes {
        quote!(<'__a>)
    } else {
        quote!()
    };
    let type_params2 = generics.type_params();
    let type_params3 = generics.const_params();

    Ok(quote! {
        #[automatically_derived]
        impl <'__a, #(#type_params2),* #(#type_params3),*> #mser::Read<'__a> for #name #tok {
            #[inline]
            fn read(r: &mut &'__a [u8]) -> ::core::result::Result<Self, ::mser::Error> {
                #read
            }
        }
    })
}
