use crate::{Attrs, Ty, V7MAX, V21MAX, ident_case, parse_fields, ty};
use alloc::string::String;
use alloc::vec::Vec;
use quote::quote;

pub fn serialize_struct(
    input: syn::DeriveInput,
    cratename: syn::Path,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => unreachable!(),
    };
    let fields = parse_fields(fields)?;
    let write = fields
        .iter()
        .map(|(field, attrs, member)| write_ty(&cratename, field, attrs, member));
    let len_s = fields
        .iter()
        .map(|(field, attrs, member)| len_ty(&cratename, field, attrs, member));

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::#cratename::Write for #name #ty_generics #where_clause {
            #[inline]
            unsafe fn write(&self, __w: &mut ::#cratename::Writer) {
                unsafe {
                    #(#write);*
                }
            }

            #[inline]
            fn len_s(&self) -> usize {
                let mut __l = 0usize;
                #(__l += #len_s;)*
                __l
            }
        }
    })
}

fn len_ty(
    cratename: &syn::Path,
    field: &&syn::Field,
    attrs: &crate::FieldAttrs,
    member: &syn::Member,
) -> proc_macro2::TokenStream {
    let t = ty(&field.ty);
    if attrs.varint {
        let w = match t {
            Ty::I32 => quote!(::#cratename::V32(self.#member as u32)),
            Ty::U32 => quote!(::#cratename::V32(self.#member)),
            Ty::I64 => quote!(::#cratename::V64(self.#member as u64)),
            _ => quote!(::#cratename::V64(self.#member)),
        };
        quote!(::#cratename::Write::len_s(&#w))
    } else if matches!(t, Ty::U8Array) {
        quote!(self.#member.len())
    } else {
        quote!(::#cratename::Write::len_s(&self.#member))
    }
}

fn write_ty(
    cratename: &syn::Path,
    field: &&syn::Field,
    attrs: &crate::FieldAttrs,
    member: &syn::Member,
) -> proc_macro2::TokenStream {
    let t = ty(&field.ty);
    if attrs.varint {
        let w = match t {
            Ty::I32 => quote!(::#cratename::V32(self.#member as u32)),
            Ty::U32 => quote!(::#cratename::V32(self.#member)),
            Ty::I64 => quote!(::#cratename::V64(self.#member as u64)),
            _ => quote!(::#cratename::V64(self.#member)),
        };
        quote!(::#cratename::Write::write(&#w, __w))
    } else if matches!(t, Ty::U8Array) {
        quote!(__w.write(&self.#member))
    } else {
        quote!(::#cratename::Write::write(&self.#member, __w))
    }
}

fn write_ty_enum(
    cratename: &syn::Path,
    field: &syn::Field,
    attrs: &crate::FieldAttrs,
    member: &syn::Ident,
) -> proc_macro2::TokenStream {
    let t = ty(&field.ty);
    if attrs.varint {
        let w = match t {
            Ty::I32 => quote!(::#cratename::V32(*#member as u32)),
            Ty::U32 => quote!(::#cratename::V32(*#member)),
            Ty::I64 => quote!(::#cratename::V64(*#member as u64)),
            _ => quote!(::#cratename::V64(*#member)),
        };
        quote!(::#cratename::Write::write(&#w, __w))
    } else if matches!(t, Ty::U8Array) {
        quote!(__w.write(#member))
    } else {
        quote!(::#cratename::Write::write(#member, __w))
    }
}

fn len_ty_enum(
    cratename: &syn::Path,
    field: &&syn::Field,
    attrs: &crate::FieldAttrs,
    member: &syn::Ident,
) -> proc_macro2::TokenStream {
    let t = ty(&field.ty);
    if attrs.varint {
        let w = match t {
            Ty::I32 => quote!(::#cratename::V32(*#member as u32)),
            Ty::U32 => quote!(::#cratename::V32(*#member)),
            Ty::I64 => quote!(::#cratename::V64(*#member as u64)),
            _ => quote!(::#cratename::V64(*#member)),
        };
        quote!(::#cratename::Write::len_s(&#w))
    } else if matches!(t, Ty::U8Array) {
        quote!(#member.len())
    } else {
        quote!(::#cratename::Write::len_s(#member))
    }
}

pub fn serialize_enum(
    input: syn::DeriveInput,
    cratename: syn::Path,
    attrs: Attrs,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let mut repr = None;
    let varint = attrs.varint;
    let variants = match &input.data {
        syn::Data::Enum(data) => &data.variants,
        _ => unreachable!(),
    };
    let len = variants.len();
    for attr in &input.attrs {
        if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                let path = meta.path.get_ident().unwrap();
                repr = Some(if !varint {
                    quote!(*self as #path)
                } else if len > u32::MAX as usize {
                    quote!(::#cratename::V64(*self as u64))
                } else if len > V21MAX {
                    quote!(::#cratename::V32(*self as u32))
                } else if len > V7MAX {
                    quote!(::#cratename::V21(*self as u32))
                } else {
                    quote!(*self as u8)
                });
                Ok(())
            })?;
        }
    }
    let (write, len_s) = match repr {
        Some(x) => (
            quote!(::#cratename::Write::write(&(#x), __w);),
            quote!(
            ::#cratename::Write::len_s(&(#x))),
        ),
        None => {
            let header = match &attrs.header {
                Some(x) => x,
                None => return Err(syn::Error::new_spanned(input, "expected header")),
            };
            let header = variants
                .iter()
                .map(|variant| {
                    let variant_name = &variant.ident;
                    let header_variant = syn::Ident::new(
                        &ident_case(&attrs, variant_name),
                        proc_macro2::Span::call_site(),
                    );
                    quote! {
                        Self::#variant_name { .. } => #header::#header_variant,
                    }
                })
                .collect::<Vec<_>>();
            let header = &header[..];
            let mut write = Vec::with_capacity(variants.len());

            for variant in variants.iter() {
                let variant_name = &variant.ident;
                let fields = variant.fields.members();
                let fields2 = parse_fields(&variant.fields)?;
                write.push(match &variant.fields {
                    syn::Fields::Named(_) => {
                        let fields2 = fields2.iter().map(|(field, attr, _)| {
                            write_ty_enum(&cratename, field, attr, field.ident.as_ref().unwrap())
                        });
                        quote! {
                            Self::#variant_name { #(#fields),* } => {
                                #(#fields2);*
                            }
                        }
                    }
                    syn::Fields::Unnamed(_) => {
                        let mut s = String::with_capacity(8);
                        s += "__self_";
                        let mut ss = Vec::new();
                        let fields = fields.map(|m| match m {
                            syn::Member::Unnamed(x) => {
                                let mut s = String::with_capacity(8);
                                s += "__self_";
                                s += itoa::Buffer::new().format(x.index);
                                ss.push(s);
                                syn::Ident::new(ss.last().unwrap(), proc_macro2::Span::call_site())
                            }
                            _ => unreachable!(),
                        });
                        let fields2 = fields2.iter().map(|(field, attr, m)| {
                            write_ty_enum(
                                &cratename,
                                field,
                                attr,
                                &match m {
                                    syn::Member::Unnamed(x) => {
                                        s.truncate(7);
                                        s += itoa::Buffer::new().format(x.index);
                                        syn::Ident::new(&s, proc_macro2::Span::call_site())
                                    }
                                    _ => unreachable!(),
                                },
                            )
                        });
                        quote! {
                            Self::#variant_name(#(#fields),*) => {
                                #(#fields2;)*
                            }
                        }
                    }
                    syn::Fields::Unit => quote! {
                        Self::#variant_name {} => {}
                    },
                });
            }
            let mut len = Vec::with_capacity(variants.len());
            for variant in variants.iter() {
                let variant_name = &variant.ident;
                let fields = variant.fields.members();
                let fields2 = parse_fields(&variant.fields)?;
                len.push(match variant.fields {
                    syn::Fields::Named(_) if !variant.fields.is_empty() => {
                        let fields2 = fields2.iter().map(|(field, attr, _)| {
                            len_ty_enum(&cratename, field, attr, field.ident.as_ref().unwrap())
                        });
                        quote! {
                            Self::#variant_name { #(#fields),* } => {
                                #(#fields2)+*
                            }
                        }
                    }
                    syn::Fields::Unnamed(_) if !variant.fields.is_empty() => {
                        let mut s = String::with_capacity(8);
                        s += "__self_";
                        let mut ss = Vec::new();
                        let fields = fields.map(|m| match m {
                            syn::Member::Unnamed(x) => {
                                let mut s = String::with_capacity(8);
                                s += "__self_";
                                s += itoa::Buffer::new().format(x.index);
                                ss.push(s);
                                syn::Ident::new(ss.last().unwrap(), proc_macro2::Span::call_site())
                            }
                            _ => unreachable!(),
                        });
                        let fields2 = fields2.iter().map(|(field, attr, m)| {
                            len_ty_enum(
                                &cratename,
                                field,
                                attr,
                                &match m {
                                    syn::Member::Unnamed(x) => {
                                        s.truncate(7);
                                        s += itoa::Buffer::new().format(x.index);
                                        syn::Ident::new(&s, proc_macro2::Span::call_site())
                                    }
                                    _ => unreachable!(),
                                },
                            )
                        });
                        quote! {
                            Self::#variant_name(#(#fields),*) => {
                                #(#fields2)+*
                            }
                        }
                    }
                    _ => {
                        quote! {
                            Self::#variant_name {} => 0,
                        }
                    }
                });
            }
            (
                quote! {
                    ::#cratename::Write::write(&match self {
                        #(#header)*
                    }, __w);
                    match self {
                        #(#write)*
                    }
                },
                quote! {
                    #cratename::Write::len_s(&match self {
                        #(#header)*
                    }) + match self {
                        #(#len)*
                    }
                },
            )
        }
    };
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::#cratename::Write for #name #ty_generics #where_clause {
            #[inline]
            unsafe fn write(&self, __w: &mut ::#cratename::Writer) {
                unsafe {
                    #write
                }
            }

            #[inline]
            fn len_s(&self) -> usize {
                #len_s
            }
        }
    })
}
