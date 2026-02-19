use crate::{V7MAX, V21MAX, kw};
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
            Some(ident) => quote!(::#cratename::Write::write(&self.#ident, __w);),
            None => {
                let l = proc_macro2::Literal::usize_unsuffixed(idx);
                quote!(::#cratename::Write::write(&self.#l, __w);)
            }
        });
    let len_s = fields
        .iter()
        .enumerate()
        .map(|(idx, field)| match &field.ident {
            Some(ident) => quote!(::#cratename::Write::len_s(&self.#ident)),
            None => {
                let l = proc_macro2::Literal::usize_unsuffixed(idx);
                quote!(::#cratename::Write::len_s(&self.#l))
            }
        });

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::#cratename::Write for #name #ty_generics #where_clause {
            #[inline]
            unsafe fn write(&self, __w: &mut ::#cratename::UnsafeWriter) {
                unsafe {
                    #(#write)*
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
                    if varint && len > V7MAX {
                        repr = Some(quote!(::#cratename::V21(*self as u32)));
                    } else {
                        repr = Some(quote!(*self as u8));
                    }
                } else if meta.path.is_ident("u16") {
                    if varint && len > V7MAX {
                        repr = Some(quote!(::#cratename::V21(*self as u32)));
                    } else {
                        repr = Some(quote!(*self as u16));
                    }
                } else if meta.path.is_ident("u32") {
                    if varint && len > V21MAX {
                        repr = Some(quote!(::#cratename::V32(*self as u32)));
                    } else if varint && len > V7MAX {
                        repr = Some(quote!(::#cratename::V21(*self as u32)));
                    } else {
                        repr = Some(quote!(*self as u32));
                    }
                } else if meta.path.is_ident("u64") {
                    if varint && len > u32::MAX as usize {
                        repr = Some(quote!(::#cratename::V64(*self as u64)));
                    } else if varint && len > V21MAX {
                        repr = Some(quote!(::#cratename::V32(*self as u64 as u32)));
                    } else if varint && len > V7MAX {
                        repr = Some(quote!(::#cratename::V21(*self as u64 as u32)));
                    } else {
                        repr = Some(quote!(*self as u64));
                    }
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
        impl #impl_generics ::#cratename::Write for #name #ty_generics #where_clause {
            #[inline]
            unsafe fn write(&self, w: &mut ::#cratename::UnsafeWriter) {
                unsafe {
                    ::#cratename::Write::write(&(#repr), w);
                }
            }

            #[inline]
            fn len_s(&self) -> usize {
                ::#cratename::Write::len_s(&(#repr))
            }
        }
    })
}
