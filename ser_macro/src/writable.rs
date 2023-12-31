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
                    quote!(Write::write(&#x, w);),
                    quote!(Write::len(&#x) #(+ #needed)*),
                ),
                None => (quote!(), quote!(#(#needed)+*)),
            };
            let (impl_params, ty_params, where_clause) =
                common_tokens(&input, &types, Trait::Writable);

            Ok(quote! {
                #[automatically_derived]
                impl<#impl_params> Write for #name #ty_params #where_clause {
                    #[inline]
                    fn write(&self, w: &mut UnsafeWriter) {
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
            if flag && variants.len() <= 0x7f {
                let x = quote! {
                    #[automatically_derived]
                    impl Write for #name {
                        #[inline]
                        fn write(&self, w: &mut UnsafeWriter) {
                            w.write_byte(*self as u8);
                        }

                        #[inline]
                        fn len(&self) -> usize {
                            1
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
        BasicType::V32 => quote!(Write::len(&V32(#name.len() as u32)) + ),
        BasicType::V21 => quote!(Write::len(&V21(#name.len() as u32)) + ),
        BasicType::U8 => quote!(1 + ),
        BasicType::None => quote!(),
    };

    let q = match field.ty.inner() {
        Ty::String | Ty::CowStr(..) | Ty::RefStr(..) | Ty::RefSliceU8(..) => {
            quote!(#needed #name.len())
        }
        Ty::RefSliceStr(..) => {
            quote! {#needed Write::len(#name) }
        }
        Ty::HashMap(..)
        | Ty::HashSet(..)
        | Ty::BTreeMap(..)
        | Ty::BTreeSet(..)
        | Ty::CowHashMap(..)
        | Ty::CowHashSet(..)
        | Ty::CowBTreeMap(..)
        | Ty::CowBTreeSet(..)
        | Ty::Array(..) => quote!(#needed Write::len(#name)),
        Ty::Primitive(PrimitiveTy::I32) if field.varint => {
            quote!(Write::len(&V32(#name as u32)))
        }
        Ty::Primitive(PrimitiveTy::U32) if field.varint => {
            quote!(Write::len(&V32(#name)))
        }
        Ty::Primitive(PrimitiveTy::I64) if field.varint => {
            quote!(Write::len(&V64(#name as i64)))
        }
        Ty::Primitive(PrimitiveTy::U64) if field.varint => {
            quote!(Write::len(&V64(#name)))
        }
        Ty::Type(syn::Type::Reference(..)) if !field.expand => quote!(Write::len(#name)),
        _ if !field.expand => quote!(Write::len(&#name)),
        _ => quote! {#needed {
            let mut a___ = 0usize;
            for a in #name {
                a___ += Write::len(a);
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
        quote!(Write::len(&#add) + #x)
    } else {
        x
    }
}

fn write_field_body(field: &Field) -> TokenStream {
    let name = field.var_name();

    let l = match field.len_type.unwrap_or(DEFAULT_LENGTH_TYPE) {
        BasicType::V32 => quote! { Write::write(&V32(#name.len() as u32), w); },
        BasicType::V21 => quote! { Write::write(&V21(#name.len() as u32), w); },
        BasicType::U8 => quote! { w.write_byte(#name.len() as u8); },
        BasicType::None => quote! {},
    };

    let body = match field.ty.inner() {
        Ty::String | Ty::CowStr(..) | Ty::RefStr(..) => quote! {
            #l
            Write::write(#name.as_bytes(), w);
        },
        Ty::RefSliceU8(..) => quote! {
            #l
            w.write(#name);
        },
        Ty::RefSliceStr(..) => quote! {
            #l
            Write::write(#name, w);
        },
        Ty::Primitive(x) => {
            let x = *x;
            if field.varint {
                match x {
                    PrimitiveTy::U32 => quote!(Write::write(&V32(*#name), w);),
                    PrimitiveTy::U64 => quote!(Write::write(&V64(*#name), w);),
                    PrimitiveTy::I32 => quote!(Write::write(&V32(*#name as u32), w);),
                    PrimitiveTy::I64 => quote!(Write::write(&V64(#name as u64), w);),
                    _ => unimplemented!(),
                }
            } else {
                quote!(w.write(&#name.to_be_bytes());)
            }
        }
        _ if !field.expand => quote! { Write::write(#name, w); },
        _ => quote! {
            #l
            for x in #name {
                Write::write(x, w);
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
            Write::write(&#add, w);
            #body
        }}
    } else {
        body
    }
}
