use crate::kw;
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
    let write = fields
        .iter()
        .enumerate()
        .map(|(idx, field)| match &field.ident {
            Some(ident) => quote!(#cratename::Write::write(&self.#ident, w);),
            None => {
                let ident = syn::Ident::new(
                    itoa::Buffer::new().format(idx),
                    proc_macro2::Span::call_site(),
                );
                quote!(#cratename::Write::write(&self.#ident, w);)
            }
        });
    let sz = fields
        .iter()
        .enumerate()
        .map(|(idx, field)| match &field.ident {
            Some(ident) => quote!(#cratename::Write::sz(&self.#ident)),
            None => {
                let ident = syn::Ident::new(
                    itoa::Buffer::new().format(idx),
                    proc_macro2::Span::call_site(),
                );
                quote!(#cratename::Write::sz(&self.#ident))
            }
        });

    Ok(quote! {
        #[automatically_derived]
        unsafe impl #impl_generics #cratename::Write for #name #ty_generics #where_clause {
            #[inline]
            unsafe fn write(&self, w: &mut #cratename::UnsafeWriter) {
                #(#write)*
            }

            #[inline]
            unsafe fn sz(&self) -> usize {
                let mut __l = 0usize;
                #(__l += #sz;)*
                __l
            }
        }
    })
}

pub fn serialize_enum(
    input: syn::DeriveInput,
    cratename: syn::Path,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let mut repr = None;
    let mut varint = false;
    let variants = match &input.data {
        syn::Data::Enum(data) => &data.variants,
        _ => unreachable!(),
    };
    let len = variants.len();
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
                        repr = Some(quote!(#cratename::V21(*self as u32)));
                    } else {
                        repr = Some(quote!(*self as u8));
                    }
                } else if meta.path.is_ident("u16") {
                    if varint && len > mser::V7MAX {
                        repr = Some(quote!(#cratename::V21(*self as u32)));
                    } else {
                        repr = Some(quote!(*self as u16));
                    }
                } else if meta.path.is_ident("u32") {
                    if varint && len > mser::V21MAX {
                        repr = Some(quote!(#cratename::V32(*self as u32)));
                    } else if varint && len > mser::V7MAX {
                        repr = Some(quote!(#cratename::V21(*self as u32)));
                    } else {
                        repr = Some(quote!(*self as u32));
                    }
                } else if meta.path.is_ident("u64") {
                    if varint && len > u32::MAX as usize {
                        repr = Some(quote!(#cratename::V64(*self as u64)));
                    } else if varint && len > mser::V21MAX {
                        repr = Some(quote!(#cratename::V32(*self as u64 as u32)));
                    } else if varint && len > mser::V7MAX {
                        repr = Some(quote!(#cratename::V21(*self as u64 as u32)));
                    } else {
                        repr = Some(quote!(*self as u64));
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
    let repr =
        repr.ok_or_else(|| syn::Error::new_spanned(&input, "expected `#[repr(...)]` attribute"))?;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        unsafe impl #impl_generics #cratename::Write for #name #ty_generics #where_clause {
            #[inline]
            unsafe fn write(&self, w: &mut #cratename::UnsafeWriter) {
                #cratename::Write::write(&(#repr), w);
            }

            #[inline]
            unsafe fn sz(&self) -> usize {
                #cratename::Write::sz(&(#repr))
            }
        }
    })
}
