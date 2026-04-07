use crate::{Attrs, Ty, V7MAX, V21MAX, parse_fields, ty};
use alloc::boxed::Box;
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
    let fields = parse_fields(fields)
        .map(|(f, a, m)| (f, a.unwrap(), m))
        .collect::<Box<_>>();

    let write = fields.iter().map(|(field, attrs, member)| {
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
    });
    let len_s = fields.iter().map(|(field, attrs, member)| {
        let t = ty(&field.ty);
        if attrs.varint {
            if matches!(t, Ty::I32) {
                quote!(::#cratename::Write::len_s(&::#cratename::V32(self.#member as u32)))
            } else {
                quote!(::#cratename::Write::len_s(&::#cratename::V32(self.#member)))
            }
        } else if matches!(t, Ty::U8Array) {
            quote!(self.#member.len())
        } else {
            quote!(::#cratename::Write::len_s(&self.#member))
        }
    });

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
    let repr = match repr {
        Some(x) => x,
        None => {
            return Err(syn::Error::new_spanned(
                &input,
                "expected `#[repr(...)]` attribute",
            ));
        }
    };
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::#cratename::Write for #name #ty_generics #where_clause {
            #[inline]
            unsafe fn write(&self, w: &mut ::#cratename::Writer) {
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
