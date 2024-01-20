use super::{
    collect_toplevel_struct_attributes, common_tokens, parse_attributes, BasicType, Field, Opt,
    PrimitiveTy, Struct, ToplevelStructAttribute, Trait, Ty, DEFAULT_LENGTH_TYPE,
};
use alloc::vec::Vec;
use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DataStruct};

pub fn impl_writable(input: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let name = &input.ident;
    let mut types = Vec::new();
    match input.data {
        Data::Struct(DataStruct { ref fields, .. }) => {
            let attrs = parse_attributes::<ToplevelStructAttribute>(&input.attrs)?;
            let attrs = collect_toplevel_struct_attributes(attrs)?;
            let st = Struct::new(fields, &attrs)?;
            let _ = st.kind;
            let fields = st.fields.iter().filter(|f| !f.skip);

            let assignments = fields.clone().map(|field| {
                let var_name = field.var_name();
                let name = field.name();

                match field.ty.inner() {
                    Ty::RefSlice(..)
                    | Ty::RefSliceStr(..)
                    | Ty::RefSliceU8(..)
                    | Ty::RefStr(..)
                    | Ty::Type(syn::Type::Reference(..)) => quote!(let #var_name = self.#name;),
                    _ => quote!(let ref #var_name = self.#name;),
                }
            });
            let body = fields.clone().map(write_field_body);
            let needed = fields.clone().map(needed_body);
            types.extend(fields.flat_map(|x| x.bound_types()));

            let (a, b) = match attrs.prefix {
                Some(ref x) => (
                    quote!(::mser::Write::write(&#x, w);),
                    quote!(::mser::Write::len(&#x) #(+ #needed)*),
                ),
                None => (quote!(), quote!(#(#needed)+*)),
            };
            let (impl_params, ty_params, where_clause) =
                common_tokens(&input, &types, Trait::Writable);

            Ok(quote! {
                #[automatically_derived]
                impl<#impl_params> ::mser::Write for #name #ty_params #where_clause {
                    #[inline]
                    fn write(&self, w: &mut ::mser::UnsafeWriter) {
                        #(#assignments)*
                        #a
                        #(#body)*
                    }

                    #[inline]
                    fn len(&self) -> usize {
                        #b
                    }
                }
            })
        }
        Data::Enum(syn::DataEnum {
            enum_token,
            variants,
            ..
        }) => {
            let mut flag = false;
            for attr in input.attrs {
                let x = attr.meta.require_list()?;
                if x.path.require_ident()? == "repr" {
                    if let Some(TokenTree::Ident(x)) = x.tokens.clone().into_iter().next() {
                        if x == "u8" {
                            flag = true;
                        }
                    }
                }
            }
            let max = (variants.len() - 1) as u32;

            if flag && max <= 0x7f {
                let x = quote! {
                    #[automatically_derived]
                    impl ::mser::Write for #name {
                        #[inline]
                        fn write(&self, w: &mut ::mser::UnsafeWriter) {
                            w.write_byte(*self as u8);
                        }

                        #[inline]
                        fn len(&self) -> usize {
                            1
                        }
                    }

                    #[automatically_derived]
                    impl ::mser::Read for #name {
                        #[inline]
                        fn read(buf: &mut &[u8]) -> Option<Self> {
                            match <::mser::V32 as ::mser::Read>::read(buf) {
                                Some(::mser::V32(x)) => {
                                    if x <= #max {
                                        unsafe { Some(core::mem::transmute(x as u8)) }
                                    } else {
                                        None
                                    }
                                },
                                None => None,
                            }
                        }
                    }
                };
                Ok(x)
            } else if flag && max <= 0xff {
                let x = quote! {
                    #[automatically_derived]
                    impl ::mser::Write for #name {
                        #[inline]
                        fn write(&self, w: &mut ::mser::UnsafeWriter) {
                            let n = self as u8;
                            if n < 0x80 {
                                w.write_byte(n);
                            } else {
                                w.write(&[n as u8, 1]);
                            }
                        }

                        #[inline]
                        fn len(&self) -> usize {
                            let n = self as u8;
                            if n < 0x80 {
                                1
                            } else {
                                2
                            }
                        }
                    }

                    #[automatically_derived]
                    impl ::mser::Read for #name {
                        #[inline]
                        fn read(buf: &mut &[u8]) -> Option<Self> {
                            match <::mser::V32 as ::mser::Read>::read(buf) {
                                Some(::mser::V32(x)) => {
                                    if x <= #max {
                                        unsafe { Some(core::mem::transmute(x as u8)) }
                                    } else {
                                        None
                                    }
                                 },
                                 None => None,
                            }
                        }
                    }
                };
                Ok(x)
            } else {
                Err(syn::Error::new(enum_token.span(), "unimplemented"))
            }
        }
        Data::Union(syn::DataUnion { union_token, .. }) => {
            Err(syn::Error::new(union_token.span(), "unimplemented"))
        }
    }
}

fn needed_body(field: &Field) -> TokenStream {
    let name = field.name();
    let name = if let Opt::Option(_) = &field.ty {
        quote!(x___)
    } else {
        quote!(self.#name)
    };

    let needed = match field.len_type.unwrap_or(DEFAULT_LENGTH_TYPE) {
        BasicType::V32 => quote!(::mser::Write::len(&::mser::V32(#name.len() as u32)) + ),
        BasicType::V21 => quote!(::mser::Write::len(&::mser::V21(#name.len() as u32)) + ),
        BasicType::U8 => quote!(1 + ),
        BasicType::None => quote!(),
    };

    let q = match field.ty.inner() {
        Ty::String | Ty::CowStr(..) | Ty::RefStr(..) | Ty::RefSliceU8(..) => {
            quote!(#needed #name.len())
        }
        Ty::RefSliceStr(..) => {
            quote! {#needed ::mser::Write::len(#name) }
        }
        Ty::HashMap(..)
        | Ty::HashSet(..)
        | Ty::BTreeMap(..)
        | Ty::BTreeSet(..)
        | Ty::CowHashMap(..)
        | Ty::CowHashSet(..)
        | Ty::CowBTreeMap(..)
        | Ty::CowBTreeSet(..)
        | Ty::Array(..) => quote!(#needed ::mser::Write::len(#name)),
        Ty::Primitive(PrimitiveTy::I32) if field.varint => {
            quote!(::mser::Write::len(&::mser::V32(#name as u32)))
        }
        Ty::Primitive(PrimitiveTy::U32) if field.varint => {
            quote!(::mser::Write::len(&::mser::V32(#name)))
        }
        Ty::Primitive(PrimitiveTy::I64) if field.varint => {
            quote!(::mser::Write::len(&::mser::V64(#name as i64)))
        }
        Ty::Primitive(PrimitiveTy::U64) if field.varint => {
            quote!(::mser::Write::len(&::mser::V64(#name)))
        }
        Ty::Type(syn::Type::Reference(..)) if !field.expand => quote!(::mser::Write::len(#name)),
        _ if !field.expand => quote!(::mser::Write::len(&#name)),
        _ => quote! {#needed {
            let mut a___ = 0usize;
            for a in #name {
                a___ += ::mser::Write::len(a);
            }
            a___
        }},
    };

    let x = if let Opt::Option(_) = &field.ty {
        let name1 = field.name();
        quote! {
            match self.#name1 {
                Some(x___) => 1 + #q,
                None => 1,
            }
        }
    } else {
        q
    };
    if let Some(ref add) = field.add {
        quote!(::mser::Write::len(&#add) + #x)
    } else {
        x
    }
}

fn write_field_body(field: &Field) -> TokenStream {
    let name = field.var_name();

    let l = match field.len_type.unwrap_or(DEFAULT_LENGTH_TYPE) {
        BasicType::V32 => quote! { ::mser::Write::write(&::mser::V32(#name.len() as u32), w); },
        BasicType::V21 => quote! { ::mser::Write::write(&::mser::V21(#name.len() as u32), w); },
        BasicType::U8 => quote! { w.write_byte(#name.len() as u8); },
        BasicType::None => quote! {},
    };

    let body = match field.ty.inner() {
        Ty::String | Ty::CowStr(..) | Ty::RefStr(..) => quote! {
            #l
            ::mser::Write::write(#name.as_bytes(), w);
        },
        Ty::RefSliceU8(..) => quote! {
            #l
            w.write(#name);
        },
        Ty::RefSliceStr(..) => quote! {
            #l
            ::mser::Write::write(#name, w);
        },
        Ty::Primitive(x) => {
            let x = *x;
            if field.varint {
                match x {
                    PrimitiveTy::U32 => quote!(::mser::Write::write(&V32(*#name), w);),
                    PrimitiveTy::U64 => quote!(::mser::Write::write(&V64(*#name), w);),
                    PrimitiveTy::I32 => quote!(::mser::Write::write(&V32(*#name as u32), w);),
                    PrimitiveTy::I64 => quote!(::mser::Write::write(&V64(#name as u64), w);),
                    _ => unimplemented!(),
                }
            } else {
                quote!(w.write(&#name.to_be_bytes());)
            }
        }
        _ if !field.expand => quote! { ::mser::Write::write(#name, w); },
        _ => quote! {
            #l
            for x in #name {
                ::mser::Write::write(x, w);
            }
        },
    };

    let body = match field.ty {
        Opt::Plain(_) => body,
        Opt::Option(_) => quote! {
            if let Some(#name) = #name {
                w.write_byte(1);
                #body
            } else {
                w.write_byte(0);
            }
        },
    };

    if let Some(ref add) = field.add {
        quote! {{
            ::mser::Write::write(&#add, w);
            #body
        }}
    } else {
        body
    }
}
