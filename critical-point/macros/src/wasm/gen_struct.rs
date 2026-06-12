use anyhow::{Result, anyhow};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::ParseStream;
use syn::{Attribute, Fields, FieldsNamed, FieldsUnnamed, Ident, ItemStruct, LitInt, Token};

use crate::wasm::base::collect_builtin_derives;

pub fn gen_struct(attr: TokenStream, input: &ItemStruct) -> Result<(String, Vec<TokenStream2>)> {
    let struct_name = &input.ident;

    let derives = collect_builtin_derives(&input.attrs);
    let repr_attrs = extract_repr_attrs(&input.attrs);

    let mut rust_asserts = vec![];
    let code;
    match &input.fields {
        // struct A;
        Fields::Unit => {
            code = generate_unit_struct(struct_name, &derives, &repr_attrs)?;
        }
        // struct A(u32, f32);
        Fields::Unnamed(fields) => {
            (code, rust_asserts) = generate_tuple_struct(struct_name, &derives, &repr_attrs, fields)?;
        }
        // struct B { a: f32, b: String }
        Fields::Named(fields) => {
            (code, rust_asserts) = generate_named_struct(struct_name, &derives, &repr_attrs, fields)?;
        }
    };

    let mut wasm_asserts = String::new();
    let size_align = parse_struct_size_align(&attr)?;
    match size_align {
        Some((size, align)) => {
            rust_asserts.push(quote! {
                static_assertions::const_assert_eq!(std::mem::size_of::<#struct_name>(), #size);
                static_assertions::const_assert_eq!(std::mem::align_of::<#struct_name>(), #align);
            });

            wasm_asserts = (quote! {
                static_assertions::const_assert_eq!(std::mem::size_of::<#struct_name>(), #size);
                static_assertions::const_assert_eq!(std::mem::align_of::<#struct_name>(), #align);
            })
            .to_string();
        }
        None => {
            match &input.fields {
                Fields::Unit => {}
                Fields::Unnamed(fields) if fields.unnamed.len() <= 1 => {}
                Fields::Named(fields) if fields.named.len() <= 1 => {}
                _ => {
                    return Err(anyhow!("Invalid size and align"));
                }
            };
        }
    }

    Ok((format!("{}\n{}\n\n", code, wasm_asserts), rust_asserts))
}

fn parse_struct_size_align(attr: &TokenStream) -> Result<Option<(usize, usize)>> {
    if attr.is_empty() {
        return Ok(None);
    }

    let attr_ts: TokenStream2 = attr.clone().into();
    let (size, align) = syn::parse::Parser::parse2(
        |input: ParseStream| -> syn::Result<(usize, usize)> {
            let size_lit: LitInt = input.parse()?;
            let size: usize = size_lit.base10_parse()?;
            input.parse::<Token![,]>()?;
            let align_lit: LitInt = input.parse()?;
            let align: usize = align_lit.base10_parse()?;
            Ok((size, align))
        },
        attr_ts,
    )?;
    Ok(Some((size, align)))
}

fn extract_repr_attrs(attrs: &[Attribute]) -> Vec<&Attribute> {
    attrs.iter().filter(|attr| attr.path().is_ident("repr")).collect()
}

fn generate_unit_struct(struct_name: &Ident, derives: &[Ident], repr_attrs: &[&Attribute]) -> Result<String> {
    let tokens = if derives.is_empty() {
        quote! {
            #(#repr_attrs)*
            pub struct #struct_name;
        }
    }
    else {
        quote! {
            #(#repr_attrs)*
            #[derive(#(#derives),*)]
            pub struct #struct_name;
        }
    };
    Ok(tokens.to_string())
}

fn generate_tuple_struct(
    struct_name: &Ident,
    derives: &[Ident],
    repr_attrs: &[&Attribute],
    fields: &FieldsUnnamed,
) -> Result<(String, Vec<TokenStream2>)> {
    let mut field_tokens = Vec::with_capacity(fields.unnamed.len());
    let mut asserts = vec![];

    for f in &fields.unnamed {
        if let Some((size, align)) = extract_wasm_hide(&f.attrs)? {
            let array_type = map_align_type(align)?;
            let array_len = size / align;
            if array_len == 0 || size % align != 0 {
                return Err(anyhow!("Invalid size and align"));
            }
            field_tokens.push(quote! { [#array_type; #array_len] });

            let ty = &f.ty;
            asserts.push(quote! {
                static_assertions::const_assert_eq!(std::mem::size_of::<#ty>(), #size);
                static_assertions::const_assert_eq!(std::mem::align_of::<#ty>(), #align);
            });
        }
        else {
            let vis = &f.vis;
            let ty = &f.ty;
            field_tokens.push(quote! { #vis #ty });
        }
    }

    let tokens = if derives.is_empty() {
        quote! {
            #(#repr_attrs)*
            pub struct #struct_name(#(#field_tokens),*);
        }
    }
    else {
        quote! {
            #(#repr_attrs)*
            #[derive(#(#derives),*)]
            pub struct #struct_name(#(#field_tokens),*);
        }
    };
    Ok((tokens.to_string(), asserts))
}

fn generate_named_struct(
    struct_name: &Ident,
    derives: &[Ident],
    repr_attrs: &[&Attribute],
    fields: &FieldsNamed,
) -> Result<(String, Vec<TokenStream2>)> {
    let mut field_tokens = Vec::with_capacity(fields.named.len());
    let mut asserts = vec![];

    for f in &fields.named {
        if let Some((size, align)) = extract_wasm_hide(&f.attrs)? {
            let array_type = map_align_type(align)?;
            let array_len = size / align;
            if array_len == 0 || size % align != 0 {
                return Err(anyhow!("Invalid size and align"));
            }
            let name = &f.ident;
            let ty = &f.ty;
            field_tokens.push(quote! { #name: [#array_type; #array_len], });

            asserts.push(quote! {
                static_assertions::const_assert_eq!(std::mem::size_of::<#ty>(), #size);
                static_assertions::const_assert_eq!(std::mem::align_of::<#ty>(), #align);
            });
        }
        else {
            let vis = &f.vis;
            let name = &f.ident;
            let ty = &f.ty;
            field_tokens.push(quote! { #vis #name: #ty, });
        }
    }

    let tokens = if derives.is_empty() {
        quote! {
            #(#repr_attrs)*
            pub struct #struct_name {
                #(#field_tokens)*
            }
        }
    }
    else {
        quote! {
            #(#repr_attrs)*
            #[derive(#(#derives),*)]
            pub struct #struct_name {
                #(#field_tokens)*
            }
        }
    };
    Ok((tokens.to_string(), asserts))
}

// #[wasm_hide(size, align)]
fn extract_wasm_hide(attrs: &[Attribute]) -> Result<Option<(usize, usize)>> {
    for attr in attrs {
        if !attr.path().is_ident("wasm_hide") {
            continue;
        }

        let (size, align): (usize, usize) = attr.parse_args_with(|input: ParseStream| {
            let size_lit: LitInt = input.parse()?;
            let size: usize = size_lit.base10_parse()?;
            input.parse::<Token![,]>()?;
            let align_lit: LitInt = input.parse()?;
            let align: usize = align_lit.base10_parse()?;
            Ok((size, align))
        })?;
        return Ok(Some((size, align)));
    }
    Ok(None)
}

fn map_align_type(align: usize) -> Result<TokenStream2> {
    match align {
        1 => Ok(quote! { u8 }),
        2 => Ok(quote! { u16 }),
        4 => Ok(quote! { u32 }),
        8 => Ok(quote! { u64 }),
        16 => Ok(quote! { u128 }),
        _ => Err(anyhow!(
            "wasm_hide: unsupported align {}, only 1, 2, 4, 8, 16 are supported",
            align
        )),
    }
}
