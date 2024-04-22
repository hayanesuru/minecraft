#![recursion_limit = "128"]
#![no_std]

extern crate alloc;

mod writable;

use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{format, vec};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{parenthesized, parse_macro_input, Expr, Lifetime, Lit, Token, Type, TypeParam};

#[proc_macro_derive(Writable, attributes(ser))]
pub fn writable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let tokens = writable::impl_writable(input).unwrap_or_else(|err| err.to_compile_error());
    proc_macro::TokenStream::from(tokens)
}

mod kw {
    syn::custom_keyword!(tag_type);
    syn::custom_keyword!(tag);
    syn::custom_keyword!(peek_tag);
    syn::custom_keyword!(always);

    syn::custom_keyword!(head);
    syn::custom_keyword!(varint);
    syn::custom_keyword!(prefix);
    syn::custom_keyword!(expand);
    syn::custom_keyword!(skip);
    syn::custom_keyword!(add);
    syn::custom_keyword!(u8);
    syn::custom_keyword!(v21);
    syn::custom_keyword!(v32);
    syn::custom_keyword!(none);
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Trait {
    Readable,
    Writable,
}

fn possibly_uses_generic_ty(generic_types: &[&syn::Ident], ty: &Type) -> bool {
    match ty {
        Type::Path(syn::TypePath {
            qself: None,
            path:
                syn::Path {
                    leading_colon: _,
                    segments,
                },
        }) => {
            segments.iter().any(|segment| {
                if generic_types.iter().any(|&ident| *ident == segment.ident) {
                    return true;
                }

                match segment.arguments {
                    syn::PathArguments::None => false,
                    syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        ref args,
                        ..
                    }) => {
                        args.iter().any(|arg| {
                            match arg {
                                syn::GenericArgument::Lifetime(..) => false,
                                syn::GenericArgument::Type(inner_ty) => {
                                    possibly_uses_generic_ty(generic_types, inner_ty)
                                }
                                syn::GenericArgument::AssocType(assoc_type) => {
                                    possibly_uses_generic_ty(generic_types, &assoc_type.ty)
                                }
                                // TODO: How to handle these?
                                syn::GenericArgument::Constraint(..) => true,
                                syn::GenericArgument::Const(..) => true,
                                syn::GenericArgument::AssocConst(_) => true,
                                _ => true,
                            }
                        })
                    }
                    _ => true,
                }
            })
        }
        Type::Slice(syn::TypeSlice { elem, .. }) => possibly_uses_generic_ty(generic_types, elem),
        Type::Tuple(syn::TypeTuple { elems, .. }) => elems
            .iter()
            .any(|elem| possibly_uses_generic_ty(generic_types, elem)),
        Type::Reference(syn::TypeReference { elem, .. }) => {
            possibly_uses_generic_ty(generic_types, elem)
        }
        Type::Paren(syn::TypeParen { elem, .. }) => possibly_uses_generic_ty(generic_types, elem),
        Type::Ptr(syn::TypePtr { elem, .. }) => possibly_uses_generic_ty(generic_types, elem),
        Type::Group(syn::TypeGroup { elem, .. }) => possibly_uses_generic_ty(generic_types, elem),
        Type::Array(syn::TypeArray { elem, len, .. }) => {
            if possibly_uses_generic_ty(generic_types, elem) {
                return true;
            }

            // This is probably too conservative.
            !matches!(len, syn::Expr::Lit(..))
        }
        Type::Never(..) => false,
        _ => true,
    }
}

