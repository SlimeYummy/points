use anyhow::{Result, anyhow};
use quote::{format_ident, quote};
use syn::ItemEnum;

use crate::utils::extract_attr_raw;
use crate::wasm::base::collect_builtin_derives;

pub fn gen_enum(input: &ItemEnum) -> Result<String> {
    let raw = extract_attr_raw(&input.attrs, "repr")?;
    if !matches!(
        raw.as_str(),
        "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64"
    ) {
        return Err(anyhow!("Need a #[repr(i8|u8|i16|u16|i32|u32|i64|u64)]"));
    }

    let enum_name = format_ident!("{}", input.ident.to_string());
    let repr_type = format_ident!("{}", raw);

    let mut variant_tokens = Vec::with_capacity(input.variants.len());
    for variant in &input.variants {
        if !variant.fields.is_empty() {
            return Err(anyhow!("Unsupported enum with fields"));
        }

        let variant_name = &variant.ident;
        let has_default = variant.attrs.iter().any(|attr| attr.path().is_ident("default"));

        match &variant.discriminant {
            Some((_, expr)) => {
                let expr_tokens = expr.clone();
                if has_default {
                    variant_tokens.push(quote! { #[default] #variant_name = #expr_tokens, });
                }
                else {
                    variant_tokens.push(quote! { #variant_name = #expr_tokens, });
                }
            }
            None => {
                if has_default {
                    variant_tokens.push(quote! { #[default] #variant_name, });
                }
                else {
                    variant_tokens.push(quote! { #variant_name, });
                }
            }
        }
    }

    let derives = collect_builtin_derives(&input.attrs);
    let code = if derives.is_empty() {
        let tokens = quote! {
            #[repr(#repr_type)]
            pub enum #enum_name {
                #(#variant_tokens)*
            }
        };
        tokens.to_string()
    }
    else {
        let tokens = quote! {
            #[repr(#repr_type)]
            #[derive(#(#derives),*)]
            pub enum #enum_name {
                #(#variant_tokens)*
            }
        };
        tokens.to_string()
    };

    Ok(format!("{}\n\n", code))
}
