use anyhow::Result;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use std::collections::HashMap;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Attribute, Fields, ItemStruct, Meta, Token};

pub fn extract_attr_raw(attrs: &[Attribute], name: &str) -> Result<String> {
    let mut raw = String::new();
    if let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident(name)) {
        attr.parse_nested_meta(|meta| {
            raw = meta.path.into_token_stream().to_string();
            Ok(())
        })?;
    }
    Ok(raw)
}

pub fn parse_token_stream_args(attr: TokenStream) -> Result<Vec<String>> {
    let mut args = Vec::new();
    if !attr.is_empty() {
        let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
        let attr_ts: TokenStream2 = attr.clone().into();
        let nested = parser.parse2(attr_ts)?;
        for meta in nested {
            args.push(meta.path().to_token_stream().to_string());
        }
    }
    Ok(args)
}

pub fn parse_int_or_consts(consts: &HashMap<String, u32>, num: &str) -> Result<u32> {
    let n = match consts.get(num) {
        Some(c) => *c,
        None => num.parse()?,
    };
    Ok(n)
}

pub fn remove_field_attrs(input: &mut ItemStruct, attr: &str) {
    match &mut input.fields {
        Fields::Named(fields) => {
            for field in &mut fields.named {
                field.attrs.retain(|a| !a.path().is_ident(attr));
            }
        }
        Fields::Unnamed(fields) => {
            for field in &mut fields.unnamed {
                field.attrs.retain(|a| !a.path().is_ident(attr));
            }
        }
        Fields::Unit => {}
    }
}