#[test]
fn test_possibly_uses_generic_ty() {
    macro_rules! assert_test {
        ($result:expr, $($token:tt)+) => {
            assert_eq!(
                possibly_uses_generic_ty(&[&syn::Ident::new("T", proc_macro2::Span::call_site())], &syn::parse2(quote! { $($token)+ }).unwrap()),
                $result
           );
        }
    }

    assert_test!(false, String);
    assert_test!(false, Cow<'a, BTreeMap<u8, u8>>);
    assert_test!(false, Cow<'a, [u8]>);
    assert_test!(false, ());
    assert_test!(false, (u8));
    assert_test!(false, (u8, u8));
    assert_test!(false, &u8);
    assert_test!(false, *const u8);
    assert_test!(false, !);
    assert_test!(false, [u8; 2]);
    assert_test!(true, T);
    assert_test!(true, Dummy::T);
    assert_test!(true, Cow<'a, BTreeMap<T, u8>>);
    assert_test!(true, Cow<'a, BTreeMap<u8, T>>);
    assert_test!(true, Cow<'a, [T]>);
    assert_test!(true, (T));
    assert_test!(true, (u8, T));
    assert_test!(true, &T);
    assert_test!(true, *const T);
    assert_test!(true, [T; 2]);
    assert_test!(true, Vec<T>);
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum PrimitiveTy {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
}

fn parse_primitive_ty(ty: &Type) -> Option<PrimitiveTy> {
    match ty {
        Type::Path(syn::TypePath {
            qself: None,
            path:
                syn::Path {
                    leading_colon: _,
                    segments,
                },
        }) => {
            if segments.len() != 1 {
                return None;
            }

            let segment = &segments[0];
            let ident = segment.ident.to_string();
            match ident.as_str() {
                "u8" => Some(PrimitiveTy::U8),
                "u16" => Some(PrimitiveTy::U16),
                "u32" => Some(PrimitiveTy::U32),
                "u64" => Some(PrimitiveTy::U64),
                "u128" => Some(PrimitiveTy::U128),
                "i8" => Some(PrimitiveTy::I8),
                "i16" => Some(PrimitiveTy::I16),
                "i32" => Some(PrimitiveTy::I32),
                "i64" => Some(PrimitiveTy::I64),
                "i128" => Some(PrimitiveTy::I128),
                "f32" => Some(PrimitiveTy::F32),
                "f64" => Some(PrimitiveTy::F64),
                _ => None,
            }
        }
        _ => None,
    }
}

fn common_tokens(
    ast: &syn::DeriveInput,
    types: &[Type],
    trait_variant: Trait,
) -> (TokenStream, TokenStream, TokenStream) {
    let impl_params = {
        let lifetime_params = ast.generics.lifetimes().map(|alpha| quote! { #alpha });
        let type_params = ast.generics.type_params().map(|ty| {
            let ty_without_default = TypeParam {
                default: None,
                ..ty.clone()
            };
            quote! { #ty_without_default }
        });
        quote! {
            #(#lifetime_params),* #(#type_params),*
        }
    };

    let ty_params = {
        let lifetime_params = ast.generics.lifetimes().map(|alpha| quote! { #alpha });
        let type_params = ast.generics.type_params().map(|ty| {
            let ident = &ty.ident;
            quote! { #ident }
        });
        let mut params = lifetime_params.chain(type_params).peekable();
        if params.peek().is_none() {
            quote! {}
        } else {
            quote! { <#(#params),*> }
        }
    };

    let generics: Vec<_> = ast.generics.type_params().map(|ty| &ty.ident).collect();
    let where_clause = {
        let constraints = types.iter().filter_map(|ty| {
            let possibly_generic = possibly_uses_generic_ty(&generics, ty);
            match (trait_variant, possibly_generic) {
                (Trait::Readable, true) => Some(quote! { #ty: Readable< 'a_, C_ > }),
                (Trait::Readable, false) => None,
                (Trait::Writable, true) => Some(quote! { #ty: Writable< C_ > }),
                (Trait::Writable, false) => None,
            }
        });

        let mut predicates = Vec::new();
        if let Some(where_clause) = ast.generics.where_clause.as_ref() {
            predicates = where_clause
                .predicates
                .iter()
                .map(|pred| quote! { #pred })
                .collect();
        }

        if trait_variant == Trait::Readable {
            for lifetime in ast.generics.lifetimes() {
                predicates.push(quote! { 'a_: #lifetime });
                predicates.push(quote! { #lifetime: 'a_ });
            }
        }

        let items = constraints.chain(predicates);
        if items.clone().next().is_none() {
            quote! {}
        } else {
            quote! { where #(#items),* }
        }
    };

    (impl_params, ty_params, where_clause)
}

#[derive(Copy, Clone)]
enum BasicType {
    U8,
    V32,
    V21,
    None,
}

const DEFAULT_LENGTH_TYPE: BasicType = BasicType::V21;

impl Parse for BasicType {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        let ty = if lookahead.peek(kw::u8) {
            input.parse::<kw::u8>()?;
            BasicType::U8
        } else if lookahead.peek(kw::v32) {
            input.parse::<kw::v32>()?;
            BasicType::V32
        } else if lookahead.peek(kw::v21) {
            input.parse::<kw::v21>()?;
            BasicType::V21
        } else if lookahead.peek(kw::none) {
            input.parse::<kw::none>()?;
            BasicType::None
        } else {
            return Err(lookahead.error());
        };

        Ok(ty)
    }
}

enum IsPrimitive {
    Always,
}

impl Parse for IsPrimitive {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        let ty = if lookahead.peek(kw::always) {
            input.parse::<kw::always>()?;
            IsPrimitive::Always
        } else {
            return Err(lookahead.error());
        };

        Ok(ty)
    }
}

enum ToplevelStructAttribute {
    Prefix { key_token: kw::prefix, expr: Expr },
    StructAttribute(StructAttribute),
}

enum VariantAttribute {
    Tag {},
}

enum StructAttribute {}

enum EnumAttribute {
    TagType {},
    PeekTag {},
}

enum VariantOrStructAttribute {
    Variant(VariantAttribute),
    Struct(StructAttribute),
}

fn parse_variant_attribute(
    input: &syn::parse::ParseStream,
    lookahead: &syn::parse::Lookahead1,
) -> syn::parse::Result<Option<VariantAttribute>> {
    let attribute = if lookahead.peek(kw::tag) {
        let _key_token = input.parse::<kw::tag>()?;
        let _: Token![=] = input.parse()?;
        let raw_tag: syn::LitInt = input.parse()?;
        let _tag = raw_tag
            .base10_parse::<u64>()
            .map_err(|err| syn::Error::new(raw_tag.span(), err))?;

        VariantAttribute::Tag {}
    } else {
        return Ok(None);
    };

    Ok(Some(attribute))
}

fn parse_struct_attribute(
    _input: &syn::parse::ParseStream,
    _lookahead: &syn::parse::Lookahead1,
) -> syn::parse::Result<Option<StructAttribute>> {
    Ok(None)
}

fn parse_enum_attribute(
    input: &syn::parse::ParseStream,
    lookahead: &syn::parse::Lookahead1,
) -> syn::parse::Result<Option<EnumAttribute>> {
    let attribute = if lookahead.peek(kw::tag_type) {
        let _key_token = input.parse::<kw::tag_type>()?;
        let _: Token![=] = input.parse()?;
        let _ty: BasicType = input.parse()?;

        EnumAttribute::TagType {}
    } else if lookahead.peek(kw::peek_tag) {
        let _key_token = input.parse::<kw::peek_tag>()?;
        EnumAttribute::PeekTag {}
    } else {
        return Ok(None);
    };

    Ok(Some(attribute))
}

impl Parse for ToplevelStructAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::prefix) {
            let key_token = input.parse::<kw::prefix>()?;
            let _: Token![=] = input.parse()?;
            let expr: Expr = input.parse()?;
            Ok(ToplevelStructAttribute::Prefix { key_token, expr })
        } else if let Some(attr) = parse_struct_attribute(&input, &lookahead)? {
            Ok(ToplevelStructAttribute::StructAttribute(attr))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for StructAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        parse_struct_attribute(&input, &lookahead)?.ok_or_else(|| lookahead.error())
    }
}

impl Parse for EnumAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        parse_enum_attribute(&input, &lookahead)?.ok_or_else(|| lookahead.error())
    }
}

impl Parse for VariantOrStructAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        if let Some(attr) = parse_variant_attribute(&input, &lookahead)? {
            return Ok(VariantOrStructAttribute::Variant(attr));
        }

        if let Some(attr) = parse_struct_attribute(&input, &lookahead)? {
            return Ok(VariantOrStructAttribute::Struct(attr));
        }

        Err(lookahead.error())
    }
}

struct StructAttributes {}

struct ToplevelStructAttributes {
    prefix: Option<Expr>,
    struct_attributes: StructAttributes,
}

impl core::ops::Deref for ToplevelStructAttributes {
    type Target = StructAttributes;
    fn deref(&self) -> &Self::Target {
        &self.struct_attributes
    }
}

fn parse_attributes<T: Parse>(attrs: &[syn::Attribute]) -> Result<Vec<T>, syn::Error> {
    struct RawAttributes<T>(Punctuated<T, Token![,]>);

    impl<T: Parse> Parse for RawAttributes<T> {
        fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
            let content;
            parenthesized!(content in input);
            Ok(RawAttributes(
                content.parse_terminated(T::parse, Token![,])?,
            ))
        }
    }

    let mut output = Vec::new();
    for raw_attr in attrs {
        let path = raw_attr.path().to_token_stream().to_string();
        if path != "ser" {
            continue;
        }

        let meta_list = raw_attr.meta.require_list()?;
        let parsed_attrs = syn::parse::Parser::parse2(
            Punctuated::<T, Token![,]>::parse_terminated,
            meta_list.tokens.clone(),
        )?;
        for attr in parsed_attrs {
            output.push(attr);
        }
    }

    Ok(output)
}

fn collect_struct_attributes(attrs: Vec<StructAttribute>) -> Result<StructAttributes, syn::Error> {
    for _attr in attrs {}

    Ok(StructAttributes {})
}

fn collect_toplevel_struct_attributes(
    attrs: Vec<ToplevelStructAttribute>,
) -> Result<ToplevelStructAttributes, syn::Error> {
    let mut struct_attributes = Vec::new();
    let mut prefix = None;
    for attr in attrs {
        match attr {
            ToplevelStructAttribute::Prefix { key_token, expr } => {
                if prefix.is_some() {
                    let message = "Duplicate 'prefix'";
                    return Err(syn::Error::new(key_token.span(), message));
                }
                prefix = Some(expr);
            }
            ToplevelStructAttribute::StructAttribute(attr) => {
                struct_attributes.push(attr);
            }
        }
    }

    let struct_attributes = collect_struct_attributes(struct_attributes)?;
    Ok(ToplevelStructAttributes {
        prefix,
        struct_attributes,
    })
}

#[derive(PartialEq)]
enum StructKind {
    Unit,
    Named,
    Unnamed,
}

struct Struct<'a> {
    fields: Vec<Field<'a>>,
    kind: StructKind,
}

impl<'a> Struct<'a> {
    fn new(fields: &'a syn::Fields, _attrs: &StructAttributes) -> Result<Self, syn::Error> {
        let structure = match fields {
            syn::Fields::Unit => Struct {
                fields: Vec::new(),
                kind: StructKind::Unit,
            },
            syn::Fields::Named(syn::FieldsNamed { ref named, .. }) => Struct {
                fields: get_fields(named)?,
                kind: StructKind::Named,
            },
            syn::Fields::Unnamed(syn::FieldsUnnamed { ref unnamed, .. }) => Struct {
                fields: get_fields(unnamed)?,
                kind: StructKind::Unnamed,
            },
        };

        Ok(structure)
    }
}

#[derive(Clone)]
struct Field<'a> {
    index: usize,
    name: Option<&'a syn::Ident>,
    raw_ty: &'a Type,
    expand: bool,
    len_type: Option<BasicType>,
    ty: Opt<Ty>,
    skip: bool,
    varint: bool,
    add: Option<Expr>,
}

impl<'a> Field<'a> {
    fn var_name(&self) -> syn::Ident {
        if let Some(name) = self.name {
            name.clone()
        } else {
            syn::Ident::new(&format!("t{}", self.index), Span::call_site())
        }
    }

    fn name(&self) -> syn::Member {
        if let Some(name) = self.name {
            syn::Member::Named(name.clone())
        } else {
            syn::Member::Unnamed(syn::Index {
                index: self.index as u32,
                span: Span::call_site(),
            })
        }
    }

    fn bound_types(&self) -> Vec<Type> {
        match self.ty.inner() {
            Ty::Array(inner_ty, ..)
            | Ty::Vec(inner_ty)
            | Ty::HashSet(inner_ty)
            | Ty::BTreeSet(inner_ty)
            | Ty::CowHashSet(inner_ty)
            | Ty::CowBTreeSet(inner_ty)
            | Ty::CowSlice(inner_ty)
            | Ty::RefSlice(inner_ty) => vec![inner_ty.clone()],
            Ty::HashMap(key_ty, value_ty)
            | Ty::BTreeMap(key_ty, value_ty)
            | Ty::CowHashMap(key_ty, value_ty)
            | Ty::CowBTreeMap(key_ty, value_ty) => vec![key_ty.clone(), value_ty.clone()],
            Ty::RefSliceStr
            | Ty::RefSliceU8
            | Ty::RefStr
            | Ty::String
            | Ty::CowStr
            | Ty::Primitive(..) => {
                vec![]
            }
            Ty::Type(_) => vec![self.raw_ty.clone()],
        }
    }
}

enum LengthKind {
    Expr(Expr),
    UntilEndOfFile,
}

impl Parse for LengthKind {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        if input.peek(Token![..]) {
            let _: Token![..] = input.parse()?;
            Ok(LengthKind::UntilEndOfFile)
        } else {
            let expr: Expr = input.parse()?;
            Ok(LengthKind::Expr(expr))
        }
    }
}

enum FieldAttribute {
    Len { key_span: Span, ty: BasicType },
    Skip { key_span: Span },
    Add { key_span: Span, prefix: Expr },
    Expand { key_span: Span },
    VarInt { key_span: Span },
}

impl Parse for FieldAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let lookahead = input.lookahead1();
        let value = if lookahead.peek(kw::expand) {
            let key_token = input.parse::<kw::expand>()?;
            FieldAttribute::Expand {
                key_span: key_token.span(),
            }
        } else if lookahead.peek(kw::head) {
            let key_token = input.parse::<kw::head>()?;
            let _: Token![=] = input.parse()?;
            let ty: BasicType = input.parse()?;
            FieldAttribute::Len {
                key_span: key_token.span(),
                ty,
            }
        } else if lookahead.peek(kw::skip) {
            let key_token = input.parse::<kw::skip>()?;
            FieldAttribute::Skip {
                key_span: key_token.span(),
            }
        } else if lookahead.peek(kw::add) {
            let key_token = input.parse::<kw::add>()?;
            let _: Token![=] = input.parse()?;
            let expr: Expr = input.parse()?;

            FieldAttribute::Add {
                key_span: key_token.span(),
                prefix: expr,
            }
        } else if lookahead.peek(kw::varint) {
            let key_token = input.parse::<kw::varint>()?;
            FieldAttribute::VarInt {
                key_span: key_token.span(),
            }
        } else {
            return Err(lookahead.error());
        };

        Ok(value)
    }
}

#[derive(Clone)]
enum Opt<T> {
    Plain(T),
    Option(T),
}

impl<T> Opt<T> {
    fn inner(&self) -> &T {
        match *self {
            Opt::Option(ref inner) => inner,
            Opt::Plain(ref inner) => inner,
        }
    }
}

#[derive(Clone)]
enum Ty {
    String,
    Vec(Type),
    CowSlice(Type),
    CowStr,
    HashMap(Type, Type),
    HashSet(Type),
    BTreeMap(Type, Type),
    BTreeSet(Type),

    CowHashMap(Type, Type),
    CowHashSet(Type),
    CowBTreeMap(Type, Type),
    CowBTreeSet(Type),

    RefSliceStr,
    RefSliceU8,
    RefSlice(Type),
    RefStr,

    Array(Type, u32),

    Primitive(PrimitiveTy),
    Type(Type),
}

fn extract_inner_ty(args: &Punctuated<syn::GenericArgument, syn::token::Comma>) -> Option<&Type> {
    if args.len() != 1 {
        return None;
    }

    match args[0] {
        syn::GenericArgument::Type(ref ty) => Some(ty),
        _ => None,
    }
}

fn extract_inner_ty_2(
    args: &Punctuated<syn::GenericArgument, syn::token::Comma>,
) -> Option<(&Type, &Type)> {
    if args.len() != 2 {
        return None;
    }

    let ty_1 = match args[0] {
        syn::GenericArgument::Type(ref ty) => ty,
        _ => return None,
    };

    let ty_2 = match args[1] {
        syn::GenericArgument::Type(ref ty) => ty,
        _ => return None,
    };

    Some((ty_1, ty_2))
}

fn extract_lifetime_and_inner_ty(
    args: &Punctuated<syn::GenericArgument, syn::token::Comma>,
) -> Option<(&Lifetime, &Type)> {
    if args.len() != 2 {
        return None;
    }

    let lifetime = match args[0] {
        syn::GenericArgument::Lifetime(ref lifetime) => lifetime,
        _ => return None,
    };

    let ty = match args[1] {
        syn::GenericArgument::Type(ref ty) => ty,
        _ => return None,
    };

    Some((lifetime, ty))
}

fn extract_slice_inner_ty(ty: &Type) -> Option<&Type> {
    match *ty {
        Type::Slice(syn::TypeSlice { ref elem, .. }) => Some(elem),
        _ => None,
    }
}

fn is_bare_ty(ty: &Type, name: &str) -> bool {
    match *ty {
        Type::Path(syn::TypePath {
            path:
                syn::Path {
                    leading_colon: None,
                    ref segments,
                },
            qself: None,
        }) if segments.len() == 1 => segments[0].ident == name && segments[0].arguments.is_empty(),
        _ => false,
    }
}

fn extract_option_inner_ty(ty: &Type) -> Option<&Type> {
    match *ty {
        Type::Path(syn::TypePath {
            path:
                syn::Path {
                    leading_colon: None,
                    ref segments,
                },
            qself: None,
        }) if segments.len() == 1 && segments[0].ident == "Option" => match segments[0].arguments {
            syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                colon2_token: None,
                ref args,
                ..
            }) if args.len() == 1 => match args[0] {
                syn::GenericArgument::Type(ref ty) => Some(ty),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn parse_ty(ty: &Type) -> Ty {
    parse_special_ty(ty).unwrap_or_else(|| Ty::Type(ty.clone()))
}

fn parse_special_ty(ty: &Type) -> Option<Ty> {
    if let Some(ty) = parse_primitive_ty(ty) {
        return Some(Ty::Primitive(ty));
    }

    match *ty {
        Type::Path(syn::TypePath {
            path:
                syn::Path {
                    leading_colon: None,
                    ref segments,
                },
            qself: None,
        }) if segments.len() == 1 => {
            let name = &segments[0].ident;
            match segments[0].arguments {
                syn::PathArguments::None => {
                    if name == "String" {
                        Some(Ty::String)
                    } else {
                        None
                    }
                }
                syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    colon2_token: None,
                    ref args,
                    ..
                }) => {
                    if name == "Vec" {
                        Some(Ty::Vec(extract_inner_ty(args)?.clone()))
                    } else if name == "HashSet" {
                        Some(Ty::HashSet(extract_inner_ty(args)?.clone()))
                    } else if name == "BTreeSet" {
                        Some(Ty::BTreeSet(extract_inner_ty(args)?.clone()))
                    } else if name == "Cow" {
                        let (_, ty) = extract_lifetime_and_inner_ty(args)?;
                        if let Some(inner_ty) = extract_slice_inner_ty(ty) {
                            Some(Ty::CowSlice(inner_ty.clone()))
                        } else if is_bare_ty(ty, "str") {
                            Some(Ty::CowStr)
                        } else {
                            match *ty {
                                Type::Path(syn::TypePath {
                                    path:
                                        syn::Path {
                                            leading_colon: None,
                                            ref segments,
                                        },
                                    qself: None,
                                }) if segments.len() == 1 => {
                                    let inner_name = &segments[0].ident;
                                    match segments[0].arguments {
                                        syn::PathArguments::AngleBracketed(
                                            syn::AngleBracketedGenericArguments {
                                                colon2_token: None,
                                                ref args,
                                                ..
                                            },
                                        ) => {
                                            if inner_name == "HashSet" {
                                                Some(Ty::CowHashSet(
                                                    extract_inner_ty(args)?.clone(),
                                                ))
                                            } else if inner_name == "BTreeSet" {
                                                Some(Ty::CowBTreeSet(
                                                    extract_inner_ty(args)?.clone(),
                                                ))
                                            } else if inner_name == "HashMap" {
                                                let (key_ty, value_ty) = extract_inner_ty_2(args)?;
                                                Some(Ty::CowHashMap(
                                                    key_ty.clone(),
                                                    value_ty.clone(),
                                                ))
                                            } else if inner_name == "BTreeMap" {
                                                let (key_ty, value_ty) = extract_inner_ty_2(args)?;
                                                Some(Ty::CowBTreeMap(
                                                    key_ty.clone(),
                                                    value_ty.clone(),
                                                ))
                                            } else {
                                                None
                                            }
                                        }
                                        _ => None,
                                    }
                                }
                                _ => None,
                            }
                        }
                    } else if name == "HashMap" {
                        let (key_ty, value_ty) = extract_inner_ty_2(args)?;
                        Some(Ty::HashMap(key_ty.clone(), value_ty.clone()))
                    } else if name == "BTreeMap" {
                        let (key_ty, value_ty) = extract_inner_ty_2(args)?;
                        Some(Ty::BTreeMap(key_ty.clone(), value_ty.clone()))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        Type::Array(syn::TypeArray {
            ref elem,
            len:
                Expr::Lit(syn::ExprLit {
                    ref attrs,
                    lit: Lit::Int(ref literal),
                }),
            ..
        }) if attrs.is_empty() => {
            if let Ok(length) = literal.base10_parse::<u32>() {
                Some(Ty::Array((**elem).clone(), length))
            } else {
                None
            }
        }
        Type::Reference(syn::TypeReference {
            lifetime: Some(_),
            mutability: None,
            ref elem,
            ..
        }) => {
            if let Some(inner_ty) = extract_slice_inner_ty(elem) {
                if let Type::Reference(syn::TypeReference {
                    lifetime: Some(_),
                    mutability: _,
                    elem,
                    ..
                }) = inner_ty
                {
                    if is_bare_ty(elem, "str") {
                        Some(Ty::RefSliceStr)
                    } else {
                        Some(Ty::RefSlice(inner_ty.clone()))
                    }
                } else if is_bare_ty(inner_ty, "u8") {
                    Some(Ty::RefSliceU8)
                } else {
                    Some(Ty::RefSlice(inner_ty.clone()))
                }
            } else if is_bare_ty(elem, "str") {
                Some(Ty::RefStr)
            } else if let Type::Array(syn::TypeArray {
                elem,
                len:
                    Expr::Lit(syn::ExprLit {
                        attrs,
                        lit: Lit::Int(literal),
                    }),
                ..
            }) = &**elem
            {
                if attrs.is_empty() {
                    if let Ok(length) = literal.base10_parse::<u32>() {
                        Some(Ty::Array((**elem).clone(), length))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_fields(fields: &Punctuated<syn::Field, Comma>) -> Result<Vec<Field>, syn::Error> {
    fields.into_iter().enumerate().map(get_field).collect()
}

fn get_field(field: (usize, &syn::Field)) -> Result<Field<'_>, syn::Error> {
    let (index, field) = field;

    let mut expand = false;
    let mut len_type = None;
    let mut skip = false;
    let mut varint = false;
    let mut add = None;
    for attr in parse_attributes::<FieldAttribute>(&field.attrs)? {
        match attr {
            FieldAttribute::Expand { key_span } => {
                if expand {
                    let message = "duplicate \"expand\"";
                    return Err(syn::Error::new(key_span, message));
                }
                expand = true;
            }
            FieldAttribute::Len { key_span, ty } => {
                if len_type.is_some() {
                    let message = "duplicate \"len_type\"";
                    return Err(syn::Error::new(key_span, message));
                }

                len_type = Some((key_span, ty));
            }
            FieldAttribute::Skip { key_span } => {
                if skip {
                    let message = "duplicate \"skip\"";
                    return Err(syn::Error::new(key_span, message));
                }
                skip = true;
            }
            FieldAttribute::Add { key_span, prefix } => {
                if add.is_some() {
                    let message = "duplicate \"add\"";
                    return Err(syn::Error::new(key_span, message));
                }
                add = Some(prefix);
            }
            FieldAttribute::VarInt { key_span } => {
                if varint {
                    let message = "duplicate \"varint\"";
                    return Err(syn::Error::new(key_span, message));
                }
                varint = true;
            }
        }
    }
    let ty = if let Some(ty) = extract_option_inner_ty(&field.ty) {
        Opt::Option(parse_ty(ty))
    } else {
        Opt::Plain(parse_ty(&field.ty))
    };
    if len_type.is_some() {
        match ty {
            Opt::Plain(Ty::Array(..))
            | Opt::Option(Ty::Array(..))
            | Opt::Plain(Ty::Primitive(..))
            | Opt::Option(Ty::Primitive(..))
            | Opt::Plain(Ty::Type(..))
            | Opt::Option(Ty::Type(..)) => {
                return Err(
                    syn::Error::new(
                        field.ty.span(),
                        "The 'len_type' attribute is only supported for `Vec`, `String`, `Cow<[_]>`, `Cow<str>`, `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, `Cow<HashMap>`, `Cow<HashSet>`, `Cow<BTreeMap>`, `Cow<BTreeSet>`, `&[u8]`, `&str` and for `Option<T>` where `T` is one of these types"
                   )
               );
            }
            _ => {}
        }
    }
    Ok(Field {
        index,
        name: field.ident.as_ref(),
        raw_ty: &field.ty,
        len_type: len_type.map(|x| x.1),
        expand,
        ty,
        skip,
        varint,
        add,
    })
}
